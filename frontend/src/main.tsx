// 应用入口 — 配置 antd 中文 locale + 主题

import React from "react";
import ReactDOM from "react-dom/client";
import { ConfigProvider, theme } from "antd";
import zhCN from "antd/locale/zh_CN";
import App from "./App";

// 全局样式：去除默认 margin，全高度
const globalStyle = `
* { margin: 0; padding: 0; box-sizing: border-box; }
html, body, #root { height: 100%; }
body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", sans-serif;
}
`;

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <style>{globalStyle}</style>
    <ConfigProvider
      locale={zhCN}
      theme={{
        algorithm: theme.defaultAlgorithm,
        token: {
          colorPrimary: "#3B82F6",
          borderRadius: 8,
        },
      }}
    >
      <App />
    </ConfigProvider>
  </React.StrictMode>
);
