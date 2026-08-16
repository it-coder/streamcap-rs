//! streamcap-rs — Rust直播流录制工具
//!
//! 基于 streamget-rs 库解析直播流URL，通过 FFmpeg 进行直播录制。
//! 支持两种部署模式：
//! - desktop: Tauri 桌面客户端（默认）
//! - server: B/S 架构 HTTP 服务器

pub mod models;
pub mod config;
pub mod stream;
pub mod recording;
pub mod broadcaster;
pub mod disk;
pub mod notifier;
pub mod jobs;
pub mod cleanup;

#[cfg(feature = "desktop")]
pub mod commands;

#[cfg(feature = "server")]
pub mod server;

pub use recording::manager::RecordingManager;
pub use broadcaster::{EventBroadcaster, ShutdownPayload};
