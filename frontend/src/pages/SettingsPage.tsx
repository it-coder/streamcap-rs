// 设置页 — 手风琴分组：存储与目录 / 默认录制参数 / 失败重试 / 格式转换 / 网络与通知 / 平台 Cookie

import { useEffect, useState } from "react";
import {
  Form,
  Input,
  InputNumber,
  Switch,
  Button,
  Card,
  Select,
  Collapse,
  message,
  Spin,
} from "antd";
import type { AppSettings } from "../types";
import { FORMAT_OPTIONS, QUALITY_OPTIONS, PLATFORM_PATTERNS } from "../types";
import { useI18n } from "../i18n";
import { api } from "../api/provider";

const { TextArea } = Input;

/** 需要配置 Cookie 的平台 key（与后端 detect_platform 对应，label 走 i18n platform.*） */
const COOKIE_PLATFORM_KEYS = PLATFORM_PATTERNS.map((p) => p.key);

/** 组内小标题：在不新增分组的前提下保持「目录结构」「分段」等子项的可读性 */
function SubTitle({ text }: { text: string }) {
  return (
    <div
      style={{
        fontWeight: 600,
        fontSize: 13,
        color: "#555",
        margin: "8px 0 4px",
      }}
    >
      {text}
    </div>
  );
}

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
  const [cleaning, setCleaning] = useState(false);

  const handleCleanup = async () => {
    try {
      setCleaning(true);
      const deleted = await api.runCleanup();
      if (deleted > 0) {
        message.success(t("settings.cleanupDone", { n: deleted }));
      } else {
        message.info(t("settings.cleanupNothing"));
      }
    } catch (e) {
      message.error(t("settings.cleanupFail", { error: String(e) }));
    } finally {
      setCleaning(false);
    }
  };

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

  // 平台 Cookie：每个平台一个嵌套折叠面板，默认全部折叠，避免 9 个文本框一次性铺开
  const cookieItems = COOKIE_PLATFORM_KEYS.map((key) => ({
    key,
    label: t(`platform.${key}`),
    children: (
      <Form.Item name={["cookies_by_platform", key]} noStyle>
        <TextArea
          rows={2}
          placeholder={t("settings.cookiePlaceholder", {
            label: t(`platform.${key}`),
          })}
          autoSize={{ minRows: 1, maxRows: 3 }}
        />
      </Form.Item>
    ),
  }));

  const collapseItems = [
    {
      key: "storage",
      label: t("settings.groupStorage"),
      children: (
        <>
          <Form.Item
            label={t("settings.outputDir")}
            name="output_dir"
            tooltip={t("settings.outputDirTooltip")}
          >
            <Input placeholder={t("settings.outputDirPlaceholder")} />
          </Form.Item>

          <Form.Item
            label={t("settings.diskThreshold")}
            name="recording_space_threshold_gb"
            tooltip={t("settings.diskThresholdTooltip")}
          >
            <InputNumber min={0} style={{ width: "100%" }} />
          </Form.Item>

          <SubTitle text={t("settings.concurrency")} />
          <Form.Item
            label={t("settings.maxConcurrent")}
            name="max_concurrent_recordings"
            tooltip={t("settings.maxConcurrentTooltip")}
          >
            <InputNumber min={0} max={100} style={{ width: "100%" }} placeholder="3" />
          </Form.Item>

          <SubTitle text={t("settings.cleanup")} />
          <Form.Item
            label={t("settings.autoCleanup")}
            name="auto_cleanup"
            valuePropName="checked"
            tooltip={t("settings.autoCleanupTooltip")}
          >
            <Switch />
          </Form.Item>
          <Button
            loading={cleaning}
            onClick={handleCleanup}
            style={{ marginBottom: 8 }}
          >
            {t("settings.cleanupNow")}
          </Button>

          <SubTitle text={t("settings.dirStructure")} />
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

          <SubTitle text={t("settings.segment")} />
          <Form.Item
            label={t("settings.segmentDuration")}
            name="segment_duration_seconds"
            tooltip={t("settings.segmentDurationTooltip")}
          >
            <InputNumber min={0} max={86400} style={{ width: "100%" }} placeholder="1800" />
          </Form.Item>
        </>
      ),
    },
    {
      key: "recording",
      label: t("settings.defaults"),
      children: (
        <>
          <Form.Item
            label={t("settings.detectInterval")}
            name="loop_interval_seconds"
            tooltip={t("settings.detectIntervalTooltip")}
          >
            <InputNumber min={30} max={3600} style={{ width: "100%" }} />
          </Form.Item>
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
        </>
      ),
    },
    {
      key: "retry",
      label: t("settings.retry"),
      children: (
        <>
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
        </>
      ),
    },
    {
      key: "conversion",
      label: t("settings.conversion"),
      children: (
        <>
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
        </>
      ),
    },
    {
      key: "network",
      label: t("settings.groupNetwork"),
      children: (
        <>
          <Form.Item label={t("settings.enableProxy")} name="enable_proxy" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label={t("settings.proxyUrl")} name="proxy_url">
            <Input
              placeholder={t("settings.proxyUrlPlaceholder")}
              disabled={!enableProxy}
            />
          </Form.Item>
          <Form.Item
            label={t("settings.webhookUrl")}
            name="webhook_url"
            tooltip={t("settings.webhookTooltip")}
          >
            <Input placeholder={t("settings.webhookUrlPlaceholder")} allowClear />
          </Form.Item>
        </>
      ),
    },
    {
      key: "desktop",
      label: t("settings.groupDesktop"),
      children: (
        <>
          <Form.Item
            label={t("settings.minimizeToTray")}
            name="minimize_to_tray"
            valuePropName="checked"
            tooltip={t("settings.minimizeToTrayTooltip")}
          >
            <Switch />
          </Form.Item>
          <Form.Item
            label={t("settings.autoLaunch")}
            name="auto_launch"
            valuePropName="checked"
            tooltip={t("settings.autoLaunchTooltip")}
          >
            <Switch />
          </Form.Item>
        </>
      ),
    },
    {
      key: "ffmpeg",
      label: t("settings.groupFfmpeg"),
      children: (
        <>
          <Form.Item
            label={t("settings.ffmpegPath")}
            name="ffmpeg_path"
            tooltip={t("settings.ffmpegPathTooltip")}
          >
            <Input placeholder={t("settings.ffmpegPathPlaceholder")} allowClear />
          </Form.Item>
          <div style={{ marginTop: -8, marginBottom: 16, fontSize: 12, color: "#6B7280" }}>
            {t("settings.ffmpegPathHint")}
          </div>
        </>
      ),
    },
    {
      key: "cookie",
      label: t("settings.cookie"),
      extra: t("settings.cookieHint"),
      children: (
        <Collapse
          items={cookieItems}
          size="small"
          defaultActiveKey={[]}
          ghost
        />
      ),
    },
  ];

  return (
    <Card style={{ maxWidth: 600 }}>
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        initialValues={settings}
      >
        <Collapse items={collapseItems} defaultActiveKey={["storage", "recording"]} />

        <div
          style={{
            position: "sticky",
            bottom: 0,
            marginTop: 16,
            padding: "12px 0 0",
            background: "#fff",
            borderTop: "1px solid #f0f0f0",
          }}
        >
          <Button type="primary" htmlType="submit" block>
            {t("settings.save")}
          </Button>
        </div>
      </Form>
    </Card>
  );
}
