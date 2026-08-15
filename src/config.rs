//! 配置管理 — 设置持久化、录制任务存储

use crate::models::{AppSettings, PostProcessJob, RecordingConfig, RecordingHistoryEntry};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// 应用全局状态
pub struct AppState {
    pub settings: RwLock<AppSettings>,
    pub recordings: RwLock<Vec<RecordingConfig>>,
    /// 录制历史（持久化到 history.json）
    pub history: RwLock<Vec<RecordingHistoryEntry>>,
    /// 后处理任务（内存态，无需持久化）
    pub jobs: RwLock<HashMap<String, PostProcessJob>>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        let data_dir = std::env::var("STREAMCAP_DATA_DIR")
            .ok()
            .map(PathBuf::from)
            .or_else(|| dirs::data_dir().map(|d| d.join("streamcap-rs")))
            .unwrap_or_else(|| PathBuf::from("."));

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
            history: RwLock::new(Self::load_history(&data_dir)),
            jobs: RwLock::new(HashMap::new()),
            data_dir,
        })
    }

    fn settings_path(data_dir: &PathBuf) -> PathBuf {
        data_dir.join("settings.json")
    }

    fn recordings_path(data_dir: &PathBuf) -> PathBuf {
        data_dir.join("recordings.json")
    }

    fn history_path(data_dir: &PathBuf) -> PathBuf {
        data_dir.join("history.json")
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

    pub async fn save_settings(&self) -> Result<(), String> {
        // 在独立作用域中序列化，确保锁守卫在 .await 前释放（否则 future 非 Send）
        let json = {
            let settings = self.settings.read();
            serde_json::to_string_pretty(&*settings).map_err(|e| e.to_string())?
        };

        let path = Self::settings_path(&self.data_dir);
        // 原子写入：先写临时文件，再 rename
        let tmp_path = path.with_extension("json.tmp");
        tokio::fs::write(&tmp_path, json)
            .await
            .map_err(|e| e.to_string())?;
        tokio::fs::rename(&tmp_path, &path)
            .await
            .map_err(|e| e.to_string())?;
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

    pub async fn save_recordings(&self) -> Result<(), String> {
        // 在独立作用域中序列化，确保锁守卫在 .await 前释放（否则 future 非 Send）
        let json = {
            let recordings = self.recordings.read();
            serde_json::to_string_pretty(&*recordings).map_err(|e| e.to_string())?
        };

        let path = Self::recordings_path(&self.data_dir);
        let tmp_path = path.with_extension("json.tmp");
        tokio::fs::write(&tmp_path, json)
            .await
            .map_err(|e| e.to_string())?;
        tokio::fs::rename(&tmp_path, &path)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ========================================
    // 录制历史
    // ========================================

    pub fn load_history(data_dir: &PathBuf) -> Vec<RecordingHistoryEntry> {
        let path = Self::history_path(data_dir);
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// 追加一条历史记录并持久化（最新的排在最前）
    pub async fn add_history_entry(&self, entry: RecordingHistoryEntry) -> Result<(), String> {
        {
            let mut history = self.history.write();
            history.insert(0, entry);
        }
        self.save_history().await
    }

    /// 删除一条历史记录（并可选删除其关联文件）
    pub async fn remove_history_entry(
        &self,
        id: &str,
        delete_file: bool,
    ) -> Result<Option<RecordingHistoryEntry>, String> {
        let removed = {
            let mut history = self.history.write();
            let pos = history.iter().position(|h| h.id == id);
            pos.map(|i| history.remove(i))
        };
        if let (Some(entry), true) = (&removed, delete_file) {
            if let Some(path) = &entry.file_path {
                let _ = std::fs::remove_file(path);
            }
        }
        self.save_history().await.map(|_| removed)
    }

    pub async fn save_history(&self) -> Result<(), String> {
        let json = {
            let history = self.history.read();
            serde_json::to_string_pretty(&*history).map_err(|e| e.to_string())?
        };
        let path = Self::history_path(&self.data_dir);
        let tmp_path = path.with_extension("json.tmp");
        tokio::fs::write(&tmp_path, json)
            .await
            .map_err(|e| e.to_string())?;
        tokio::fs::rename(&tmp_path, &path)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
