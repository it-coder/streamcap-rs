// 与 Rust 后端 models.rs 对应的 TypeScript 类型定义

export type VideoQuality = "OD" | "UHD" | "HD" | "SD" | "LD";

export type OutputFormat = "mp4" | "ts" | "mkv" | "flv" | "mov";

export type RecordingStatus =
  | "monitoring"
  | "checking"
  | "live"
  | "recording"
  | "stopped"
  | "offline"
  | "error"
  | "not_scheduled"
  | "retrying";

export interface TimeRange {
  start: string; // "HH:MM"
  end: string; // "HH:MM"
}

export interface RecordingConfig {
  id: string;
  url: string;
  platform: string;
  anchor_name: string;
  title: string;
  monitor_enabled: boolean;
  is_recording: boolean;
  is_live: boolean;
  quality: VideoQuality;
  output_format: OutputFormat;
  output_dir: string | null;
  recording_dir: string | null;
  schedule: TimeRange[];
  created_at: string;
  updated_at: string;
  last_check_at: string | null;
  recording_started_at: string | null;
  error_message: string | null;
  segment_count: number;
  retry_count: number;
  thumbnail: string | null;
}

export interface AppSettings {
  default_quality: VideoQuality;
  default_format: OutputFormat;
  output_dir: string;
  loop_interval_seconds: number;
  recording_space_threshold_gb: number;
  enable_proxy: boolean;
  proxy_url: string | null;
  folder_by_platform: boolean;
  folder_by_anchor: boolean;
  folder_by_date: boolean;
  folder_by_title: boolean;
  segment_duration_seconds: number;
  enable_conversion: boolean;
  conversion_format: OutputFormat;
  delete_original_after_conversion: boolean;
  cookies_by_platform: Record<string, string>;
  max_retries: number;
  retry_delay_seconds: number;
  webhook_url: string | null;
}

// 录制历史状态
export type HistoryStatus = "completed" | "failed" | "cancelled";

// 录制历史条目
export interface RecordingHistoryEntry {
  id: string;
  recording_id: string;
  url: string;
  platform: string;
  anchor_name: string;
  title: string;
  status: HistoryStatus;
  started_at: string;
  ended_at: string;
  duration_seconds: number;
  file_path: string | null;
  file_size: number;
  thumbnail: string | null;
  error_message: string | null;
  created_at: string;
}

// 后处理任务状态
export type JobStatus = "pending" | "running" | "done" | "failed";

// 后处理任务
export interface PostProcessJob {
  id: string;
  kind: string; // convert | extract-audio | trim
  input: string;
  output: string | null;
  status: JobStatus;
  progress: number;
  error: string | null;
  created_at: string;
}

// 启动后处理任务的请求
export interface PostProcessRequest {
  kind: string;
  input: string;
  target_format?: string;
  start_seconds?: number;
  end_seconds?: number;
}

export interface RecordingProgress {
  recording_id: string;
  status: RecordingStatus;
  duration_seconds: number;
  file_size_bytes: number;
  download_speed_kbps: number;
}

export interface ShutdownPayload {
  stage: "start" | "done";
  activeCount: number;
  message: string;
}

// 应用版本信息（来自后端 get_version）
export interface AppVersion {
  version: string;
}

// 文件条目（来自后端 list_files）
export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: string | null;
}

// 视频质量选项（供 Select 使用）
export const QUALITY_OPTIONS: { value: VideoQuality; label: string }[] = [
  { value: "OD", label: "原画" },
  { value: "UHD", label: "超清" },
  { value: "HD", label: "高清" },
  { value: "SD", label: "标清" },
  { value: "LD", label: "低清" },
];

export const FORMAT_OPTIONS: { value: OutputFormat; label: string }[] = [
  { value: "ts", label: "TS (推荐)" },
  { value: "mp4", label: "MP4" },
  { value: "mkv", label: "MKV" },
  { value: "flv", label: "FLV" },
  { value: "mov", label: "MOV" },
];

// 平台检测规则（供 URL 输入时自动识别）
export const PLATFORM_PATTERNS: { pattern: string; key: string; label: string; icon: string }[] = [
  { pattern: "douyin.com", key: "douyin", label: "抖音直播", icon: "🎵" },
  { pattern: "bilibili.com", key: "bilibili", label: "哔哩哔哩直播", icon: "📺" },
  { pattern: "twitch.tv", key: "twitch", label: "Twitch", icon: "🎮" },
  { pattern: "youtube.com", key: "youtube", label: "YouTube", icon: "▶️" },
  { pattern: "huya.com", key: "huya", label: "虎牙直播", icon: "🐯" },
  { pattern: "kuaishou.com", key: "kuaishou", label: "快手直播", icon: "📱" },
  { pattern: "douyu.com", key: "douyu", label: "斗鱼直播", icon: "🐟" },
  { pattern: "tiktok.com", key: "tiktok", label: "TikTok", icon: "🎵" },
  { pattern: "xiaohongshu.com", key: "rednote", label: "小红书直播", icon: "📕" },
];
