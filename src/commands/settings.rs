//! 设置管理 Tauri 命令

use crate::config::AppState;
use crate::models::AppSettings;
use std::sync::Arc;
use tauri::State;

/// 获取设置
#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Result<AppSettings, String> {
    let settings = state.settings.read();
    Ok(settings.clone())
}

/// 更新设置
#[tauri::command]
pub fn update_settings(
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> Result<(), String> {
    *state.settings.write() = settings;
    state.save_settings().map_err(|e| format!("保存失���: {}", e))
}

/// 检查 FFmpeg 是否可用
#[tauri::command]
pub fn check_ffmpeg() -> Result<String, String> {
    crate::recording::ffmpeg::check_ffmpeg_available()
}
