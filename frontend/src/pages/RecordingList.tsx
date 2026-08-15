// 录制列表页 — 卡片网格展示所有录制任务（支持搜索与状态筛选）

import { useMemo, useState } from "react";
import { Row, Col, Empty, Spin, Input, Select, Space } from "antd";
import { SearchOutlined } from "@ant-design/icons";
import { RecordingCard } from "../components/RecordingCard";
import type { UseRecordingsResult } from "../hooks/useRecordings";
import type { RecordingConfig } from "../types";

/** 推导录制状态 key（与 StatusBadge 保持一致） */
function statusKey(r: RecordingConfig): string {
  if (r.is_recording && r.retry_count > 0) return "retrying";
  if (r.is_recording) return "recording";
  if (r.is_live) return "live";
  if (r.monitor_enabled) return "monitoring";
  if (r.error_message) return "error";
  return "offline";
}

const STATUS_FILTERS = [
  { value: "all", label: "全部" },
  { value: "recording", label: "录制中" },
  { value: "live", label: "直播中" },
  { value: "monitoring", label: "监控中" },
  { value: "retrying", label: "重试中" },
  { value: "error", label: "错误" },
  { value: "offline", label: "已停止" },
];

interface Props {
  recordingsHook: UseRecordingsResult;
  onNavigateToAdd: () => void;
}

export function RecordingList({ recordingsHook, onNavigateToAdd }: Props) {
  const {
    recordings,
    progressMap,
    loading,
    toggleMonitor,
    startRecording,
    stopRecording,
    removeRecording,
  } = recordingsHook;

  const [keyword, setKeyword] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");

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
            暂无录制任务
            <br />
            <a onClick={onNavigateToAdd}>点击添加任务开始监控直播</a>
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
          placeholder="搜索主播 / 标题 / 链接 / 平台"
          value={keyword}
          onChange={(e) => setKeyword(e.target.value)}
          style={{ width: 280 }}
        />
        <Select
          value={statusFilter}
          onChange={setStatusFilter}
          options={STATUS_FILTERS}
          style={{ width: 140 }}
        />
      </Space>

      {filtered.length === 0 ? (
        <Empty description="没有匹配的录制任务" />
      ) : (
        <Row gutter={[16, 16]}>
          {filtered.map((r: RecordingConfig) => (
            <Col key={r.id} xs={24} sm={12} md={12} lg={8} xl={8}>
              <RecordingCard
                recording={r}
                progress={progressMap[r.id]}
                onToggleMonitor={toggleMonitor}
                onStartRecording={startRecording}
                onStopRecording={stopRecording}
                onDelete={removeRecording}
              />
            </Col>
          ))}
        </Row>
      )}
    </>
  );
}
