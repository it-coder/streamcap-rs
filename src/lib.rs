//! streamcap-rs — Rust直播流录制工具
//!
//! 基于 streamget-rs 库解析直播流URL，通过 FFmpeg 进行直播录制。

pub mod models;
pub mod config;
pub mod stream;
pub mod recording;
pub mod commands;

pub use recording::manager::RecordingManager;
