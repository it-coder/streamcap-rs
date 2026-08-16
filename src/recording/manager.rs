//! 录制管理器 — 核心状态机 + 轮询调度
//!
//! 参考 StreamCap 的 record_manager.py

use crate::broadcaster::{EventBroadcaster, ShutdownPayload};
use crate::config::AppState;
use crate::disk;
use crate::models::{
    HistoryStatus, RecordingConfig, RecordingHistoryEntry, RecordingProgress, RecordingStatus,
    StreamInfo, TimeRange,
};
use crate::notifier::{EVENT_COMPLETED, EVENT_FAILED, EVENT_STARTED};
use crate::recording::ffmpeg::{self, FFmpegRecorder};
use crate::stream::resolver;
use chrono::{Local, Utc};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, watch, Mutex};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};
use uuid::Uuid;

/// 活跃录制任务跟踪
struct ActiveRecording {
    stop_tx: watch::Sender<bool>,
    task_handle: JoinHandle<()>,
}

/// 录制管理器（线程安全共享）
pub struct RecordingManager {
    pub(crate) app_state: Arc<AppState>,
    broadcaster: Arc<dyn EventBroadcaster>,
    active_tasks: Arc<Mutex<HashMap<String, ActiveRecording>>>,
    status_tx: broadcast::Sender<RecordingConfig>,
    poll_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl RecordingManager {
    pub fn new(app_state: Arc<AppState>, broadcaster: Arc<dyn EventBroadcaster>) -> Arc<Self> {
        let (tx, _) = broadcast::channel(100);
        Arc::new(Self {
            app_state,
            broadcaster,
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
        // 推送到前端（Tauri emit 或 WebSocket fan-out）
        self.broadcaster.broadcast_status(&config);
    }

    /// 推送关闭事件到前端
    pub fn broadcast_shutdown(&self, payload: &ShutdownPayload) {
        self.broadcaster.broadcast_shutdown(payload);
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
        let (proxy, quality, segment_duration_secs, threshold_gb, cookies_by_platform, settings_output_dir) = {
            let settings = self.app_state.settings.read();
            (
                settings.proxy_url.clone(),
                config.quality.clone(),
                settings.segment_duration_seconds,
                settings.recording_space_threshold_gb,
                settings.cookies_by_platform.clone(),
                settings.output_dir.clone(),
            )
        };

        info!("开始检测直播: {}", config.url);

        // 磁盘空间预检查（启动前）
        let base_output_dir = config
            .output_dir
            .clone()
            .unwrap_or(settings_output_dir);
        if let Err(e) = disk::check_space(&PathBuf::from(&base_output_dir), threshold_gb) {
            warn!("磁盘空间不足，拒绝启动录制: {}", e);
            {
                let mut recordings = self.app_state.recordings.write();
                if let Some(r) = recordings.iter_mut().find(|r| r.id == recording_id) {
                    r.error_message = Some(e.clone());
                    r.updated_at = Utc::now();
                }
            }
            let _ = self.app_state.save_recordings().await;
            return Err(e);
        }

        // 按平台获取 Cookie
        let (platform_key, _) = resolver::detect_platform(&config.url);
        let cookie = cookies_by_platform
            .get(platform_key)
            .filter(|s| !s.is_empty())
            .cloned();

        // 解析流URL
        let stream_info =
            resolver::resolve_stream(&config.url, &quality, proxy.as_deref(), cookie.as_deref())
                .await?;

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
        self.broadcast(updated_config.clone());
        let _ = self.app_state.save_recordings().await;

        // Webhook：录制开始通知（不阻塞主流程）
        {
            let hook_cfg = updated_config.clone();
            let hook_app = self.app_state.clone();
            tokio::spawn(async move {
                crate::notifier::fire_webhook(hook_app, EVENT_STARTED, &hook_cfg).await;
            });
        }

        // 构建输出路径
        let output_dir = Self::build_output_dir(self.app_state.clone(), &config, &stream_info);
        tokio::fs::create_dir_all(&output_dir)
            .await
            .map_err(|e| format!("创建输出目录失败: {}", e))?;

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
        let broadcaster = self.broadcaster.clone();
        let opts = output_path.clone();
        let schedule = config.schedule.clone();

        // 启动录制任务（支持分段循环 + 失败自动重试）
        let handle = tokio::spawn(async move {
            // 读取重试配置
            let (max_retries, retry_delay_seconds) = {
                let settings = app_state.settings.read();
                (settings.max_retries, settings.retry_delay_seconds)
            };

            // 重试循环：仅在真实 FFmpeg 错误时重试；
            // 主播下播 / 用户停止 / 磁盘满 均通过 run_ffmpeg_recording 的 Ok 分支结束，不会重试
            let mut attempt: u32 = 0;
            let final_result: Result<(), String> = loop {
                let result = run_ffmpeg_recording(
                    &rid,
                    &stream_url,
                    &output_format,
                    &user_agent,
                    proxy_url.as_deref(),
                    &opts,
                    stop_rx.clone(),
                    segment_duration_secs,
                    threshold_gb,
                    schedule.clone(),
                    app_state.clone(),
                    status_tx.clone(),
                    broadcaster.clone(),
                )
                .await;

                match result {
                    Ok(()) => {
                        // 正常结束：清除重试计数
                        {
                            let mut recs = app_state.recordings.write();
                            if let Some(r) = recs.iter_mut().find(|r| r.id == rid) {
                                r.retry_count = 0;
                            }
                        }
                        break Ok(());
                    }
                    Err(e) => {
                        if attempt >= max_retries {
                            error!("录制 {} 最终失败（已重试 {} 次）: {}", rid, attempt, e);
                            break Err(e);
                        }
                        attempt += 1;
                        warn!("录制 {} 失败，{}/{} 次重试: {}", rid, attempt, max_retries, e);
                        // 广播"重试中"状态
                        {
                            let mut recs = app_state.recordings.write();
                            if let Some(r) = recs.iter_mut().find(|r| r.id == rid) {
                                r.retry_count = attempt;
                                r.error_message = None;
                                r.updated_at = Utc::now();
                            }
                        }
                        let cfg = {
                            let recs = app_state.recordings.read();
                            recs.iter().find(|r| r.id == rid).cloned()
                        };
                        let _ = app_state.save_recordings().await;
                        if let Some(c) = cfg {
                            let _ = status_tx.send(c.clone());
                            broadcaster.broadcast_status(&c);
                        }
                        // 可中断退避：等待 retry_delay_seconds * attempt 秒，期间用户停止则立即放弃
                        if !cancellable_sleep(&stop_rx, retry_delay_seconds * attempt as u64).await {
                            info!("退避期间收到停止信号，放弃重试: {}", rid);
                            break Err(format!("录制 {} 已被用户停止", rid));
                        }
                    }
                }
            };

            // 录制结束，更新状态（块作用域确保锁守卫在 .await 前释放）
            let updated = {
                let mut recordings = app_state.recordings.write();
                if let Some(r) = recordings.iter_mut().find(|r| r.id == rid) {
                    r.is_recording = false;
                    r.is_live = false;
                    r.retry_count = 0;
                    r.updated_at = Utc::now();
                    match &final_result {
                        Ok(_) => info!("录制 {} 完成", rid),
                        Err(e) => {
                            error!("录制 {} 失败: {}", rid, e);
                            r.error_message = Some(e.clone());
                        }
                    }
                }
                recordings.iter().find(|r| r.id == rid).cloned()
            };
            let _ = app_state.save_recordings().await;
            if let Some(finished) = updated.clone() {
                let _ = status_tx.send(finished.clone());
                broadcaster.broadcast_status(&finished);

                // 写入录制历史（成功/失败均记录）
                let history_entry = make_history_entry(&finished, &final_result);
                let _ = app_state.add_history_entry(history_entry).await;

                // Webhook：录制完成 / 失败通知
                let hook_app = app_state.clone();
                let hook_event = if final_result.is_ok() {
                    EVENT_COMPLETED
                } else {
                    EVENT_FAILED
                };
                let hook_cfg = finished.clone();
                tokio::spawn(async move {
                    crate::notifier::fire_webhook(hook_app, hook_event, &hook_cfg).await;
                });
            }

            // 定时录制周期重排：成功完成后，若设置了 recurrence，计算下一次开始时间
            if final_result.is_ok() {
                if let Some(fin) = &updated {
                    if let Some(rec) = &fin.recurrence {
                        if *rec != crate::models::Recurrence::Once {
                            let next = match rec {
                                crate::models::Recurrence::Daily => {
                                    Utc::now() + chrono::Duration::days(1)
                                }
                                crate::models::Recurrence::Weekly => {
                                    Utc::now() + chrono::Duration::days(7)
                                }
                                _ => Utc::now(),
                            };
                            {
                                let mut recs = app_state.recordings.write();
                                if let Some(r) = recs.iter_mut().find(|r| r.id == rid) {
                                    r.scheduled_start = Some(next);
                                    r.updated_at = Utc::now();
                                }
                            }
                            let _ = app_state.save_recordings().await;
                        }
                    }
                }
            }
        });

        // 注册活跃任务
        self.active_tasks.lock().await.insert(
            recording_id.to_string(),
            ActiveRecording {
                stop_tx,
                task_handle: handle,
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

    /// 获取当前活跃录制任务数
    pub async fn active_count(&self) -> usize {
        self.active_tasks.lock().await.len()
    }

    /// 关闭应用前优雅停止所有录制并保存状态
    /// 返回被停止的活跃任务数
    pub async fn shutdown_all(&self) -> usize {
        // 1. 先停止轮询循环，避免关闭过程中触发新录制
        if let Some(handle) = self.poll_handle.lock().await.take() {
            handle.abort();
            let _ = handle.await;
            info!("shutdown: 轮询循环已停止");
        }

        // 2. 取出所有活跃任务（drain 并尽早释放锁）
        let mut tasks = self.active_tasks.lock().await;
        let active: Vec<ActiveRecording> = tasks.drain().map(|(_, v)| v).collect();
        let count = active.len();
        drop(tasks);

        if count == 0 {
            let _ = self.app_state.save_recordings().await;
            return 0;
        }

        // 3. 发送停止信号
        for ar in &active {
            let _ = ar.stop_tx.send(true);
        }
        info!("shutdown: 已发送 {} 个停止信号", count);

        // 4. 等待所有录制任务真正完成（总超时 30s，超时由 kill_on_drop 兜底）
        let handles: Vec<JoinHandle<()>> = active.into_iter().map(|ar| ar.task_handle).collect();
        let result = tokio::time::timeout(
            Duration::from_secs(30),
            async {
                for handle in handles {
                    let _ = handle.await;
                }
            },
        )
        .await;

        match result {
            Ok(_) => info!("shutdown: 所有录制任务已优雅退出"),
            Err(_) => warn!("shutdown: 部分录制任务 30s 未完成，将被 kill_on_drop 强杀"),
        }

        let _ = self.app_state.save_recordings().await;
        count
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
            // 磁盘自动清理周期任务：每 5 分钟检查一次（settings.auto_cleanup 开启时生效）
            let cleanup_app = this.app_state.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(300)).await;
                    crate::cleanup::run_cleanup(&cleanup_app).await;
                }
            });

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

                    // 调度时间窗口检查：不在窗口内则跳过检测
                    if !TimeRange::any_contains(&recording.schedule, Local::now()) {
                        info!("{} 不在录制时间窗口内，跳过检测", recording.url);
                        continue;
                    }

                    // 定时开录：尚未到点则跳过检测，保持"已排期"状态（前端据 scheduled_start 展示）
                    if let Some(start) = recording.scheduled_start {
                        if Utc::now() < start {
                            continue;
                        }
                    }

                    // 在独立作用域中读取 settings，确保守卫在 .await 前完全释放
                    let (proxy, quality, cookies_by_platform) = {
                        let settings = this.app_state.settings.read();
                        (
                            settings.proxy_url.clone(),
                            recording.quality.clone(),
                            settings.cookies_by_platform.clone(),
                        )
                    };

                    // 按平台获取 Cookie
                    let (platform_key, _) = resolver::detect_platform(&recording.url);
                    let cookie = cookies_by_platform
            .get(platform_key)
            .filter(|s| !s.is_empty())
            .cloned();

                    match resolver::resolve_stream(
                        &recording.url,
                        &quality,
                        proxy.as_deref(),
                        cookie.as_deref(),
                    )
                    .await
                    {
                        Ok(stream_info) => {
                            info!(
                                "检测到开播: {} [{}]",
                                stream_info.anchor_name, stream_info.platform
                            );

                            // 更新信息并推送事件
                            let cfg_to_broadcast = {
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
                                    Some(r.clone())
                                } else {
                                    None
                                }
                            };
                            if let Some(cfg) = cfg_to_broadcast {
                                this.broadcast(cfg);
                            }

                            // 并发上限：达到上限则暂不启动，等待空位（前端据 active 数展示"排队中"）
                            let max_concurrent = {
                                let s = this.app_state.settings.read();
                                s.max_concurrent_recordings
                            };
                            if max_concurrent > 0 {
                                let active = this.active_count().await;
                                if active >= max_concurrent as usize {
                                    info!(
                                        "并发已达上限({}/{}), 任务 {} 进入排队",
                                        active, max_concurrent, recording.id
                                    );
                                    continue;
                                }
                            }

                            // 并行启动录制，不阻塞后续直播间的检测
                            // start_recording 内部会再次 broadcast
                            let this_clone = this.clone();
                            let rid = recording.id.clone();
                            tokio::spawn(async move {
                                if let Err(e) = this_clone.start_recording(&rid).await {
                                    error!("启动录制失败: {}", e);
                                }
                            });
                        }
                        Err(_) => {
                            info!("未开播......{}", &recording.url);
                            // 未开播，更新状态并推送事件
                            let cfg_to_broadcast = {
                                let mut list = this.app_state.recordings.write();
                                if let Some(r) = list.iter_mut().find(|r| r.id == recording.id) {
                                    r.is_live = false;
                                    r.last_check_at = Some(Utc::now());
                                    Some(r.clone())
                                } else {
                                    None
                                }
                            };
                            if let Some(cfg) = cfg_to_broadcast {
                                this.broadcast(cfg);
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
///
/// 支持分段录制：当 `segment_duration_secs > 0` 时，每隔指定时长自动切换到新文件。
/// 文件名格式：`{原始文件名}_part{N}.{ext}`（第一段无后缀）。
async fn run_ffmpeg_recording(
    recording_id: &str,
    stream_url: &str,
    output_format: &crate::models::OutputFormat,
    user_agent: &str,
    proxy_url: Option<&str>,
    output_path: &PathBuf,
    stop_rx: watch::Receiver<bool>,
    segment_duration_secs: u64,
    threshold_gb: u64,
    schedule: Vec<TimeRange>,
    app_state: Arc<AppState>,
    status_tx: broadcast::Sender<RecordingConfig>,
    broadcaster: Arc<dyn EventBroadcaster>,
) -> Result<(), String> {
    let ext = output_format.extension();
    let stem = output_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "recording".to_string());
    let dir = output_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let recording_start = tokio::time::Instant::now();
    let mut segment_num: u32 = 0;

    loop {
        // 构建当前分段的输出路径
        let segment_path = if segment_num == 0 {
            output_path.clone()
        } else {
            dir.join(format!("{}_part{}.{}", stem, segment_num, ext))
        };

        info!(
            "开始录制分段 {}: {:?}",
            segment_num + 1,
            segment_path
        );

        // 更新 segment_count 并广播
        {
            let mut recordings = app_state.recordings.write();
            if let Some(r) = recordings.iter_mut().find(|r| r.id == recording_id) {
                r.segment_count = segment_num + 1;
                r.updated_at = Utc::now();
            }
        }
        let _ = app_state.save_recordings().await;
        // 广播状态更新
        {
            let recordings = app_state.recordings.read();
            if let Some(cfg) = recordings.iter().find(|r| r.id == recording_id).cloned() {
                let _ = status_tx.send(cfg.clone());
                broadcaster.broadcast_status(&cfg);
            }
        }

        // 启动 FFmpeg
        let mut recorder = FFmpegRecorder::new(recording_id, segment_path.clone());
        recorder
            .start(stream_url, output_format, user_agent, proxy_url)
            .await?;

        // 监控循环：等待停止信号 / 流结束 / 分段超时 / 磁盘满 / 窗口结束
        let segment_start = tokio::time::Instant::now();
        let segment_dur = Duration::from_secs(segment_duration_secs);
        let mut stream_ended = false;
        let mut stop_requested = false;
        let mut disk_full = false;
        let mut last_disk_check = tokio::time::Instant::now();
        let mut last_progress_time = tokio::time::Instant::now();
        let mut last_size: u64 = 0;
        let mut last_thumb_time = tokio::time::Instant::now();

        loop {
            // 检查停止信号
            if *stop_rx.borrow() {
                info!("收到停止信号，优雅停止 FFmpeg...");
                recorder.stop_gracefully().await;
                stop_requested = true;
                break;
            }

            // 检查分段超时
            if segment_duration_secs > 0 && segment_start.elapsed() >= segment_dur {
                info!(
                    "分段时长到达 ({}秒)，切换到新文件...",
                    segment_duration_secs
                );
                recorder.stop_gracefully().await;
                break;
            }

            // 磁盘空间检查（每 30 秒）
            if threshold_gb > 0 && last_disk_check.elapsed() >= Duration::from_secs(30) {
                last_disk_check = tokio::time::Instant::now();
                if let Err(e) = disk::check_space(&dir, threshold_gb) {
                    warn!("{} — 停止录制", e);
                    recorder.stop_gracefully().await;
                    disk_full = true;
                    {
                        let mut recordings = app_state.recordings.write();
                        if let Some(r) = recordings.iter_mut().find(|r| r.id == recording_id) {
                            r.error_message = Some(e);
                            r.updated_at = Utc::now();
                        }
                    }
                    break;
                }
            }

            // 调度时间窗口检查：窗口结束则停止录制
            if !TimeRange::any_contains(&schedule, Local::now()) {
                info!("不在录制时间窗口内，停止录制");
                recorder.stop_gracefully().await;
                stop_requested = true;
                break;
            }

            // 录制进度推送（每 5 秒）
            if last_progress_time.elapsed() >= Duration::from_secs(5) {
                let now = tokio::time::Instant::now();
                let size = tokio::fs::metadata(&segment_path)
                    .await
                    .map(|m| m.len())
                    .unwrap_or(0);
                let dt = now.duration_since(last_progress_time).as_secs_f64();
                let speed_kbps = if dt > 0.0 && size >= last_size {
                    ((size - last_size) as f64 / dt) / 1024.0
                } else {
                    0.0
                };
                last_size = size;
                last_progress_time = now;

                let progress = RecordingProgress {
                    recording_id: recording_id.to_string(),
                    status: RecordingStatus::Recording,
                    duration_seconds: recording_start.elapsed().as_secs(),
                    file_size_bytes: size,
                    download_speed_kbps: speed_kbps,
                };
                broadcaster.broadcast_progress(&progress);
            }

            // 封面帧快照（每 60 秒）：抓取当前分段首帧作为预览，失败忽略
            if last_thumb_time.elapsed() >= Duration::from_secs(60) {
                last_thumb_time = tokio::time::Instant::now();
                let thumb_path = dir.join(format!("{}_thumb.jpg", recording_id));
                if ffmpeg::capture_thumbnail(&segment_path, &thumb_path).await.is_ok() {
                    let updated = {
                        let mut recordings = app_state.recordings.write();
                        if let Some(r) = recordings.iter_mut().find(|r| r.id == recording_id) {
                            r.thumbnail = Some(thumb_path.to_string_lossy().to_string());
                            r.updated_at = Utc::now();
                            Some(r.clone())
                        } else {
                            None
                        }
                    };
                    if let Some(cfg) = updated {
                        let _ = app_state.save_recordings().await;
                        let _ = status_tx.send(cfg.clone());
                        broadcaster.broadcast_status(&cfg);
                    }
                }
            }

            // 检查 FFmpeg 进程状态
            match recorder.check_status() {
                Ok(true) => {
                    info!("FFmpeg 进程已退出（流结束）");
                    stream_ended = true;
                    break;
                }
                Ok(false) => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // 后处理：格式转换（可配置）
        let (enable_conversion, conversion_format, delete_original) = {
            let settings = app_state.settings.read();
            (
                settings.enable_conversion,
                settings.conversion_format.clone(),
                settings.delete_original_after_conversion,
            )
        };

        if enable_conversion {
            match ffmpeg::convert_format(&segment_path, &conversion_format, delete_original)
                .await
            {
                Ok(converted_path) => {
                    info!("格式转换完成: {:?}", converted_path);
                }
                Err(e) => {
                    error!("格式转换失败: {}", e);
                }
            }
        }

        // 判断是否继续下一段
        if stop_requested || stream_ended || disk_full {
            break;
        }

        segment_num += 1;
        info!("切换到分段 {}", segment_num + 1);
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

/// 可中断的退避等待：每 1 秒检查一次停止信号，被停止则返回 false
async fn cancellable_sleep(stop_rx: &tokio::sync::watch::Receiver<bool>, total_secs: u64) -> bool {
    let mut slept = 0u64;
    while slept < total_secs {
        if *stop_rx.borrow() {
            return false;
        }
        let step = std::cmp::min(1u64, total_secs - slept);
        tokio::time::sleep(Duration::from_secs(step)).await;
        slept += step;
    }
    true
}

/// 扫描录制输出目录，返回（代表媒体文件, 总大小）
///
/// 代表文件取体积最大的媒体文件（用于回放/下载）；总大小为目录内所有文件之和。
fn scan_recording_output(dir: &Option<String>) -> (Option<String>, u64) {
    let dir = match dir {
        Some(d) => PathBuf::from(d),
        None => return (None, 0),
    };
    if !dir.is_dir() {
        return (None, 0);
    }
    let media_exts = [
        "mp4", "ts", "mkv", "flv", "mov", "webm", "avi", "m4a", "mp3",
    ];
    let mut total: u64 = 0;
    let mut largest: Option<(u64, PathBuf)> = None;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.is_file() {
                let size = meta.len();
                total += size;
                let ext = entry
                    .path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if media_exts.contains(&ext.as_str()) {
                    let bigger = largest.as_ref().map(|(s, _)| size > *s).unwrap_or(true);
                    if bigger {
                        largest = Some((size, entry.path()));
                    }
                }
            }
        }
    }
    (
        largest.map(|(_, p)| p.to_string_lossy().to_string()),
        total,
    )
}

/// 根据录制配置与最终结果构造一条历史记录
fn make_history_entry(
    config: &RecordingConfig,
    final_result: &Result<(), String>,
) -> RecordingHistoryEntry {
    let now = Utc::now();
    let started_at = config.recording_started_at.unwrap_or(now);
    let duration = (now - started_at).num_seconds().max(0) as u64;
    let (file_path, file_size) = scan_recording_output(&config.recording_dir);
    let status = match final_result {
        Ok(_) => HistoryStatus::Completed,
        Err(_) => HistoryStatus::Failed,
    };
    RecordingHistoryEntry {
        id: Uuid::new_v4().to_string(),
        recording_id: config.id.clone(),
        url: config.url.clone(),
        platform: config.platform.clone(),
        anchor_name: config.anchor_name.clone(),
        title: config.title.clone(),
        status,
        started_at,
        ended_at: now,
        duration_seconds: duration,
        file_path,
        file_size,
        recording_dir: config.recording_dir.clone(),
        thumbnail: config.thumbnail.clone(),
        error_message: match final_result {
            Ok(_) => None,
            Err(e) => config
                .error_message
                .clone()
                .or_else(|| Some(e.clone())),
        },
        created_at: now,
    }
}
