// 录制列表页 — 卡片网格展示所有录制任务

import { Row, Col, Empty, Spin } from "antd";
import { RecordingCard } from "../components/RecordingCard";
import type { UseRecordingsResult } from "../hooks/useRecordings";
import type { RecordingConfig } from "../types";

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
    <Row gutter={[16, 16]}>
      {recordings.map((r: RecordingConfig) => (
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
  );
}
