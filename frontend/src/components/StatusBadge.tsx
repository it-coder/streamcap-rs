// 状态徽章组件 — 根据录制状态显示对应颜色和标签

import { Tag } from "antd";
import type { RecordingConfig } from "../types";
import { useI18n } from "../i18n";

interface Props {
  recording: RecordingConfig;
  /** 当前正在录制的路数（用于判断"排队中"） */
  activeCount?: number;
  /** 最大同时录制路数（0=不限制） */
  maxConcurrent?: number;
}

function getStatusKey(
  recording: RecordingConfig,
  activeCount?: number,
  maxConcurrent?: number,
): string {
  if (recording.is_recording && recording.retry_count > 0) return "retrying";
  if (recording.is_recording) return "recording";
  // 已排期（定时录制未到点）：优先于 live 展示
  if (
    recording.scheduled_start &&
    new Date(recording.scheduled_start).getTime() > Date.now()
  ) {
    return "scheduled";
  }
  // 并发已满且已开播但未开始录制 → 排队中
  if (
    recording.is_live &&
    !recording.is_recording &&
    maxConcurrent !== undefined &&
    maxConcurrent > 0 &&
    activeCount !== undefined &&
    activeCount >= maxConcurrent
  ) {
    return "queued";
  }
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
  scheduled: "status.scheduled",
  queued: "status.queued",
  offline: "status.offline",
  error: "status.error",
};

const STATUS_COLOR: Record<string, string> = {
  monitoring: "default",
  checking: "warning",
  live: "success",
  recording: "red",
  retrying: "gold",
  scheduled: "blue",
  queued: "orange",
  offline: "default",
  error: "error",
};

export function StatusBadge({ recording, activeCount, maxConcurrent }: Props) {
  const { t } = useI18n();
  const status = getStatusKey(recording, activeCount, maxConcurrent);
  const color = STATUS_COLOR[status] || STATUS_COLOR.offline;
  const label = t(STATUS_KEY_TO_LABEL[status] || STATUS_KEY_TO_LABEL.offline);
  return <Tag color={color}>{label}</Tag>;
}
