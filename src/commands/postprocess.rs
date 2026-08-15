//! 后处理任务 Tauri 命令 — 仅桌面模式可用
//!
//! 桌面模式无 WebSocket，前端通过 get_postprocess 轮询任务进度。

use crate::config::AppState;
use crate::jobs;
use crate::models::{PostProcessJob, PostProcessRequest};
use std::sync::Arc;
use tauri::State;

/// 启动一个后处理任务（格式转换 / 提取音频 / 片段截取）
#[tauri::command]
pub async fn start_postprocess(
    state: State<'_, Arc<AppState>>,
    req: PostProcessRequest,
) -> Result<PostProcessJob, String> {
    // 桌面模式不通过 WebSocket 广播，前端轮询 get_postprocess
    jobs::start_postprocess(state.inner().clone(), None, req).await
}

/// 查询后处理任务状态
#[tauri::command]
pub async fn get_postprocess(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<PostProcessJob, String> {
    let jobs = state.jobs.read();
    jobs.get(&id)
        .cloned()
        .ok_or_else(|| format!("后处理任务 {} 不存在", id))
}
