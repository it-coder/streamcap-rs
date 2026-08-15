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
import { FORMAT_OPTIONS } from "../types";
import { useI18n } from "../i18n";

const { TextArea } = Input;

/** 平台列表（key 对应后端 detect_platform 返回的标识） */
const COOKIE_PLATFORMS = [
  { key: "douyin", label: "抖音" },
  { key: "bilibili", label: "哔哩哔哩" },
  { key: "huya", label: "虎牙" },
  { key: "kuaishou", label: "快手" },
  { key: "douyu", label: "斗鱼" },
  { key: "twitch", label: "Twitch" },
  { key: "youtube", label: "YouTube" },
  { key: "tiktok", label: "TikTok" },
  { key: "rednote", label: "小红书" },
];

interface Props {
  settings: AppSettings | null;
  loading: boolean;
  onSave: (settings: AppSettings) => Promise<void>;
}

export function SettingsPage({ settings, loading, onSave }: Props) {
  const { t } = useI18n();
  const [form] = Form.useForm<AppSettings>();
  const enableConversion = Form.useWatch("enable_conversion", form);

  useEffect(() => {
    if (settings) {
      form.setFieldsValue(settings);
    }
  }, [settings, form]);

  const handleSubmit = async (values: AppSettings) => {
    try {
      await onSave(values);
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
            <Input placeholder={t("settings.proxyUrlPlaceholder")} />
          </Form.Item>
        </Card>

        <Card
          type="inner"
          title={t("settings.cookie")}
          style={{ marginBottom: 16 }}
          extra={t("settings.cookieHint")}
        >
          {COOKIE_PLATFORMS.map((p) => (
            <Form.Item
              key={p.key}
              label={t(`platform.${p.key}`)}
              name={["cookies_by_platform", p.key]}
            >
              <TextArea
                rows={2}
                placeholder={t("settings.cookiePlaceholder", {
                  label: t(`platform.${p.key}`),
                })}
                autoSize={{ minRows: 1, maxRows: 3 }}
              />
            </Form.Item>
          ))}
        </Card>

        <Form.Item label={t("settings.defaultQuality")} name="default_quality">
          <Input placeholder="OD" disabled />
        </Form.Item>

        <Form.Item>
          <Button type="primary" htmlType="submit" block>
            {t("settings.save")}
          </Button>
        </Form.Item>
      </Form>
    </Card>
  );
}
