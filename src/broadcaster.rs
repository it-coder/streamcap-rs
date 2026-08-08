//! 事件广播器 — 解耦 RecordingManager 对具体框架的依赖
//!
//! 桌面模式使用 TauriBroadcaster（emit Tauri 事件）
//! 服务器模式使用 WsBroadcaster（tokio broadcast channel → WebSocket fan-out）

use crate::models::RecordingConfig;
use serde::Serialize;
use std::sync::Arc;

/// 关闭事件载荷
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShutdownPayload {
    pub stage: String,
    pub active_count: usize,
    pub message: String,
}

impl ShutdownPayload {
    pub fn start(active_count: usize) -> Self {
        Self {
            stage: "start".to_string(),
            active_count,
            message: if active_count > 0 {
                format!("正在停止 {} 个录制任务并保存状态...", active_count)
            } else {
                "正在保存状态...".to_string()
            },
        }
    }

    pub fn done(active_count: usize) -> Self {
        Self {
            stage: "done".to_string(),
            active_count,
            message: "已完成，正在退出应用...".to_string(),
        }
    }
}

/// 事件广播器 trait — 框架无关的事件推送接口
///
/// 两个实现都是同步的：
/// - TauriBroadcaster: AppHandle.emit() 是同步的
/// - WsBroadcaster: broadcast::Sender.send() 是同步的
pub trait EventBroadcaster: Send + Sync {
    /// 推送录制状态变更
    fn broadcast_status(&self, status: &RecordingConfig);
    /// 推送应用关闭事件
    fn broadcast_shutdown(&self, payload: &ShutdownPayload);
}

// ========================================
// WsBroadcaster — 服务器模式实现
// ========================================

/// WebSocket 广播器：通过 tokio broadcast channel fan-out 到所有连接的 WS 客户端
pub struct WsBroadcaster {
    status_tx: tokio::sync::broadcast::Sender<RecordingConfig>,
    shutdown_tx: tokio::sync::broadcast::Sender<ShutdownPayload>,
}

impl WsBroadcaster {
    pub fn new() -> Arc<Self> {
        let (status_tx, _) = tokio::sync::broadcast::channel(100);
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(16);
        Arc::new(Self {
            status_tx,
            shutdown_tx,
        })
    }

    /// 订阅状态变更事件（每个 WebSocket 连接调用一次）
    pub fn subscribe_status(&self) -> tokio::sync::broadcast::Receiver<RecordingConfig> {
        self.status_tx.subscribe()
    }

    /// 订阅关闭事件
    pub fn subscribe_shutdown(&self) -> tokio::sync::broadcast::Receiver<ShutdownPayload> {
        self.shutdown_tx.subscribe()
    }
}

impl EventBroadcaster for WsBroadcaster {
    fn broadcast_status(&self, status: &RecordingConfig) {
        let _ = self.status_tx.send(status.clone());
    }

    fn broadcast_shutdown(&self, payload: &ShutdownPayload) {
        let _ = self.shutdown_tx.send(payload.clone());
    }
}

// ========================================
// TauriBroadcaster — 桌面模式实现
// ========================================

#[cfg(feature = "desktop")]
pub struct TauriBroadcaster {
    app_handle: tauri::AppHandle,
}

#[cfg(feature = "desktop")]
impl TauriBroadcaster {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

#[cfg(feature = "desktop")]
impl EventBroadcaster for TauriBroadcaster {
    fn broadcast_status(&self, status: &RecordingConfig) {
        use tauri::Emitter;
        let _ = self.app_handle.emit("recording_status", status);
    }

    fn broadcast_shutdown(&self, payload: &ShutdownPayload) {
        use tauri::Emitter;
        let _ = self.app_handle.emit("app:shutdown", payload);
    }
}
