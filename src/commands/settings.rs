//! 设置管理 Tauri 命令

use crate::config::AppState;
use crate::models::{AppSettings, FfmpegCheck};
use crate::recording::ffmpeg;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

/// 获取设置
#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Result<AppSettings, String> {
    let settings = state.settings.read();
    Ok(settings.clone())
}

/// 更新设置
#[tauri::command]
pub async fn update_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> Result<(), String> {
    settings
        .validate()
        .map_err(|e| format!("设置校验失败: {e}"))?;
    *state.settings.write() = settings.clone();
    state
        .save_settings()
        .await
        .map_err(|e| format!("保存失败: {}", e))?;

    // 应用开机自启（仅桌面端；server 模式不加载此命令）
    let manager = app.autolaunch();
    let _ = if settings.auto_launch {
        manager.enable()
    } else {
        manager.disable()
    };
    Ok(())
}

/// 检查 FFmpeg 是否可用（结构化返回：可用状态 + 版本 + 实际路径）
#[tauri::command]
pub fn check_ffmpeg(state: State<'_, Arc<AppState>>) -> Result<FfmpegCheck, String> {
    let bin = ffmpeg::resolve_ffmpeg_bin(&state.settings.read().ffmpeg_path);
    let path = bin.clone();
    match ffmpeg::check_ffmpeg_available(&bin) {
        Ok(version) => Ok(FfmpegCheck {
            available: true,
            version,
            path,
        }),
        Err(_) => Ok(FfmpegCheck {
            available: false,
            version: String::new(),
            path,
        }),
    }
}

/// 一键安装 FFmpeg（桌面端本地执行下载）。
///
/// 从 ffmpeg-static 发布下载适配当前平台的静态二进制到 data_dir/ffmpeg/，
/// 校验通过后写入 settings.ffmpeg_path 并保存，最后返回结构化检测结果。
#[tauri::command]
pub async fn install_ffmpeg(state: State<'_, Arc<AppState>>) -> Result<FfmpegCheck, String> {
    let data_dir = state.data_dir.clone();
    let installed_path = ffmpeg::install_ffmpeg(&data_dir).await?;

    {
        let mut settings = state.settings.write();
        settings.ffmpeg_path = Some(installed_path.clone());
    }
    state
        .save_settings()
        .await
        .map_err(|e| format!("保存设置失败: {}", e))?;

    match ffmpeg::check_ffmpeg_available(&installed_path) {
        Ok(version) => Ok(FfmpegCheck {
            available: true,
            version,
            path: installed_path,
        }),
        Err(e) => Err(format!("安装后校验失败: {}", e)),
    }
}

/// 获取应用版本号（单一可信源 = Cargo.toml version）
#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
