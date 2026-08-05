import { useMemo, useState } from "react";
import type { TrendPoint } from "../types";

interface TrendChartProps { data: TrendPoint[]; }
type Metric = "total" | "input" | "cache" | "output";

const METRICS: Array<{ key: Metric; label: string; color: string }> = [
  { key: "total", label: "总计", color: "var(--brand)" },
  { key: "input", label: "Input", color: "var(--ico-input)" },
  { key: "cache", label: "Cache", color: "var(--ico-cache)" },
  { key: "output", label: "Output", color: "var(--ico-output)" },
];

export default function TrendChart({ data }: TrendChartProps) {
  const [metric, setMetric] = useState<Metric>("total");
  const active = METRICS.find((item) => item.key === metric)!;
  const values = useMemo(() => data.map((point) => {
    if (metric === "total") {
      return {
        claimed: point.input_claimed + point.cache_claimed + point.output_claimed,
        detected: point.input_detected + point.cache_detected + point.output_detected,
      };
    }
    return {
      claimed: point[`${metric}_claimed`],
      detected: point[`${metric}_detected`],
    };
  }), [data, metric]);

  const width = 760;
  const height = 220;
  const pad = { left: 10, right: 54, top: 18, bottom: 20 };
  const max = Math.max(1, ...values.flatMap((value) => [value.claimed, value.detected]));
  const toX = (index: number) => pad.left + (index / Math.max(1, values.length - 1)) * (width - pad.left - pad.right);
  const toY = (value: number) => pad.top + (height - pad.top - pad.bottom) * (1 - value / (max * 1.12));
  const path = (key: "claimed" | "detected") => values.map((value, index) =>
    `${index === 0 ? "M" : "L"}${toX(index).toFixed(1)},${toY(value[key]).toFixed(1)}`,
  ).join(" ");
  const area = values.length ? `${path("claimed")} ${[...values].reverse().map((value, reverseIndex) => {
    const index = values.length - 1 - reverseIndex;
    return `L${toX(index).toFixed(1)},${toY(value.detected).toFixed(1)}`;
  }).join(" ")} Z` : "";
  const latest = values[values.length - 1] ?? { claimed: 0, detected: 0 };
  const diff = latest.claimed - latest.detected;
  const rate = latest.claimed > 0 ? (diff / latest.claimed) * 100 : 0;

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="mb-2 flex shrink-0 items-center justify-between gap-3">
        <div className="flex rounded-md bg-gray-100 p-0.5">
          {METRICS.map((item) => (
            <button key={item.key} type="button" onClick={() => setMetric(item.key)}
              className={`rounded px-2.5 py-1 text-[10px] font-semibold transition-colors ${metric === item.key ? "bg-white text-gray-800 shadow-xs" : "text-gray-400 hover:text-gray-600"}`}>
              {item.label}
            </button>
          ))}
        </div>
        <div className="flex items-baseline gap-1.5 text-right">
          <span className="text-[10px] text-gray-400">最新差异</span>
          <span className={`font-mono text-sm font-bold ${Math.abs(rate) < 3 ? "text-success" : "text-danger"}`}>
            {diff > 0 ? "+" : ""}{diff} <span className="text-[10px] font-normal">({rate > 0 ? "+" : ""}{rate.toFixed(1)}%)</span>
          </span>
        </div>
      </div>
      <svg className="min-h-0 flex-1" viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none">
        {[0, 0.5, 1].map((ratio) => {
          const value = max * ratio;
          const y = toY(value);
          return <g key={ratio}><line x1={pad.left} y1={y} x2={width - pad.right} y2={y} stroke="var(--gray-200)" strokeWidth="0.7" /><text x={width - pad.right + 8} y={y + 3} fill="var(--gray-400)" fontSize="9">{value >= 1000 ? `${(value / 1000).toFixed(1)}K` : Math.round(value)}</text></g>;
        })}
        <path d={area} fill={active.color} fillOpacity="0.1" />
        <path d={path("claimed")} fill="none" stroke={active.color} strokeOpacity="0.38" strokeWidth="1.5" strokeDasharray="5 4" />
        <path d={path("detected")} fill="none" stroke={active.color} strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
      <div className="flex shrink-0 items-center justify-center gap-5 pt-2 text-[10px] text-gray-400">
        <span className="flex items-center gap-1.5"><i className="h-0.5 w-4 rounded bg-gray-500" />本地检测</span>
        <span className="flex items-center gap-1.5"><i className="h-px w-4 border-t border-dashed border-gray-400" />Provider 声称</span>
        <span className="flex items-center gap-1.5"><i className="h-2 w-4 rounded-sm bg-brand-muted" />差异区间</span>
      </div>
    </div>
  );
}
