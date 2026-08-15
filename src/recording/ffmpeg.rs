//! FFmpeg 子进程管理器 — 启动、监控、优雅停止 FFmpeg 录制进程

use crate::models::OutputFormat;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::io::AsyncWriteExt;
use tracing::{info, warn, error};

/// FFmpeg 录制器 — 管理单个录制任务的 FFmpeg 子进程
pub struct FFmpegRecorder {
    /// 录制任务ID
    recording_id: String,
    /// FFmpeg 子进程
    process: Option<Child>,
    /// 输出文件路径
    output_path: PathBuf,
    /// 是否已请求停止
    should_stop: bool,
}

impl FFmpegRecorder {
    pub fn new(recording_id: &str, output_path: PathBuf) -> Self {
        Self {
            recording_id: recording_id.to_string(),
            process: None,
            output_path,
            should_stop: false,
        }
    }

    /// 构建 FFmpeg 命令行参数
    pub fn build_ffmpeg_command(
        stream_url: &str,
        output_path: &PathBuf,
        format: &OutputFormat,
        user_agent: &str,
        proxy_url: Option<&str>,
    ) -> Vec<String> {
        let mut args = vec![
            "ffmpeg".to_string(),
            "-y".to_string(),
            "-v".to_string(), "verbose".to_string(),
            "-rw_timeout".to_string(), "15000000".to_string(),
            "-loglevel".to_string(), "error".to_string(),
            "-hide_banner".to_string(),
            "-user_agent".to_string(), user_agent.to_string(),
            "-protocol_whitelist".to_string(),
            "rtmp,crypto,file,http,https,tcp,tls,udp,rtp,httpproxy".to_string(),
            "-thread_queue_size".to_string(), "1024".to_string(),
            "-analyzeduration".to_string(), "20000000".to_string(),
            "-probesize".to_string(), "10000000".to_string(),
            "-fflags".to_string(), "+discardcorrupt+igndts".to_string(),
            "-re".to_string(),
        ];

        // 代理设置
        if let Some(proxy) = proxy_url {
            if !proxy.is_empty() {
                args.push("-http_proxy".to_string());
                args.push(proxy.to_string());
            }
        }

        // 输入URL
        args.push("-i".to_string());
        args.push(stream_url.to_string());

        // 通用输出参数
        args.extend_from_slice(&[
            "-bufsize".to_string(), "8000k".to_string(),
            "-sn".to_string(), "-dn".to_string(),
            "-reconnect_delay_max".to_string(), "60".to_string(),
            "-reconnect_streamed".to_string(),
            "-reconnect_at_eof".to_string(),
            "-max_muxing_queue_size".to_string(), "1024".to_string(),
            "-correct_ts_overflow".to_string(), "1".to_string(),
            "-avoid_negative_ts".to_string(), "1".to_string(),
            "-flush_packets".to_string(), "1".to_string(),
        ]);

        // 格式特定参数（流复制，避免转码）
        match format {
            OutputFormat::TS => {
                args.extend_from_slice(&[
                    "-c:v".to_string(), "copy".to_string(),
                    "-c:a".to_string(), "copy".to_string(),
                    "-f".to_string(), "mpegts".to_string(),
                ]);
            }
            OutputFormat::MP4 | OutputFormat::MOV => {
                args.extend_from_slice(&[
                    "-c:v".to_string(), "copy".to_string(),
                    "-c:a".to_string(), "copy".to_string(),
                ]);
            }
            OutputFormat::MKV => {
                args.extend_from_slice(&[
                    "-c:v".to_string(), "copy".to_string(),
                    "-c:a".to_string(), "copy".to_string(),
                    "-f".to_string(), "matroska".to_string(),
                ]);
            }
            OutputFormat::FLV => {
                args.extend_from_slice(&[
                    "-c:v".to_string(), "copy".to_string(),
                    "-c:a".to_string(), "copy".to_string(),
                    "-f".to_string(), "flv".to_string(),
                ]);
            }
        }

        args.push(output_path.to_string_lossy().to_string());
        args
    }

