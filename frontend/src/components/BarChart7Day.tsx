import { useMemo, useState } from "react";
import type { DailyBreakdown } from "../types";

interface BarChart7DayProps {
  data: DailyBreakdown[];
}

interface BarGroup {
  day: string;
  date: string;
  input: { audit: number; claimed: number };
  cache: { audit: number; claimed: number };
  output: { audit: number; claimed: number };
}

const ICO_COLORS = {
  input: "var(--ico-input)",
  cache: "var(--ico-cache)",
  output: "var(--ico-output)",
};

export default function BarChart7Day({ data }: BarChart7DayProps) {
  const [hovered, setHovered] = useState<{ dayIdx: number; ico: string } | null>(null);

  const groups: BarGroup[] = useMemo(() => {
    return data.map((d) => {
      const parts = d.date.split("-");
      const label = parts.length === 3 ? `${parts[1]}-${parts[2]}` : d.date;
      return {
        day: label,
        date: d.date,
        input: { audit: d.input_detected, claimed: d.input_claimed },
        cache: { audit: d.cache_detected, claimed: d.cache_claimed },
        output: { audit: d.output_detected, claimed: d.output_claimed },
      };
    });
  }, [data]);

  if (!groups.length) {
    return (
      <div className="flex items-center justify-center h-full text-sm text-gray-500">
        暂无统计数据
      </div>
    );
  }

  // Y-axis range
  const allVals = groups.flatMap(g => [
    g.input.audit, g.input.claimed,
    g.cache.audit, g.cache.claimed,
    g.output.audit, g.output.claimed,
  ]);
  const maxVal = Math.max(...allVals, 1);
  const yMax = Math.ceil(maxVal / 1000) * 1000 || 1000;

  const padding = { top: 4, right: 4, bottom: 22, left: 28 };
  const svgW = 500, svgH = 200;
  const chartW = svgW - padding.left - padding.right;
  const chartH = svgH - padding.top - padding.bottom;

  const groupW = chartW / groups.length;
  const barW = groupW * 0.23;
  const gap = groupW * 0.04;

  const toY = (v: number) => padding.top + chartH - (v / yMax) * chartH;

  const yTicks = [0, Math.round(yMax / 2), yMax];
  const icoKeys = ["input", "cache", "output"] as const;

  const formatK = (v: number) => v >= 1000 ? `${(v / 1000).toFixed(0)}K` : String(v);
  const formatKAlways = (v: number) => (v / 1000).toFixed(1).replace(/\.0$/, "") + "K";

  return (
    <svg className="w-full h-full" viewBox={`0 0 ${svgW} ${svgH}`} preserveAspectRatio="xMidYMid meet">
      {/* Grid lines */}
      {yTicks.map((v, i) => (
        <g key={i}>
          <line x1={padding.left} y1={toY(v)} x2={svgW - padding.right} y2={toY(v)}
            stroke="var(--gray-300)" strokeWidth="0.5" />
          <text x={padding.left - 6} y={toY(v) + 5}
            fill="var(--gray-500)" fontSize="11" fontWeight="500" textAnchor="end">
            {formatKAlways(v)}
          </text>
        </g>
      ))}

      {/* Bars */}
      {groups.map((g, gi) => {
        const x0 = padding.left + gi * groupW + groupW / 2;
        return icoKeys.map((ico) => {
          const v = g[ico];
          const barCenterX = x0 + (icoKeys.indexOf(ico) - 1) * (barW + gap);
          const auditH = v.audit > 0 ? (v.audit / yMax) * chartH : 0;
          const claimedH = v.claimed > 0 ? (v.claimed / yMax) * chartH : 0;

          const isHovered = hovered?.dayIdx === gi && hovered?.ico === ico;
          const color = ICO_COLORS[ico];

          return (
            <g key={`${gi}-${ico}`}
              onMouseEnter={() => setHovered({ dayIdx: gi, ico })}
              onMouseLeave={() => setHovered(null)}
              className="cursor-pointer"
            >
              {/* Audit bar (solid fill) */}
              {auditH > 0 && (
                <rect
                  x={barCenterX - barW / 2}
                  y={toY(v.audit)}
                  width={barW}
                  height={Math.max(auditH, 1)}
                  fill={color}
                  fillOpacity={isHovered ? 1 : 0.9}
                  rx={1.5}
                />
              )}
              {/* Claimed outline (dashed stroke) */}
              {claimedH > 0 && (
                <rect
                  x={barCenterX - barW / 2}
                  y={toY(v.claimed)}
                  width={barW}
                  height={Math.max(claimedH, 1)}
                  fill="none"
                  stroke={color}
                  strokeOpacity={0.85}
                  strokeWidth={2}
                  strokeDasharray="4 2"
                  rx={1.5}
                />
              )}
            </g>
          );
        });
      })}

      {/* X-axis labels */}
      {groups.map((g, gi) => {
        const x0 = padding.left + gi * groupW + groupW / 2;
        return (
          <text key={gi} x={x0} y={svgH - 5}
            fill="var(--gray-600)" fontSize="12" fontWeight="500" textAnchor="middle">
            {g.day}
          </text>
        );
      })}

      {/* Tooltip */}
      {hovered && (() => {
        const g = groups[hovered.dayIdx];
        const v = g[hovered.ico as keyof BarGroup] as { audit: number; claimed: number };
        const x0 = padding.left + hovered.dayIdx * groupW + groupW / 2;
        const icoIdx = icoKeys.indexOf(hovered.ico as typeof icoKeys[number]);
        const barCenterX = x0 + (icoIdx - 1) * (barW + gap);
        const tooltipW = 110, tooltipH = 38;
        const tx = Math.min(Math.max(barCenterX - tooltipW / 2, 2), svgW - tooltipW - 2);
        return (
          <g>
            <rect x={tx} y={4} width={tooltipW} height={tooltipH} rx={6}
              fill="var(--gray-900)" fillOpacity={0.9} />
            <text x={tx + tooltipW / 2} y={19} fill="white" fontSize="11" textAnchor="middle" fontWeight={700}>
              {hovered.ico.toUpperCase()}
            </text>
            <text x={tx + tooltipW / 2} y={35} fill="var(--gray-300)" fontSize="10" textAnchor="middle">
              审计 {formatK(v.audit)} · 声称 {formatK(v.claimed)}
            </text>
          </g>
        );
      })()}
    </svg>
  );
}
