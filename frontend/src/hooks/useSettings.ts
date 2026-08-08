// 设置 hook

import { useEffect, useState, useCallback } from "react";
import { api } from "../api/provider";
import type { AppSettings } from "../types";

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const s = await api.getSettings();
      setSettings(s);
    } catch (e) {
      console.error("加载设置失败:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const save = useCallback(async (newSettings: AppSettings) => {
    await api.updateSettings(newSettings);
    setSettings(newSettings);
  }, []);

  return { settings, loading, save, refresh };
}
