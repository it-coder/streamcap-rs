//! 录制管理 Tauri 命令

use crate::config::AppState;
use crate::models::RecordingConfig;
use crate::RecordingManager;
use crate::stream::resolver;
use chrono::Utc;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

/// 添加录制任务
#[tauri::command]
pub async fn add_recording(
    state: State<'_, Arc<AppState>>,
    url: String,
    monitor_enabled: Option<bool>,
    quality: Option<String>,
) -> Result<RecordingConfig, String> {
    let id = Uuid::new_v4().to_string();
    let (platform_key, _platform_name) = resolver::detect_platform(&url);
    let quality = quality.unwrap_or_else(|| "OD".to_string());

    let config = RecordingConfig {
        id,
        url,
        platform: platform_key.to_string(),
        anchor_name: String::new(),
        title: String::new(),
        monitor_enabled: monitor_enabled.unwrap_or(true),
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

    {
        let mut recordings = state.recordings.write();
        recordings.push(config.clone());
    }

    state
        .save_recordings()
        .await
        .map_err(|e| format!("保存失败: {}", e))?;

    Ok(config)
}

/// 删除录制任务
#[tauri::command]
pub async fn remove_recording(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    {
        let mut recordings = state.recordings.write();
        recordings.retain(|r| r.id != id);
    }

    state
        .save_recordings()
        .await
        .map_err(|e| format!("保存失败: {}", e))
}

/// 更新录制任务
#[tauri::command]
pub async fn update_recording(
    state: State<'_, Arc<AppState>>,
    config: RecordingConfig,
) -> Result<(), String> {
    let id = config.id.clone();
    let mut found = false;
    {
        let mut recordings = state.recordings.write();
        if let Some(existing) = recordings.iter_mut().find(|r| r.id == id) {
            found = true;
            crate::models::merge_recording_update(existing, config)?;
        }
    }
    if !found {
        return Err(format!("未找到录制任务: {}", id));
    }

    state
        .save_recordings()
        .await
        .map_err(|e| format!("保存失败: {}", e))
}

/// 获取录制列表
#[tauri::command]
pub fn list_recordings(state: State<'_, Arc<AppState>>) -> Result<Vec<RecordingConfig>, String> {
    let recordings = state.recordings.read();
    Ok(recordings.clone())
}

/// 启用监控
#[tauri::command]
pub async fn start_monitor(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    {
        let mut recordings = state.recordings.write();
        if let Some(r) = recordings.iter_mut().find(|r| r.id == id) {
            r.monitor_enabled = true;
            r.updated_at = Utc::now();
        }
    }
    state
        .save_recordings()
        .await
        .map_err(|e| format!("保存失败: {}", e))
}

/// 停止监控
#[tauri::command]
pub async fn stop_monitor(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    {
        let mut recordings = state.recordings.write();
        if let Some(r) = recordings.iter_mut().find(|r| r.id == id) {
            r.monitor_enabled = false;
            r.updated_at = Utc::now();
        }
    }
    state
        .save_recordings()
        .await
        .map_err(|e| format!("保存失败: {}", e))
}

/// 手动开始录制
#[tauri::command]
pub async fn start_recording(
    manager: State<'_, Arc<RecordingManager>>,
    id: String,
) -> Result<(), String> {
    manager.start_recording(&id).await
}

/// 手动停止���制
#[tauri::command]
pub async fn stop_recording(
    manager: State<'_, Arc<RecordingManager>>,
    id: String,
) -> Result<(), String> {
    manager.stop_recording(&id).await
}

/// 获取录制状态
#[tauri::command]
pub fn get_recording_status(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Option<RecordingConfig>, String> {
    let recordings = state.recordings.read();
    Ok(recordings.iter().find(|r| r.id == id).cloned())
}
