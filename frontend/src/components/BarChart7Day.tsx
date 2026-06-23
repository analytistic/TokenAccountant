import { useMemo, useState, useRef, useEffect, useCallback } from "react";
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
  const svgRef = useRef<SVGSVGElement>(null);
  const [width, setWidth] = useState(0);

  // Observe the parent element (card-body) which actually resizes with the grid
  useEffect(() => {
    const parent = svgRef.current?.parentElement;
    if (!parent) return;
    const ro = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setWidth(entry.contentRect.width);
      }
    });
    ro.observe(parent);
    return () => ro.disconnect();
  }, []);

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

  // Compute layout based on actual parent width
  const layout = useMemo(() => {
    const w = Math.max(width || 300, 200);
    const h = Math.max(130, w * 0.4);
    const padL = Math.max(26, w * 0.07);
    const pad = { top: 4, right: 6, bottom: 20, left: padL };
    const chartW = w - pad.left - pad.right;
    const chartH = h - pad.top - pad.bottom;
    return { svgW: w, svgH: h, pad, chartW, chartH };
  }, [width]);

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

  const { svgW, svgH, pad, chartW, chartH } = layout;
  const groupW = chartW / groups.length;
  const barW = Math.max(4, groupW * 0.23);
  const gap = groupW * 0.04;

  const toY = useCallback(
    (v: number) => pad.top + chartH - (v / yMax) * chartH,
    [pad.top, chartH, yMax]
  );

  const yTicks = [0, Math.round(yMax / 2), yMax];
  const icoKeys = ["input", "cache", "output"] as const;

  const formatK = (v: number) => v >= 1000 ? `${(v / 1000).toFixed(0)}K` : String(v);
  const formatKAlways = (v: number) => (v / 1000).toFixed(1).replace(/\.0$/, "") + "K";

  return (
    <svg ref={svgRef} className="w-full h-full" viewBox={`0 0 ${layout.svgW} ${layout.svgH}`}
      preserveAspectRatio="xMidYMid meet">
      {/* Grid lines */}
      {yTicks.map((v, i) => (
        <g key={i}>
          <line x1={pad.left} y1={toY(v)} x2={svgW - pad.right} y2={toY(v)}
            stroke="var(--gray-300)" strokeWidth="0.5" />
          <text x={pad.left - 6} y={toY(v) + 5}
            fill="var(--gray-500)" fontSize={Math.max(9, pad.left * 0.35)} fontWeight="500" textAnchor="end">
            {formatKAlways(v)}
          </text>
        </g>
      ))}

      {/* Bars */}
      {groups.map((g, gi) => {
        const x0 = pad.left + gi * groupW + groupW / 2;
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
        const x0 = pad.left + gi * groupW + groupW / 2;
        return (
          <text key={gi} x={x0} y={svgH - 4}
            fill="var(--gray-600)" fontSize={Math.max(10, groupW * 0.14)} fontWeight="500" textAnchor="middle">
            {g.day}
          </text>
        );
      })}

      {/* Tooltip */}
      {hovered && (() => {
        const g = groups[hovered.dayIdx];
        const v = g[hovered.ico as keyof BarGroup] as { audit: number; claimed: number };
        const x0 = pad.left + hovered.dayIdx * groupW + groupW / 2;
        const icoIdx = icoKeys.indexOf(hovered.ico as typeof icoKeys[number]);
        const barCenterX = x0 + (icoIdx - 1) * (barW + gap);
        const tooltipW = Math.min(120, svgW * 0.25), tooltipH = 38;
        const tx = Math.min(Math.max(barCenterX - tooltipW / 2, 2), svgW - tooltipW - 2);
        const tooltipFont = Math.max(10, tooltipW * 0.09);
        return (
          <g>
            <rect x={tx} y={4} width={tooltipW} height={tooltipH} rx={6}
              fill="var(--gray-900)" fillOpacity={0.9} />
            <text x={tx + tooltipW / 2} y={19} fill="white" fontSize={tooltipFont} textAnchor="middle" fontWeight={700}>
              {hovered.ico.toUpperCase()}
            </text>
            <text x={tx + tooltipW / 2} y={35} fill="var(--gray-300)" fontSize={tooltipFont} textAnchor="middle">
              审计 {formatK(v.audit)} · 声称 {formatK(v.claimed)}
            </text>
          </g>
        );
      })()}
    </svg>
  );
}
