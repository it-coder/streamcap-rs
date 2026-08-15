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
      message.success("设置已保存");
    } catch (e) {
      message.error(`保存失败: ${e}`);
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
          label="输出目录"
          name="output_dir"
          tooltip="录制文件保存的根目录"
        >
          <Input placeholder="录制文件保存路径" />
        </Form.Item>

        <Form.Item
          label="检测间隔（秒）"
          name="loop_interval_seconds"
          tooltip="建议 120-300 秒，过于频繁可能导致 IP 被封"
        >
          <InputNumber min={30} max={3600} style={{ width: "100%" }} />
        </Form.Item>

        <Form.Item
          label="磁盘空间阈值（GB）"
          name="recording_space_threshold_gb"
          tooltip="低于此值自动停止录制，0 表示不限制"
        >
          <InputNumber min={0} style={{ width: "100%" }} />
        </Form.Item>

        <Card type="inner" title="目录结构" style={{ marginBottom: 16 }}>
          <Form.Item label="按平台创建子文件夹" name="folder_by_platform" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label="按主播创建子文件夹" name="folder_by_anchor" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label="按日期创建子文件夹" name="folder_by_date" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label="按标题创建子文件夹" name="folder_by_title" valuePropName="checked">
            <Switch />
          </Form.Item>
        </Card>

        <Card type="inner" title="录制分段" style={{ marginBottom: 16 }}>
          <Form.Item
            label="分段时长（秒）"
            name="segment_duration_seconds"
            tooltip="录制达到指定时长后自动切换到新文件，0 表示不分段。默认 1800 秒（30 分钟）"
          >
            <InputNumber min={0} max={86400} style={{ width: "100%" }} placeholder="1800" />
          </Form.Item>
        </Card>

        <Card type="inner" title="格式转换" style={{ marginBottom: 16 }}>
          <Form.Item
            label="启用录制后格式转换"
            name="enable_conversion"
            valuePropName="checked"
            tooltip="录制完成后自动转换为目标格式（流复制 remux，无重编码，速度快且无损）。默认关闭"
          >
            <Switch />
          </Form.Item>
          <Form.Item
            label="转换目标格式"
            name="conversion_format"
            tooltip="转换后的视频容器格式"
          >
            <Select
              disabled={!enableConversion}
              options={[
                { value: "mp4", label: "MP4" },
                { value: "mkv", label: "MKV" },
                { value: "mov", label: "MOV" },
                { value: "flv", label: "FLV" },
                { value: "ts", label: "TS" },
              ]}
            />
          </Form.Item>
          <Form.Item
            label="转换后删除原文件"
            name="delete_original_after_conversion"
            valuePropName="checked"
            tooltip="转换成功后自动删除原始录制文件。转换失败时始终保留原文件"
          >
            <Switch disabled={!enableConversion} />
          </Form.Item>
        </Card>

        <Card type="inner" title="代理设置" style={{ marginBottom: 16 }}>
          <Form.Item label="启用代理" name="enable_proxy" valuePropName="checked">
            <Switch />
          </Form.Item>
          <Form.Item label="代理地址" name="proxy_url">
            <Input placeholder="http://127.0.0.1:7890" />
          </Form.Item>
        </Card>

        <Card
          type="inner"
          title="Cookie 配置"
          style={{ marginBottom: 16 }}
          extra="按平台配置，留空则不使用"
        >
          {COOKIE_PLATFORMS.map((p) => (
            <Form.Item
              key={p.key}
              label={p.label}
              name={["cookies_by_platform", p.key]}
            >
              <TextArea
                rows={2}
                placeholder={`粘贴 ${p.label} 的 Cookie（如 SESSDATA=xxx; ...）`}
                autoSize={{ minRows: 1, maxRows: 3 }}
              />
            </Form.Item>
          ))}
        </Card>

        <Form.Item label="默认视频质量" name="default_quality">
          <Input placeholder="OD" disabled />
        </Form.Item>

        <Form.Item>
          <Button type="primary" htmlType="submit" block>
            保存设置
          </Button>
        </Form.Item>
      </Form>
    </Card>
  );
}
