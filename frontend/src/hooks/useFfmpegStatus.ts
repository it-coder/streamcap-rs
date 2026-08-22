// FFmpeg 状态 hook

import { useEffect, useState } from "react";
import { api } from "../api/provider";
import type { FfmpegCheck } from "../types";

export function useFfmpegStatus() {
  const [status, setStatus] = useState<"checking" | "available" | "missing">("checking");
  const [info, setInfo] = useState<FfmpegCheck | null>(null);

  const refresh = () => {
    setStatus("checking");
    api
      .checkFfmpeg()
      .then((res) => {
        setInfo(res);
        setStatus(res.available ? "available" : "missing");
      })
      .catch(() => {
        setStatus("missing");
      });
  };

  useEffect(() => {
    refresh();
  }, []);

  return { status, info, refresh };
}
