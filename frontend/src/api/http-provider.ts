// HttpApiProvider — B/S 服务器模式实现
//
// 使用 fetch() 进行 REST API 调用，WebSocket 进行实时事件订阅。
// 单个 WebSocket 连接复用所有事件订阅。

import type { ApiProvider } from "./provider";
import type {
  RecordingConfig,
  AppSettings,
  VideoQuality,
  ShutdownPayload,
  RecordingProgress,
  AppVersion,
  FileEntry,
  RecordingHistoryEntry,
  PostProcessJob,
  PostProcessRequest,
} from "../types";

export class HttpApiProvider implements ApiProvider {
  private ws: WebSocket | null = null;
  private wsReady: Promise<void> | null = null;
  private statusCallbacks: Set<(status: RecordingConfig) => void> = new Set();
  private progressCallbacks: Set<(progress: RecordingProgress) => void> = new Set();
  private shutdownCallbacks: Set<(payload: ShutdownPayload) => void> = new Set();
  private jobCallbacks: Set<(job: PostProcessJob) => void> = new Set();

  // ========================================
  // REST API 辅助方法
  // ========================================

  private async fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
    const res = await fetch(url, {
      headers: { "Content-Type": "application/json" },
      ...options,
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: res.statusText }));
      throw new Error(err.error || `HTTP ${res.status}`);
    }
    return res.json();
  }

  private async fetchVoid(url: string, options?: RequestInit): Promise<void> {
    const res = await fetch(url, {
      headers: { "Content-Type": "application/json" },
      ...options,
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: res.statusText }));
      throw new Error(err.error || `HTTP ${res.status}`);
    }
  }

  // ========================================
  // 录制任务
  // ========================================

  async listRecordings(): Promise<RecordingConfig[]> {
    return this.fetchJson<RecordingConfig[]>("/api/recordings");
  }

  async addRecording(params: {
    url: string;
    monitorEnabled?: boolean;
    quality?: VideoQuality;
  }): Promise<RecordingConfig> {
    return this.fetchJson<RecordingConfig>("/api/recordings", {
      method: "POST",
      body: JSON.stringify({
        url: params.url,
        monitor_enabled: params.monitorEnabled ?? true,
        quality: params.quality ?? "OD",
      }),
    });
  }

  async removeRecording(id: string): Promise<void> {
    return this.fetchVoid(`/api/recordings/${id}`, { method: "DELETE" });
  }

  async updateRecording(config: RecordingConfig): Promise<void> {
    return this.fetchVoid(`/api/recordings/${config.id}`, {
      method: "PUT",
      body: JSON.stringify(config),
    });
  }

  async startMonitor(id: string): Promise<void> {
    return this.fetchVoid(`/api/recordings/${id}/monitor`, { method: "POST" });
  }

  async stopMonitor(id: string): Promise<void> {
    return this.fetchVoid(`/api/recordings/${id}/monitor`, {
      method: "DELETE",
    });
  }

  async startRecording(id: string): Promise<void> {
    return this.fetchVoid(`/api/recordings/${id}/recording`, {
      method: "POST",
    });
  }

  async stopRecording(id: string): Promise<void> {
    return this.fetchVoid(`/api/recordings/${id}/recording`, {
      method: "DELETE",
    });
  }

  async getRecordingStatus(id: string): Promise<RecordingConfig | null> {
    const res = await fetch(`/api/recordings/${id}/status`);
    if (res.status === 404) return null;
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  // ========================================
  // 设置
  // ========================================

  async getSettings(): Promise<AppSettings> {
    return this.fetchJson<AppSettings>("/api/settings");
  }

  async updateSettings(settings: AppSettings): Promise<void> {
    return this.fetchVoid("/api/settings", {
      method: "PUT",
      body: JSON.stringify(settings),
    });
  }

  async checkFfmpeg(): Promise<string> {
    const data = await this.fetchJson<{
      available: boolean;
      version?: string;
      error?: string;
    }>("/api/ffmpeg/check");
    if (data.available) {
      return data.version || "unknown";
    }
    throw new Error(data.error || "FFmpeg not available");
  }

  // ========================================
  // 应用元信息 & 文件浏览
  // ========================================

  async getVersion(): Promise<AppVersion> {
    return this.fetchJson<AppVersion>("/api/version");
  }

  async listFiles(dir?: string): Promise<FileEntry[]> {
    const q = dir ? `?dir=${encodeURIComponent(dir)}` : "";
    const data = await this.fetchJson<{ entries: FileEntry[] }>(
      `/api/files${q}`,
    );
    return data.entries;
  }

  async openFile(path: string): Promise<void> {
    // B/S 模式：通过下载接口获取文件
    const res = await fetch(
      `/api/files/download?path=${encodeURIComponent(path)}`,
    );
    if (!res.ok) {
      throw new Error(`下载失败: HTTP ${res.status}`);
    }
    const blob = await res.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = path.split(/[\\/]/).pop() || "download";
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);
  }

  async getThumbnail(path: string): Promise<string> {
    // server 模式：返回内联预览 URL（同源）
    return `/api/files/raw?path=${encodeURIComponent(path)}`;
  }

  // ========================================
  // 录制历史 & 后处理
  // ========================================

  async listHistory(): Promise<RecordingHistoryEntry[]> {
    return this.fetchJson<RecordingHistoryEntry[]>("/api/history");
  }

  async deleteHistory(id: string, deleteFile = false): Promise<void> {
    return this.fetchVoid(`/api/history/${id}?delete_file=${deleteFile}`, {
      method: "DELETE",
    });
  }

  getPlaybackUrl(path: string): string | null {
    // server 模式：返回内联流地址（支持 Range，<video> 可拖动）
    return `/api/files/raw?path=${encodeURIComponent(path)}`;
  }

  async startPostProcess(req: PostProcessRequest): Promise<PostProcessJob> {
    return this.fetchJson<PostProcessJob>("/api/postprocess", {
      method: "POST",
      body: JSON.stringify(req),
    });
  }

  async getPostProcess(id: string): Promise<PostProcessJob | null> {
    const res = await fetch(`/api/postprocess/${id}`);
    if (res.status === 404) return null;
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }

  // ========================================
  // WebSocket 事件订阅
  // ========================================

  private ensureWs(): Promise<void> {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      return Promise.resolve();
    }
    if (this.wsReady) return this.wsReady;

    this.wsReady = new Promise((resolve, reject) => {
      const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
      const wsUrl = `${protocol}//${window.location.host}/ws/events`;

      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => resolve();
      this.ws.onerror = () => reject(new Error("WebSocket connection failed"));
      this.ws.onclose = () => {
        this.ws = null;
        this.wsReady = null;
      };

      this.ws.onmessage = (event) => {
        try {
          const msg = JSON.parse(event.data);
          if (msg.type === "recording_status") {
            this.statusCallbacks.forEach((cb) => cb(msg.data));
          } else if (msg.type === "recording_progress") {
            this.progressCallbacks.forEach((cb) => cb(msg.data));
          } else if (msg.type === "app:shutdown") {
            this.shutdownCallbacks.forEach((cb) => cb(msg.data));
          } else if (msg.type === "job_progress") {
            this.jobCallbacks.forEach((cb) => cb(msg.data));
          }
        } catch (e) {
          console.error("WebSocket message parse error:", e);
        }
      };
    });

    return this.wsReady;
  }

  async onStatusChange(
    callback: (status: RecordingConfig) => void,
  ): Promise<() => void> {
    await this.ensureWs();
    this.statusCallbacks.add(callback);
    return () => {
      this.statusCallbacks.delete(callback);
    };
  }

  async onProgressChange(
    callback: (progress: RecordingProgress) => void,
  ): Promise<() => void> {
    await this.ensureWs();
    this.progressCallbacks.add(callback);
    return () => {
      this.progressCallbacks.delete(callback);
    };
  }

  async onShutdown(
    callback: (payload: ShutdownPayload) => void,
  ): Promise<() => void> {
    await this.ensureWs();
    this.shutdownCallbacks.add(callback);
    return () => {
      this.shutdownCallbacks.delete(callback);
    };
  }

  async onJobProgress(
    callback: (job: PostProcessJob) => void,
  ): Promise<() => void> {
    await this.ensureWs();
    this.jobCallbacks.add(callback);
    return () => {
      this.jobCallbacks.delete(callback);
    };
  }
}
