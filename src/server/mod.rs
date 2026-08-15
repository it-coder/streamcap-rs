//! B/S 服务器模式 — Axum HTTP + WebSocket
//!
//! 提供 REST API 和 WebSocket 实时事件推送，替代 Tauri IPC。

pub mod handlers;
pub mod ws;

use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use axum::{
    routing::{get, post, put},
    Router,
};

use crate::broadcaster::WsBroadcaster;
use crate::config::AppState;
use crate::RecordingManager;

/// 服务器共享状态 — 注入到所有 Axum handler
#[derive(Clone)]
pub struct ServerState {
    pub app_state: Arc<AppState>,
    pub recording_manager: Arc<RecordingManager>,
    pub ws_broadcaster: Arc<WsBroadcaster>,
    /// 服务器启动时刻（用于 /health 的 uptime）
    pub started_at: Instant,
}

/// 构建 Axum 路由
pub fn build_router(state: Arc<ServerState>, static_dir: &str) -> Router {
    let index_path = format!("{}/index.html", static_dir);

    Router::new()
        // ========================================
        // 录制任务 API
        // ========================================
        .route(
            "/api/recordings",
            get(handlers::list_recordings).post(handlers::add_recording),
        )
        .route(
            "/api/recordings/:id",
            put(handlers::update_recording).delete(handlers::remove_recording),
        )
        .route(
            "/api/recordings/:id/monitor",
            post(handlers::start_monitor).delete(handlers::stop_monitor),
        )
        .route(
            "/api/recordings/:id/recording",
            post(handlers::start_recording).delete(handlers::stop_recording),
        )
        .route("/api/recordings/:id/status", get(handlers::get_recording_status))
        // ========================================
        // 设置 API
        // ========================================
        .route(
            "/api/settings",
            get(handlers::get_settings).put(handlers::update_settings),
        )
        .route("/api/ffmpeg/check", get(handlers::check_ffmpeg))
        .route("/api/version", get(handlers::get_version))
        .route("/api/health", get(handlers::health))
        // ========================================
        // 文件浏览 API
        // ========================================
        .route("/api/files", get(handlers::list_files))
        .route("/api/files/download", get(handlers::download_file))
        // ========================================
        // WebSocket 实时事件
        // ========================================
        .route("/ws/events", get(ws::ws_handler))
        // ========================================
        // 静态文件服务 (SPA fallback → index.html)
        // ========================================
        .fallback_service(
            ServeDir::new(static_dir).fallback(ServeFile::new(index_path)),
        )
        // CORS — 允许开发模式下跨域访问
        .layer(CorsLayer::permissive())
        .with_state(state)
}
