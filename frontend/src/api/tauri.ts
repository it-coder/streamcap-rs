// Tauri 后端命令调用封装
// 所有 invoke 调用集中在此，组件不直接接触 Tauri API

import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, RecordingConfig, VideoQuality } from "../types";

// ========================================
// 录制任务命令
// ========================================

export async function listRecordings(): Promise<RecordingConfig[]> {
  return invoke<RecordingConfig[]>("list_recordings");
}

export async function addRecording(params: {
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

export async function removeRecording(id: string): Promise<void> {
  return invoke<void>("remove_recording", { id });
}

export async function updateRecording(config: RecordingConfig): Promise<void> {
  return invoke<void>("update_recording", { config });
}

export async function startMonitor(id: string): Promise<void> {
  return invoke<void>("start_monitor", { id });
}

export async function stopMonitor(id: string): Promise<void> {
  return invoke<void>("stop_monitor", { id });
}

export async function startRecording(id: string): Promise<void> {
  return invoke<void>("start_recording", { id });
}

export async function stopRecording(id: string): Promise<void> {
  return invoke<void>("stop_recording", { id });
}

export async function getRecordingStatus(id: string): Promise<RecordingConfig | null> {
  return invoke<RecordingConfig | null>("get_recording_status", { id });
}

// ========================================
// 设置命令
// ========================================

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function updateSettings(settings: AppSettings): Promise<void> {
  return invoke<void>("update_settings", { settings });
}

export async function checkFfmpeg(): Promise<string> {
  return invoke<string>("check_ffmpeg");
}
