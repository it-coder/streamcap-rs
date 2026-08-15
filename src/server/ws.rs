//! WebSocket 处理器 — 实时事件推送
//!
//! 客户端连接 /ws/events 后，服务器推送以下事件：
//! - { type: "recording_status", data: RecordingConfig }
//! - { type: "recording_progress", data: RecordingProgress }
//! - { type: "app:shutdown", data: ShutdownPayload }

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tracing::{error, info};

use crate::server::ServerState;

/// WebSocket 升级处理器
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_connection(socket, state))
}

/// 处理单个 WebSocket 连接
async fn handle_ws_connection(socket: WebSocket, state: Arc<ServerState>) {
    info!("WebSocket 客户端已连接");

    let (mut sender, mut receiver) = socket.split();

    // 订阅广播频道
    let mut status_rx = state.ws_broadcaster.subscribe_status();
    let mut progress_rx = state.ws_broadcaster.subscribe_progress();
    let mut shutdown_rx = state.ws_broadcaster.subscribe_shutdown();

    // 主循环：监听广播事件 + 客户端消息
    loop {
        tokio::select! {
            // 录制状态变更
            Ok(status) = status_rx.recv() => {
                let msg = json!({
                    "type": "recording_status",
                    "data": status,
                });
                if sender.send(Message::Text(msg.to_string())).await.is_err() {
                    break;
                }
            }
            // 录制进度（时长/文件大小/速度）
            Ok(progress) = progress_rx.recv() => {
                let msg = json!({
                    "type": "recording_progress",
                    "data": progress,
                });
                if sender.send(Message::Text(msg.to_string())).await.is_err() {
                    break;
                }
            }
            // 关闭事件
            Ok(shutdown) = shutdown_rx.recv() => {
                let msg = json!({
                    "type": "app:shutdown",
                    "data": shutdown,
                });
                if sender.send(Message::Text(msg.to_string())).await.is_err() {
                    break;
                }
            }
            // 客户端消息（目前仅用于心跳/断开检测）
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sender.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("WebSocket 客户端已断开");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("WebSocket 连接已关闭");
}