    /// 启动 FFmpeg 子进程
    ///
    /// 返回 Ok(()) 表示进程启动成功，开始录制循环
    pub async fn start(
        &mut self,
        stream_url: &str,
        format: &OutputFormat,
        user_agent: &str,
        proxy_url: Option<&str>,
    ) -> Result<(), String> {
        let ffmpeg_args = Self::build_ffmpeg_command(
            stream_url,
            &self.output_path,
            format,
            user_agent,
            proxy_url,
        );

        // 跳过 "ffmpeg" 本身
        let program = &ffmpeg_args[0];
        let args = &ffmpeg_args[1..];

        info!(
            "启动 FFmpeg: {} {}",
            program,
            args.join(" ")
        );

        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("启动 FFmpeg 失败: {} (请确认 FFmpeg 已安装并在 PATH 中)", e))?;

        // 消费 stderr 管道：不读取会导致管道缓冲区（约64KB）写满后 FFmpeg 阻塞。
        // 逐行读取并输出到 tracing 日志，FFmpeg 退出后管道关闭，task 自然结束。
        if let Some(stderr) = child.stderr.take() {
            let rid = self.recording_id.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::debug!("[FFmpeg:{}] {}", rid, line);
                }
            });
        }

        self.process = Some(child);
        self.should_stop = false;
        Ok(())
    }

    /// 录制监控循环
    ///
    /// 持续监控 FFmpeg 进程状态，直到进程退出或收到停止信号。
    /// 返回录制是否正常结束。
    pub async fn monitoring_loop(&mut self) -> Result<bool, String> {
        let mut process = self.process.take().ok_or("录制进程未启动")?;

        loop {
            if self.should_stop {
                info!("收到停止信号，正在优雅停止 FFmpeg...");
                Self::graceful_stop(&mut process).await;
                return Ok(true);
            }

            match process.try_wait() {
                Ok(Some(status)) => {
                    if status.success() {
                        info!("FFmpeg 正常退出: {:?}", status);
                        return Ok(true);
                    } else {
                        // 退出码 255 视为正常（流结束）
                        if status.code() == Some(255) {
                            info!("FFmpeg 退出(流结束): {:?}", status);
                            return Ok(true);
                        }
                        error!("FFmpeg 异常退出: {:?}", status);
                        return Err(format!("FFmpeg 异常退出: {:?}", status));
                    }
                }
                Ok(None) => {
                    // 进程仍在运行，等待1秒后继续检查
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
                Err(e) => {
                    return Err(format!("检查 FFmpeg 进程状态失败: {}", e));
                }
            }
        }
    }

    /// 请求停止录制
    pub fn request_stop(&mut self) {
        self.should_stop = true;
    }

    /// 优雅停止 FFmpeg 进程
    ///
    /// 发送 'q' 到 FFmpeg stdin，等待最多 15 秒，超时则强制终止。
    /// 用于分段切换和手动停止。
    pub async fn stop_gracefully(&mut self) {
        if let Some(ref mut process) = self.process {
            // 发送 'q' 命令到 FFmpeg stdin
            if let Some(mut stdin) = process.stdin.take() {
                let _ = stdin.write_all(b"q").await;
                let _ = stdin.flush().await;
            }

            // 等待进程退出（最多 15 秒）
            for _ in 0..15 {
                match process.try_wait() {
                    Ok(Some(_)) => {
                        info!("FFmpeg 已优雅退出");
                        return;
                    }
                    _ => {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }

            // 超时强制终止
            warn!("FFmpeg 未响应优雅停止，强制终止");
            let _ = process.kill().await;
        }
    }

    /// 检查进程状态（非阻塞）
    ///
    /// 返回 Ok(true) 表示进程已退出，Ok(false) 表示仍在运行
    pub fn check_status(&mut self) -> Result<bool, String> {
        if let Some(ref mut process) = self.process {
            match process.try_wait() {
                Ok(Some(status)) => {
                    if status.success() || status.code() == Some(255) {
                        info!("FFmpeg 已退出: {:?}", status);
                        Ok(true)
                    } else {
                        Err(format!("FFmpeg 异常退出: {:?}", status))
                    }
                }
                Ok(None) => Ok(false),
                Err(e) => Err(format!("检查进程状态失败: {}", e)),
            }
        } else {
            Ok(true) // 没有进程 = 已退出
        }
    }

    /// 优雅停止 FFmpeg 进程
    ///
    /// - 首先尝试发送 'q' 到 stdin
    /// - 如果 15 秒后仍未退出，发送 SIGTERM/KILL
    async fn graceful_stop(process: &mut Child) {
        // 方式1: 发送 'q' 命令
        if let Some(mut stdin) = process.stdin.take() {
            let _ = stdin.write_all(b"q").await;
            let _ = stdin.flush().await;
        }

        // 等待进程退出
        for _ in 0..15 {
            match process.try_wait() {
                Ok(Some(_)) => {
                    info!("FFmpeg 已优雅退出");
                    return;
                }
                _ => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }

        // 方式2: 强制终止
        warn!("FFmpeg 未响应优雅停止，强制终止");
        let _ = process.kill().await;
    }
}

/// 后处理：格式转换（流复制 remux，无重编码）
///
/// 支持任意录制格式之间的互转，全部使用 stream copy（-c copy）。
/// 特殊处理：
/// - TS → MP4/MOV：添加 `-bsf:a aac_adtstoasc`（ADTS→ASC 音频流过滤）
/// - → MP4：添加 `-movflags +faststart`（moov atom 前置，Web 播放友好）
/// - → MKV：添加 `-f matroska`
/// - → FLV：添加 `-f flv`
/// - → TS：添加 `-f mpegts`
///
/// 源格式 == 目标格式时跳过转换，直接返回原路径。
pub async fn convert_format(
    input: &PathBuf,
    target_format: &OutputFormat,
    delete_original: bool,
) -> Result<PathBuf, String> {
    let input_ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let target_ext = target_format.extension();

    // 源格式 == 目标格式，跳过
    if input_ext == target_ext {
        info!("源格式与目标格式相同({}), 跳过转换", target_ext);
        return Ok(input.clone());
    }

    let output = input.with_extension(target_ext);
    info!("格式转换: {:?} ({}) -> {:?}", input, input_ext, output);

    let mut args: Vec<String> = vec![
        "-i".into(),
        input.to_string_lossy().into(),
        "-c:v".into(), "copy".into(),
        "-c:a".into(), "copy".into(),
        "-y".into(),
    ];

    // TS → MP4/MOV 需要 aac_adtstoasc 比特流过滤器
    if input_ext == "ts" && matches!(target_format, OutputFormat::MP4 | OutputFormat::MOV) {
        args.push("-bsf:a".into());
        args.push("aac_adtstoasc".into());
    }

    // 目标格式特定参数
    match target_format {
        OutputFormat::MP4 => {
            args.push("-movflags".into());
            args.push("+faststart".into());
        }
        OutputFormat::MKV => {
            args.push("-f".into());
            args.push("matroska".into());
        }
        OutputFormat::FLV => {
            args.push("-f".into());
            args.push("flv".into());
        }
        OutputFormat::TS => {
            args.push("-f".into());
            args.push("mpegts".into());
        }
        OutputFormat::MOV => {}
    }

    args.push(output.to_string_lossy().into());

    let status = Command::new("ffmpeg")
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| format!("格式转换失败: {}", e))?;

    if status.success() {
        // 仅在成功且配置允许时删除原文件
        if delete_original {
            match tokio::fs::remove_file(input).await {
                Ok(_) => info!("已删除原文件: {:?}", input),
                Err(e) => warn!("删除原文件失败: {:?} - {}", input, e),
            }
        }
        info!("格式转换完成: {:?}", output);
        Ok(output)
    } else {
        let err = format!("格式转换失败, exit code: {:?}", status.code());
        error!("{}", err);
        Err(err)
    }
}

/// 检查 FFmpeg 是否可用
pub fn check_ffmpeg_available() -> Result<String, String> {
    let output = std::process::Command::new("ffmpeg")
        .args(["-version"])
        .output()
        .map_err(|_| "FFmpeg 未安装或不在 PATH 中".to_string())?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("FFmpeg (version unknown)")
            .to_string();
        Ok(version)
    } else {
        Err("FFmpeg 不可用".to_string())
    }
}

/// 抓取封面帧：从录制输出文件中抽取一帧作为缩略图（best-effort）
///
/// 用于录制中定期生成封面预览。从本地分段文件读取第一帧，不消耗额外网络带宽。
/// TS 等流式容器可正常抽取；MP4 等需 moov 原子的容器在录制中（未 finalize）可能失败，调用方应忽略错误。
pub async fn capture_thumbnail(input: &PathBuf, output: &PathBuf) -> Result<(), String> {
    let args: Vec<String> = vec![
        "-y".into(),
        "-loglevel".into(), "error".into(),
        "-rw_timeout".into(), "15000000".into(),
        "-fflags".into(), "+discardcorrupt".into(),
        "-t".into(), "5".into(),
        "-i".into(), input.to_string_lossy().into(),
        "-frames:v".into(), "1".into(),
        "-q:v".into(), "3".into(),
        output.to_string_lossy().into(),
    ];

    let result = tokio::time::timeout(
        Duration::from_secs(15),
        Command::new("ffmpeg")
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status(),
    )
    .await;

    match result {
        Ok(Ok(status)) if status.success() => Ok(()),
        Ok(Ok(status)) => Err(format!("封面帧抓取失败, exit code: {:?}", status.code())),
        Ok(Err(e)) => Err(format!("封面帧抓取失败: {}", e)),
        Err(_) => Err("封面帧抓取超时".to_string()),
    }
}

/// 运行一条简单的 FFmpeg 命令（无实时进度），成功返回 Ok(())
async fn run_ffmpeg(args: &[String]) -> Result<(), String> {
    let status = Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| format!("FFmpeg 执行失败: {}", e))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("FFmpeg 退出码: {:?}", status.code()))
    }
}

