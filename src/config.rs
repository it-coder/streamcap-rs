//! 配置管理 — 设置持久化、录制任务存储

use crate::models::{AppSettings, RecordingConfig};
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;

/// 应���全局状态
pub struct AppState {
    pub settings: RwLock<AppSettings>,
    pub recordings: RwLock<Vec<RecordingConfig>>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("streamcap-rs");

        std::fs::create_dir_all(&data_dir).ok();

        let settings = Self::load_settings(&data_dir);
        let mut recordings = Self::load_recordings(&data_dir);

        // 重启后清空运行时状态：FFmpeg 进程已不存在，这些标记均过期
        for r in &mut recordings {
            r.is_recording = false;
            r.is_live = false;
            r.error_message = None;
            r.recording_dir = None;
            r.recording_started_at = None;
        }

        Arc::new(Self {
            settings: RwLock::new(settings),
            recordings: RwLock::new(recordings),
            data_dir,
        })
    }

    fn settings_path(data_dir: &PathBuf) -> PathBuf {
        data_dir.join("settings.json")
    }

    fn recordings_path(data_dir: &PathBuf) -> PathBuf {
        data_dir.join("recordings.json")
    }

    pub fn load_settings(data_dir: &PathBuf) -> AppSettings {
        let path = Self::settings_path(data_dir);
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            AppSettings::default()
        }
    }

    pub fn save_settings(&self) -> Result<(), String> {
        let settings = self.settings.read();
        let json = serde_json::to_string_pretty(&*settings).map_err(|e| e.to_string())?;

        let path = Self::settings_path(&self.data_dir);
        // 原子写入：先写临时文件，再 rename
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp_path, &path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_recordings(data_dir: &PathBuf) -> Vec<RecordingConfig> {
        let path = Self::recordings_path(data_dir);
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    pub fn save_recordings(&self) -> Result<(), String> {
        let recordings = self.recordings.read();
        let json = serde_json::to_string_pretty(&*recordings).map_err(|e| e.to_string())?;

        let path = Self::recordings_path(&self.data_dir);
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp_path, &path).map_err(|e| e.to_string())?;
        Ok(())
    }
}
