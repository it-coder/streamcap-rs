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

/// 定时录制的重复方式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Recurrence {
    /// 仅一次（不重复）
    #[serde(rename = "once")]
    Once,
    /// 每天重复
    #[serde(rename = "daily")]
    Daily,
    /// 每周重复（按 7 天周期）
    #[serde(rename = "weekly")]
    Weekly,
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
    /// 定时开始录制（绝对时间 UTC）；到点前不自动开录（None=立即按现有逻辑）
    #[serde(default)]
    pub scheduled_start: Option<DateTime<Utc>>,
    /// 录制完成后的重复方式（once=不重复；daily/weekly 会自动排期下一次）
    #[serde(default)]
    pub recurrence: Option<Recurrence>,
}

/// 合并录制任务的编辑更新（供 update_recording 调用）。
///
/// 限制条件：
/// - 保留所有运行时/派生字段（id、created_at、is_recording、is_live、
///   recording_dir、last_check_at、recording_started_at、segment_count、
///   retry_count、thumbnail、error_message），仅覆盖用户可编辑字段，
///   防止编辑误覆盖正在进行的录制状态。
/// - 录制进行中（is_recording=true）禁止修改 url 与 output_dir，
///   因为它们与正在运行的 FFmpeg 进程绑定；违者返回 Err。
pub fn merge_recording_update(
    existing: &mut RecordingConfig,
    incoming: RecordingConfig,
) -> Result<(), String> {
    if existing.is_recording
        && (existing.url != incoming.url || existing.output_dir != incoming.output_dir)
    {
        return Err("录制进行中无法修改直播间地址或输出目录".to_string());
    }

    // 先拷贝需要保留的运行时/派生字段，避免与下方 *existing = 赋值产生借用冲突
    let id = existing.id.clone();
    let is_recording = existing.is_recording;
    let is_live = existing.is_live;
    let recording_dir = existing.recording_dir.clone();
    let created_at = existing.created_at;
    let last_check_at = existing.last_check_at;
    let recording_started_at = existing.recording_started_at;
    let error_message = existing.error_message.clone();
    let segment_count = existing.segment_count;
    let retry_count = existing.retry_count;
    let thumbnail = existing.thumbnail.clone();

    *existing = RecordingConfig {
        id,
        url: incoming.url,
        platform: incoming.platform,
        anchor_name: incoming.anchor_name,
        title: incoming.title,
        monitor_enabled: incoming.monitor_enabled,
        is_recording,
        is_live,
        quality: incoming.quality,
        output_format: incoming.output_format,
        output_dir: incoming.output_dir,
        recording_dir,
        schedule: incoming.schedule,
        created_at,
        updated_at: Utc::now(),
        last_check_at,
        recording_started_at,
        error_message,
        segment_count,
        retry_count,
        thumbnail,
        scheduled_start: incoming.scheduled_start,
        recurrence: incoming.recurrence,
    };
    Ok(())
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

/// 录制历史状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HistoryStatus {
    /// 录制成功完成
    #[serde(rename = "completed")]
    Completed,
    /// 录制失败
    #[serde(rename = "failed")]
    Failed,
    /// 用户取消/未实际产出文件
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl HistoryStatus {
    pub fn label(&self) -> &str {
        match self {
            HistoryStatus::Completed => "完成",
            HistoryStatus::Failed => "失败",
            HistoryStatus::Cancelled => "已取消",
        }
    }
}

/// 录制历史条目（录制结束后持久化，供「录制历史」页查看与回放）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingHistoryEntry {
    /// 历史条目唯一标识
    pub id: String,
    /// 原始录制任务 ID
    pub recording_id: String,
    /// 直播间 URL
    pub url: String,
    /// 平台标识
    pub platform: String,
    /// 主播名称
    pub anchor_name: String,
    /// 直播标题
    pub title: String,
    /// 录制结果状态
    pub status: HistoryStatus,
    /// 开始时间
    pub started_at: DateTime<Utc>,
    /// 结束时间
    pub ended_at: DateTime<Utc>,
    /// 录制时长（秒）
    pub duration_seconds: u64,
    /// 代表文件（最大的媒体文件，用于回放/下载）
    pub file_path: Option<String>,
    /// 该次录制产出文件总大小（字节）
    pub file_size: u64,
    /// 封面帧快照路径
    #[serde(default)]
    pub thumbnail: Option<String>,
    /// 该次录制产出文件所在目录（用于自动清理时整体删除）
    #[serde(default)]
    pub recording_dir: Option<String>,
    /// 错误信息（失败时）
    #[serde(default)]
    pub error_message: Option<String>,
    /// 创建时间
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

/// 后处理任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "failed")]
    Failed,
}

/// 后处理任务（格式转换 / 提取音频 / 片段截取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostProcessJob {
    /// 任务 ID
    pub id: String,
    /// 操作类型：convert | extract-audio | trim
    pub kind: String,
    /// 输入文件路径
    pub input: String,
    /// 输出文件路径（运行时填充）
    #[serde(default)]
    pub output: Option<String>,
    /// 任务状态
    pub status: JobStatus,
    /// 进度 0-100
    #[serde(default)]
    pub progress: u8,
    /// 错误信息
    #[serde(default)]
    pub error: Option<String>,
    /// 创建时间
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