/// 后处理：提取音频为 MP3（使用 libmp3lame 重编码，兼容性最好）
pub async fn extract_audio(input: &PathBuf, output: &PathBuf) -> Result<PathBuf, String> {
    let args: Vec<String> = vec![
        "-y".into(),
        "-i".into(),
        input.to_string_lossy().into(),
        "-vn".into(),
        "-acodec".into(),
        "libmp3lame".into(),
        "-q:a".into(),
        "2".into(),
        output.to_string_lossy().into(),
    ];
    run_ffmpeg(&args)
        .await
        .map(|_| output.clone())
        .map_err(|e| format!("提取音频失败: {}", e))
}

/// 后处理：截取片段（流复制，无重编码）
///
/// `start` 为起始秒；`end` 为结束秒（None 表示截到结尾）。
pub async fn trim(
    input: &PathBuf,
    output: &PathBuf,
    start: f64,
    end: Option<f64>,
) -> Result<PathBuf, String> {
    let mut args: Vec<String> = vec![
        "-y".into(),
        "-ss".into(),
        format!("{:.3}", start),
        "-i".into(),
        input.to_string_lossy().into(),
        "-c".into(),
        "copy".into(),
    ];
    if let Some(end) = end {
        args.push("-to".into());
        args.push(format!("{:.3}", end));
    }
    args.push(output.to_string_lossy().into());
    run_ffmpeg(&args)
        .await
        .map(|_| output.clone())
        .map_err(|e| format!("片段截取失败: {}", e))
}
