// 状态徽章组件 — 根据录制状态显示对应颜色和标签

import { Tag } from "antd";
import type { RecordingConfig } from "../types";

interface Props {
  recording: RecordingConfig;
}

function getStatus(recording: RecordingConfig) {
  if (recording.is_recording && recording.retry_count > 0) return "retrying";
  if (recording.is_recording) return "recording";
  if (recording.is_live) return "live";
  if (recording.monitor_enabled) return "monitoring";
  if (recording.error_message) return "error";
  return "offline";
}

const STATUS_CONFIG: Record<
  string,
  { label: string; color: string }
> = {
  monitoring: { label: "监控中", color: "default" },
  checking: { label: "检查中", color: "warning" },
  live: { label: "直播中", color: "success" },
  recording: { label: "录制中", color: "red" },
  retrying: { label: "重试中", color: "gold" },
  offline: { label: "离线", color: "default" },
  error: { label: "错误", color: "error" },
};

export function StatusBadge({ recording }: Props) {
  const status = getStatus(recording);
  const config = STATUS_CONFIG[status] || STATUS_CONFIG.offline;
  return <Tag color={config.color}>{config.label}</Tag>;
}
