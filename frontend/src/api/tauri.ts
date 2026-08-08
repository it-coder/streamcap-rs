// 向后兼容的 re-export — 新代码请直接从 "./provider" 导入
import { api } from "./provider";
import type { AppSettings, RecordingConfig, VideoQuality } from "../types";

export const listRecordings = () => api.listRecordings();
export const addRecording = (params: {
  url: string;
  monitorEnabled?: boolean;
  quality?: VideoQuality;
}) => api.addRecording(params);
export const removeRecording = (id: string) => api.removeRecording(id);
export const updateRecording = (config: RecordingConfig) =>
  api.updateRecording(config);
export const startMonitor = (id: string) => api.startMonitor(id);
export const stopMonitor = (id: string) => api.stopMonitor(id);
export const startRecording = (id: string) => api.startRecording(id);
export const stopRecording = (id: string) => api.stopRecording(id);
export const getRecordingStatus = (id: string) =>
  api.getRecordingStatus(id);
export const getSettings = () => api.getSettings();
export const updateSettings = (settings: AppSettings) =>
  api.updateSettings(settings);
export const checkFfmpeg = () => api.checkFfmpeg();
