//! 磁盘清理相关 Tauri 命令

use crate::cleanup;
use crate::config::AppState;
use std::sync::Arc;
use tauri::State;

/// 立即执行一次磁盘自动清理，返回被删除的录制条数
#[tauri::command]
pub async fn run_cleanup(state: State<'_, Arc<AppState>>) -> Result<u32, String> {
    Ok(cleanup::run_cleanup(state.inner()).await)
}
