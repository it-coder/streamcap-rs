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
  | "not_scheduled";

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
}

export interface ShutdownPayload {
  stage: "start" | "done";
  activeCount: number;
  message: string;
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
];

// 平台检测规则（供 URL 输入时自动识别）
export const PLATFORM_PATTERNS: { pattern: string; label: string; icon: string }[] = [
  { pattern: "douyin.com", label: "抖音直播", icon: "🎵" },
  { pattern: "bilibili.com", label: "哔哩哔哩直播", icon: "📺" },
  { pattern: "twitch.tv", label: "Twitch", icon: "🎮" },
  { pattern: "youtube.com", label: "YouTube", icon: "▶️" },
  { pattern: "huya.com", label: "虎牙直播", icon: "🐯" },
  { pattern: "kuaishou.com", label: "快手直播", icon: "📱" },
  { pattern: "douyu.com", label: "斗鱼直播", icon: "🐟" },
  { pattern: "tiktok.com", label: "TikTok", icon: "🎵" },
  { pattern: "xiaohongshu.com", label: "小红书直播", icon: "📕" },
];
