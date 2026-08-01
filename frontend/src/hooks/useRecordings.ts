// 录制列表 hook — 加载数据 + 事件订阅自动刷新
//
// 核心优化：通过 Tauri 事件系统监听后端推送的 "recording_status" 事件，
// 当后端轮询检测到开播、录制开始/结束等状态变更时，自动更新前端列表，
// 无需手动刷新或轮询。

import { useEffect, useState, useCallback } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import * as api from "../api/tauri";
import type { RecordingConfig, VideoQuality } from "../types";

export type UseRecordingsResult = {
  recordings: RecordingConfig[];
  loading: boolean;
  refresh: () => Promise<void>;
  addRecording: (url: string, monitorEnabled: boolean, quality: VideoQuality) => Promise<RecordingConfig>;
  removeRecording: (id: string) => Promise<void>;
  toggleMonitor: (id: string, enabled: boolean) => Promise<void>;
  startRecording: (id: string) => Promise<void>;
  stopRecording: (id: string) => Promise<void>;
};

export function useRecordings() {
  const [recordings, setRecordings] = useState<RecordingConfig[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const list = await api.listRecordings();
      setRecordings(list);
    } catch (e) {
      console.error("加载录制列表失败:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();

    // 订阅后端状态变更事件
    // 后端在 RecordingManager.broadcast() 中 emit("recording_status", config)
    let unlisten: UnlistenFn | undefined;
    let cancelled = false;

    listen<RecordingConfig>("recording_status", (event) => {
      if (cancelled) return;
      const updated = event.payload;
      setRecordings((prev) =>
        prev.map((r) => (r.id === updated.id ? updated : r))
      );
    }).then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [refresh]);

  // 操作方法（操作后无需手动 refresh，事件会自动推送更新）
  const addRecording = useCallback(
    async (url: string, monitorEnabled: boolean, quality: VideoQuality) => {
      const config = await api.addRecording({ url, monitorEnabled, quality });
      // 手动追加，因为 add 命令不触发 broadcast
      setRecordings((prev) => [...prev, config]);
      return config;
    },
    []
  );

  const removeRecording = useCallback(async (id: string) => {
    await api.removeRecording(id);
    setRecordings((prev) => prev.filter((r) => r.id !== id));
  }, []);

  const toggleMonitor = useCallback(async (id: string, enabled: boolean) => {
    if (enabled) {
      await api.startMonitor(id);
    } else {
      await api.stopMonitor(id);
    }
    // 本地先更新，避免等事件推送
    setRecordings((prev) =>
      prev.map((r) =>
        r.id === id ? { ...r, monitor_enabled: enabled, updated_at: new Date().toISOString() } : r
      )
    );
  }, []);

  const startRecording = useCallback(async (id: string) => {
    await api.startRecording(id);
    // 状态变更由事件推送自动更新
  }, []);

  const stopRecording = useCallback(async (id: string) => {
    await api.stopRecording(id);
    // 状态变更由事件推送自动更新
  }, []);

  return {
    recordings,
    loading,
    refresh,
    addRecording,
    removeRecording,
    toggleMonitor,
    startRecording,
    stopRecording,
  };
}
