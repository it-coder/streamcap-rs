//! HTTP 请求处理器 — REST API 端点实现
//!
//! 与 Tauri commands 一一对应，委托给共享的 AppState / RecordingManager。

use std::io::SeekFrom;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use crate::disk;
use crate::jobs;
use crate::models::{
    AppSettings, FileEntry, PostProcessJob, PostProcessRequest, RecordingConfig,
    RecordingHistoryEntry,
};
use crate::recording::ffmpeg;
use crate::server::ServerState;
use crate::stream::resolver;
use chrono::Utc;
use uuid::Uuid;

/// 统一错误响应
pub struct ApiError(pub String);

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": self.0 })),
        )
            .into_response()
    }
}

// ========================================
// 录制任务 API
// ========================================

/// GET /api/recordings — 获取所有录制任务
pub async fn list_recordings(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let recordings = state.app_state.recordings.read();
    Json(recordings.clone())
}

/// POST /api/recordings — 添加录制任务
#[derive(Deserialize)]
pub struct AddRecordingRequest {
    pub url: String,
    pub monitor_enabled: Option<bool>,
    pub quality: Option<String>,
}

pub async fn add_recording(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<AddRecordingRequest>,
) -> Result<Json<RecordingConfig>, ApiError> {
    let id = Uuid::new_v4().to_string();
    let (platform_key, _platform_name) = resolver::detect_platform(&req.url);
    let quality = req.quality.unwrap_or_else(|| "OD".to_string());

    let config = RecordingConfig {
        id,
        url: req.url,
        platform: platform_key.to_string(),
        anchor_name: String::new(),
        title: String::new(),
        monitor_enabled: req.monitor_enabled.unwrap_or(true),
        is_recording: false,
        is_live: false,
        quality: match quality.as_str() {
            "UHD" => crate::models::VideoQuality::UltraHD,
            "HD" => crate::models::VideoQuality::HD,
            "SD" => crate::models::VideoQuality::SD,
            "LD" => crate::models::VideoQuality::LD,
            _ => crate::models::VideoQuality::Original,
        },
        output_format: crate::models::OutputFormat::TS,
        output_dir: None,
        recording_dir: None,
        schedule: Vec::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_check_at: None,
        recording_started_at: None,
        error_message: None,
        segment_count: 0,
        retry_count: 0,
        thumbnail: None,
        scheduled_start: None,
        recurrence: None,
    };

    // 块作用域确保锁守卫在 .await 前释放（否则 future 非 Send）
    {
        let mut recordings = state.app_state.recordings.write();
        recordings.push(config.clone());
    }

    state
        .app_state
        .save_recordings()
        .await
        .map_err(|e| ApiError(format!("保存失败: {}", e)))?;

    Ok(Json(config))
}

/// DELETE /api/recordings/:id — 删除录制任务
pub async fn remove_recording(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    {
        let mut recordings = state.app_state.recordings.write();
        recordings.retain(|r| r.id != id);
    }

    state
        .app_state
        .save_recordings()
        .await
        .map_err(|e| ApiError(format!("保存失败: {}", e)))?;

    Ok(Json(json!({ "success": true })))
}

/// PUT /api/recordings/:id — 更新录制任务
pub async fn update_recording(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(config): Json<RecordingConfig>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut found = false;
    {
        let mut recordings = state.app_state.recordings.write();
        if let Some(existing) = recordings.iter_mut().find(|r| r.id == id) {
            found = true;
            crate::models::merge_recording_update(existing, config)
                .map_err(ApiError)?;
        }
    }
    if !found {
        return Err(ApiError(format!("录制任务 {} 不存在", id)));
    }

    state
        .app_state
        .save_recordings()
        .await
        .map_err(|e| ApiError(format!("保存失败: {}", e)))?;

    Ok(Json(json!({ "success": true })))
}

/// POST /api/cleanup — 立即执行一次磁盘自动清理，返回删除条数
pub async fn cleanup(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let deleted = crate::cleanup::run_cleanup(&state.app_state).await;
    Ok(Json(json!({ "deleted": deleted })))
}

