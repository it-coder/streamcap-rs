// 应用关闭时的全屏遮罩提示
//
// 监听后端 emit("app:shutdown") 事件：
//   stage="start" — 显示"正在停止录制并保存状态..."
//   stage="done"  — 显示"已完成，正在退出应用..."
//
// 使用 antd Modal + Spin + Progress 实现友好提示，防止用户在关闭过程中误操作。

import { useEffect, useState } from "react";
import { Modal, Spin, Typography, Progress } from "antd";
import { LoadingOutlined, CheckCircleOutlined } from "@ant-design/icons";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

const { Text } = Typography;

interface ShutdownPayload {
  stage: "start" | "done";
  activeCount: number;
  message: string;
}

export function ShutdownOverlay() {
  const [visible, setVisible] = useState(false);
  const [payload, setPayload] = useState<ShutdownPayload | null>(null);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let cancelled = false;

    listen<ShutdownPayload>("app:shutdown", (event) => {
      if (cancelled) return;
      setPayload(event.payload);
      setVisible(true);
    }).then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const isDone = payload?.stage === "done";
  const percent = isDone ? 100 : 65;

  return (
    <Modal
      open={visible}
      closable={false}
      maskClosable={false}
      keyboard={false}
      footer={null}
      width={420}
      centered
      styles={{ mask: { background: "rgba(0, 0, 0, 0.55)" } }}
    >
      <div style={{ textAlign: "center", padding: "12px 0 4px" }}>
        {isDone ? (
          <CheckCircleOutlined
            style={{ fontSize: 44, color: "#52C41A", marginBottom: 16 }}
          />
        ) : (
          <Spin
            indicator={<LoadingOutlined style={{ fontSize: 44 }} />}
            size="large"
          />
        )}

        <div style={{ marginTop: 16, marginBottom: 8 }}>
          <Text strong style={{ fontSize: 16 }}>
            {isDone ? "安全关闭完成" : "正在安全关闭应用"}
          </Text>
        </div>

        <Text type="secondary" style={{ fontSize: 13 }}>
          {payload?.message || "请稍候..."}
        </Text>

        <Progress
          percent={percent}
          status={isDone ? "success" : "active"}
          showInfo={false}
          strokeColor={isDone ? "#52C41A" : "#1677FF"}
          style={{ marginTop: 20, marginBottom: 0 }}
        />

        {!isDone && payload?.activeCount && payload.activeCount > 0 && (
          <Text
            type="secondary"
            style={{ fontSize: 12, display: "block", marginTop: 12 }}
          >
            ⚠️ 检测到 {payload.activeCount} 个录制任务正在运行，正在发送停止信号并等待 FFmpeg 优雅退出...
          </Text>
        )}
      </div>
    </Modal>
  );
}
