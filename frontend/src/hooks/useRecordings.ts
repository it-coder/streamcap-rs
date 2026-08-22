// 录制列表 hook — 加载数据 + 事件订阅自动刷新
//
// 通过 ApiProvider 抽象层订阅后端推送的状态变更事件，
// 当后端轮询检测到开播、录制开始/结束等状态变更时，自动更新前端列表。

import { useEffect, useState, useCallback, useRef } from "react";
import { message } from "antd";
import { api } from "../api/provider";
import { useI18n } from "../i18n";
import type { RecordingConfig, VideoQuality, RecordingProgress, TimeRange } from "../types";

export type UseRecordingsResult = {
  recordings: RecordingConfig[];
  progressMap: Record<string, RecordingProgress>;
  loading: boolean;
  refresh: () => Promise<void>;
  addRecording: (url: string, monitorEnabled: boolean, quality: VideoQuality, schedule?: TimeRange[]) => Promise<RecordingConfig>;
  removeRecording: (id: string) => Promise<void>;
  toggleMonitor: (id: string, enabled: boolean) => Promise<void>;
  startRecording: (id: string) => Promise<void>;
  stopRecording: (id: string) => Promise<void>;
  updateRecording: (config: RecordingConfig) => Promise<void>;
};

export function useRecordings() {
  const { t } = useI18n();
  const [recordings, setRecordings] = useState<RecordingConfig[]>([]);
  const [progressMap, setProgressMap] = useState<Record<string, RecordingProgress>>({});
  const [loading, setLoading] = useState(true);
  // 记录每个录制任务上一次的状态，用于检测「开始 / 完成 / 失败」跳变并弹出 toast
  const prevState = useRef<Record<string, { is_recording: boolean }>>({});

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
    let unlistenProgress: (() => void) | undefined;
    let cancelled = false;

    // 先注册事件监听，确保不会漏掉 refresh 期间后端推送的状态变更
    // 注册完成后再加载数据，避免竞态条件
    api
      .onStatusChange((updated) => {
        if (cancelled) return;

        // 站内通知：检测 开始 / 完成 / 失败 跳变
        const prev = prevState.current[updated.id];
        const name =
          updated.anchor_name || updated.title || t("card.unknownPlatform");
        if (prev) {
          if (!prev.is_recording && updated.is_recording) {
            message.info(t("notify.recordingStarted", { name }));
          } else if (prev.is_recording && !updated.is_recording) {
            if (updated.error_message) {
              message.error(
                t("notify.recordingFailed", { name }) +
                  `: ${updated.error_message}`
              );
            } else {
              message.success(t("notify.recordingDone", { name }));
            }
          }
        }
        prevState.current[updated.id] = {
          is_recording: updated.is_recording,
        };

        setRecordings((prev) =>
          prev.map((r) => (r.id === updated.id ? updated : r))
        );
        // 录制结束时清除进度
        if (!updated.is_recording) {
          setProgressMap((prev) => {
            const next = { ...prev };
            delete next[updated.id];
            return next;
          });
        }
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

    // 录制进度订阅
    api
      .onProgressChange((progress) => {
        if (cancelled) return;
        setProgressMap((prev) => ({
          ...prev,
          [progress.recording_id]: progress,
        }));
      })
      .then((fn) => {
        if (cancelled) {
          fn();
        } else {
          unlistenProgress = fn;
        }
      })
      .catch(() => {});

    return () => {
      cancelled = true;
      unlisten?.();
      unlistenProgress?.();
    };
  }, [refresh]);

  // 操作方法（操作后无需手动 refresh，事件会自动推送更新）
  const addRecording = useCallback(
    async (url: string, monitorEnabled: boolean, quality: VideoQuality, schedule?: TimeRange[]) => {
      const config = await api.addRecording({ url, monitorEnabled, quality });
      // 如果有调度窗口，创建后立即更新（add 命令不支持 schedule 参数）
      if (schedule && schedule.length > 0) {
        const updated = { ...config, schedule };
        await api.updateRecording(updated);
        setRecordings((prev) => [...prev, updated]);
        return updated;
      }
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

  const updateRecording = useCallback(async (config: RecordingConfig) => {
    await api.updateRecording(config);
    // update_recording 不触发状态广播，主动刷新一次以拉取合并后的最新配置
    await refresh();
  }, [refresh]);

  return {
    recordings,
    progressMap,
    loading,
    refresh,
    addRecording,
    removeRecording,
    toggleMonitor,
    startRecording,
    stopRecording,
    updateRecording,
  };
}
