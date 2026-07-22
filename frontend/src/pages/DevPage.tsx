import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

interface DevTraceEvent {
  model: string;
  detect_text: string;
  store_text: string;
}

interface DevTrace extends DevTraceEvent {
  id: number;
  turn?: number;
}

const EMPTY_TEXT = "(empty)";

export default function DevPage() {
  const [traces, setTraces] = useState<DevTrace[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [fontSize, setFontSize] = useState(13);

  useEffect(() => {
    let nextId = 1;
    invoke<Array<DevTraceEvent & { turn: number }>>("list_dev_traces")
      .then((items) => {
        const recent = items.slice(-5).map((item) => ({ ...item, id: item.turn }));
        nextId = Math.max(1, ...recent.map((item) => item.id + 1));
        setTraces(recent);
        if (recent.length > 0) setSelectedId(recent[recent.length - 1].id);
      })
      .catch((error) => console.error("Failed to load dev traces:", error));
    const unlisten = listen<DevTraceEvent>("dev-trace", ({ payload }) => {
      const trace = { ...payload, id: nextId++ };
      setTraces((current) => [...current, trace].slice(-5));
      setSelectedId(trace.id);
    });
    return () => {
      unlisten.then((dispose) => dispose());
    };
  }, []);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (!event.metaKey && !event.ctrlKey) return;
      if (event.key === "+" || event.key === "=") {
        event.preventDefault();
        setFontSize((size) => Math.min(24, size + 1));
      } else if (event.key === "-") {
        event.preventDefault();
        setFontSize((size) => Math.max(9, size - 1));
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const current = traces.find((trace) => trace.id === selectedId) ?? traces[traces.length - 1] ?? null;

  return (
    <div className="flex h-full flex-col bg-gray-950 text-gray-200">
      <header className="flex shrink-0 items-center justify-between border-b border-gray-800 px-6 py-4">
        <div className="flex items-center gap-3">
          <div>
            <h1 className="text-lg font-semibold tracking-tight text-white">开发者工具</h1>
            <p className="mt-0.5 text-xs text-gray-500">比较 Store 与 Detect 的模板渲染结果</p>
          </div>
          {current && (
            <span className="rounded-full border border-warning/30 bg-warning/10 px-2.5 py-1 font-mono text-[11px] text-warning">
              {current.model || "unknown"}
            </span>
          )}
        </div>
        <div className="flex items-center gap-3 text-xs text-gray-500">
          <span>⌘/Ctrl +/- 调整字号</span>
          <span className="rounded bg-gray-800 px-2 py-1 font-mono text-gray-400">{fontSize}px</span>
          {traces.length > 0 && (
            <button
              type="button"
              className="rounded px-2 py-1 text-gray-400 transition-colors hover:bg-gray-800 hover:text-white"
              onClick={async () => {
                try {
                  await invoke("clear_dev_traces");
                  setTraces([]);
                  setSelectedId(null);
                } catch (error) {
                  console.error("Failed to clear dev traces:", error);
                }
              }}
            >
              清空
            </button>
          )}
        </div>
      </header>

      {current ? (
        <div className="grid min-h-0 flex-1 grid-cols-2">
          <TracePane label="Store 展平文本" text={current.store_text} fontSize={fontSize} />
          <TracePane label="Detect 展平文本" text={current.detect_text} fontSize={fontSize} border />
        </div>
      ) : (
        <div className="flex flex-1 items-center justify-center">
          <div className="text-center">
            <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-xl border border-gray-800 bg-gray-900 font-mono text-lg text-gray-600">
              &lt;/&gt;
            </div>
            <p className="text-sm font-medium text-gray-400">等待审计数据...</p>
            <p className="mt-1 text-xs text-gray-600">代理收到请求后，渲染结果会实时显示在这里</p>
          </div>
        </div>
      )}

      <footer className="flex h-12 shrink-0 items-center gap-2 border-t border-gray-800 bg-gray-900/70 px-4">
        <span className="mr-1 text-[11px] font-medium uppercase tracking-wide text-gray-600">Recent</span>
        {traces.length === 0 ? (
          <span className="text-xs text-gray-600">暂无 Trace</span>
        ) : (
          traces.map((trace, index) => (
            <button
              key={trace.id}
              type="button"
              onClick={() => setSelectedId(trace.id)}
              className={`max-w-40 truncate rounded-md border px-3 py-1.5 font-mono text-[11px] transition-colors ${
                current?.id === trace.id
                  ? "border-brand bg-brand/20 text-brand-light"
                  : "border-gray-700 text-gray-500 hover:border-gray-600 hover:text-gray-300"
              }`}
            >
              #{index + 1} {trace.model || "unknown"}
            </button>
          ))
        )}
      </footer>
    </div>
  );
}

function TracePane({
  label,
  text,
  fontSize,
  border = false,
}: {
  label: string;
  text: string;
  fontSize: number;
  border?: boolean;
}) {
  return (
    <section className={`flex min-w-0 flex-col ${border ? "border-l border-gray-800" : ""}`}>
      <div className="shrink-0 border-b border-gray-800 bg-gray-900/40 px-4 py-2.5 text-[11px] font-semibold uppercase tracking-wider text-gray-500">
        {label}
      </div>
      <pre
        className="flex-1 overflow-auto whitespace-pre-wrap break-words p-4 font-mono leading-relaxed text-gray-300"
        style={{ fontSize: `${fontSize}px` }}
      >
        {text || EMPTY_TEXT}
      </pre>
    </section>
  );
}
