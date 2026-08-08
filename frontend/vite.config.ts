import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri 期望资源从相对路径加载，base 设为相对路径
// B/S 模式下相对路径同样适用（ServeDir 提供静态文件服务）
export default defineConfig({
  plugins: [react()],
  base: "./",
  server: {
    // Tauri dev 模式下前端跑在 1420 端口
    port: 1420,
    strictPort: true,
    // B/S 开发模式代理：将 API 和 WebSocket 请求转发到后端服务器
    // Tauri 模式下不影响（前端使用 IPC 而非 HTTP）
    proxy: {
      "/api": {
        target: "http://localhost:8080",
        changeOrigin: true,
      },
      "/ws": {
        target: "ws://localhost:8080",
        ws: true,
      },
    },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    assetsInlineLimit: 4096,
  },
});
