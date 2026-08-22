// 编辑任务弹窗 — 复用添加表单的字段，并补充可编辑的「主播名 / 标题」，
// 录制进行中锁定直播间地址（url）以免破坏正在运行的 FFmpeg 进程。

import { useState } from "react";
import {
  Modal,
  Form,
  Input,
  Select,
  Switch,
  Button,
  Card,
  Space,
  TimePicker,
  DatePicker,
  message,
} from "antd";
import { LinkOutlined, PlusOutlined, MinusCircleOutlined } from "@ant-design/icons";
import dayjs, { type Dayjs } from "dayjs";
import type {
  VideoQuality,
  OutputFormat,
  RecordingConfig,
  TimeRange,
} from "../types";
import { QUALITY_OPTIONS, FORMAT_OPTIONS, PLATFORM_PATTERNS } from "../types";
import { useI18n } from "../i18n";

interface Props {
  entry: RecordingConfig;
  onSave: (config: RecordingConfig) => Promise<void>;
  onClose: () => void;
}

interface FormValues {
  anchor_name: string;
  title: string;
  url: string;
  quality: VideoQuality;
  format: OutputFormat;
  monitor: boolean;
  schedule?: { range: [Dayjs, Dayjs] }[];
  scheduledStart?: Dayjs;
  recurrence?: string;
}

/** 根据 URL 推断平台标识（与前端检测规则保持一致） */
function detectPlatformKey(url: string): string {
  const found = PLATFORM_PATTERNS.find((p) => url.includes(p.pattern));
  return found ? found.key : "custom";
}

export function EditTask({ entry, onSave, onClose }: Props) {
  const { t } = useI18n();
  const [form] = Form.useForm<FormValues>();
  const [submitting, setSubmitting] = useState(false);

  // 录制进行中：锁定直播间地址（与活动 FFmpeg 进程绑定）
  const locked = entry.is_recording;

  const initialSchedule = entry.schedule.map((s) => ({
    range: [dayjs(s.start, "HH:mm"), dayjs(s.end, "HH:mm")] as [Dayjs, Dayjs],
  }));

  const handleSubmit = async (values: FormValues) => {
    setSubmitting(true);
    try {
      const schedule: TimeRange[] | undefined =
        values.schedule && values.schedule.length > 0
          ? values.schedule
              .filter((s) => s && s.range && s.range[0] && s.range[1])
              .map((s) => ({
                start: s.range[0].format("HH:mm"),
                end: s.range[1].format("HH:mm"),
              }))
          : undefined;

      const scheduledStart: string | null = values.scheduledStart
        ? values.scheduledStart.toISOString()
        : null;
      const recurrence: string | null = values.recurrence || "once";

      // 合并：保留运行时字段（来自 entry），仅覆盖用户可编辑字段
      const updated: RecordingConfig = {
        ...entry,
        anchor_name: values.anchor_name?.trim() ?? "",
        title: values.title?.trim() ?? "",
        // 录制中锁定 url / platform / output_dir，保持原值
        url: locked ? entry.url : values.url?.trim() ?? entry.url,
        platform: locked
          ? entry.platform
          : detectPlatformKey(values.url?.trim() ?? entry.url),
        quality: values.quality,
        output_format: values.format,
        monitor_enabled: values.monitor,
        schedule: schedule ?? [],
        scheduled_start: scheduledStart,
        recurrence: recurrence as RecordingConfig["recurrence"],
      };

      await onSave(updated);
      message.success(t("edit.success"));
      onClose();
    } catch (e) {
      message.error(t("edit.fail", { error: String(e) }));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal
      title={t("edit.title")}
      open
      onCancel={onClose}
      footer={null}
      destroyOnClose
      width={600}
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        initialValues={{
          anchor_name: entry.anchor_name,
          title: entry.title,
          url: entry.url,
          quality: entry.quality,
          format: entry.output_format,
          monitor: entry.monitor_enabled,
          schedule: initialSchedule,
          scheduledStart: entry.scheduled_start ? dayjs(entry.scheduled_start) : undefined,
          recurrence: entry.recurrence ?? "once",
        }}
      >
        <Form.Item label={t("edit.anchorName")} name="anchor_name">
          <Input placeholder={t("edit.anchorNamePlaceholder")} allowClear />
        </Form.Item>

        <Form.Item label={t("edit.titleLabel")} name="title">
          <Input placeholder={t("edit.titlePlaceholder")} allowClear />
        </Form.Item>

        <Form.Item
          label={t("add.urlLabel")}
          name="url"
          rules={[{ required: true, message: t("add.urlRequired") }]}
          extra={
            locked ? (
              <span style={{ color: "#D97706" }}>{t("edit.urlLockedHint")}</span>
            ) : undefined
          }
        >
          <Input
            prefix={<LinkOutlined />}
            placeholder={t("add.urlPlaceholder")}
            disabled={locked}
          />
        </Form.Item>

        <Space style={{ display: "flex", marginBottom: 0 }} size="large">
          <Form.Item
            label={t("add.quality")}
            name="quality"
            style={{ flex: 1, minWidth: 160 }}
          >
            <Select
              options={QUALITY_OPTIONS.map((q) => ({
                value: q.value,
                label: t(`quality.${q.value}`),
              }))}
            />
          </Form.Item>

          <Form.Item
            label={t("add.format")}
            name="format"
            style={{ flex: 1, minWidth: 160 }}
          >
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
                    <Form.Item name={[name, "range"]} style={{ marginBottom: 0 }}>
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

        <Card
          type="inner"
          title={t("add.scheduleRecordTitle")}
          style={{ marginBottom: 16 }}
          size="small"
          extra={t("add.scheduleRecordHint")}
        >
          <Form.Item label={t("add.scheduledStart")} name="scheduledStart">
            <DatePicker
              showTime
              format="YYYY-MM-DD HH:mm"
              placeholder={t("add.scheduledStartPlaceholder")}
              style={{ width: "100%" }}
            />
          </Form.Item>
          <Form.Item label={t("add.recurrence")} name="recurrence">
            <Select
              options={[
                { value: "once", label: t("add.recurrenceOnce") },
                { value: "daily", label: t("add.recurrenceDaily") },
                { value: "weekly", label: t("add.recurrenceWeekly") },
              ]}
            />
          </Form.Item>
        </Card>

        <Form.Item>
          <Button type="primary" htmlType="submit" block loading={submitting}>
            {t("edit.save")}
          </Button>
        </Form.Item>
      </Form>
    </Modal>
  );
}
