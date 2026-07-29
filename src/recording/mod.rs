//! 录制引擎模块
//!
//! 包含：
//! - `RecordingManager` — 状态机 + 轮询调度（对应 StreamCap 的 record_manager.py）
//! - `FFmpegRecorder` — FFmpeg 子进程管理（对应 stream_manager.py）
//! - `DirectDownloader` — FLV 直链下载（对应 direct_downloader.py）

pub mod manager;
pub mod ffmpeg;
pub mod downloader;
