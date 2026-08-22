//! 后处理任务运行器 — 格式转换 / 提取音频 / 片段截取
//!
//! 任务存储在 AppState.jobs（内存态）。每个任务在后台 tokio 任务中运行 FFmpeg，
//! 运行期间定期更新进度并广播（服务器模式经 WebSocket 推送，桌面模式由前端轮询）。

use crate::broadcaster::WsBroadcaster;
use crate::config::AppState;
use crate::models::{JobStatus, OutputFormat, PostProcessJob, PostProcessRequest};
use crate::recording::ffmpeg;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// 启动一个后处理任务（沙箱限制在输出目录内）
pub async fn start_postprocess(
    app_state: Arc<AppState>,
    job_broadcaster: Option<Arc<WsBroadcaster>>,
    req: PostProcessRequest,
) -> Result<PostProcessJob, String> {
    let input = PathBuf::from(&req.input);
    if !input.exists() {
        return Err(format!("输入文件不存在: {}", req.input));
    }

    // 沙箱校验：仅允许处理输出目录内的文件
    let root = {
        let settings = app_state.settings.read();
        PathBuf::from(&settings.output_dir)
    };
    let root_abs = root
        .canonicalize()
        .map_err(|e| format!("输出目录无效: {}", e))?;
    let input_abs = input
        .canonicalize()
        .map_err(|e| format!("输入文件无效: {}", e))?;
    if !input_abs.starts_with(&root_abs) {
        return Err("非法路径：仅允许处理输出目录内的文件".into());
    }

    let input_size = input_abs.metadata().map(|m| m.len()).unwrap_or(0);
    let output = compute_output_path(&input_abs, &req)?;
    let job_id = Uuid::new_v4().to_string();

    let job = PostProcessJob {
        id: job_id.clone(),
        kind: req.kind.clone(),
        input: req.input.clone(),
        output: Some(output.to_string_lossy().to_string()),
        status: JobStatus::Pending,
        progress: 0,
        error: None,
        created_at: Utc::now(),
    };

    {
        let mut jobs = app_state.jobs.write();
        jobs.insert(job_id.clone(), job.clone());
    }
    if let Some(b) = &job_broadcaster {
        b.broadcast_job(&job);
    }

    tokio::spawn(run_job(
        app_state,
        job_broadcaster,
        job_id,
        req,
        input_abs,
        output,
        input_size,
    ));

    Ok(job)
}

/// 计算输出文件路径（按操作类型推导扩展名，避免覆盖源文件）
fn compute_output_path(input: &PathBuf, req: &PostProcessRequest) -> Result<PathBuf, String> {
    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".into());
    let parent = input
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let out = match req.kind.as_str() {
        "convert" => {
            let ext = req.target_format.as_deref().unwrap_or("mp4");
            parent.join(format!("{}.{}", stem, ext))
        }
        "extract-audio" => parent.join(format!("{}.mp3", stem)),
        "trim" => {
            let ext = input
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("mp4");
            parent.join(format!("{}_trim.{}", stem, ext))
        }
        other => return Err(format!("不支持的后处理类型: {}", other)),
    };
    Ok(out)
}

fn parse_format(s: &str) -> Result<OutputFormat, String> {
    match s {
        "mp4" => Ok(OutputFormat::MP4),
        "ts" => Ok(OutputFormat::TS),
        "mkv" => Ok(OutputFormat::MKV),
        "flv" => Ok(OutputFormat::FLV),
        "mov" => Ok(OutputFormat::MOV),
        _ => Err(format!("不支持的目标格式: {}", s)),
    }
}

/// 后台运行后处理任务，更新进度并广播
async fn run_job(
    app_state: Arc<AppState>,
    job_broadcaster: Option<Arc<WsBroadcaster>>,
    job_id: String,
    req: PostProcessRequest,
    input: PathBuf,
    output: PathBuf,
    input_size: u64,
) {
    update_job(&app_state, &job_broadcaster, &job_id, |j| {
        j.status = JobStatus::Running;
    });

    // 进度监视：每 1s 读取输出文件大小估算进度（基于与输入文件的大小比）
    let monitor = {
        let app_state = app_state.clone();
        let job_broadcaster = job_broadcaster.clone();
        let job_id = job_id.clone();
        let output = output.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let size = output.metadata().map(|m| m.len()).unwrap_or(0);
                let pct = if input_size > 0 {
                    ((size as f64 / input_size as f64) * 100.0).min(99.0) as u8
                } else {
                    0
                };
                update_job(&app_state, &job_broadcaster, &job_id, |j| {
                    j.progress = pct;
                });
                let done = {
                    let jobs = app_state.jobs.read();
                    jobs.get(&job_id)
                        .map(|j| j.status != JobStatus::Running)
                        .unwrap_or(true)
                };
                if done {
                    break;
                }
            }
        })
    };

    let result = match req.kind.as_str() {
        "convert" => {
            let fmt = match req.target_format.as_deref() {
                Some(f) => match parse_format(f) {
                    Ok(fmt) => fmt,
                    Err(e) => {
                        update_job(&app_state, &job_broadcaster, &job_id, |j| {
                            j.status = JobStatus::Failed;
                            j.error = Some(e);
                        });
                        return;
                    }
                },
                None => OutputFormat::MP4,
            };
            let ffmpeg_bin = ffmpeg::resolve_ffmpeg_bin(&app_state.settings.read().ffmpeg_path);
            ffmpeg::convert_format(&ffmpeg_bin, &input, &fmt, false).await
        }
        "extract-audio" => {
            let ffmpeg_bin = ffmpeg::resolve_ffmpeg_bin(&app_state.settings.read().ffmpeg_path);
            ffmpeg::extract_audio(&ffmpeg_bin, &input, &output).await
        }
        "trim" => {
            let start = req.start_seconds.unwrap_or(0.0);
            let end = req.end_seconds;
            let ffmpeg_bin = ffmpeg::resolve_ffmpeg_bin(&app_state.settings.read().ffmpeg_path);
            ffmpeg::trim(&ffmpeg_bin, &input, &output, start, end).await
        }
        _ => Err(format!("不支持的后处理类型: {}", req.kind)),
    };

    monitor.abort();

    match result {
        Ok(_) => update_job(&app_state, &job_broadcaster, &job_id, |j| {
            j.status = JobStatus::Done;
            j.progress = 100;
        }),
        Err(e) => update_job(&app_state, &job_broadcaster, &job_id, |j| {
            j.status = JobStatus::Failed;
            j.error = Some(e);
        }),
    }
}

/// 更新任务状态并广播（若有广播器）
fn update_job<F>(
    app_state: &AppState,
    job_broadcaster: &Option<Arc<WsBroadcaster>>,
    job_id: &str,
    f: F,
) where
    F: FnOnce(&mut PostProcessJob),
{
    let job = {
        let mut jobs = app_state.jobs.write();
        match jobs.get_mut(job_id) {
            Some(j) => {
                f(j);
                Some(j.clone())
            }
            None => None,
        }
    };
    if let (Some(j), Some(b)) = (job, job_broadcaster) {
        b.broadcast_job(&j);
    }
}