/// GET /api/recordings/:id/status — 获取单个录制状态
pub async fn get_recording_status(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let recordings = state.app_state.recordings.read();
    match recordings.iter().find(|r| r.id == id) {
        Some(config) => Ok(Json(config.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("录制任务 {} 不存在", id) })),
        )),
    }
}

/// POST /api/recordings/:id/monitor — 启用监控
pub async fn start_monitor(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    {
        let mut recordings = state.app_state.recordings.write();
        if let Some(r) = recordings.iter_mut().find(|r| r.id == id) {
            r.monitor_enabled = true;
            r.updated_at = Utc::now();
        }
    }
    state
        .app_state
        .save_recordings()
        .await
        .map_err(|e| ApiError(format!("保存失败: {}", e)))?;

    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/recordings/:id/monitor — 停止监控
pub async fn stop_monitor(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    {
        let mut recordings = state.app_state.recordings.write();
        if let Some(r) = recordings.iter_mut().find(|r| r.id == id) {
            r.monitor_enabled = false;
            r.updated_at = Utc::now();
        }
    }
    state
        .app_state
        .save_recordings()
        .await
        .map_err(|e| ApiError(format!("保存失败: {}", e)))?;

    Ok(Json(json!({ "success": true })))
}

/// POST /api/recordings/:id/recording — 手动开始录制
pub async fn start_recording(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state
        .recording_manager
        .start_recording(&id)
        .await
        .map_err(ApiError)?;
    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/recordings/:id/recording — 手动停止录制
pub async fn stop_recording(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state
        .recording_manager
        .stop_recording(&id)
        .await
        .map_err(ApiError)?;
    Ok(Json(json!({ "success": true })))
}

// ========================================
// 设置 API
// ========================================

/// GET /api/settings — 获取设置
pub async fn get_settings(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let settings = state.app_state.settings.read();
    Json(settings.clone())
}

/// PUT /api/settings — 更新设置
pub async fn update_settings(
    State(state): State<Arc<ServerState>>,
    Json(settings): Json<AppSettings>,
) -> Result<Json<serde_json::Value>, ApiError> {
    settings
        .validate()
        .map_err(|e| ApiError(format!("设置校验失败: {e}")))?;
    *state.app_state.settings.write() = settings;
    state
        .app_state
        .save_settings()
        .await
        .map_err(|e| ApiError(format!("保存失败: {}", e)))?;
    Ok(Json(json!({ "success": true })))
}

/// GET /api/ffmpeg/check — 检查 FFmpeg
pub async fn check_ffmpeg() -> Result<Json<serde_json::Value>, ApiError> {
    match ffmpeg::check_ffmpeg_available() {
        Ok(version) => Ok(Json(json!({ "available": true, "version": version }))),
        Err(e) => Ok(Json(json!({ "available": false, "error": e }))),
    }
}

/// GET /api/version — 获取应用版本号（单一可信源 = Cargo.toml version）
pub async fn get_version() -> impl IntoResponse {
    Json(json!({ "version": env!("CARGO_PKG_VERSION") }))
}

/// GET /api/health — 健康检查（供 Docker/k8s 探针，无需鉴权）
///
/// 返回服务状态、版本、活跃录制数、磁盘剩余空间(GiB)与运行时长。
pub async fn health(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let active_recordings = state.recording_manager.active_count().await;
    let uptime_seconds = state.started_at.elapsed().as_secs();
    // 在独立作用域读取 settings，确保锁守卫不跨 .await
    let disk_free_gb = {
        let settings = state.app_state.settings.read();
        disk::available_gb(std::path::Path::new(&settings.output_dir)).unwrap_or(0.0)
    };

    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "active_recordings": active_recordings,
        "disk_free_gb": disk_free_gb,
        "uptime_seconds": uptime_seconds,
    }))
}

// ========================================
// 录制历史 API
// ========================================

/// GET /api/history — 获取录制历史（最新在前）
pub async fn list_history(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let history = state.app_state.history.read();
    Json(history.clone())
}

