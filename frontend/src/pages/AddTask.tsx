// 添加任务页 — URL 输入 + 平台检测 + 质量/格式选择 + 时间窗口

import { useState } from "react";
import {
  Form,
  Input,
  Select,
  Switch,
  Button,
  Card,
  Space,
  TimePicker,
  message,
} from "antd";
import { LinkOutlined, PlusOutlined, MinusCircleOutlined } from "@ant-design/icons";
import type { VideoQuality, OutputFormat, RecordingConfig, TimeRange } from "../types";
import { QUALITY_OPTIONS, FORMAT_OPTIONS, PLATFORM_PATTERNS } from "../types";

interface Props {
  onAdd: (
    url: string,
    monitorEnabled: boolean,
    quality: VideoQuality,
    schedule?: TimeRange[],
  ) => Promise<RecordingConfig>;
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
    schedule?: { range: [any, any] }[];
  }) => {
    setSubmitting(true);
    try {
      // 将 TimePicker.RangePicker 的 Dayjs 值转换为 "HH:mm" 字符串
      const schedule: TimeRange[] | undefined =
        values.schedule && values.schedule.length > 0
          ? values.schedule
              .filter((s) => s && s.range && s.range[0] && s.range[1])
              .map((s) => ({
                start: s.range[0].format("HH:mm"),
                end: s.range[1].format("HH:mm"),
              }))
          : undefined;

      await onAdd(values.url, values.monitor, values.quality, schedule);
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

        <Card
          type="inner"
          title="录制时间窗口（可选）"
          style={{ marginBottom: 16 }}
          size="small"
          extra="留空 = 全天录制"
        >
          <Form.List name="schedule">
            {(fields, { add, remove }) => (
              <>
                {fields.map(({ key, name }) => (
                  <Space key={key} style={{ display: "flex", marginBottom: 8 }} align="baseline">
                    <Form.Item
                      name={[name, "range"]}
                      style={{ marginBottom: 0 }}
                    >
                      <TimePicker.RangePicker
                        format="HH:mm"
                        minuteStep={5}
                        placeholder={["开始", "结束"]}
                      />
                    </Form.Item>
                    <MinusCircleOutlined
                      onClick={() => remove(name)}
                      style={{ color: "#999" }}
                    />
                  </Space>
                ))}
                <Button
                  type="dashed"
                  onClick={() => add({})}
                  block
                  icon={<PlusOutlined />}
                  size="small"
                >
                  添加时间窗口
                </Button>
              </>
            )}
          </Form.List>
        </Card>

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
