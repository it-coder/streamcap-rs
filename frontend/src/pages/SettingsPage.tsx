// 设置页 — 输出目录、检测间隔、磁盘阈值、文件夹规则、代理

import { useEffect } from "react";
import {
  Form,
  Input,
  InputNumber,
  Switch,
  Button,
  Card,
  Select,
  message,
  Spin,
} from "antd";
import type { AppSettings } from "../types";
import { FORMAT_OPTIONS, QUALITY_OPTIONS, PLATFORM_PATTERNS } from "../types";
import { useI18n } from "../i18n";

const { TextArea } = Input;

/** 需要配置 Cookie 的平台 key（与后端 detect_platform 对应，label 走 i18n platform.*） */
const COOKIE_PLATFORM_KEYS = PLATFORM_PATTERNS.map((p) => p.key);

interface Props {
  settings: AppSettings | null;
  loading: boolean;
  onSave: (settings: AppSettings) => Promise<void>;
}

export function SettingsPage({ settings, loading, onSave }: Props) {
  const { t } = useI18n();
  const [form] = Form.useForm<AppSettings>();
  const enableConversion = Form.useWatch("enable_conversion", form);
  const enableProxy = Form.useWatch("enable_proxy", form);

  useEffect(() => {
    if (settings) {
      form.setFieldsValue(settings);
    }
  }, [settings, form]);

  const handleSubmit = async (values: AppSettings) => {
    try {
      // 合并原始设置：表单只回传已注册字段，未注册的字段（如 default_format）
      // 若不合并会被后端默认值覆盖而静默丢失
      const merged: AppSettings = { ...(settings as AppSettings), ...values };
      await onSave(merged);
      message.success(t("settings.saved"));
    } catch (e) {
      message.error(t("settings.saveFail", { error: String(e) }));
    }
  };

  if (loading || !settings) {
    return (
      <div style={{ textAlign: "center", padding: 60 }}>
        <Spin />
      </div>
    );
  }

  return (
    <Card style={{ maxWidth: 600 }}>
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        initialValues={settings}
      >
        <Form.Item
          label={t("settings.outputDir")}
          name="output_dir"
          tooltip={t("settings.outputDirTooltip")}
        >
          <Input placeholder={t("settings.outputDirPlaceholder")} />
        </Form.Item>

        <Form.Item
          label={t("settings.detectInterval")}
          name="loop_interval_seconds"
          tooltip={t("settings.detectIntervalTooltip")}
        >
          <InputNumber min={30} max={3600} style={{ width: "100%" }} />
        </Form.Item>

        <Form.Item
          label={t("settings.diskThreshold")}
          name="recording_space_threshold_gb"
          tooltip={t("settings.diskThresholdTooltip")}
        >
          <InputNumber min={0} style={{ width: "100%" }} />
        </Form.Item>

        <Card type="inner" title={t("settings.defaults")} style={{ marginBottom: 16 }}>
          <Form.Item label={t("settings.defaultQuality")} name="default_quality">
            <Select
              options={QUALITY_OPTIONS.map((q) => ({
                value: q.value,
                label: t(`quality.${q.value}`),
              }))}
            />
          </Form.Item>
          <Form.Item
            label={t("settings.defaultFormat")}
            name="default_format"
            tooltip={t("settings.defaultFormatTooltip")}
          >
            <Select
              options={FORMAT_OPTIONS.map((f) => ({
                value: f.value,
                label: t(`format.${f.value}`),
              }))}
            />
          </Form.Item>
        </Card>

        <Card type="inner" title={t("settings.dirStructure")} style={{ marginBottom: 16 }}>
          <Form.Item label={t("settings.byPlatform")} name="folder_by_platform" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label={t("settings.byAnchor")} name="folder_by_anchor" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label={t("settings.byDate")} name="folder_by_date" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label={t("settings.byTitle")} name="folder_by_title" valuePropName="checked">
            <Switch />
          </Form.Item>
        </Card>

        <Card type="inner" title={t("settings.segment")} style={{ marginBottom: 16 }}>
          <Form.Item
            label={t("settings.segmentDuration")}
            name="segment_duration_seconds"
            tooltip={t("settings.segmentDurationTooltip")}
          >
            <InputNumber min={0} max={86400} style={{ width: "100%" }} placeholder="1800" />
          </Form.Item>
        </Card>

        <Card type="inner" title={t("settings.retry")} style={{ marginBottom: 16 }}>
          <Form.Item
            label={t("settings.maxRetries")}
            name="max_retries"
            tooltip={t("settings.maxRetriesTooltip")}
          >
            <InputNumber min={0} max={10} style={{ width: "100%" }} />
          </Form.Item>
          <Form.Item
            label={t("settings.retryDelay")}
            name="retry_delay_seconds"
            tooltip={t("settings.retryDelayTooltip")}
          >
            <InputNumber min={1} max={120} style={{ width: "100%" }} />
          </Form.Item>
        </Card>

        <Card type="inner" title={t("settings.conversion")} style={{ marginBottom: 16 }}>
          <Form.Item
            label={t("settings.enableConversion")}
            name="enable_conversion"
            valuePropName="checked"
            tooltip={t("settings.enableConversionTooltip")}
          >
            <Switch />
          </Form.Item>
          <Form.Item
            label={t("settings.targetFormat")}
            name="conversion_format"
            tooltip={t("settings.targetFormatTooltip")}
          >
            <Select
              disabled={!enableConversion}
              options={FORMAT_OPTIONS.map((f) => ({
                value: f.value,
                label: t(`format.${f.value}`),
              }))}
            />
          </Form.Item>
          <Form.Item
            label={t("settings.deleteOriginal")}
            name="delete_original_after_conversion"
            valuePropName="checked"
            tooltip={t("settings.deleteOriginalTooltip")}
          >
            <Switch disabled={!enableConversion} />
          </Form.Item>
        </Card>

        <Card type="inner" title={t("settings.proxy")} style={{ marginBottom: 16 }}>
          <Form.Item label={t("settings.enableProxy")} name="enable_proxy" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label={t("settings.proxyUrl")} name="proxy_url">
            <Input
              placeholder={t("settings.proxyUrlPlaceholder")}
              disabled={!enableProxy}
            />
          </Form.Item>
        </Card>

        <Card type="inner" title={t("settings.webhook")} style={{ marginBottom: 16 }}>
          <Form.Item
            label={t("settings.webhookUrl")}
            name="webhook_url"
            tooltip={t("settings.webhookTooltip")}
          >
            <Input placeholder={t("settings.webhookUrlPlaceholder")} allowClear />
          </Form.Item>
        </Card>

        <Card
          type="inner"
          title={t("settings.cookie")}
          style={{ marginBottom: 16 }}
          extra={t("settings.cookieHint")}
        >
          {COOKIE_PLATFORM_KEYS.map((key) => (
            <Form.Item
              key={key}
              label={t(`platform.${key}`)}
              name={["cookies_by_platform", key]}
            >
              <TextArea
                rows={2}
                placeholder={t("settings.cookiePlaceholder", {
                  label: t(`platform.${key}`),
                })}
                autoSize={{ minRows: 1, maxRows: 3 }}
              />
            </Form.Item>
          ))}
        </Card>

        <Form.Item>
          <Button type="primary" htmlType="submit" block>
            {t("settings.save")}
          </Button>
        </Form.Item>
      </Form>
    </Card>
  );
}