/// GET /api/history/:id — 获取单条历史记录
pub async fn get_history_entry(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Result<Json<RecordingHistoryEntry>, ApiError> {
    let history = state.app_state.history.read();
    match history.iter().find(|h| h.id == id) {
        Some(entry) => Ok(Json(entry.clone())),
        None => Err(ApiError(format!("历史记录 {} 不存在", id))),
    }
}

/// DELETE /api/history/:id — 删除历史记录（?delete_file=true 同时删除关联文件）
#[derive(Deserialize)]
pub struct DeleteHistoryQuery {
    pub delete_file: Option<bool>,
}

pub async fn delete_history_entry(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Query(query): Query<DeleteHistoryQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state
        .app_state
        .remove_history_entry(&id, query.delete_file.unwrap_or(false))
        .await
        .map_err(|e| ApiError(format!("删除失败: {}", e)))?;
    Ok(Json(json!({ "success": true })))
}

// ========================================
// 后处理任务 API
// ========================================

/// POST /api/postprocess — 启动一个后处理任务
pub async fn start_postprocess(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<PostProcessRequest>,
) -> Result<Json<PostProcessJob>, ApiError> {
    let job = jobs::start_postprocess(
        state.app_state.clone(),
        Some(state.ws_broadcaster.clone()),
        req,
    )
    .await
    .map_err(ApiError)?;
    Ok(Json(job))
}

/// GET /api/postprocess/:id — 查询后处理任务状态
pub async fn get_postprocess(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Result<Json<PostProcessJob>, ApiError> {
    let jobs = state.app_state.jobs.read();
    match jobs.get(&id) {
        Some(job) => Ok(Json(job.clone())),
        None => Err(ApiError(format!("后处理任务 {} 不存在", id))),
    }
}

/// 文件列表查询参数
#[derive(Deserialize)]
pub struct FileListQuery {
    pub dir: Option<String>,
}

/// 文件下载查询参数
#[derive(Deserialize)]
pub struct FileDownloadQuery {
    pub path: String,
}

/// 将路径规范化为绝对规范路径
fn canonicalize_path(path: &PathBuf) -> Result<PathBuf, ApiError> {
    path.canonicalize()
        .map_err(|e| ApiError(format!("路径无效 {}: {}", path.display(), e)))
}

/// RFC 3986 非保留字符之外的字节做百分号编码（用于 Content-Disposition 文件名）
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// GET /api/files?dir=<path> — 列出目录内容（沙箱限制在 output_dir 内，防目录遍历）
pub async fn list_files(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<FileListQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let root = {
        let settings = state.app_state.settings.read();
        PathBuf::from(&settings.output_dir)
    };
    let root_abs = canonicalize_path(&root)?;

    let dir = match &query.dir {
        Some(d) => PathBuf::from(d),
        None => root.clone(),
    };
    let dir_abs = canonicalize_path(&dir)?;

    // 沙箱校验：仅允许访问 output_dir 子树
    if !dir_abs.starts_with(&root_abs) {
        return Err(ApiError("越权访问：仅允许浏览输出目录内的文件".into()));
    }

    let mut entries: Vec<FileEntry> = Vec::new();
    let mut reader = tokio::fs::read_dir(&dir_abs)
        .await
        .map_err(|e| ApiError(format!("读取目录失败: {}", e)))?;

    while let Some(entry) = reader
        .next_entry()
        .await
        .map_err(|e| ApiError(format!("读取目录失败: {}", e)))?
    {
        let meta = entry
            .metadata()
            .await
            .map_err(|e| ApiError(format!("读取文件信息失败: {}", e)))?;
        let is_dir = meta.is_dir();
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let size = if is_dir { 0 } else { meta.len() };
        let modified = meta.modified().ok().map(|t| {
            chrono::DateTime::<chrono::Local>::from(t).to_rfc3339()
        });
        entries.push(FileEntry {
            name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            size,
            modified,
        });
    }

    // 排序：目录在前，再按名称
    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            return b.is_dir.cmp(&a.is_dir);
        }
        a.name.cmp(&b.name)
    });

    // 父目录（仅当仍在沙箱内）
    let parent = dir_abs
        .parent()
        .filter(|p| p.starts_with(&root_abs))
        .map(|p| p.to_string_lossy().to_string());

    Ok(Json(json!({
        "dir": dir_abs.to_string_lossy(),
        "parent": parent,
        "entries": entries,
    })))
}

