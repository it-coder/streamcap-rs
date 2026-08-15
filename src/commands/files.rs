//! 文件浏览 Tauri 命令 — 仅桌面模式可用

use crate::config::AppState;
use crate::models::FileEntry;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tauri_plugin_shell::ShellExt;
use base64::Engine;

/// 列出目录内容（沙箱限制在 output_dir 内，防目录遍历）
#[tauri::command]
pub async fn list_files(
    state: State<'_, Arc<AppState>>,
    dir: Option<String>,
) -> Result<Vec<FileEntry>, String> {
    let root = {
        let settings = state.settings.read();
        PathBuf::from(&settings.output_dir)
    };
    let root_abs = root
        .canonicalize()
        .map_err(|e| format!("路径无效 {}: {}", root.display(), e))?;

    let dir = match dir {
        Some(d) => PathBuf::from(d),
        None => root.clone(),
    };
    let dir_abs = dir
        .canonicalize()
        .map_err(|e| format!("路径无效 {}: {}", dir.display(), e))?;

    if !dir_abs.starts_with(&root_abs) {
        return Err("越权访问：仅允许浏览输出目录内的文件".into());
    }

    let mut entries: Vec<FileEntry> = Vec::new();
    let mut reader = tokio::fs::read_dir(&dir_abs)
        .await
        .map_err(|e| format!("读取目录失败: {}", e))?;

    while let Some(entry) = reader
        .next_entry()
        .await
        .map_err(|e| format!("读取目录失败: {}", e))?
    {
        let meta = entry
            .metadata()
            .await
            .map_err(|e| format!("读取文件信息失败: {}", e))?;
        let is_dir = meta.is_dir();
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let size = if is_dir { 0 } else { meta.len() };
        let modified = meta.modified().ok().map(|t| {
            chrono::DateTime::<chrono::Local>::from(t).to_rfc3339()
        });
        entries.push(FileEntry {
            name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            size,
            modified,
        });
    }

    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            return b.is_dir.cmp(&a.is_dir);
        }
        a.name.cmp(&b.name)
    });

    Ok(entries)
}

/// 在系统默认程序中打开文件（或定位到文件）
#[tauri::command]
pub fn open_file(app: tauri::AppHandle, path: String) -> Result<(), String> {
    app.shell()
        .open(path, None)
        .map_err(|e| format!("打开文件失败: {}", e))
}

/// 读取文件并返回 data URL（用于桌面模式下在 <img> 中预览缩略图等本地文件）
///
/// 沙箱限制在 output_dir 内，防目录遍历。
#[tauri::command]
pub async fn read_file_base64(
    state: State<'_, Arc<AppState>>,
    path: String,
) -> Result<String, String> {
    let root = {
        let settings = state.settings.read();
        PathBuf::from(&settings.output_dir)
    };
    let root_abs = root
        .canonicalize()
        .map_err(|e| format!("路径无效 {}: {}", root.display(), e))?;
    let path_abs = PathBuf::from(&path)
        .canonicalize()
        .map_err(|e| format!("路径无效 {}: {}", path, e))?;

    if path_abs.is_dir() || !path_abs.starts_with(&root_abs) {
        return Err("非法路径：仅允许访问输出目录内的文件".into());
    }

    let bytes = tokio::fs::read(&path_abs)
        .await
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let mime = match path_abs
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    };

    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, encoded))
}
