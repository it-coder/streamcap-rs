// FFmpeg 状态 hook

import { useEffect, useState } from "react";
import { api } from "../api/provider";

export function useFfmpegStatus() {
  const [status, setStatus] = useState<"checking" | "available" | "missing">("checking");
  const [version, setVersion] = useState<string>("");

  useEffect(() => {
    let cancelled = false;
    api
      .checkFfmpeg()
      .then((v) => {
        if (cancelled) return;
        setVersion(v);
        setStatus("available");
      })
      .catch(() => {
        if (cancelled) return;
        setStatus("missing");
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return { status, version };
}
