// 录制卡片组件 — 展示单个录制任务信息和操作按钮

import { Card, Tag, Space, Button, Popconfirm, Tooltip, message } from "antd";
import {
  PlayCircleOutlined,
  PauseCircleOutlined,
  VideoCameraOutlined,
  StopOutlined,
  DeleteOutlined,
} from "@ant-design/icons";
import { useEffect, useState } from "react";
import { StatusBadge } from "./StatusBadge";
import type { RecordingConfig, RecordingProgress } from "../types";
import { api } from "../api/provider";
import { useI18n } from "../i18n";

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
  const { t } = useI18n();
  const [thumbUrl, setThumbUrl] = useState<string | null>(null);

  // 解析缩略图地址（server 模式返回 URL，desktop 模式返回 data URL）
  useEffect(() => {
    let cancelled = false;
    if (recording.thumbnail) {
      api
        .getThumbnail(recording.thumbnail)
        .then((u) => {
          if (!cancelled) setThumbUrl(u);
        })
        .catch(() => {
          if (!cancelled) setThumbUrl(null);
        });
    } else {
      setThumbUrl(null);
    }
    return () => {
      cancelled = true;
    };
  }, [recording.thumbnail]);

  const qualityLabel = t(`quality.${recording.quality}`);

  const showError = (actionKey: string, e: unknown) => {
    message.error(
      t("card.opFail", { action: t(`op.${actionKey}`), error: String(e) }),
    );
  };

  const handleToggleMonitor = async () => {
    try {
      await onToggleMonitor(recording.id, !recording.monitor_enabled);
    } catch (e) {
      showError("operation", e);
    }
  };

  const handleStartRecord = async () => {
    try {
      await onStartRecording(recording.id);
      message.success(t("card.started"));
    } catch (e) {
      showError("startRecord", e);
    }
  };

  const handleStopRecord = async () => {
    try {
      await onStopRecording(recording.id);
      message.success(t("card.stopped"));
    } catch (e) {
      showError("stopRecord", e);
    }
  };

  const handleDelete = async () => {
    try {
      await onDelete(recording.id);
      message.success(t("card.deleted"));
    } catch (e) {
      showError("delete", e);
    }
  };

  return (
    <Card
      size="medium"
      hoverable
      style={{ height: "100%" }}
      cover={
        thumbUrl ? (
          <img
            src={thumbUrl}
            alt="thumbnail"
            style={{
              width: "100%",
              height: 140,
              objectFit: "cover",
              background: "#000",
            }}
          />
        ) : undefined
      }
      title={
        <Space>
          <Tag color="blue">{recording.platform || t("card.unknownPlatform")}</Tag>
          <StatusBadge recording={recording} />
        </Space>
      }
      extra={
        recording.monitor_enabled ? (
          <Tag color="processing">{t("card.monitoring")}</Tag>
        ) : (
          <Tag>{t("card.paused")}</Tag>
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
        {recording.title || t("card.waitDetect")}
      </div>

      <div style={{ marginBottom: 12 }}>
        <Space size="middle" style={{ color: "#6B7280", fontSize: 12 }}>
          <span>📹 {qualityLabel}</span>
          {recording.recording_started_at && (
            <Tooltip
              title={`${t("card.startAt")}: ${new Date(
                recording.recording_started_at,
              ).toLocaleString()}`}
            >
              <span>
                ⏱️ {new Date(recording.recording_started_at).toLocaleTimeString()}
              </span>
            </Tooltip>
          )}
          {recording.schedule.length > 0 && (
            <Tooltip title={t("card.timeWindow")}>
              <span>
                ⏰ {recording.schedule.map((s) => `${s.start}-${s.end}`).join(", ")}
              </span>
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
            <span>
              ⚡{" "}
              {progress.download_speed_kbps > 0
                ? (progress.download_speed_kbps / 1024).toFixed(2) + " MB/s"
                : "-"}
            </span>
          </Space>
        </div>
      )}

      {recording.error_message && (
        <div style={{ color: "#EF4444", fontSize: 12, marginBottom: 8 }}>
          ⚠️ {recording.error_message}
        </div>
      )}

      {recording.is_recording && recording.retry_count > 0 && (
        <div style={{ color: "#D97706", fontSize: 12, marginBottom: 8 }}>
          🔄 {t("card.retryMsg", { n: recording.retry_count })}
        </div>
      )}

      <Space wrap>
        {recording.monitor_enabled ? (
          <Button
            size="small"
            icon={<PauseCircleOutlined />}
            onClick={handleToggleMonitor}
          >
            {t("card.pauseMonitor")}
          </Button>
        ) : (
          <Button
            size="small"
            type="primary"
            ghost
            icon={<PlayCircleOutlined />}
            onClick={handleToggleMonitor}
          >
            {t("card.startMonitor")}
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
            {t("card.startRecording")}
          </Button>
        )}

        {recording.is_recording && (
          <Button
            size="small"
            danger
            icon={<StopOutlined />}
            onClick={handleStopRecord}
          >
            {t("card.stopRecording")}
          </Button>
        )}

        <Popconfirm
          title={t("card.deleteConfirmTitle")}
          onConfirm={handleDelete}
          okText={t("card.delete")}
          cancelText={t("card.cancel")}
          okButtonProps={{ danger: true }}
        >
          <Button size="small" icon={<DeleteOutlined />}>
            {t("card.delete")}
          </Button>
        </Popconfirm>
      </Space>
    </Card>
  );
}
