import { useMemo, useState } from "react";
import type { DailyBreakdown } from "../types";

interface BarChart7DayProps { data: DailyBreakdown[]; }

export default function BarChart7Day({ data }: BarChart7DayProps) {
  const [hovered, setHovered] = useState<number | null>(null);
  const days = useMemo(() => data.map((day) => ({
    date: day.date,
    label: day.date.split("-").slice(1).join("-"),
    input: day.input_claimed - day.input_detected,
    cache: day.cache_claimed - day.cache_detected,
    output: day.output_claimed - day.output_detected,
    total:
      day.input_claimed - day.input_detected +
      day.cache_claimed - day.cache_detected +
      day.output_claimed - day.output_detected,
  })), [data]);

  if (!days.length) return <div className="flex h-full items-center justify-center text-sm text-gray-400">暂无 7 日审计数据</div>;

  const width = 760;
  const height = 190;
  const pad = { left: 46, right: 14, top: 15, bottom: 28 };
  const chartH = height - pad.top - pad.bottom;
  const maxAbs = Math.max(1, ...days.flatMap((day) => [Math.abs(day.input), Math.abs(day.cache), Math.abs(day.output), Math.abs(day.total)]));
  const zeroY = pad.top + chartH / 2;
  const scale = chartH / 2 / (maxAbs * 1.15);
  const groupWidth = (width - pad.left - pad.right) / days.length;
  const barWidth = Math.min(14, groupWidth * 0.17);
  const format = (value: number) => Math.abs(value) >= 1000 ? `${(value / 1000).toFixed(1)}K` : String(Math.round(value));

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="mb-1 flex shrink-0 items-center justify-between text-[10px] text-gray-400">
        <span>零线上方为多报，零线下方为少报</span>
        <div className="flex gap-3"><span className="text-ico-input">● Input</span><span className="text-ico-cache">● Cache</span><span className="text-ico-output">● Output</span></div>
      </div>
      <svg className="min-h-0 flex-1" viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none">
        <line x1={pad.left} y1={zeroY} x2={width - pad.right} y2={zeroY} stroke="var(--gray-400)" strokeWidth="1" />
        {[1, -1].map((direction) => {
          const y = zeroY - direction * maxAbs * scale;
          return <g key={direction}><line x1={pad.left} y1={y} x2={width - pad.right} y2={y} stroke="var(--gray-200)" strokeWidth="0.6" /><text x={pad.left - 7} y={y + 3} textAnchor="end" fill="var(--gray-400)" fontSize="9">{direction > 0 ? "+" : "−"}{format(maxAbs)}</text></g>;
        })}
        {days.map((day, index) => {
          const center = pad.left + groupWidth * (index + 0.5);
          const bars = [
            { value: day.input, color: "var(--ico-input)" },
            { value: day.cache, color: "var(--ico-cache)" },
            { value: day.output, color: "var(--ico-output)" },
          ];
          return (
            <g key={day.date} onMouseEnter={() => setHovered(index)} onMouseLeave={() => setHovered(null)}>
              {hovered === index && <rect x={center - groupWidth / 2 + 2} y={pad.top} width={groupWidth - 4} height={chartH} rx="4" fill="var(--gray-100)" />}
              {bars.map((bar, barIndex) => {
                const barHeight = Math.abs(bar.value) * scale;
                const x = center + (barIndex - 1) * (barWidth + 3) - barWidth / 2;
                const y = bar.value >= 0 ? zeroY - barHeight : zeroY;
                return <rect key={barIndex} x={x} y={y} width={barWidth} height={Math.max(barHeight, bar.value === 0 ? 0 : 1)} rx="2" fill={bar.color} opacity={hovered === null || hovered === index ? 0.9 : 0.35} />;
              })}
              <text x={center} y={height - 7} textAnchor="middle" fill="var(--gray-500)" fontSize="10">{day.label}</text>
              {hovered === index && <text x={center} y={pad.top + 10} textAnchor="middle" fill={day.total >= 0 ? "var(--danger)" : "var(--success)"} fontSize="10" fontWeight="700">净差异 {day.total > 0 ? "+" : ""}{format(day.total)}</text>}
            </g>
          );
        })}
      </svg>
    </div>
  );
}
