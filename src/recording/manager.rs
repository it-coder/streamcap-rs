//! 录制管理器 — 核心状态机 + 轮询调度
//!
//! 参考 StreamCap 的 record_manager.py

use crate::config::AppState;
use crate::models::{RecordingConfig, StreamInfo};
use crate::recording::ffmpeg::{self, FFmpegRecorder};
use crate::stream::resolver;
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{broadcast, watch, Mutex};
use tokio::task::JoinHandle;
use tracing::{error, info};

/// 活跃录制任务跟踪
struct ActiveRecording {
    stop_tx: watch::Sender<bool>,
    task_handle: JoinHandle<()>,
    started_at: chrono::DateTime<Utc>,
}

/// 录制管理器（线程安全共享）
pub struct RecordingManager {
    app_state: Arc<AppState>,
    app_handle: AppHandle,
    active_tasks: Arc<Mutex<HashMap<String, ActiveRecording>>>,
    status_tx: broadcast::Sender<RecordingConfig>,
    poll_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl RecordingManager {
    pub fn new(app_state: Arc<AppState>, app_handle: AppHandle) -> Arc<Self> {
        let (tx, _) = broadcast::channel(100);
        Arc::new(Self {
            app_state,
            app_handle,
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            status_tx: tx,
            poll_handle: Arc::new(Mutex::new(None)),
        })
    }

    /// 获取状态广播接收器
    pub fn status_receiver(&self) -> broadcast::Receiver<RecordingConfig> {
        self.status_tx.subscribe()
    }

    /// 广播状态变更：同时推送到 Rust 内部 broadcast channel 和前端事件
    fn broadcast(&self, config: RecordingConfig) {
        // Rust 内部广播
        let _ = self.status_tx.send(config.clone());
        // 推送到前端（React listen 订阅）
        let _ = self.app_handle.emit("recording_status", config);
    }

    /// ========================================
    /// 录制启动
    /// ========================================

    pub async fn start_recording(self: &Arc<Self>, recording_id: &str) -> Result<(), String> {
        // 读取配置
        let config = {
            let recordings = self.app_state.recordings.read();
            recordings
                .iter()
                .find(|r| r.id == recording_id)
                .cloned()
                .ok_or_else(|| format!("录制任务 {} 不存在", recording_id))?
        };

        // 在独立作用域中读取 settings，确保守卫在 .await 前完全释放
        let (proxy, quality) = {
            let settings = self.app_state.settings.read();
            (settings.proxy_url.clone(), config.quality.clone())
        };

        info!("开始检测直播: {}", config.url);

        // 解析流URL
        let stream_info =
            resolver::resolve_stream(&config.url, &quality, proxy.as_deref(), None).await?;

        info!(
            "获取流成功: {} [{}] | {}",
            stream_info.anchor_name, stream_info.platform, stream_info.title
        );

        // 更新录制信息
        {
            let mut recordings = self.app_state.recordings.write();
            if let Some(r) = recordings.iter_mut().find(|r| r.id == recording_id) {
                r.is_live = true;
                r.is_recording = true;
                r.anchor_name = stream_info.anchor_name.clone();
                r.title = stream_info.title.clone();
                if r.platform.is_empty() {
                    r.platform = stream_info.platform.clone();
                }
                r.recording_started_at = Some(Utc::now());
                r.updated_at = Utc::now();
            }
        }

        let updated_config = {
            let recordings = self.app_state.recordings.read();
            recordings
                .iter()
                .find(|r| r.id == recording_id)
                .cloned()
                .unwrap()
        };
        self.broadcast(updated_config);

        // 构建输出路径
        let output_dir = Self::build_output_dir(self.app_state.clone(), &config, &stream_info);
        std::fs::create_dir_all(&output_dir).map_err(|e| format!("创建输出目录失败: {}", e))?;

        let filename = Self::build_filename(&config, &stream_info);
        let ext = config.output_format.extension();
        let output_path = output_dir.join(format!("{}.{}", filename, ext));

        // 停止信号
        let (stop_tx, stop_rx) = watch::channel(false);

        // 克隆需要传入异步任务的变量
        let stream_url = stream_info.stream_url.clone();
        let output_format = config.output_format.clone();
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/141.0.0.0 Safari/537.36".to_string();
        let proxy_url = proxy;
        let rid = recording_id.to_string();
        let app_state = self.app_state.clone();
        let status_tx = self.status_tx.clone();
        let app_handle = self.app_handle.clone();
        let opts = output_path.clone();

        // 启动录制任务
        let handle = tokio::spawn(async move {
            let result = run_ffmpeg_recording(
                &rid,
                &stream_url,
                &output_format,
                &user_agent,
                proxy_url.as_deref(),
                &opts,
                stop_rx,
            )
            .await;

            // 录制结束，更新状态
            let mut recordings = app_state.recordings.write();
            if let Some(r) = recordings.iter_mut().find(|r| r.id == rid) {
                r.is_recording = false;
                r.is_live = false;
                r.updated_at = Utc::now();
                match &result {
                    Ok(_) => info!("录制 {} 完成", rid),
                    Err(e) => {
                        error!("录制 {} 失败: {}", rid, e);
                        r.error_message = Some(e.clone());
                    }
                }
            }
            let updated = recordings.iter().find(|r| r.id == rid).cloned();
            drop(recordings);
            if let Some(cfg) = updated {
                let _ = status_tx.send(cfg.clone());
                let _ = app_handle.emit("recording_status", cfg);
            }
        });

        // 注册活跃任务
        self.active_tasks.lock().await.insert(
            recording_id.to_string(),
            ActiveRecording {
                stop_tx,
                task_handle: handle,
                started_at: Utc::now(),
            },
        );
        info!("直播中的: {:?}", recording_id.to_string());
        {
            let mut recordings = self.app_state.recordings.write();
            if let Some(r) = recordings.iter_mut().find(|r| r.id == recording_id) {
                r.recording_dir = Some(output_dir.to_string_lossy().to_string());
                r.updated_at = Utc::now();
            }
        }

        Ok(())
    }

    /// ========================================
    /// 录制停止
    /// ========================================

    pub async fn stop_recording(&self, recording_id: &str) -> Result<(), String> {
        let mut tasks = self.active_tasks.lock().await;
        if let Some(active) = tasks.remove(recording_id) {
            // 发送停止信号
            let _ = active.stop_tx.send(true);
            info!("已发送停止信号: {}", recording_id);
            Ok(())
        } else {
            Err(format!("录制任务 {} 未在运行", recording_id))
        }
    }

    /// ========================================
    /// 轮询检测
    /// ========================================

    pub async fn start_polling(self: Arc<Self>) {
        let mut poll_handle = self.poll_handle.lock().await;
        if poll_handle.is_some() {
            return;
        }

        let this = self.clone();
        *poll_handle = Some(tokio::spawn(async move {
            info!("直播检测轮询已启动");
            loop {
                info!("直播检测轮询: {:?}", chrono::Local::now());
                let interval = {
                    let settings = this.app_state.settings.read();
                    settings.loop_interval_seconds
                };

                let recordings: Vec<RecordingConfig> = {
                    let list = this.app_state.recordings.read();
                    list.iter()
                        .filter(|r| r.monitor_enabled && !r.is_recording)
                        .cloned()
                        .collect()
                };

                for recording in &recordings {
                    info!("检查记录：{:?}", recording);
                    // 在独立作用域中读取 settings，确保守卫在 .await 前完全释放
                    let (proxy, quality) = {
                        let settings = this.app_state.settings.read();
                        (settings.proxy_url.clone(), recording.quality.clone())
                    };

                    match resolver::resolve_stream(&recording.url, &quality, proxy.as_deref(), None)
                        .await
                    {
                        Ok(stream_info) => {
                            info!(
                                "检测到开播: {} [{}]",
                                stream_info.anchor_name, stream_info.platform
                            );

                            // 更新信息并推送事件
                            {
                                let mut list = this.app_state.recordings.write();
                                if let Some(r) = list.iter_mut().find(|r| r.id == recording.id) {
                                    r.is_live = true;
                                    r.anchor_name = stream_info.anchor_name.clone();
                                    r.title = stream_info.title.clone();
                                    if r.platform.is_empty() {
                                        r.platform = stream_info.platform.clone();
                                    }
                                    r.last_check_at = Some(Utc::now());
                                    r.updated_at = Utc::now();
                                    // 推送状态更新到前端
                                    let _ = this.app_handle.emit("recording_status", r.clone());
                                }
                            }

                            // 启动录制（start_recording 内部会再次 broadcast）
                            if let Err(e) = this.start_recording(&recording.id).await {
                                error!("启动录制失败: {}", e);
                            }
                        }
                        Err(_) => {
                            info!("未开播......{}", &recording.url);
                            // 未开播，更新状态并推送事件
                            let mut list = this.app_state.recordings.write();
                            if let Some(r) = list.iter_mut().find(|r| r.id == recording.id) {
                                r.is_live = false;
                                r.last_check_at = Some(Utc::now());
                                let _ = this.app_handle.emit("recording_status", r.clone());
                            }
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
            }
        }));
    }

    /// ========================================
    /// 路径构建
    /// ========================================

    fn build_output_dir(
        app_state: Arc<AppState>,
        config: &RecordingConfig,
        info: &StreamInfo,
    ) -> PathBuf {
        let settings = app_state.settings.read();
        let base = if let Some(ref dir) = config.output_dir {
            PathBuf::from(dir)
        } else {
            PathBuf::from(&settings.output_dir)
        };

        let mut path = base;
        if settings.folder_by_platform && !info.platform.is_empty() {
            path = path.join(&info.platform);
        }
        if settings.folder_by_anchor && !info.anchor_name.is_empty() {
            path = path.join(sanitize(&info.anchor_name));
        }
        if settings.folder_by_date {
            path = path.join(chrono::Local::now().format("%Y-%m-%d").to_string());
        }
        if settings.folder_by_title && !info.title.is_empty() {
            path = path.join(sanitize(&info.title));
        }
        path
    }

    fn build_filename(_config: &RecordingConfig, info: &StreamInfo) -> String {
        let anchor = if info.anchor_name.is_empty() {
            "unknown"
        } else {
            &info.anchor_name
        };
        let title = if info.title.is_empty() {
            "live".to_string()
        } else {
            sanitize(&info.title)
        };
        let time = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        format!("{}_{}_{}", anchor, title, time)
    }
}

/// 运行 FFmpeg 录制循环（在 tokio 任务中执行）
async fn run_ffmpeg_recording(
    recording_id: &str,
    stream_url: &str,
    output_format: &crate::models::OutputFormat,
    user_agent: &str,
    proxy_url: Option<&str>,
    output_path: &PathBuf,
    stop_rx: watch::Receiver<bool>,
) -> Result<(), String> {
    let mut recorder = FFmpegRecorder::new(recording_id, output_path.clone());

    recorder
        .start(stream_url, output_format, user_agent, proxy_url)
        .await?;

    loop {
        if *stop_rx.borrow() {
            info!("收到停止信号，优雅停止 FFmpeg...");
            recorder.request_stop();
            // 给 recorder 一些时间完成
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            break;
        }

        match recorder.check_status() {
            Ok(true) => {
                // 正常结束
                break;
            }
            Ok(false) => {
                // 继续等待
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    // TS → MP4 后处理
    if *output_format == crate::models::OutputFormat::TS {
        if let Ok(mp4_path) = ffmpeg::convert_ts_to_mp4(output_path).await {
            info!("TS->MP4 完成: {:?}", mp4_path);
        }
    }

    Ok(())
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .take(50)
        .collect()
}
