// API Provider 接口 + 自动模式检测
//
// 运行时自动检测当前环境：
// - 桌面客户端模式 (window.__TAURI__ 存在) → TauriApiProvider (invoke + listen)
// - B/S 服务器模式 (浏览器) → HttpApiProvider (fetch + WebSocket)

import type { RecordingConfig, AppSettings, VideoQuality, ShutdownPayload } from "../types";

export type { ShutdownPayload } from "../types";

export interface ApiProvider {
  // 录制任务
  listRecordings(): Promise<RecordingConfig[]>;
  addRecording(params: {
    url: string;
    monitorEnabled?: boolean;
    quality?: VideoQuality;
  }): Promise<RecordingConfig>;
  removeRecording(id: string): Promise<void>;
  updateRecording(config: RecordingConfig): Promise<void>;
  startMonitor(id: string): Promise<void>;
  stopMonitor(id: string): Promise<void>;
  startRecording(id: string): Promise<void>;
  stopRecording(id: string): Promise<void>;
  getRecordingStatus(id: string): Promise<RecordingConfig | null>;

  // 设置
  getSettings(): Promise<AppSettings>;
  updateSettings(settings: AppSettings): Promise<void>;
  checkFfmpeg(): Promise<string>;

  // 事件订阅 — 返回取消订阅函数
  onStatusChange(callback: (status: RecordingConfig) => void): Promise<() => void>;
  onShutdown(callback: (payload: ShutdownPayload) => void): Promise<() => void>;
}

// 自动检测运行环境
const isTauri =
  typeof window !== "undefined" && "__TAURI__" in window;

// 动态选择 Provider — 避免在浏览器模式下载入 Tauri API
import { TauriApiProvider } from "./tauri-provider";
import { HttpApiProvider } from "./http-provider";

export const api: ApiProvider = isTauri
  ? new TauriApiProvider()
  : new HttpApiProvider();
