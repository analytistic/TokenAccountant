import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface DevTrace {
  turn: number;
  model: string;
  detect_text: string;
  store_text: string;
}

export default function DevPanel() {
  const [traces, setTraces] = useState<DevTrace[]>([]);
  const [selectedIdx, setSelectedIdx] = useState<number | null>(null);
  const [fontSize, setFontSize] = useState(13);

  const load = useCallback(() => {
    invoke<DevTrace[]>("list_dev_traces")
      .then((data) => {
        setTraces(data);
        if (data.length > 0 && selectedIdx === null) {
          setSelectedIdx(data.length - 1);
        }
      })
      .catch(console.error);
  }, [selectedIdx]);

  useEffect(() => {
    load();
    const interval = setInterval(load, 2000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey) {
        if (e.key === "=" || e.key === "+") {
          e.preventDefault();
          setFontSize((s) => Math.min(s + 1, 32));
        } else if (e.key === "-") {
          e.preventDefault();
          setFontSize((s) => Math.max(s - 1, 8));
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const current = selectedIdx !== null ? traces[selectedIdx] : null;

  return (
    <div className="bg-gray-900 flex flex-col h-full">
      {/* Main: Store / Detect side-by-side */}
      {current ? (
        <div className="flex flex-row flex-1 min-h-0">
          <div className="flex-1 flex flex-col min-w-0 border-r border-gray-700">
            <div className="text-[11px] font-medium text-gray-400 uppercase tracking-wider px-3 pt-3 pb-1 shrink-0">
              Store 展平
            </div>
            <pre
              className="flex-1 overflow-auto px-3 pb-3 text-gray-200 whitespace-pre-wrap break-all font-mono"
              style={{ fontSize: `${fontSize}px` }}
            >
              {current.store_text || "(empty)"}
            </pre>
          </div>
          <div className="flex-1 flex flex-col min-w-0">
            <div className="text-[11px] font-medium text-gray-400 uppercase tracking-wider px-3 pt-3 pb-1 shrink-0">
              Detect 展平
            </div>
            <pre
              className="flex-1 overflow-auto px-3 pb-3 text-gray-200 whitespace-pre-wrap break-all font-mono"
              style={{ fontSize: `${fontSize}px` }}
            >
              {current.detect_text || "(empty)"}
            </pre>
          </div>
        </div>
      ) : (
        <div className="flex-1 flex items-center justify-center text-sm text-gray-500">
          暂无数据
        </div>
      )}

      {/* Bottom: trace selector — ~2 lines */}
      <div className="shrink-0 border-t border-gray-700 bg-gray-800 px-3 py-2 flex items-center gap-2">
        <span className="text-xs text-gray-400 shrink-0">Trace</span>
        <select
          className="flex-1 text-xs bg-gray-700 text-gray-200 border border-gray-600 rounded px-1 py-0.5 min-w-0"
          value={selectedIdx ?? ""}
          onChange={(e) => setSelectedIdx(Number(e.target.value))}
        >
          {traces.map((t, i) => (
            <option key={i} value={i}>
              #{t.turn} {t.model}
            </option>
          ))}
        </select>
        <span className="text-[10px] text-gray-600 shrink-0">
          Cmd+/- 字号
        </span>
      </div>
    </div>
  );
}
