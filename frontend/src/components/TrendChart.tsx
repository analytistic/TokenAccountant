import { useMemo } from "react";
import type { TrendPoint } from "../types";

interface TrendChartProps {
  data: TrendPoint[];
}

// ---- Single SubChart ----
interface SubChartProps {
  label: string;
  color: string;
  auditValues: number[];
  claimedValues: number[];
}

function SubChart({ label, color, auditValues, claimedValues }: SubChartProps) {
  const padding = { left: 4, right: 52, top: 10, bottom: 6 };
  const width = 740;
  const height = 85;
  const chartW = width - padding.left - padding.right;
  const chartH = height - padding.top - padding.bottom;

  // Compute diff rate for latest point
  const lastClaimed = claimedValues[claimedValues.length - 1] || 0;
  const lastAudit = auditValues[auditValues.length - 1] || 0;
  const diffRate = lastClaimed > 0 ? ((lastClaimed - lastAudit) / lastClaimed) * 100 : 0;
  const diffColor =
    Math.abs(diffRate) < 3 ? "var(--success)" : Math.abs(diffRate) < 5 ? "var(--warning)" : "var(--danger)";

  // Scale Y
  const allValues = [...auditValues, ...claimedValues];
  const maxVal = Math.max(...allValues, 1);
  // Pad Y axis to give headroom
  const yMax = maxVal * 1.15;

  const toX = (i: number) => padding.left + (i / (auditValues.length - 1)) * chartW;
  const toY = (v: number) => padding.top + chartH - (v / yMax) * chartH;
  const baselineY = padding.top + chartH; // bottom of chart

  // Build point arrays
  const auditPts = auditValues.map((v, i) => ({ x: toX(i), y: toY(v) }));
  const claimedPts = claimedValues.map((v, i) => ({ x: toX(i), y: toY(v) }));

  // Polyline paths (straight line segments, no smoothing interpolation)
  const auditPolyline = auditPts.map((p, i) =>
    `${i === 0 ? "M" : "L"}${p.x.toFixed(1)},${p.y.toFixed(1)}`
  ).join(" ");

  const claimedPolyline = claimedPts.map((p, i) =>
    `${i === 0 ? "M" : "L"}${p.x.toFixed(1)},${p.y.toFixed(1)}`
  ).join(" ");

  // Fill area between claimed (top edge) and audit (bottom edge)
  // Polygon: forward along claimed, then backward along audit → close
  const fillPath = (() => {
    const N = auditPts.length;
    let d = "";
    // Forward along claimed line (top edge of fill)
    for (let i = 0; i < N; i++) {
      d += `${i === 0 ? "M" : "L"}${claimedPts[i].x.toFixed(1)},${claimedPts[i].y.toFixed(1)} `;
    }
    // Down to audit at last point
    d += `L${auditPts[N - 1].x.toFixed(1)},${auditPts[N - 1].y.toFixed(1)} `;
    // Backward along audit line (bottom edge of fill)
    for (let i = N - 2; i >= 0; i--) {
      d += `L${auditPts[i].x.toFixed(1)},${auditPts[i].y.toFixed(1)} `;
    }
    d += "Z";
    return d;
  })();

  // Y-axis tick values
  const ticks = [
    { value: 0, y: baselineY },
    { value: yMax / 2, y: toY(yMax / 2) },
    { value: yMax, y: toY(yMax) },
  ];

  const allZero = auditValues.every(v => v === 0) && claimedValues.every(v => v === 0);

  return (
    <div className="sub-chart flex-1 flex flex-col min-h-0">
      {/* Header */}
      <div className="flex items-center justify-between h-[14px] shrink-0 px-0.5">
        <span className="text-[9px] font-bold uppercase tracking-wider flex items-center gap-1" style={{ color }}>
          <span className="inline-block w-[5px] h-[5px] rounded-[1.5px] shrink-0" style={{ backgroundColor: color }} />
          {label}
        </span>
      </div>

      {/* Chart */}
      <svg className="w-full flex-1 min-h-0" viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none">
        {/* Grid lines */}
        {ticks.map((t, i) => (
          <g key={i}>
            <line x1={padding.left} y1={t.y} x2={width - padding.right + 4} y2={t.y}
              stroke="var(--gray-200)" strokeWidth="0.5" />
            <text x={width - padding.right + 8} y={t.y + 3}
              fill="var(--gray-400)" fontSize="8" textAnchor="start">
              {t.value >= 1000 ? `${(t.value / 1000).toFixed(0)}K` : String(Math.round(t.value))}
            </text>
          </g>
        ))}

        {/* Fill area between claimed and audit */}
        <path d={fillPath} fill={color} fillOpacity="0.12" />

        {/* Claimed line (dashed, thinner) */}
        <path d={claimedPolyline} fill="none" stroke={color} strokeWidth="1.5"
          strokeOpacity="0.4" strokeLinejoin="round" strokeLinecap="round"
          strokeDasharray="5 3" />

        {/* Audit line (solid, thicker) */}
        <path d={auditPolyline} fill="none" stroke={color} strokeWidth="2.2"
          strokeLinejoin="round" strokeLinecap="round" />
      </svg>

      {/* Diff rate badge */}
      <div className="shrink-0 h-5 flex items-center justify-end px-1">
        {allZero ? (
          <span className="text-[10px] text-gray-400">暂无趋势数据</span>
        ) : (
          <span className="text-xs font-bold font-mono" style={{ color: diffColor }}>
            {diffRate >= 0 ? "+" : ""}{diffRate.toFixed(1)}%
          </span>
        )}
      </div>
    </div>
  );
}

// ---- Main TrendChart ----
export default function TrendChart({ data }: TrendChartProps) {
  const ico = useMemo(() => {
    const input_audit: number[] = [];
    const input_claimed: number[] = [];
    const cache_audit: number[] = [];
    const cache_claimed: number[] = [];
    const output_audit: number[] = [];
    const output_claimed: number[] = [];

    data.forEach((p) => {
      input_audit.push(p.input_detected);
      input_claimed.push(p.input_claimed);
      cache_audit.push(p.cache_detected);
      cache_claimed.push(p.cache_claimed);
      output_audit.push(p.output_detected);
      output_claimed.push(p.output_claimed);
    });

    return [
      { label: "Input", color: "var(--ico-input)", audit: input_audit, claimed: input_claimed },
      { label: "Cache", color: "var(--ico-cache)", audit: cache_audit, claimed: cache_claimed },
      { label: "Output", color: "var(--ico-output)", audit: output_audit, claimed: output_claimed },
    ];
  }, [data]);

  return (
    <div className="flex flex-col flex-1 min-h-0 gap-0.5">
      {ico.map((s) => (
        <SubChart key={s.label} label={s.label} color={s.color}
          auditValues={s.audit} claimedValues={s.claimed} />
      ))}
      {/* Legend */}
      <div className="flex items-center justify-center gap-6 shrink-0 pb-1">
        <span className="flex items-center gap-1.5 text-[10px] text-gray-400">
          <span className="inline-block w-3 h-[2px] rounded bg-current" /> 审计值（基准）
        </span>
        <span className="flex items-center gap-1.5 text-[10px] text-gray-400">
          <span className="inline-block w-3 h-[1px] rounded border border-dashed border-current" /> 声称值
        </span>
        <span className="flex items-center gap-1.5 text-[10px] text-gray-400">
          <span className="inline-block w-3 h-2 rounded-sm bg-black/10" /> 差值区域
        </span>
      </div>
    </div>
  );
}
