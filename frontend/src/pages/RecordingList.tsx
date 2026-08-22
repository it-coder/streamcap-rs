// 录制列表页 — 卡片网格展示所有录制任务（支持搜索与状态筛选）

import { useMemo, useState } from "react";
import { Row, Col, Empty, Spin, Input, Select, Space } from "antd";
import { SearchOutlined } from "@ant-design/icons";
import { RecordingCard } from "../components/RecordingCard";
import type { UseRecordingsResult } from "../hooks/useRecordings";
import type { AppSettings, RecordingConfig } from "../types";
import { useI18n } from "../i18n";

/** 推导录制状态 key（与 StatusBadge 保持一致） */
function statusKey(r: RecordingConfig): string {
  if (r.is_recording && r.retry_count > 0) return "retrying";
  if (r.is_recording) return "recording";
  if (r.is_live) return "live";
  if (r.monitor_enabled) return "monitoring";
  if (r.error_message) return "error";
  return "offline";
}

interface Props {
  recordingsHook: UseRecordingsResult;
  onNavigateToAdd: () => void;
  onEdit: (recording: RecordingConfig) => void;
  settings?: AppSettings | null;
}

export function RecordingList({ recordingsHook, onNavigateToAdd, onEdit, settings }: Props) {
  const { t } = useI18n();
  const {
    recordings,
    progressMap,
    loading,
    toggleMonitor,
    startRecording,
    stopRecording,
    removeRecording,
  } = recordingsHook;

  // 当前正在录制的路数 + 最大并发（用于状态徽章判断"排队中"）
  const activeCount = recordings.filter((r) => r.is_recording).length;
  const maxConcurrent = settings?.max_concurrent_recordings ?? 0;

  const [keyword, setKeyword] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");

  const statusFilters = [
    { value: "all", label: t("filter.all") },
    { value: "recording", label: t("filter.recording") },
    { value: "live", label: t("filter.live") },
    { value: "monitoring", label: t("filter.monitoring") },
    { value: "retrying", label: t("filter.retrying") },
    { value: "error", label: t("filter.error") },
    { value: "offline", label: t("filter.offline") },
  ];

  const filtered = useMemo(() => {
    const kw = keyword.trim().toLowerCase();
    return recordings.filter((r) => {
      if (statusFilter !== "all" && statusKey(r) !== statusFilter) return false;
      if (!kw) return true;
      return (
        r.anchor_name.toLowerCase().includes(kw) ||
        r.title.toLowerCase().includes(kw) ||
        r.url.toLowerCase().includes(kw) ||
        r.platform.toLowerCase().includes(kw)
      );
    });
  }, [recordings, keyword, statusFilter]);

  if (loading) {
    return (
      <div style={{ textAlign: "center", padding: 60 }}>
        <Spin size="large" />
      </div>
    );
  }

  if (recordings.length === 0) {
    return (
      <Empty
        image={Empty.PRESENTED_IMAGE_SIMPLE}
        description={
          <span>
            {t("list.emptyTitle")}
            <br />
            <a onClick={onNavigateToAdd}>{t("list.emptyAction")}</a>
          </span>
        }
      />
    );
  }

  return (
    <>
      <Space
        style={{ marginBottom: 16, width: "100%", justifyContent: "space-between" }}
        wrap
      >
        <Input
          allowClear
          prefix={<SearchOutlined />}
          placeholder={t("list.searchPlaceholder")}
          value={keyword}
          onChange={(e) => setKeyword(e.target.value)}
          style={{ width: 280 }}
        />
        <Select
          value={statusFilter}
          onChange={setStatusFilter}
          options={statusFilters}
          style={{ width: 140 }}
        />
      </Space>

      {filtered.length === 0 ? (
        <Empty description={t("list.noMatch")} />
      ) : (
        <Row gutter={[16, 16]}>
          {filtered.map((r: RecordingConfig) => (
            <Col key={r.id} xs={24} sm={12} md={12} lg={8} xl={8}>
              <RecordingCard
                recording={r}
                progress={progressMap[r.id]}
                activeCount={activeCount}
                maxConcurrent={maxConcurrent}
                onToggleMonitor={toggleMonitor}
                onStartRecording={startRecording}
                onStopRecording={stopRecording}
                onDelete={removeRecording}
                onEdit={onEdit}
              />
            </Col>
          ))}
        </Row>
      )}
    </>
  );
}
