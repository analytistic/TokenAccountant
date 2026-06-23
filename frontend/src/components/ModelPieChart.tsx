import { useMemo, useState } from "react";
import type { ModelBreakdown } from "../types";

interface ModelPieChartProps {
  data: ModelBreakdown[];
}

const PALETTE = [
  "#60A5FA", "#FB923C", "#34D399", "#A78BFA", "#F472B6",
  "#FBBF24", "#38BDF8", "#FB7185",
];

const MAX_SECTORS = 5; // extra goes to "Other"
const CX = 50, CY = 50, R_AUDIT = 22;

export default function ModelPieChart({ data }: ModelPieChartProps) {
  const [dimIdx, setDimIdx] = useState<number | null>(null);

  const { sectors, total } = useMemo(() => {
    if (!data.length) return { sectors: [], total: 0 };

    // Sort by audit total desc
    const sorted = [...data].sort((a, b) =>
      (b.input_total + b.cache_total + b.output_total) -
      (a.input_total + a.cache_total + a.output_total)
    );

    const top = sorted.slice(0, MAX_SECTORS).map((m, i) => ({
      name: m.model,
      audit: m.input_total + m.cache_total + m.output_total,
      claimed: m.input_claimed_total + m.cache_claimed_total + m.output_claimed_total,
      color: PALETTE[i % PALETTE.length],
    }));

    // Aggregate remaining into "Other"
    if (sorted.length > MAX_SECTORS) {
      const rest = sorted.slice(MAX_SECTORS);
      const otherAudit = rest.reduce((s, m) => s + m.input_total + m.cache_total + m.output_total, 0);
      const otherClaimed = rest.reduce((s, m) => s + m.input_claimed_total + m.cache_claimed_total + m.output_claimed_total, 0);
      if (otherAudit > 0) {
        top.push({ name: "Other", audit: otherAudit, claimed: otherClaimed, color: "#9CA3AF" });
      }
    }

    const t = top.reduce((s, m) => s + m.audit, 0);
    return { sectors: top, total: t };
  }, [data]);

  if (!sectors.length) return null;

  // Polar to cartesian
  const pt = (r: number, deg: number) => ({
    x: CX + r * Math.cos((deg * Math.PI) / 180),
    y: CY + r * Math.sin((deg * Math.PI) / 180),
  });

  // Max radius for separator lines
  const maxR = Math.max(R_AUDIT, ...sectors.map(m => R_AUDIT * Math.max(1, m.claimed / Math.max(1, m.audit))));

  const formatK = (v: number) => v >= 1000 ? `${(v / 1000).toFixed(1)}K` : String(v);

  return (
    <div className="flex flex-col items-center">
      {/* Pie SVG */}
      <svg className="w-[120px] h-[120px]" viewBox="0 0 100 100">
        <g transform={`rotate(-90 ${CX} ${CY})`}>
          {sectors.reduce<React.ReactNode[]>((acc, m, i) => {
            let startAngle = 0;
            // Recalculate startAngle by summing previous
            for (let j = 0; j < i; j++) {
              startAngle += (sectors[j].audit / total) * 360;
            }
            const pct = m.audit / total;
            const endAngle = startAngle + pct * 360;
            const R_claimed = Math.max(R_AUDIT, R_AUDIT * (m.claimed / Math.max(1, m.audit)));
            const largeArc = pct > 0.5 ? 1 : 0;

            const sa = pt(R_claimed, startAngle);
            const ea = pt(R_claimed, endAngle);
            const sa_audit = pt(R_AUDIT, startAngle);
            const ea_audit = pt(R_AUDIT, endAngle);

            const dimmed = dimIdx !== null && dimIdx !== i;

            const elements = (
              <g key={i}
                className="cursor-pointer transition-opacity duration-200"
                style={{ opacity: dimmed ? 0.1 : 1 }}
                onMouseEnter={() => setDimIdx(i)}
                onMouseLeave={() => setDimIdx(null)}
              >
                {/* Claimed layer (translucent outer) */}
                <path
                  d={`M${CX},${CY} L${sa.x.toFixed(1)},${sa.y.toFixed(1)} A${R_claimed.toFixed(1)},${R_claimed.toFixed(1)} 0 ${largeArc},1 ${ea.x.toFixed(1)},${ea.y.toFixed(1)} Z`}
                  fill={m.color}
                  fillOpacity={0.25}
                />
                {/* Audit layer (solid inner) */}
                <path
                  d={`M${CX},${CY} L${sa_audit.x.toFixed(1)},${sa_audit.y.toFixed(1)} A${R_AUDIT},${R_AUDIT} 0 ${largeArc},1 ${ea_audit.x.toFixed(1)},${ea_audit.y.toFixed(1)} Z`}
                  fill={m.color}
                  fillOpacity={0.8}
                />
                {/* Separator line from center to outer edge (at start angle) */}
                <line x1={CX} y1={CY}
                  x2={pt(maxR, startAngle).x.toFixed(1)}
                  y2={pt(maxR, startAngle).y.toFixed(1)}
                  stroke="white" strokeWidth="0.8"
                />
              </g>
            );
            acc.push(elements);
            return acc;
          }, [] as React.ReactNode[])}
        </g>
        {/* Center dot */}
        <circle cx={CX} cy={CY} r={4} fill="white" />
      </svg>

      {/* Legend */}
      <div className="flex flex-col gap-0.5 mt-2 w-full">
        {sectors.map((m, i) => (
          <div
            key={i}
            className={`flex items-center gap-2 text-[11px] px-1 py-0.5 rounded cursor-default transition-colors
              ${dimIdx === i ? "bg-gray-100" : ""}`}
            onMouseEnter={() => setDimIdx(i)}
            onMouseLeave={() => setDimIdx(null)}
          >
            <span className="w-2 h-2 rounded-sm shrink-0" style={{ backgroundColor: m.color }} />
            <span className="text-gray-700 flex-1 truncate">{m.name}</span>
            <span className="font-mono text-gray-500 tabular-nums">审计 {formatK(m.audit)}</span>
            <span className="font-mono text-gray-400 tabular-nums">声称 {formatK(m.claimed)}</span>
          </div>
        ))}
      </div>
      <p className="text-[10px] text-gray-400 mt-1">实心=审计占比 · 半透明外环=声称多出部分</p>
    </div>
  );
}
