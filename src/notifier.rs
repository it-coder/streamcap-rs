//! 事件通知 — Webhook
//!
//! 当录制开始 / 完成 / 失败时，向 settings.webhook_url POST JSON 事件。
//! 用于对接外部系统（如 Discord / 飞书 / 自建通知服务）。

use crate::config::AppState;
use crate::models::RecordingConfig;
use serde_json::json;
use std::sync::Arc;

/// 事件类型
pub const EVENT_STARTED: &str = "recording.started";
pub const EVENT_COMPLETED: &str = "recording.completed";
pub const EVENT_FAILED: &str = "recording.failed";

/// 向配置的 Webhook 发送事件（无 webhook_url 时静默返回）
///
/// 调用方应自行 `tokio::spawn`，避免阻塞录制主流程。网络/HTTP 错误仅记录日志，不影响录制。
pub async fn fire_webhook(app_state: Arc<AppState>, event: &str, config: &RecordingConfig) {
    let url = {
        let settings = app_state.settings.read();
        match &settings.webhook_url {
            Some(u) if !u.trim().is_empty() => u.clone(),
            _ => return,
        }
    };

    let payload = json!({
        "event": event,
        "recording_id": config.id,
        "url": config.url,
        "platform": config.platform,
        "anchor_name": config.anchor_name,
        "title": config.title,
        "is_recording": config.is_recording,
        "error_message": config.error_message,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let client = reqwest::Client::new();
    match client
        .post(&url)
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                tracing::info!("Webhook 通知已发送: {}", event);
            } else {
                tracing::warn!("Webhook 通知返回非 2xx: {} -> {}", event, resp.status());
            }
        }
        Err(e) => {
            tracing::warn!("Webhook 通知发送失败: {} -> {}", event, e);
        }
    }
}
