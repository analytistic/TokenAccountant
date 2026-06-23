import { useMemo } from "react";
import type { TrendPoint } from "../types";

interface TrendChartProps {
  data: TrendPoint[];
}

// ---- Catmull-Rom to cubic Bezier ----
function catmullRomToBezier(points: { x: number; y: number }[]) {
  const curves: { cp1x: number; cp1y: number; cp2x: number; cp2y: number; x: number; y: number }[] = [];
  for (let i = 0; i < points.length - 1; i++) {
    const p0 = points[Math.max(0, i - 1)];
    const p1 = points[i];
    const p2 = points[i + 1];
    const p3 = points[Math.min(points.length - 1, i + 2)];
    curves.push({
      cp1x: p1.x + (p2.x - p0.x) / 6,
      cp1y: p1.y + (p2.y - p0.y) / 6,
      cp2x: p2.x - (p3.x - p1.x) / 6,
      cp2y: p2.y - (p3.y - p1.y) / 6,
      x: p2.x,
      y: p2.y,
    });
  }
  return curves;
}

// ---- Single SubChart ----
interface SubChartProps {
  label: string;
  color: string;
  auditValues: number[];
  claimedValues: number[];
}

function SubChart({ label, color, auditValues, claimedValues }: SubChartProps) {
  const padding = { left: 4, right: 52, top: 6, bottom: 4 };
  const width = 740;
  const height = 85;
  const chartW = width - padding.left - padding.right;
  const chartH = height - padding.top - padding.bottom;

  // Compute diff rate for latest point
  const lastClaimed = claimedValues[claimedValues.length - 1] || 0;
  const lastAudit = auditValues[auditValues.length - 1] || 0;
  const diffRate = lastClaimed > 0 ? ((lastClaimed - lastAudit) / lastClaimed) * 100 : 0;
  const diffColor = Math.abs(diffRate) < 3 ? "var(--success)" : Math.abs(diffRate) < 5 ? "var(--warning)" : "var(--danger)";

  // Scale Y: find max across both series
  const allValues = [...auditValues, ...claimedValues];
  const maxVal = Math.max(...allValues, 1);

  const toX = (i: number) => padding.left + (i / (auditValues.length - 1)) * chartW;
  const toY = (v: number) => padding.top + chartH - (v / maxVal) * chartH;

  // Build points
  const auditPts = auditValues.map((v, i) => ({ x: toX(i), y: toY(v) }));
  const claimedPts = claimedValues.map((v, i) => ({ x: toX(i), y: toY(v) }));

  // Catmull-Rom curves
  const auditCurves = catmullRomToBezier(auditPts);
  const claimedCurves = catmullRomToBezier(claimedPts);

  // Audit line path (solid)
  const auditPath = auditCurves.length > 0
    ? `M${auditPts[0].x},${auditPts[0].y} ` +
      auditCurves.map(c => `C${c.cp1x.toFixed(1)},${c.cp1y.toFixed(1)} ${c.cp2x.toFixed(1)},${c.cp2y.toFixed(1)} ${c.x.toFixed(1)},${c.y.toFixed(1)}`).join(" ")
    : "";

  // Claimed line path (dashed — we use stroke-dasharray on the element)
  const claimedPath = claimedCurves.length > 0
    ? `M${claimedPts[0].x},${claimedPts[0].y} ` +
      claimedCurves.map(c => `C${c.cp1x.toFixed(1)},${c.cp1y.toFixed(1)} ${c.cp2x.toFixed(1)},${c.cp2y.toFixed(1)} ${c.x.toFixed(1)},${c.y.toFixed(1)}`).join(" ")
    : "";

  // Diff area fill (between claimed and audit)
  const diffAreaPath = claimedCurves.length > 0
    ? `M${claimedPts[0].x},${claimedPts[0].y} ` +
      claimedCurves.map(c => `C${c.cp1x.toFixed(1)},${c.cp1y.toFixed(1)} ${c.cp2x.toFixed(1)},${c.cp2y.toFixed(1)} ${c.x.toFixed(1)},${c.y.toFixed(1)}`).join(" ") +
      ` L${auditPts[auditPts.length - 1].x},${auditPts[auditPts.length - 1].y} ` +
      [...auditCurves].reverse().map(c => `C${c.cp2x.toFixed(1)},${c.cp2y.toFixed(1)} ${c.cp1x.toFixed(1)},${c.cp1y.toFixed(1)} ${c.x.toFixed(1)},${c.y.toFixed(1)}`).join(" ") +
      " Z"
    : "";

  // Y-axis ticks
  const yTicks = [0, Math.round(maxVal / 2), maxVal].map(v => ({
    label: v >= 1000 ? `${(v / 1000).toFixed(0)}K` : String(v),
    y: toY(v),
  }));

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
      <svg className="w-full flex-1 min-h-0" viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="xMidYMid meet">
        {/* Y-axis ticks */}
        {yTicks.map((t, i) => (
          <g key={i}>
            <line x1={padding.left} y1={t.y} x2={width - padding.right + 4} y2={t.y}
              stroke="var(--gray-200)" strokeWidth="0.5" />
            <text x={width - padding.right + 8} y={t.y + 3}
              fill="var(--gray-400)" fontSize="8" textAnchor="start">
              {t.label}
            </text>
          </g>
        ))}

        {!allZero && (
          <>
            {/* Diff area */}
            <path d={diffAreaPath} fill={color} fillOpacity="0.10" />
            {/* Claimed line (dashed) */}
            <path d={claimedPath} fill="none" stroke={color} strokeWidth="1.5" strokeOpacity="0.35"
              strokeLinecap="round" strokeDasharray="4 3" />
            {/* Audit line (solid) */}
            <path d={auditPath} fill="none" stroke={color} strokeWidth="2" strokeLinecap="round" />
          </>
        )}
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
    <div className="flex flex-col flex-1 min-h-0 gap-1">
      {ico.map((s) => (
        <SubChart key={s.label} label={s.label} color={s.color} auditValues={s.audit} claimedValues={s.claimed} />
      ))}
      {/* Legend */}
      <div className="flex items-center justify-center gap-6 shrink-0 pb-1">
        <span className="flex items-center gap-1.5 text-[10px] text-gray-400">
          <span className="inline-block w-3 h-0.5 rounded bg-gray-600" /> 审计值（基准）
        </span>
        <span className="flex items-center gap-1.5 text-[10px] text-gray-400">
          <span className="inline-block w-3 h-0.5 rounded bg-gray-400 border-dashed" /> 声称值
        </span>
        <span className="flex items-center gap-1.5 text-[10px] text-gray-400">
          <span className="inline-block w-3 h-2 rounded-sm bg-black/10" /> 差值区域
        </span>
      </div>
    </div>
  );
}
