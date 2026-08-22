// 应用主布局 — antd Layout + Sider 侧边栏导航

import { useState, useEffect } from "react";
import {
  Layout,
  Menu,
  Typography,
  Tag,
  theme,
  ConfigProvider,
  Select,
} from "antd";
import {
  UnorderedListOutlined,
  PlusOutlined,
  SettingOutlined,
  FolderOpenOutlined,
  GlobalOutlined,
  HistoryOutlined,
} from "@ant-design/icons";
import zhCN from "antd/locale/zh_CN";
import enUS from "antd/locale/en_US";
import { RecordingList } from "./pages/RecordingList";
import { AddTask } from "./pages/AddTask";
import { EditTask } from "./pages/EditTask";
import { SettingsPage } from "./pages/SettingsPage";
import { FilesPage } from "./pages/FilesPage";
import { HistoryPage } from "./pages/HistoryPage";
import { ShutdownOverlay } from "./components/ShutdownOverlay";
import { useRecordings } from "./hooks/useRecordings";
import { useSettings } from "./hooks/useSettings";
import { useFfmpegStatus } from "./hooks/useFfmpegStatus";
import { api } from "./api/provider";
import type { RecordingConfig } from "./types";
import { LanguageProvider, useI18n, type Lang } from "./i18n";

const { Sider, Content } = Layout;
const { Title, Text } = Typography;

type PageKey = "home" | "add" | "settings" | "files" | "history";

function AppContent() {
  const [currentPage, setCurrentPage] = useState<PageKey>("home");
  const [version, setVersion] = useState<string>("");
  const [editing, setEditing] = useState<RecordingConfig | null>(null);
  const recordingsHook = useRecordings();
  const settingsHook = useSettings();
  const ffmpeg = useFfmpegStatus();
  const { token } = theme.useToken();
  const { t, lang, setLang } = useI18n();

  // 从后端获取版本号（单一可信源 = Cargo.toml）
  useEffect(() => {
    api
      .getVersion()
      .then((v) => setVersion(v.version))
      .catch(() => setVersion(""));
  }, []);

  const menuItems = [
    { key: "home", icon: <UnorderedListOutlined />, label: t("nav.recordingList") },
    { key: "add", icon: <PlusOutlined />, label: t("nav.addTask") },
    { key: "files", icon: <FolderOpenOutlined />, label: t("nav.files") },
    { key: "history", icon: <HistoryOutlined />, label: t("nav.history") },
    { key: "settings", icon: <SettingOutlined />, label: t("nav.settings") },
    {
      key: "ffmpeg-status",
      label: (
        <div style={{ padding: "8px 0" }}>
          {ffmpeg.status === "available" ? (
            <Tag color="success">{t("ffmpeg.available")}</Tag>
          ) : ffmpeg.status === "missing" ? (
            <Tag color="error">{t("ffmpeg.missing")}</Tag>
          ) : (
            <Tag color="processing">{t("ffmpeg.checking")}</Tag>
          )}
        </div>
      ),
      disabled: true,
    },
  ];

  return (
    <ConfigProvider locale={lang === "en" ? enUS : zhCN}>
      <Layout style={{ height: "100vh" }}>
        <ShutdownOverlay />
        <Sider
          width={240}
          style={{
            background: token.colorBgContainer,
            borderRight: `1px solid ${token.colorBorderSecondary}`,
          }}
        >
          <div style={{ padding: "20px 20px 16px" }}>
            <Title level={4} style={{ margin: 0 }}>
              🎬 StreamCap RS
            </Title>
            <div style={{ marginTop: 4 }}>
              <Text type="secondary" style={{ fontSize: 12 }}>
                {version ? `v${version}` : t("nav.loading")}
              </Text>
            </div>
            <div style={{ marginTop: 10 }}>
              <Select
                size="small"
                value={lang}
                onChange={(v) => setLang(v as Lang)}
                style={{ width: "100%" }}
                prefix={<GlobalOutlined />}
                options={[
                  { value: "zh", label: t("lang.zh") },
                  { value: "en", label: t("lang.en") },
                ]}
              />
            </div>
          </div>
          <Menu
            mode="inline"
            selectedKeys={[currentPage]}
            items={menuItems}
            onClick={({ key }) => {
              if (
                key === "home" ||
                key === "add" ||
                key === "settings" ||
                key === "files" ||
                key === "history"
              ) {
                setCurrentPage(key as PageKey);
              }
            }}
            style={{ borderRight: 0 }}
          />
        </Sider>

        <Content style={{ padding: 24, overflow: "auto" }}>
          {currentPage === "home" && (
            <RecordingList
              recordingsHook={recordingsHook}
              onNavigateToAdd={() => setCurrentPage("add")}
              onEdit={(r) => setEditing(r)}
              settings={settingsHook.settings}
            />
          )}
          {currentPage === "add" && (
            <AddTask
              onAdd={recordingsHook.addRecording}
              onSuccess={() => setCurrentPage("home")}
            />
          )}
          {currentPage === "files" && <FilesPage />}
          {currentPage === "history" && <HistoryPage />}
          {currentPage === "settings" && (
            <SettingsPage
              settings={settingsHook.settings}
              loading={settingsHook.loading}
              onSave={settingsHook.save}
            />
          )}
          {editing && (
            <EditTask
              entry={editing}
              onSave={recordingsHook.updateRecording}
              onClose={() => setEditing(null)}
            />
          )}
        </Content>
      </Layout>
    </ConfigProvider>
  );
}

function App() {
  return (
    <LanguageProvider>
      <AppContent />
    </LanguageProvider>
  );
}

export default App;
