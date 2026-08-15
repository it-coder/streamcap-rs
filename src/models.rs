//! 数据模型 — 录制任务、设置、流信息

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 视频质量等级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VideoQuality {
    /// 原画
    #[serde(rename = "OD")]
    Original,
    /// 超高清
    #[serde(rename = "UHD")]
    UltraHD,
    /// 高清
    #[serde(rename = "HD")]
    HD,
    /// 标清
    #[serde(rename = "SD")]
    SD,
    /// 低清
    #[serde(rename = "LD")]
    LD,
}

impl VideoQuality {
    pub fn as_streamget_quality(&self) -> &str {
        match self {
            VideoQuality::Original => "OD",
            VideoQuality::UltraHD => "UHD",
            VideoQuality::HD => "HD",
            VideoQuality::SD => "SD",
            VideoQuality::LD => "LD",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            VideoQuality::Original => "原画",
            VideoQuality::UltraHD => "超清",
            VideoQuality::HD => "高清",
            VideoQuality::SD => "标清",
            VideoQuality::LD => "低清",
        }
    }
}

impl Default for VideoQuality {
    fn default() -> Self {
        VideoQuality::Original
    }
}

/// 录制输出格式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    #[serde(rename = "mp4")]
    MP4,
    #[serde(rename = "ts")]
    TS,
    #[serde(rename = "mkv")]
    MKV,
    #[serde(rename = "flv")]
    FLV,
    #[serde(rename = "mov")]
    MOV,
}

impl OutputFormat {
    pub fn extension(&self) -> &str {
        match self {
            OutputFormat::MP4 => "mp4",
            OutputFormat::TS => "ts",
            OutputFormat::MKV => "mkv",
            OutputFormat::FLV => "flv",
            OutputFormat::MOV => "mov",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            OutputFormat::MP4 => "MP4",
            OutputFormat::TS => "TS (推荐HLS流)",
            OutputFormat::MKV => "MKV",
            OutputFormat::FLV => "FLV",
            OutputFormat::MOV => "MOV",
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::TS
    }
}

/// 录制状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecordingStatus {
    /// 监控中（等待开播）
    #[serde(rename = "monitoring")]
    Monitoring,
    /// 检查中
    #[serde(rename = "checking")]
    Checking,
    /// 直播中（准备录制）
    #[serde(rename = "live")]
    Live,
    /// 录制中
    #[serde(rename = "recording")]
    Recording,
    /// 已停止
    #[serde(rename = "stopped")]
    Stopped,
    /// 离线
    #[serde(rename = "offline")]
    Offline,
    /// 错误
    #[serde(rename = "error")]
    Error,
    /// 不在调度窗口
    #[serde(rename = "not_scheduled")]
    NotScheduled,
}

impl RecordingStatus {
    pub fn label(&self) -> &str {
        match self {
            RecordingStatus::Monitoring => "监控中",
            RecordingStatus::Checking => "检查中",
            RecordingStatus::Live => "直播中",
            RecordingStatus::Recording => "录制中",
            RecordingStatus::Stopped => "已停止",
            RecordingStatus::Offline => "离线",
            RecordingStatus::Error => "错误",
            RecordingStatus::NotScheduled => "未排期",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            RecordingStatus::Monitoring => "#6B7280",
            RecordingStatus::Checking => "#F59E0B",
            RecordingStatus::Live => "#10B981",
            RecordingStatus::Recording => "#EF4444",
            RecordingStatus::Stopped => "#9CA3AF",
            RecordingStatus::Offline => "#6B7280",
            RecordingStatus::Error => "#DC2626",
            RecordingStatus::NotScheduled => "#9CA3AF",
        }
    }
}

/// 时间范围（调度窗口）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: String, // "HH:MM"
    pub end: String,   // "HH:MM"
}

impl TimeRange {
    /// 判断给定时间是否在窗口内（支持跨零点，如 22:00-02:00）
    ///
    /// - start == end：整天
    /// - start < end（如 08:00-22:00）：start <= now < end
    /// - start > end（如 22:00-02:00，跨零点）：now >= start || now < end
    pub fn contains(&self, now: chrono::DateTime<chrono::Local>) -> bool {
        let now_min = now.format("%H:%M").to_string();
        let start = self.start.trim();
        let end = self.end.trim();

        if start.is_empty() || end.is_empty() {
            return true;
        }
        if start == end {
            return true;
        }
        let now_str = now_min.as_str();
        if start < end {
            now_str >= start && now_str < end
        } else {
            now_str >= start || now_str < end
        }
    }

    /// 一组窗口是否覆盖当前时间；空列表 = 不限制（全天）
    pub fn any_contains(
        schedule: &[TimeRange],
        now: chrono::DateTime<chrono::Local>,
    ) -> bool {
        if schedule.is_empty() {
            return true;
        }
        schedule.iter().any(|w| w.contains(now))
    }

    /// 格式化为显示字符串
    pub fn display(&self) -> String {
        format!("{}-{}", self.start, self.end)
    }
}

