import { useMemo, useState } from "react";
import type { DailyBreakdown } from "../types";

interface BarChart7DayProps {
  data: DailyBreakdown[];
}

const ICO_COLORS = {
  input: "var(--ico-input)",
  cache: "var(--ico-cache)",
  output: "var(--ico-output)",
};

// Fixed internal coordinate system — SVG will stretch to fill container
const VW = 1000, VH = 200;
const PAD = { top: 6, right: 12, bottom: 24, left: 64 };

export default function BarChart7Day({ data }: BarChart7DayProps) {
  const [hovered, setHovered] = useState<{ dayIdx: number; ico: string } | null>(null);

  const groups = useMemo(() => {
    return data.map((d) => {
      const parts = d.date.split("-");
      return {
        day: parts.length === 3 ? `${parts[1]}-${parts[2]}` : d.date,
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

  const allVals = groups.flatMap(g => [
    g.input.audit, g.input.claimed,
    g.cache.audit, g.cache.claimed,
    g.output.audit, g.output.claimed,
  ]);
  const maxVal = Math.max(...allVals, 1);
  const yMax = Math.ceil(maxVal / 1000) * 1000 || 1000;

  const chartW = VW - PAD.left - PAD.right;
  const chartH = VH - PAD.top - PAD.bottom;
  const groupW = chartW / groups.length;
  const barW = Math.max(4, groupW * 0.14);
  const gap = groupW * 0.04;

  const toY = (v: number) => PAD.top + chartH - (v / yMax) * chartH;
  const yTicks = [0, Math.round(yMax / 2), yMax];
  const icoKeys = ["input", "cache", "output"] as const;

  const formatK = (v: number) => v >= 1000 ? `${(v / 1000).toFixed(0)}K` : String(v);
  const fmtY = (v: number) => (v / 1000).toFixed(1).replace(/\.0$/, "") + "K";

  return (
    <svg
      viewBox={`0 0 ${VW} ${VH}`}
      preserveAspectRatio="none"
      style={{ width: "100%", height: "100%", display: "block" }}
    >
      {/* Grid lines */}
      {yTicks.map((v, i) => (
        <g key={i}>
          <line x1={PAD.left} y1={toY(v)} x2={VW - PAD.right} y2={toY(v)}
            stroke="var(--gray-300)" strokeWidth="0.5" />
          <text x={PAD.left - 8} y={toY(v) + 6}
            fill="var(--gray-500)" fontSize="14" fontWeight="500" textAnchor="end">
            {fmtY(v)}
          </text>
        </g>
      ))}

      {/* Bars */}
      {groups.map((g, gi) => {
        const x0 = PAD.left + gi * groupW + groupW / 2;
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
        const x0 = PAD.left + gi * groupW + groupW / 2;
        return (
          <text key={gi} x={x0} y={VH - 4}
            fill="var(--gray-600)" fontSize="14" fontWeight="500" textAnchor="middle">
            {g.day}
          </text>
        );
      })}

      {/* Tooltip */}
      {hovered && (() => {
        const g = groups[hovered.dayIdx];
        const icoKey = hovered.ico as typeof icoKeys[number];
        const v = g[icoKey];
        const x0 = PAD.left + hovered.dayIdx * groupW + groupW / 2;
        const icoIdx = icoKeys.indexOf(hovered.ico as typeof icoKeys[number]);
        const barCenterX = x0 + (icoIdx - 1) * (barW + gap);
        const tW = 200, tH = 56;
        const barTop = toY(Math.max(v.audit, v.claimed));
        const ty = Math.max(2, barTop - tH - 14);
        const tx = Math.min(Math.max(barCenterX - tW / 2, 2), VW - tW - 2);
        return (
          <g>
            <rect x={tx} y={ty} width={tW} height={tH} rx={6}
              fill="var(--gray-900)" fillOpacity={0.9} />
            <text x={tx + tW / 2} y={ty + 22} fill="white" fontSize="20" textAnchor="middle" fontWeight="700">
              {icoKey.toUpperCase()}
            </text>
            <text x={tx + tW / 2} y={ty + 46} fill="var(--gray-300)" fontSize="16" textAnchor="middle">
              审计 {formatK(v.audit)} · 声称 {formatK(v.claimed)}
            </text>
          </g>
        );
      })()}
    </svg>
  );
}
