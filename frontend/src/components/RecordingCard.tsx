// 录制卡片组件 — 展示单个录制任务信息和操作按钮

import { Card, Tag, Space, Button, Popconfirm, Tooltip, message } from "antd";
import {
  PlayCircleOutlined,
  PauseCircleOutlined,
  VideoCameraOutlined,
  StopOutlined,
  DeleteOutlined,
} from "@ant-design/icons";
import { StatusBadge } from "./StatusBadge";
import type { RecordingConfig, RecordingProgress } from "../types";
import { QUALITY_OPTIONS } from "../types";

interface Props {
  recording: RecordingConfig;
  progress?: RecordingProgress;
  onToggleMonitor: (id: string, enabled: boolean) => Promise<void>;
  onStartRecording: (id: string) => Promise<void>;
  onStopRecording: (id: string) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

/** 格式化时长秒为 HH:MM:SS */
function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  return [h, m, s].map((v) => String(v).padStart(2, "0")).join(":");
}

/** 格式化文件大小 */
function formatSize(bytes: number): string {
  if (bytes >= 1_073_741_824) return (bytes / 1_073_741_824).toFixed(2) + " GB";
  if (bytes >= 1_048_576) return (bytes / 1_048_576).toFixed(1) + " MB";
  if (bytes >= 1024) return (bytes / 1024).toFixed(0) + " KB";
  return bytes + " B";
}

export function RecordingCard({
  recording,
  progress,
  onToggleMonitor,
  onStartRecording,
  onStopRecording,
  onDelete,
}: Props) {
  const qualityLabel =
    QUALITY_OPTIONS.find((q) => q.value === recording.quality)?.label || "原画";

  const showError = (action: string, e: unknown) => {
    message.error(`${action}失败: ${e}`);
  };

  const handleToggleMonitor = async () => {
    try {
      await onToggleMonitor(recording.id, !recording.monitor_enabled);
    } catch (e) {
      showError("操作", e);
    }
  };

  const handleStartRecord = async () => {
    try {
      await onStartRecording(recording.id);
      message.success("录制已启动");
    } catch (e) {
      showError("启动录制", e);
    }
  };

  const handleStopRecord = async () => {
    try {
      await onStopRecording(recording.id);
      message.success("录制已停止");
    } catch (e) {
      showError("停止录制", e);
    }
  };

  const handleDelete = async () => {
    try {
      await onDelete(recording.id);
      message.success("已删除");
    } catch (e) {
      showError("删除", e);
    }
  };

  return (
    <Card
      size="medium"
      hoverable
      style={{ height: "100%" }}
      title={
        <Space>
          <Tag color="blue">{recording.platform || "未识别"}</Tag>
          <StatusBadge recording={recording} />
        </Space>
      }
      extra={
        recording.monitor_enabled ? (
          <Tag color="processing">🔔 监控中</Tag>
        ) : (
          <Tag>⏸️ 已暂停</Tag>
        )
      }
    >
      <div style={{ marginBottom: 8 }}>
        <strong style={{ fontSize: 15 }}>
          {recording.anchor_name || recording.url}
        </strong>
      </div>
      <div
        style={{
          color: "#6B7280",
          fontSize: 13,
          marginBottom: 12,
          whiteSpace: "nowrap",
          overflow: "hidden",
          textOverflow: "ellipsis",
        }}
      >
        {recording.title || "等待检测..."}
      </div>

      <div style={{ marginBottom: 12 }}>
        <Space size="middle" style={{ color: "#6B7280", fontSize: 12 }}>
          <span>📹 {qualityLabel}</span>
          {recording.recording_started_at && (
            <Tooltip title={`开始: ${new Date(recording.recording_started_at).toLocaleString()}`}>
              <span>⏱️ {new Date(recording.recording_started_at).toLocaleTimeString()}</span>
            </Tooltip>
          )}
          {recording.schedule.length > 0 && (
            <Tooltip title="录制时间窗口">
              <span>⏰ {recording.schedule.map((s) => `${s.start}-${s.end}`).join(", ")}</span>
            </Tooltip>
          )}
        </Space>
      </div>

      {recording.is_recording && progress && (
        <div
          style={{
            background: "#f0f9ff",
            borderRadius: 6,
            padding: "6px 10px",
            marginBottom: 12,
            fontSize: 12,
            color: "#374151",
          }}
        >
          <Space size="middle">
            <span>⏱️ {formatDuration(progress.duration_seconds)}</span>
            <span>📦 {formatSize(progress.file_size_bytes)}</span>
            <span>⚡ {progress.download_speed_kbps > 0 ? (progress.download_speed_kbps / 1024).toFixed(2) + " MB/s" : "-"}</span>
          </Space>
        </div>
      )}

      {recording.error_message && (
        <div style={{ color: "#EF4444", fontSize: 12, marginBottom: 8 }}>
          ⚠️ {recording.error_message}
        </div>
      )}

      <Space wrap>
        {recording.monitor_enabled ? (
          <Button
            size="small"
            icon={<PauseCircleOutlined />}
            onClick={handleToggleMonitor}
          >
            暂停监控
          </Button>
        ) : (
          <Button
            size="small"
            type="primary"
            ghost
            icon={<PlayCircleOutlined />}
            onClick={handleToggleMonitor}
          >
            开始监控
          </Button>
        )}

        {!recording.is_recording && recording.is_live && (
          <Button
            size="small"
            type="primary"
            danger
            icon={<VideoCameraOutlined />}
            onClick={handleStartRecord}
          >
            开始录制
          </Button>
        )}

        {recording.is_recording && (
          <Button
            size="small"
            danger
            icon={<StopOutlined />}
            onClick={handleStopRecord}
          >
            停止录制
          </Button>
        )}

        <Popconfirm
          title="确定删除此录制任务？"
          onConfirm={handleDelete}
          okText="删除"
          cancelText="取消"
          okButtonProps={{ danger: true }}
        >
          <Button size="small" icon={<DeleteOutlined />}>
            删除
          </Button>
        </Popconfirm>
      </Space>
    </Card>
  );
}