/// 启动后处理任务的请求体
#[derive(Debug, Clone, Deserialize)]
pub struct PostProcessRequest {
    /// 操作类型：convert | extract-audio | trim
    pub kind: String,
    /// 输入文件路径（位于输出目录内）
    pub input: String,
    /// convert 时的目标格式（mp4/ts/mkv/flv/mov）
    #[serde(default)]
    pub target_format: Option<String>,
    /// trim 时的起始秒数
    #[serde(default)]
    pub start_seconds: Option<f64>,
    /// trim 时的结束秒数（不填表示到结尾）
    #[serde(default)]
    pub end_seconds: Option<f64>,
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
    /// 事件通知 Webhook URL（录制开始/完成/失败时 POST JSON；为空则不发送）
    #[serde(default)]
    pub webhook_url: Option<String>,
    /// 最大同时录制路数（0 表示不限制）
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_recordings: u32,
    /// 磁盘空间不足时自动清理最旧的已完成录制（需配合 recording_space_threshold_gb > 0）
    #[serde(default)]
    pub auto_cleanup: bool,
    /// 关闭窗口时最小化到系统托盘（而非退出应用）；默认 true
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    /// 开机自启（仅桌面端生效；server 模式忽略）
    #[serde(default)]
    pub auto_launch: bool,
    /// 自定义 FFmpeg 可执行文件路径；None 时回退到 PATH 中的 "ffmpeg"。
    /// 由「一键安装 FFmpeg」自动写入（下载的静态二进制路径）。
    #[serde(default)]
    pub ffmpeg_path: Option<String>,
}

/// FFmpeg 可用性检测结果（供前端结构化展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegCheck {
    /// 是否可用
    pub available: bool,
    /// 版本字符串（不可用时为空）
    pub version: String,
    /// 实际使用的可执行文件路径（PATH 查找时为 "ffmpeg"）
    pub path: String,
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

fn default_max_concurrent() -> u32 {
    3
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
            webhook_url: None,
            max_concurrent_recordings: 3,
            auto_cleanup: false,
            minimize_to_tray: true,
            auto_launch: false,
            ffmpeg_path: None,
        }
    }
}

impl AppSettings {
    /// 校验设置合法性。返回 Err(原因) 表示非法；Ok(()) 表示通过。
    /// 后端在保存前调用，避免把明显错误的配置写盘。
    pub fn validate(&self) -> Result<(), String> {
        // 输出目录必须非空（后续录制依赖它，空目录会静默失败）
        if self.output_dir.trim().is_empty() {
            return Err("输出目录不能为空".to_string());
        }

        // 检测间隔：不能为 0（过于频繁可能触发平台封禁），上限 1 小时
        if self.loop_interval_seconds == 0 || self.loop_interval_seconds > 3600 {
            return Err("检测间隔需在 1-3600 秒之间".to_string());
        }

        // 磁盘阈值：0 表示不限制，上限给一个宽松上限避免误填
        if self.recording_space_threshold_gb > 10_000_000 {
            return Err("磁盘空间阈值过大（上限 10000000 GB）".to_string());
        }

        // 分段时长：0 表示不分段，上限 24 小时
        if self.segment_duration_seconds > 86400 {
            return Err("分段时长不能超过 86400 秒（24 小时）".to_string());
        }

        // 重试次数 / 退避：合理范围
        if self.max_retries > 50 {
            return Err("最大重试次数不能超过 50".to_string());
        }
        if self.retry_delay_seconds == 0 || self.retry_delay_seconds > 3600 {
            return Err("重试退避需在 1-3600 秒之间".to_string());
        }

        // 代理：启用时必须填写且为合法 URL（带协议头）
        if self.enable_proxy {
            match &self.proxy_url {
                Some(u) if !u.trim().is_empty() => {
                    if !has_url_scheme(u) {
                        return Err("代理地址需包含协议头，如 http:// 或 socks5://".to_string());
                    }
                }
                _ => {
                    return Err("启用代理时必须填写代理地址".to_string());
                }
            }
        }

        // Webhook：填写时必须为空合法的 http(s) 地址
        if let Some(u) = &self.webhook_url {
            if !u.trim().is_empty() && !is_http_url(u) {
                return Err("Webhook 地址必须是 http(s) URL".to_string());
            }
        }

        // 并发上限：0 表示不限制，给一个宽松上限避免误填
        if self.max_concurrent_recordings > 100 {
            return Err("最大同时录制路数不能超过 100".to_string());
        }

        Ok(())
    }
}

/// 是否包含 URL 协议头（xxx://）
fn has_url_scheme(s: &str) -> bool {
    s.trim().contains("://")
}

/// 是否为 http(s) URL
fn is_http_url(s: &str) -> bool {
    let t = s.trim().to_ascii_lowercase();
    t.starts_with("http://") || t.starts_with("https://")
}
