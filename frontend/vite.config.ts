import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri 期望资源从相对路径加载，base 设为相对路径
export default defineConfig({
  plugins: [react()],
  // 生产构建用相对路径，适配 Tauri 的 custom protocol
  base: "./",
  // Tauri dev 模式下前端跑在 1420 端口
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    // 输出到 dist 目录，tauri.conf.json 的 frontendDist 指向这里
    outDir: "dist",
    emptyOutDir: true,
    // 静态资源内联（小于 4KB），减少 Tauri 打包文件数
    assetsInlineLimit: 4096,
  },
});
