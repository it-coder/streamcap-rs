// 应用主布局 — antd Layout + Sider 侧边栏导航

import { useState } from "react";
import { Layout, Menu, Typography, Tag, theme } from "antd";
import {
  UnorderedListOutlined,
  PlusOutlined,
  SettingOutlined,
} from "@ant-design/icons";
import { RecordingList } from "./pages/RecordingList";
import { AddTask } from "./pages/AddTask";
import { SettingsPage } from "./pages/SettingsPage";
import { ShutdownOverlay } from "./components/ShutdownOverlay";
import { useRecordings } from "./hooks/useRecordings";
import { useSettings } from "./hooks/useSettings";
import { useFfmpegStatus } from "./hooks/useFfmpegStatus";

const { Sider, Content } = Layout;
const { Title, Text } = Typography;

type PageKey = "home" | "add" | "settings";

const MENU_ITEMS = [
  { key: "home", icon: <UnorderedListOutlined />, label: "录制列表" },
  { key: "add", icon: <PlusOutlined />, label: "添加任务" },
  { key: "settings", icon: <SettingOutlined />, label: "设置" },
];

function App() {
  const [currentPage, setCurrentPage] = useState<PageKey>("home");
  const recordingsHook = useRecordings();
  const settingsHook = useSettings();
  const ffmpeg = useFfmpegStatus();
  const { token } = theme.useToken();

  const menuItems = [
    ...MENU_ITEMS,
    {
      key: "ffmpeg-status",
      label: (
        <div style={{ padding: "8px 0" }}>
          {ffmpeg.status === "available" ? (
            <Tag color="success">✅ FFmpeg 可用</Tag>
          ) : ffmpeg.status === "missing" ? (
            <Tag color="error">❌ FFmpeg 未安装</Tag>
          ) : (
            <Tag color="processing">检查 FFmpeg...</Tag>
          )}
        </div>
      ),
      disabled: true,
    },
  ];

  return (
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
          <Text type="secondary" style={{ fontSize: 12 }}>
            v0.1.0
          </Text>
        </div>
        <Menu
          mode="inline"
          selectedKeys={[currentPage]}
          items={menuItems}
          onClick={({ key }) => {
            if (key === "home" || key === "add" || key === "settings") {
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
          />
        )}
        {currentPage === "add" && (
          <AddTask
            onAdd={recordingsHook.addRecording}
            onSuccess={() => setCurrentPage("home")}
          />
        )}
        {currentPage === "settings" && (
          <SettingsPage
            settings={settingsHook.settings}
            loading={settingsHook.loading}
            onSave={settingsHook.save}
          />
        )}
      </Content>
    </Layout>
  );
}

export default App;
