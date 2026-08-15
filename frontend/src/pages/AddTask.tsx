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
import { useI18n } from "../i18n";

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
  const { t } = useI18n();
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
    setPlatformHint(found ? `${found.icon} ${t(`platform.${found.key}`)}` : t("add.customStream"));
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
      message.success(t("add.success"));
      form.resetFields();
      setPlatformHint("");
      onSuccess();
    } catch (e) {
      message.error(t("add.fail", { error: String(e) }));
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
          label={t("add.urlLabel")}
          name="url"
          rules={[{ required: true, message: t("add.urlRequired") }]}
          extra={platformHint}
        >
          <Input
            prefix={<LinkOutlined />}
            placeholder={t("add.urlPlaceholder")}
            onChange={handleUrlChange}
          />
        </Form.Item>

        <Space style={{ display: "flex", marginBottom: 0 }} size="large">
          <Form.Item label={t("add.quality")} name="quality" style={{ flex: 1, minWidth: 160 }}>
            <Select
              options={QUALITY_OPTIONS.map((q) => ({
                value: q.value,
                label: t(`quality.${q.value}`),
              }))}
            />
          </Form.Item>

          <Form.Item label={t("add.format")} name="format" style={{ flex: 1, minWidth: 160 }}>
            <Select
              options={FORMAT_OPTIONS.map((f) => ({
                value: f.value,
                label: t(`format.${f.value}`),
              }))}
            />
          </Form.Item>
        </Space>

        <Card
          type="inner"
          title={t("add.timeWindowTitle")}
          style={{ marginBottom: 16 }}
          size="small"
          extra={t("add.timeWindowHint")}
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
                        placeholder={[t("add.start"), t("add.end")]}
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
                  {t("add.addTimeWindow")}
                </Button>
              </>
            )}
          </Form.List>
        </Card>

        <Form.Item label={t("add.monitorNow")} name="monitor" valuePropName="checked">
          <Switch />
        </Form.Item>

        <Form.Item>
          <Button type="primary" htmlType="submit" block loading={submitting}>
            {t("add.confirm")}
          </Button>
        </Form.Item>
      </Form>
    </Card>
  );
}
