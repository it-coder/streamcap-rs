//! 磁盘空间检查工具

use std::path::Path;

/// 获取指定路径所在磁盘的可用空间（GB）
///
/// 如果路径不存在，会逐级向上查找已存在的父目录。
pub fn available_gb(path: &Path) -> Result<f64, String> {
    // 找到一个实际存在的祖先目录
    let existing = path
        .ancestors()
        .find(|p| p.exists())
        .ok_or_else(|| format!("路径及其父目录均不存在: {:?}", path))?;

    let available_bytes = fs2::available_space(existing)
        .map_err(|e| format!("获取磁盘可用空间失败: {}", e))?;

    Ok(available_bytes as f64 / 1_073_741_824.0) // bytes -> GB
}

/// 检查磁盘空间是否充足
///
/// `threshold_gb` 为 0 表示不检查。
/// 返回 Ok(()) 表示充足，Err(message) 表示不足。
pub fn check_space(path: &Path, threshold_gb: u64) -> Result<(), String> {
    if threshold_gb == 0 {
        return Ok(());
    }

    let available = available_gb(path)?;
    if available < threshold_gb as f64 {
        return Err(format!(
            "磁盘空间不足: 可用 {:.1} GB < 阈值 {} GB",
            available, threshold_gb
        ));
    }
    Ok(())
}
