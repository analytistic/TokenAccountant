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
    <div className="w-96 border-l border-gray-200 bg-white flex flex-col h-full">
      {/* Header */}
      <div className="px-3 py-2 border-b border-gray-200 flex items-center justify-between shrink-0">
        <h2 className="text-sm font-semibold text-gray-700">渲染检查</h2>
        <select
          className="text-xs border border-gray-300 rounded px-1 py-0.5"
          value={selectedIdx ?? ""}
          onChange={(e) => setSelectedIdx(Number(e.target.value))}
        >
          {traces.map((t, i) => (
            <option key={i} value={i}>
              #{t.turn} {t.model}
            </option>
          ))}
        </select>
      </div>

      {/* Traces */}
      {current ? (
        <div className="flex-1 overflow-auto p-3 space-y-4">
          <TraceSection
            label="Store 展平"
            text={current.store_text}
            fontSize={fontSize}
          />
          <TraceSection
            label="Detect 展平"
            text={current.detect_text}
            fontSize={fontSize}
          />
        </div>
      ) : (
        <div className="flex-1 flex items-center justify-center text-sm text-gray-400">
          暂无数据
        </div>
      )}

      {/* Footer */}
      <div className="px-3 py-1.5 border-t border-gray-200 text-[10px] text-gray-400 shrink-0">
        Cmd+/- 调整字号 · 每 2s 自动刷新
      </div>
    </div>
  );
}

function TraceSection({
  label,
  text,
  fontSize,
}: {
  label: string;
  text: string;
  fontSize: number;
}) {
  return (
    <div>
      <div className="text-[11px] font-medium text-gray-500 mb-1 uppercase tracking-wider">
        {label}
      </div>
      <pre
        className="bg-gray-50 border border-gray-200 rounded p-2 overflow-auto max-h-96 whitespace-pre-wrap break-all"
        style={{ fontSize: `${fontSize}px`, fontFamily: "SF Mono, Menlo, Monaco, Consolas, monospace" }}
      >
        {text || "(empty)"}
      </pre>
    </div>
  );
}
