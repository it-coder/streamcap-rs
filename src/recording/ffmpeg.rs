//! FFmpeg 子进程管理器 — 启动、监控、优雅停止 FFmpeg 录制进程

use crate::models::OutputFormat;
use std::path::PathBuf;
use std::process::Stdio;
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

        let child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("启动 FFmpeg 失败: {} (请确认 FFmpeg 已安装并在 PATH 中)", e))?;

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

/// 后处理：TS转MP4（流复制，无转码开销）
pub async fn convert_ts_to_mp4(input: &PathBuf) -> Result<PathBuf, String> {
    let output = input.with_extension("mp4");

    info!("TS -> MP4 转换: {:?} -> {:?}", input, output);

    let status = Command::new("ffmpeg")
        .args([
            "-i",
            &input.to_string_lossy(),
            "-c:v", "copy",
            "-c:a", "copy",
            "-f", "mp4",
            "-y",
            &output.to_string_lossy(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| format!("TS转MP4失败: {}", e))?;

    if status.success() {
        // 删除原TS文件
        let _ = std::fs::remove_file(input);
        Ok(output)
    } else {
        Err(format!("TS转MP4失败, exit code: {:?}", status.code()))
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