/// GET /api/files/download?path=<path> — 下载文件（流式，沙箱限制在 output_dir 内）
pub async fn download_file(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<FileDownloadQuery>,
) -> Result<Response, ApiError> {
    let root = {
        let settings = state.app_state.settings.read();
        PathBuf::from(&settings.output_dir)
    };
    let root_abs = canonicalize_path(&root)?;
    let path_abs = canonicalize_path(&PathBuf::from(&query.path))?;

    if path_abs.is_dir() || !path_abs.starts_with(&root_abs) {
        return Err(ApiError("非法下载路径：仅允许下载输出目录内的文件".into()));
    }

    let file = tokio::fs::File::open(&path_abs)
        .await
        .map_err(|e| ApiError(format!("打开文件失败: {}", e)))?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let filename = path_abs
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "download".to_string());
    let encoded = percent_encode(&filename);

    Response::builder()
        .header("Content-Type", "application/octet-stream")
        .header(
            "Content-Disposition",
            format!("attachment; filename*=UTF-8''{}", encoded),
        )
        .body(body)
        .map_err(|e| ApiError(format!("构建响应失败: {}", e)))
}

/// GET /api/files/raw?path=<path> — 内联返回文件（供 <img>/<video> 预览，沙箱限制在 output_dir 内）
///
/// 支持 HTTP Range 请求（206 Partial Content），使 <video> 可拖动进度条。
pub async fn serve_file_inline(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    Query(query): Query<FileDownloadQuery>,
) -> Result<Response, ApiError> {
    let root = {
        let settings = state.app_state.settings.read();
        PathBuf::from(&settings.output_dir)
    };
    let root_abs = canonicalize_path(&root)?;
    let path_abs = canonicalize_path(&PathBuf::from(&query.path))?;

    if path_abs.is_dir() || !path_abs.starts_with(&root_abs) {
        return Err(ApiError("非法路径：仅允许访问输出目录内的文件".into()));
    }

    let content_type = match path_abs
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ts" => "video/mp2t",
        "mkv" => "video/x-matroska",
        "mov" => "video/quicktime",
        "flv" => "video/x-flv",
        _ => "application/octet-stream",
    };

    let mut file = tokio::fs::File::open(&path_abs)
        .await
        .map_err(|e| ApiError(format!("打开文件失败: {}", e)))?;
    let total = file
        .metadata()
        .await
        .map_err(|e| ApiError(format!("读取文件信息失败: {}", e)))?
        .len();

    // 解析 Range 请求头
    let range_hdr = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| parse_range(s, total));

    if let Some((start, end)) = range_hdr {
        if start > end || start >= total {
            return Err(ApiError("Range 请求不合法".into()));
        }
        file.seek(SeekFrom::Start(start))
            .await
            .map_err(|e| ApiError(format!("定位文件偏移失败: {}", e)))?;
        let len = end - start + 1;
        let stream = ReaderStream::new(file.take(len));
        let body = Body::from_stream(stream);
        return Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
            .header("Content-Type", content_type)
            .header("Content-Range", format!("bytes {}-{}/{}", start, end, total))
            .header("Accept-Ranges", "bytes")
            .header("Content-Length", len)
            .header("Content-Disposition", "inline")
            .body(body)
            .map_err(|e| ApiError(format!("构建响应失败: {}", e)));
    }

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Response::builder()
        .header("Content-Type", content_type)
        .header("Accept-Ranges", "bytes")
        .header("Content-Disposition", "inline")
        .body(body)
        .map_err(|e| ApiError(format!("构建响应失败: {}", e)))
}

/// 解析 HTTP Range 头（"bytes=start-end" / "bytes=-suffix"），返回含闭区间 [start, end]
fn parse_range(range: &str, total: u64) -> Option<(u64, u64)> {
    let range = range.trim();
    let spec = range.strip_prefix("bytes=")?;
    let (start_s, end_s) = spec.split_once('-')?;
    let start = if start_s.is_empty() {
        // 后缀范围：bytes=-N 取最后 N 字节
        total.checked_sub(end_s.parse::<u64>().ok()?)?
    } else {
        start_s.parse::<u64>().ok()?
    };
    let end = if end_s.is_empty() {
        total.saturating_sub(1)
    } else {
        end_s.parse::<u64>().ok()?
    };
    Some((start, end))
}
