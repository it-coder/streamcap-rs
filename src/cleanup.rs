//! 磁盘自动清理 — 当可用空间低于阈值时，按"最旧优先"删除已完成的录制

use crate::config::AppState;
use crate::disk;
use crate::models::HistoryStatus;
use std::path::PathBuf;
use std::sync::Arc;

/// 执行一次清理：循环删除最旧的已完成录制目录，直到可用空间 >= 阈值或无可删项。
///
/// 返回被删除的录制条数。仅当 `settings.auto_cleanup` 开启且 `recording_space_threshold_gb > 0` 才生效。
pub async fn run_cleanup(app_state: &Arc<AppState>) -> u32 {
    let (auto_cleanup, threshold_gb, output_dir) = {
        let s = app_state.settings.read();
        (
            s.auto_cleanup,
            s.recording_space_threshold_gb,
            s.output_dir.clone(),
        )
    };

    if !auto_cleanup || threshold_gb == 0 {
        return 0;
    }

    let root = PathBuf::from(output_dir);
    let mut deleted: u32 = 0;

    loop {
        let available = match disk::available_gb(&root) {
            Ok(v) => v,
            Err(_) => break,
        };
        if available >= threshold_gb as f64 {
            break;
        }

        // 找出最旧的、已完成且记录了目录的历史条目
        let oldest = {
            let history = app_state.history.read();
            history
                .iter()
                .filter(|h| h.status == HistoryStatus::Completed && h.recording_dir.is_some())
                .min_by_key(|h| h.ended_at)
                .cloned()
        };

        let entry = match oldest {
            Some(e) => e,
            None => break,
        };

        // 删除整个录制目录（包含所有分段 / 转换文件）
        if let Some(dir) = &entry.recording_dir {
            let _ = std::fs::remove_dir_all(dir);
        }

        // 移除历史条目（delete_file=false，因为目录已整体删除）
        let _ = app_state.remove_history_entry(&entry.id, false).await;
        deleted += 1;
        tracing::info!(
            "自动清理: 删除最旧录制 {} (剩余空间 {:.1} GB)",
            entry.recording_dir.unwrap_or_default(),
            disk::available_gb(&root).unwrap_or(0.0)
        );
    }

    if deleted > 0 {
        let _ = app_state.save_recordings().await;
    }

    deleted
}
