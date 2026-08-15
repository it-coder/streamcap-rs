//! 录制历史 Tauri 命令 — 仅桌面模式可用

use crate::config::AppState;
use crate::models::RecordingHistoryEntry;
use std::sync::Arc;
use tauri::State;

/// 获取全部录制历史（最新在前）
#[tauri::command]
pub fn list_history(state: State<'_, Arc<AppState>>) -> Vec<RecordingHistoryEntry> {
    state.history.read().clone()
}

/// 删除一条录制历史（delete_file=true 同时删除关联文件）
#[tauri::command]
pub async fn delete_history(
    state: State<'_, Arc<AppState>>,
    id: String,
    delete_file: Option<bool>,
) -> Result<(), String> {
    state
        .remove_history_entry(&id, delete_file.unwrap_or(false))
        .await
        .map(|_| ())
}
