// 录制历史页面 — 列表 + 播放器 + 后处理
//
// 复用 http-provider / tauri-provider 的 listHistory / getPlaybackUrl /
// startPostProcess / getPostProcess / openFile 接口，双模式通用。

import { useState, useEffect, useRef } from "react";
import {
  Typography,
  Input,
  List,
  Card,
  Button,
  Space,
  Tag,
  Modal,
  Select,
  Progress,
  message,
  Popconfirm,
  Empty,
  Tooltip,
} from "antd";
import {
  PlayCircleOutlined,
  ToolOutlined,
  DeleteOutlined,
  FolderOpenOutlined,
} from "@ant-design/icons";
import { api } from "../api/provider";
import { useI18n } from "../i18n";
import type {
  RecordingHistoryEntry,
  PostProcessJob,
  PostProcessRequest,
  OutputFormat,
} from "../types";

const { Title, Text } = Typography;

function formatDuration(sec: number): string {
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  const s = sec % 60;
  if (h > 0) return `${h}h${m}m`;
  if (m > 0) return `${m}m${s}s`;
  return `${s}s`;
}

function formatSize(bytes: number): string {
  if (bytes <= 0) return "-";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let v = bytes;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(v < 10 && i > 0 ? 1 : 0)} ${units[i]}`;
}

const FORMAT_CHOICES: { value: OutputFormat; label: string }[] = [
  { value: "mp4", label: "MP4" },
  { value: "mkv", label: "MKV" },
  { value: "ts", label: "TS" },
  { value: "flv", label: "FLV" },
  { value: "mov", label: "MOV" },
];

export function HistoryPage() {
  const { t } = useI18n();
  const [history, setHistory] = useState<RecordingHistoryEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [search, setSearch] = useState("");

  const [playEntry, setPlayEntry] = useState<RecordingHistoryEntry | null>(null);
  const [ppEntry, setPpEntry] = useState<RecordingHistoryEntry | null>(null);

  const load = async () => {
    setLoading(true);
    try {
      const data = await api.listHistory();
      setHistory(data);
    } catch (e) {
      message.error(t("history.loadFail", { error: String(e) }));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, []);

  const filtered = history.filter((h) => {
    if (!search) return true;
    const q = search.toLowerCase();
    return (
      h.anchor_name.toLowerCase().includes(q) ||
      h.title.toLowerCase().includes(q) ||
      h.platform.toLowerCase().includes(q)
    );
  });

  const onDelete = async (entry: RecordingHistoryEntry, deleteFile: boolean) => {
    try {
      await api.deleteHistory(entry.id, deleteFile);
      message.success(t("history.deleted"));
      load();
    } catch (e) {
      message.error(t("history.loadFail", { error: String(e) }));
    }
  };

  return (
    <div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 16,
        }}
      >
        <Title level={4} style={{ margin: 0 }}>
          📼 {t("history.title")}
        </Title>
        <Input.Search
          allowClear
          placeholder={t("history.searchPlaceholder")}
          style={{ width: 320 }}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      {filtered.length === 0 ? (
        <Empty description={t("history.empty")} style={{ marginTop: 80 }} />
      ) : (
        <List
          grid={{ gutter: 16, xs: 1, sm: 1, md: 2, lg: 2, xl: 3 }}
          dataSource={filtered}
          loading={loading}
          renderItem={(entry) => (
            <List.Item>
              <Card
                size="small"
                cover={
                  entry.thumbnail ? (
                    <HistoryThumb path={entry.thumbnail} />
                  ) : (
                    <div
                      style={{
                        height: 140,
                        background: "#f0f0f0",
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "center",
                        color: "#999",
                      }}
                    >
                      🎬
                    </div>
                  )
                }
                actions={[
                  <Tooltip title={t("history.play")} key="play">
                    <PlayCircleOutlined
                      onClick={() => setPlayEntry(entry)}
                      style={{ color: entry.file_path ? undefined : "#ccc" }}
                    />
                  </Tooltip>,
                  <Tooltip title={t("history.postprocess")} key="pp">
                    <ToolOutlined
                      onClick={() => {
                        setPlayEntry(null);
                        setPpEntry(entry);
                      }}
                      style={{ color: entry.file_path ? undefined : "#ccc" }}
                    />
                  </Tooltip>,
                  <Popconfirm
                    key="del"
                    title={t("history.deleteConfirm")}
                    description={
                      <PopconfirmDelete onConfirm={(df) => onDelete(entry, df)} />
                    }
                    icon={null}
                  >
                    <DeleteOutlined />
                  </Popconfirm>,
                ]}
              >
                <Card.Meta
                  title={
                    <Space>
                      <Text strong>{entry.anchor_name || t("card.unknownPlatform")}</Text>
                      <StatusTag entry={entry} />
                    </Space>
                  }
                  description={
                    <div style={{ fontSize: 12 }}>
                      <div style={{ marginBottom: 2 }}>{entry.title || "-"}</div>
                      <Space size={12}>
                        <span>{formatDuration(entry.duration_seconds)}</span>
                        <span>{formatSize(entry.file_size)}</span>
                      </Space>
                      {entry.status === "failed" && entry.error_message && (
                        <div style={{ color: "#cf1322", marginTop: 4 }}>
                          {t("history.errMsg", { error: entry.error_message })}
                        </div>
                      )}
                    </div>
                  }
                />
              </Card>
            </List.Item>
          )}
        />
      )}

      <PlayerModal entry={playEntry} onClose={() => setPlayEntry(null)} />
      <PostProcessModal entry={ppEntry} onClose={() => setPpEntry(null)} onDone={load} />
    </div>
  );
}

// 缩略图：getThumbnail 在 desktop 模式返回 data URL（异步），server 模式返回 URL
function HistoryThumb({ path }: { path: string }) {
  const [src, setSrc] = useState<string>("");
  useEffect(() => {
    let alive = true;
    api
      .getThumbnail(path)
      .then((u) => {
        if (alive) setSrc(u);
      })
      .catch(() => {});
    return () => {
      alive = false;
    };
  }, [path]);

  if (!src) {
    return (
      <div
        style={{
          height: 140,
          background: "#000",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "#666",
        }}
      >
        …
      </div>
    );
  }
  return (
    <img
      alt="thumb"
      src={src}
      style={{ height: 140, objectFit: "cover", background: "#000" }}
    />
  );
}

function StatusTag({ entry }: { entry: RecordingHistoryEntry }) {
  const { t } = useI18n();
  const color =
    entry.status === "completed"
      ? "success"
      : entry.status === "failed"
      ? "error"
      : "default";
  return <Tag color={color}>{t(`history.status.${entry.status}`)}</Tag>;
}

// 删除确认气泡内的「同时删除文件」复选框
function PopconfirmDelete({ onConfirm }: { onConfirm: (deleteFile: boolean) => void }) {
  const { t } = useI18n();
  const [deleteFile, setDeleteFile] = useState(false);
  return (
    <div
      onClick={(e) => e.stopPropagation()}
      onDoubleClick={(e) => e.stopPropagation()}
    >
      <label style={{ display: "block", marginBottom: 8 }}>
        <input
          type="checkbox"
          checked={deleteFile}
          onChange={(e) => setDeleteFile(e.target.checked)}
        />{" "}
        {t("history.deleteFile")}
      </label>
      <Space>
        <Button size="small" onClick={(e) => { e.stopPropagation(); onConfirm(deleteFile); }}>
          {t("history.delete")}
        </Button>
      </Space>
    </div>
  );
}

function PlayerModal({
  entry,
  onClose,
}: {
  entry: RecordingHistoryEntry | null;
  onClose: () => void;
}) {
  const { t } = useI18n();
  const url = entry?.file_path ? api.getPlaybackUrl(entry.file_path) : null;

  return (
    <Modal
      title={t("player.title")}
      open={!!entry}
      onCancel={onClose}
      footer={null}
      width={800}
      destroyOnClose
    >
      {entry?.file_path ? (
        url ? (
          <video
            controls
            src={url}
            style={{ width: "100%", maxHeight: "60vh", background: "#000" }}
          />
        ) : (
          <Space direction="vertical">
            <Text type="secondary">{t("player.supportedHint")}</Text>
            <Button
              icon={<FolderOpenOutlined />}
              onClick={() => api.openFile(entry.file_path!)}
            >
              {t("player.openExternal")}
            </Button>
          </Space>
        )
      ) : (
        <Empty description={t("player.noFile")} />
      )}
    </Modal>
  );
}

function PostProcessModal({
  entry,
  onClose,
  onDone,
}: {
  entry: RecordingHistoryEntry | null;
  onClose: () => void;
  onDone: () => void;
}) {
  const { t } = useI18n();
  const [kind, setKind] = useState<string>("convert");
  const [targetFormat, setTargetFormat] = useState<OutputFormat>("mp4");
  const [startSec, setStartSec] = useState<string>("");
  const [endSec, setEndSec] = useState<string>("");
  const [job, setJob] = useState<PostProcessJob | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const pollRef = useRef<number | null>(null);

  // 切换条目时重置
  useEffect(() => {
    setJob(null);
    setSubmitting(false);
    setKind("convert");
    setTargetFormat("mp4");
    setStartSec("");
    setEndSec("");
  }, [entry]);

  useEffect(() => {
    return () => {
      if (pollRef.current) window.clearInterval(pollRef.current);
    };
  }, []);

  const poll = (id: string) => {
    if (pollRef.current) window.clearInterval(pollRef.current);
    pollRef.current = window.setInterval(async () => {
      const j = await api.getPostProcess(id);
      if (!j) return;
      setJob(j);
      if (j.status === "done" || j.status === "failed") {
        if (pollRef.current) window.clearInterval(pollRef.current);
        pollRef.current = null;
        if (j.status === "done") {
          message.success(t("postprocess.done"));
          onDone();
        } else {
          message.error(t("postprocess.fail", { error: j.error || "" }));
        }
      }
    }, 1000);
  };

  const onSubmit = async () => {
    if (!entry?.file_path) return;
    setSubmitting(true);
    try {
      const req: PostProcessRequest = {
        kind,
        input: entry.file_path,
        target_format: kind === "convert" ? targetFormat : undefined,
        start_seconds:
          kind === "trim" && startSec !== "" ? Number(startSec) : undefined,
        end_seconds: kind === "trim" && endSec !== "" ? Number(endSec) : undefined,
      };
      const j = await api.startPostProcess(req);
      setJob(j);
      message.success(t("postprocess.started"));
      poll(j.id);
    } catch (e) {
      message.error(t("postprocess.fail", { error: String(e) }));
    } finally {
      setSubmitting(false);
    }
  };

  const running = job?.status === "pending" || job?.status === "running";

  return (
    <Modal
      title={t("postprocess.title")}
      open={!!entry}
      onCancel={onClose}
      footer={null}
      destroyOnClose
    >
      {!entry?.file_path ? (
        <Empty description={t("player.noFile")} />
      ) : (
        <Space direction="vertical" style={{ width: "100%" }} size="middle">
          <div>
            <Text type="secondary">{entry.file_path}</Text>
          </div>

          <div>
            <Text>{t("postprocess.kind")}</Text>
            <Select
              value={kind}
              style={{ width: "100%", marginTop: 4 }}
              disabled={running}
              onChange={setKind}
              options={[
                { value: "convert", label: t("postprocess.convert") },
                { value: "extract-audio", label: t("postprocess.extractAudio") },
                { value: "trim", label: t("postprocess.trim") },
              ]}
            />
          </div>

          {kind === "convert" && (
            <div>
              <Text>{t("postprocess.targetFormat")}</Text>
              <Select
                value={targetFormat}
                style={{ width: "100%", marginTop: 4 }}
                disabled={running}
                onChange={setTargetFormat}
                options={FORMAT_CHOICES}
              />
            </div>
          )}

          {kind === "trim" && (
            <>
              <div>
                <Text>{t("postprocess.startSeconds")}</Text>
                <Input
                  value={startSec}
                  type="number"
                  style={{ marginTop: 4 }}
                  disabled={running}
                  onChange={(e) => setStartSec(e.target.value)}
                />
              </div>
              <div>
                <Text>{t("postprocess.endSeconds")}</Text>
                <Input
                  value={endSec}
                  type="number"
                  style={{ marginTop: 4 }}
                  disabled={running}
                  onChange={(e) => setEndSec(e.target.value)}
                />
              </div>
            </>
          )}

          <Button
            type="primary"
            loading={submitting}
            disabled={running}
            onClick={onSubmit}
          >
            {running ? t("postprocess.running") : t("postprocess.start")}
          </Button>

          {job && (
            <div>
              <Text>
                {t("postprocess.progress")}: {Math.round(job.progress * 100)}%
              </Text>
              <Progress percent={Math.round(job.progress * 100)} />
              {job.output && (
                <Button
                  size="small"
                  icon={<PlayCircleOutlined />}
                  onClick={() => {
                    const u = api.getPlaybackUrl(job.output!);
                    if (u) window.open(u, "_blank");
                    else api.openFile(job.output!);
                  }}
                >
                  {t("history.play")}
                </Button>
              )}
            </div>
          )}
        </Space>
      )}
    </Modal>
  );
}
