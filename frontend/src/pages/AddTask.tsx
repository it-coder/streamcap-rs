// 添加任务页 — URL 输入 + 平台检测 + 质量/格式选择

import { useState } from "react";
import {
  Form,
  Input,
  Select,
  Switch,
  Button,
  Card,
  Space,
  message,
} from "antd";
import { LinkOutlined } from "@ant-design/icons";
import type { VideoQuality, OutputFormat, RecordingConfig } from "../types";
import { QUALITY_OPTIONS, FORMAT_OPTIONS, PLATFORM_PATTERNS } from "../types";

interface Props {
  onAdd: (url: string, monitorEnabled: boolean, quality: VideoQuality) => Promise<RecordingConfig>;
  onSuccess: () => void;
}

export function AddTask({ onAdd, onSuccess }: Props) {
  const [form] = Form.useForm();
  const [platformHint, setPlatformHint] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const handleUrlChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const url = e.target.value.trim();
    if (!url) {
      setPlatformHint("");
      return;
    }
    const found = PLATFORM_PATTERNS.find((p) => url.includes(p.pattern));
    setPlatformHint(found ? `${found.icon} ${found.label}` : "🔗 自定义流 / M3U8 URL");
  };

  const handleSubmit = async (values: {
    url: string;
    quality: VideoQuality;
    format: OutputFormat;
    monitor: boolean;
  }) => {
    setSubmitting(true);
    try {
      await onAdd(values.url, values.monitor, values.quality);
      message.success("任务已添加");
      form.resetFields();
      setPlatformHint("");
      onSuccess();
    } catch (e) {
      message.error(`添加失败: ${e}`);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Card style={{ maxWidth: 600 }}>
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        initialValues={{
          quality: "OD" as VideoQuality,
          format: "ts" as OutputFormat,
          monitor: true,
        }}
      >
        <Form.Item
          label="直播间 URL"
          name="url"
          rules={[{ required: true, message: "请输入直播间 URL" }]}
          extra={platformHint}
        >
          <Input
            prefix={<LinkOutlined />}
            placeholder="例如: https://live.bilibili.com/12345"
            onChange={handleUrlChange}
          />
        </Form.Item>

        <Space style={{ display: "flex", marginBottom: 0 }} size="large">
          <Form.Item label="视频质量" name="quality" style={{ flex: 1, minWidth: 160 }}>
            <Select options={QUALITY_OPTIONS} />
          </Form.Item>

          <Form.Item label="输出格式" name="format" style={{ flex: 1, minWidth: 160 }}>
            <Select options={FORMAT_OPTIONS} />
          </Form.Item>
        </Space>

        <Form.Item label="添加后立即开始监控" name="monitor" valuePropName="checked">
          <Switch />
        </Form.Item>

        <Form.Item>
          <Button type="primary" htmlType="submit" block loading={submitting}>
            确认添加
          </Button>
        </Form.Item>
      </Form>
    </Card>
  );
}
