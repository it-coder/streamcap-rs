//! HTTP 请求处理器 — REST API 端点实现
//!
//! 与 Tauri commands 一一对应，委托给共享的 AppState / RecordingManager。

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::models::{RecordingConfig, AppSettings};
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
    };

    let mut recordings = state.app_state.recordings.write();
    recordings.push(config.clone());
    drop(recordings);

    state
        .app_state
        .save_recordings()
        .map_err(|e| ApiError(format!("保存失败: {}", e)))?;

    Ok(Json(config))
}

/// DELETE /api/recordings/:id — 删除录制任务
pub async fn remove_recording(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut recordings = state.app_state.recordings.write();
    recordings.retain(|r| r.id != id);
    drop(recordings);

    state
        .app_state
        .save_recordings()
        .map_err(|e| ApiError(format!("保存失败: {}", e)))?;

    Ok(Json(json!({ "success": true })))
}

/// PUT /api/recordings/:id — 更新录制任务
pub async fn update_recording(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
    Json(config): Json<RecordingConfig>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut recordings = state.app_state.recordings.write();
    if let Some(existing) = recordings.iter_mut().find(|r| r.id == id) {
        *existing = RecordingConfig {
            updated_at: Utc::now(),
            ..config
        };
    }
    drop(recordings);

    state
        .app_state
        .save_recordings()
        .map_err(|e| ApiError(format!("保存失败: {}", e)))?;

    Ok(Json(json!({ "success": true })))
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
    let mut recordings = state.app_state.recordings.write();
    if let Some(r) = recordings.iter_mut().find(|r| r.id == id) {
        r.monitor_enabled = true;
        r.updated_at = Utc::now();
    }
    drop(recordings);
    state
        .app_state
        .save_recordings()
        .map_err(|e| ApiError(format!("保存失败: {}", e)))?;

    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/recordings/:id/monitor — 停止监控
pub async fn stop_monitor(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut recordings = state.app_state.recordings.write();
    if let Some(r) = recordings.iter_mut().find(|r| r.id == id) {
        r.monitor_enabled = false;
        r.updated_at = Utc::now();
    }
    drop(recordings);
    state
        .app_state
        .save_recordings()
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
    *state.app_state.settings.write() = settings;
    state
        .app_state
        .save_settings()
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
