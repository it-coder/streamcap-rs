// 状态徽章组件 — 根据录制状态显示对应颜色和标签

import { Tag } from "antd";
import type { RecordingConfig } from "../types";
import { useI18n } from "../i18n";

interface Props {
  recording: RecordingConfig;
}

function getStatusKey(recording: RecordingConfig): string {
  if (recording.is_recording && recording.retry_count > 0) return "retrying";
  if (recording.is_recording) return "recording";
  if (recording.is_live) return "live";
  if (recording.monitor_enabled) return "monitoring";
  if (recording.error_message) return "error";
  return "offline";
}

const STATUS_KEY_TO_LABEL: Record<string, string> = {
  monitoring: "status.monitoring",
  checking: "status.checking",
  live: "status.live",
  recording: "status.recording",
  retrying: "status.retrying",
  offline: "status.offline",
  error: "status.error",
};

const STATUS_COLOR: Record<string, string> = {
  monitoring: "default",
  checking: "warning",
  live: "success",
  recording: "red",
  retrying: "gold",
  offline: "default",
  error: "error",
};

export function StatusBadge({ recording }: Props) {
  const { t } = useI18n();
  const status = getStatusKey(recording);
  const color = STATUS_COLOR[status] || STATUS_COLOR.offline;
  const label = t(STATUS_KEY_TO_LABEL[status] || STATUS_KEY_TO_LABEL.offline);
  return <Tag color={color}>{label}</Tag>;
}
