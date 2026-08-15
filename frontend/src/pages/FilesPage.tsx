// 文件管理页 — 浏览录制输出目录（沙箱在 output_dir 内），支持进子目录、下载/打开文件

import { useEffect, useCallback, useState } from "react";
import { Breadcrumb, Table, Button, Space, Spin, message } from "antd";
import type { ColumnsType } from "antd/es/table";
import {
  FolderOutlined,
  FileOutlined,
  ArrowLeftOutlined,
  DownloadOutlined,
  FolderOpenOutlined,
} from "@ant-design/icons";
import { api, isTauri } from "../api/provider";
import type { FileEntry } from "../types";
import { useI18n } from "../i18n";

/** 格式化文件大小 */
function formatSize(bytes: number): string {
  if (bytes >= 1_073_741_824) return (bytes / 1_073_741_824).toFixed(2) + " GB";
  if (bytes >= 1_048_576) return (bytes / 1_048_576).toFixed(1) + " MB";
  if (bytes >= 1024) return (bytes / 1024).toFixed(0) + " KB";
  return bytes + " B";
}

export function FilesPage() {
  const { t } = useI18n();
  const [currentDir, setCurrentDir] = useState<string | undefined>(undefined);
  const [entries, setEntries] = useState<FileEntry[]>([]);
  const [loading, setLoading] = useState(false);

  const load = useCallback(async (dir?: string) => {
    setLoading(true);
    try {
      const list = await api.listFiles(dir);
      setEntries(list);
      setCurrentDir(dir);
    } catch (e) {
      message.error(t("files.loadFail", { error: String(e) }));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    load(undefined);
  }, [load]);

  const handleOpen = async (entry: FileEntry) => {
    if (entry.is_dir) {
      await load(entry.path);
      return;
    }
    try {
      await api.openFile(entry.path);
    } catch (e) {
      message.error(t("files.openFail", { error: String(e) }));
    }
  };

  // 面包屑（兼容 Windows / 类 Unix 路径分隔符）
  const sep = currentDir && currentDir.includes("\\") ? "\\" : "/";
  const parts = currentDir ? currentDir.split(sep).filter(Boolean) : [];
  const breadcrumbItems = parts.map((name, i) => {
    const path = parts.slice(0, i + 1).join(sep);
    return { title: <a onClick={() => load(path)}>{name}</a> };
  });
  const parentPath = parts.slice(0, -1).join(sep) || undefined;

  const columns: ColumnsType<FileEntry> = [
    {
      title: t("files.name"),
      dataIndex: "name",
      render: (_: string, entry: FileEntry) => (
        <Space>
          {entry.is_dir ? (
            <FolderOutlined style={{ color: "#F59E0B" }} />
          ) : (
            <FileOutlined />
          )}
          <a onClick={() => handleOpen(entry)}>{entry.name}</a>
        </Space>
      ),
    },
    {
      title: t("files.size"),
      dataIndex: "size",
      width: 120,
      render: (size: number, entry: FileEntry) =>
        entry.is_dir ? "-" : formatSize(size),
    },
    {
      title: t("files.modified"),
      dataIndex: "modified",
      width: 200,
      render: (m: string | null) => (m ? new Date(m).toLocaleString() : "-"),
    },
    {
      title: t("files.action"),
      key: "action",
      width: 120,
      render: (_: unknown, entry: FileEntry) => (
        <Button
          size="small"
          icon={
            entry.is_dir ? (
              <FolderOpenOutlined />
            ) : isTauri ? (
              <FolderOpenOutlined />
            ) : (
              <DownloadOutlined />
            )
          }
          onClick={() => handleOpen(entry)}
        >
          {entry.is_dir
            ? t("files.enter")
            : isTauri
              ? t("files.open")
              : t("files.download")}
        </Button>
      ),
    },
  ];

  return (
    <div>
      <div
        style={{
          marginBottom: 12,
          display: "flex",
          alignItems: "center",
          gap: 12,
          flexWrap: "wrap",
        }}
      >
        <Button
          icon={<ArrowLeftOutlined />}
          disabled={parts.length === 0}
          onClick={() => load(parentPath)}
        >
          {t("files.up")}
        </Button>
        <Breadcrumb items={breadcrumbItems} />
      </div>
      <Spin spinning={loading}>
        <Table
          rowKey="path"
          columns={columns}
          dataSource={entries}
          pagination={false}
          size="middle"
          locale={{ emptyText: t("files.folderEmpty") }}
        />
      </Spin>
    </div>
  );
}
