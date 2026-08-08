// 录制列表 hook — 加载数据 + 事件订阅自动刷新
//
// 通过 ApiProvider 抽象层订阅后端推送的状态变更事件，
// 当后端轮询检测到开播、录制开始/结束等状态变更时，自动更新前端列表。

import { useEffect, useState, useCallback } from "react";
import { api } from "../api/provider";
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
      // merge 而非直接覆盖：
      // 如果本地已有该 id 且 updated_at 更新（来自事件推送的较新数据），保留本地的
      // 防止 refresh() 返回的列表覆盖掉事件刚收到的状态更新
      setRecordings((prev) => {
        if (prev.length === 0) return list;
        return list.map((item) => {
          const existing = prev.find((p) => p.id === item.id);
          if (!existing) return item;
          const prevTs = new Date(existing.updated_at).getTime();
          const newTs = new Date(item.updated_at).getTime();
          // 本地数据更新（来自事件推送），保留本地的
          return prevTs > newTs ? existing : item;
        });
      });
    } catch (e) {
      console.error("加载录制列表失败:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    // 先注册事件监听，确保不会漏掉 refresh 期间后端推送的状态变更
    // 注册完成后再加载数据，避免竞态条件
    api
      .onStatusChange((updated) => {
        if (cancelled) return;
        setRecordings((prev) =>
          prev.map((r) => (r.id === updated.id ? updated : r))
        );
      })
      .then((fn) => {
        if (cancelled) {
          fn();
        } else {
          unlisten = fn;
          // 事件监听注册成功后再加载数据
          refresh();
        }
      })
      .catch(() => {
        // 事件订阅失败，仍然加载数据
        refresh();
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