/// 录制任务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    /// 唯一标识
    pub id: String,
    /// 直播间URL
    pub url: String,
    /// 平台标识（douyin, bilibili, etc.）
    pub platform: String,
    /// 主播名称
    #[serde(default)]
    pub anchor_name: String,
    /// 直播标题
    #[serde(default)]
    pub title: String,
    /// 是否启用监控
    #[serde(default)]
    pub monitor_enabled: bool,
    /// 当前录制中
    #[serde(default)]
    pub is_recording: bool,
    /// 直播中
    #[serde(default)]
    pub is_live: bool,
    /// 视频质量
    #[serde(default)]
    pub quality: VideoQuality,
    /// 输出格式
    #[serde(default)]
    pub output_format: OutputFormat,
    /// 输出目录
    #[serde(default)]
    pub output_dir: Option<String>,
    /// 实际录制目录(运行时填充)
    #[serde(default)]
    pub recording_dir: Option<String>,
    /// 调度时间窗口
    #[serde(default)]
    pub schedule: Vec<TimeRange>,
    /// 创建时间
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    /// 更新时间
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
    /// 上次检测时间
    #[serde(default)]
    pub last_check_at: Option<DateTime<Utc>>,
    /// 录制开始时间
    #[serde(default)]
    pub recording_started_at: Option<DateTime<Utc>>,
    /// 错误信息
    #[serde(default)]
    pub error_message: Option<String>,
    /// 录制段数
    #[serde(default)]
    pub segment_count: u32,
    /// 当前重试次数（0=未重试，仅录制失败自动重试时 >0）
    #[serde(default)]
    pub retry_count: u32,
    /// 封面帧快照路径（录制中定期抓取，可能为 null）
    #[serde(default)]
    pub thumbnail: Option<String>,
}

/// 文件条目（文件浏览 API 返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// 文件名
    pub name: String,
    /// 完整路径
    pub path: String,
    /// 是否为目录
    pub is_dir: bool,
    /// 文件大小（字节），目录为 0
    pub size: u64,
    /// 最后修改时间（RFC3339），无法获取时为 null
    pub modified: Option<String>,
}

/// 流信息（从 streamget-rs 解析后返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    /// 流URL
    pub stream_url: String,
    /// 平台
    pub platform: String,
    /// 主播名称
    pub anchor_name: String,
    /// 直播标题
    pub title: String,
    /// 是否FLV流（需要直接下载）
    pub is_flv: bool,
}

/// 录制进度（传给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingProgress {
    pub recording_id: String,
    pub status: RecordingStatus,
    pub duration_seconds: u64,
    pub file_size_bytes: u64,
    pub download_speed_kbps: f64,
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 默认视频质量
    #[serde(default)]
    pub default_quality: VideoQuality,
    /// 默认输出格式
    #[serde(default)]
    pub default_format: OutputFormat,
    /// 输出根目录
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    /// 开播检测轮询间隔(秒)
    #[serde(default = "default_loop_interval")]
    pub loop_interval_seconds: u64,
    /// 磁盘空间阈值(GB)，低于此值停止录制
    #[serde(default)]
    pub recording_space_threshold_gb: u64,
    /// 启用代理
    #[serde(default)]
    pub enable_proxy: bool,
    /// 代理地址
    #[serde(default)]
    pub proxy_url: Option<String>,
    /// 目录结构：按平台分文件夹
    #[serde(default)]
    pub folder_by_platform: bool,
    /// 目录结构：按主播分文件夹
    #[serde(default)]
    pub folder_by_anchor: bool,
    /// 目录结构：按日期分文件夹
    #[serde(default = "default_true")]
    pub folder_by_date: bool,
    /// 目录结构：按标题分文件夹
    #[serde(default)]
    pub folder_by_title: bool,
    /// 分段时长(秒)，0表示不分段
    #[serde(default)]
    pub segment_duration_seconds: u64,
    /// 是否启用录制后格式转换（转封装 remux）
    #[serde(default)]
    pub enable_conversion: bool,
    /// 转换的目标格式
    #[serde(default = "default_conversion_format")]
    pub conversion_format: OutputFormat,
    /// 转换后是否删除原始文件（仅成功时删除）
    #[serde(default = "default_true")]
    pub delete_original_after_conversion: bool,
    /// 各平台 Cookie 配置（key=平台标识，value=Cookie 字符串）
    /// 用于解析需要登录才能获取高画质的直播间流
    #[serde(default)]
    pub cookies_by_platform: HashMap<String, String>,
    /// 录制失败最大重试次数（0=不重试）
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// 重试退避基础秒数（第 n 次重试等待 retry_delay_seconds * n 秒）
    #[serde(default = "default_retry_delay")]
    pub retry_delay_seconds: u64,
}

fn default_output_dir() -> String {
    dirs_next().to_string_lossy().to_string()
}

fn default_loop_interval() -> u64 {
    180
}

fn default_true() -> bool {
    true
}

fn default_conversion_format() -> OutputFormat {
    OutputFormat::MP4
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    10
}

/// 获取默认下载目录
fn dirs_next() -> std::path::PathBuf {
    dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_quality: VideoQuality::Original,
            default_format: OutputFormat::TS,
            output_dir: default_output_dir(),
            loop_interval_seconds: 180,
            recording_space_threshold_gb: 0,
            enable_proxy: false,
            proxy_url: None,
            folder_by_platform: false,
            folder_by_anchor: false,
            folder_by_date: true,
            folder_by_title: false,
            segment_duration_seconds: 1800,
            enable_conversion: false,
            conversion_format: OutputFormat::MP4,
            delete_original_after_conversion: true,
            cookies_by_platform: HashMap::new(),
            max_retries: 3,
            retry_delay_seconds: 10,
        }
    }
}
