// TauriApiProvider — 桌面客户端模式实现
//
// 使用 Tauri IPC (invoke) 进行命令调用，Tauri Events (listen) 进行事件订阅。

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ApiProvider } from "./provider";
import type {
  RecordingConfig,
  AppSettings,
  VideoQuality,
  ShutdownPayload,
  RecordingProgress,
  AppVersion,
  FileEntry,
} from "../types";

export class TauriApiProvider implements ApiProvider {
  // ========================================
  // 录制任务
  // ========================================

  async listRecordings(): Promise<RecordingConfig[]> {
    return invoke<RecordingConfig[]>("list_recordings");
  }

  async addRecording(params: {
    url: string;
    monitorEnabled?: boolean;
    quality?: VideoQuality;
  }): Promise<RecordingConfig> {
    return invoke<RecordingConfig>("add_recording", {
      url: params.url,
      monitorEnabled: params.monitorEnabled ?? true,
      quality: params.quality ?? "OD",
    });
  }

  async removeRecording(id: string): Promise<void> {
    return invoke<void>("remove_recording", { id });
  }

  async updateRecording(config: RecordingConfig): Promise<void> {
    return invoke<void>("update_recording", { config });
  }

  async startMonitor(id: string): Promise<void> {
    return invoke<void>("start_monitor", { id });
  }

  async stopMonitor(id: string): Promise<void> {
    return invoke<void>("stop_monitor", { id });
  }

  async startRecording(id: string): Promise<void> {
    return invoke<void>("start_recording", { id });
  }

  async stopRecording(id: string): Promise<void> {
    return invoke<void>("stop_recording", { id });
  }

  async getRecordingStatus(id: string): Promise<RecordingConfig | null> {
    return invoke<RecordingConfig | null>("get_recording_status", { id });
  }

  // ========================================
  // 设置
  // ========================================

  async getSettings(): Promise<AppSettings> {
    return invoke<AppSettings>("get_settings");
  }

  async updateSettings(settings: AppSettings): Promise<void> {
    return invoke<void>("update_settings", { settings });
  }

  async checkFfmpeg(): Promise<string> {
    return invoke<string>("check_ffmpeg");
  }

  // ========================================
  // 应用元信息 & 文件浏览
  // ========================================

  async getVersion(): Promise<AppVersion> {
    const v = await invoke<string>("get_version");
    return { version: v };
  }

  async listFiles(dir?: string): Promise<FileEntry[]> {
    return invoke<FileEntry[]>("list_files", { dir: dir ?? null });
  }

  async openFile(path: string): Promise<void> {
    return invoke<void>("open_file", { path });
  }

  // ========================================
  // 事件订阅
  // ========================================

  async onStatusChange(
    callback: (status: RecordingConfig) => void,
  ): Promise<() => void> {
    return listen<RecordingConfig>("recording_status", (event) => {
      callback(event.payload);
    });
  }

  async onProgressChange(
    callback: (progress: RecordingProgress) => void,
  ): Promise<() => void> {
    return listen<RecordingProgress>("recording_progress", (event) => {
      callback(event.payload);
    });
  }

  async onShutdown(
    callback: (payload: ShutdownPayload) => void,
  ): Promise<() => void> {
    return listen<ShutdownPayload>("app:shutdown", (event) => {
      callback(event.payload);
    });
  }
}
