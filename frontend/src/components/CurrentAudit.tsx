import type { CurrentAudit as CurrentAuditType } from "../types";

interface CurrentAuditProps {
  audit: CurrentAuditType | null;
}

const ICO = [
  { key: "input" as const, label: "Input", color: "var(--ico-input)" },
  { key: "cache" as const, label: "Cache", color: "var(--ico-cache)" },
  { key: "output" as const, label: "Output", color: "var(--ico-output)" },
];

const CX = 32, CY = 32, R = 20;

function formatK(v: number) {
  return v >= 1000 ? `${(v / 1000).toFixed(1)}K` : String(v);
}

function PieSlice({
  startAngle,
  endAngle,
  r,
  color,
  opacity,
}: {
  startAngle: number;
  endAngle: number;
  r: number;
  color: string;
  opacity: number;
}) {
  const pct = (endAngle - startAngle) / 360;
  const largeArc = pct > 0.5 ? 1 : 0;
  const toCart = (angle: number) => ({
    x: CX + r * Math.cos((angle * Math.PI) / 180),
    y: CY + r * Math.sin((angle * Math.PI) / 180),
  });
  const sa = toCart(startAngle);
  const ea = toCart(endAngle);

  // Full circle: use two 180° arcs
  let d: string;
  if (pct > 0.999) {
    const mid = toCart(startAngle + 180);
    d = `M${CX},${CY} L${sa.x.toFixed(1)},${sa.y.toFixed(1)} A${r},${r} 0 0,1 ${mid.x.toFixed(1)},${mid.y.toFixed(1)} A${r},${r} 0 0,1 ${ea.x.toFixed(1)},${ea.y.toFixed(1)} Z`;
  } else {
    d = `M${CX},${CY} L${sa.x.toFixed(1)},${sa.y.toFixed(1)} A${r},${r} 0 ${largeArc},1 ${ea.x.toFixed(1)},${ea.y.toFixed(1)} Z`;
  }
  return <path d={d} fill={color} fillOpacity={opacity} />;
}

export default function CurrentAudit({ audit }: CurrentAuditProps) {
  if (!audit) {
    return (
      <div className="flex items-center justify-center h-full text-sm text-gray-400">
        等待新请求...
      </div>
    );
  }

  const auditVals = [audit.input_audit, audit.cache_audit, audit.output_audit];
  const claimedVals = [audit.input_claimed, audit.cache_claimed, audit.output_claimed];
  const auditTotal = auditVals.reduce((s, v) => s + v, 0);
  const claimedTotal = claimedVals.reduce((s, v) => s + v, 0);

  return (
    <div className="flex flex-col gap-3">
      {/* Pass/fail */}
      <div className="flex items-center gap-2">
        <span
          className={`w-5 h-5 rounded-full flex items-center justify-center shrink-0
            ${audit.audit_passed ? "bg-success-subtle text-success" : "bg-danger-subtle text-danger"}`}
        >
          {audit.audit_passed ? (
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          ) : (
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          )}
        </span>
        <span className="text-xs font-semibold text-gray-600">
          {audit.audit_passed ? "审计通过" : "审计未通过"}
        </span>
      </div>

      {/* Two pie charts side by side: audit + claimed */}
      <div className="flex items-start gap-4">
        {/* Audit pie */}
        <div className="flex flex-col items-center gap-1">
          <span className="text-[10px] text-gray-400 font-medium">审计</span>
          <svg className="w-[64px] h-[64px] shrink-0" viewBox="0 0 64 64">
            <g transform={`rotate(-90 ${CX} ${CY})`}>
              {(() => {
                let a = 0;
                return ICO.map((ico, i) => {
                  const pct = auditTotal > 0 ? auditVals[i] / auditTotal : 0;
                  const sa = a;
                  a += pct * 360;
                  return <PieSlice key={i} startAngle={sa} endAngle={a} r={R} color={ico.color} opacity={0.85} />;
                });
              })()}
            </g>
            <circle cx={CX} cy={CY} r={8} fill="white" />
            <text x={CX} y={CY + 4} textAnchor="middle" fontSize="8" fontWeight={700} fill="var(--gray-700)">
              {formatK(auditTotal)}
            </text>
          </svg>
        </div>

        {/* Claimed pie */}
        <div className="flex flex-col items-center gap-1">
          <span className="text-[10px] text-gray-400 font-medium">声称</span>
          <svg className="w-[64px] h-[64px] shrink-0" viewBox="0 0 64 64">
            <g transform={`rotate(-90 ${CX} ${CY})`}>
              {(() => {
                let a = 0;
                return ICO.map((ico, i) => {
                  const pct = claimedTotal > 0 ? claimedVals[i] / claimedTotal : 0;
                  const sa = a;
                  a += pct * 360;
                  return <PieSlice key={i} startAngle={sa} endAngle={a} r={R} color={ico.color} opacity={0.4} />;
                });
              })()}
            </g>
            <circle cx={CX} cy={CY} r={8} fill="white" />
            <text x={CX} y={CY + 4} textAnchor="middle" fontSize="8" fontWeight={700} fill="var(--gray-500)">
              {formatK(claimedTotal)}
            </text>
          </svg>
        </div>

        {/* Legend */}
        <div className="flex flex-col gap-1 flex-1">
          {ICO.map((ico, i) => {
            const claimed = claimedVals[i];
            const real = auditVals[i];
            const diff = claimed - real;
            const diffRate = claimed > 0 ? (diff / claimed) * 100 : 0;
            const color = Math.abs(diffRate) < 3 ? "var(--success)" : Math.abs(diffRate) < 5 ? "var(--warning)" : "var(--danger)";
            return (
              <div key={ico.key} className="flex items-center gap-1.5">
                <span className="w-2 h-2 rounded-sm shrink-0" style={{ backgroundColor: ico.color }} />
                <span className="text-[10px] text-gray-500">{ico.label}</span>
                <span className="text-[10px] font-mono font-semibold ml-auto" style={{ color }}>
                  {diff >= 0 ? "+" : ""}{formatK(diff)} ({diffRate >= 0 ? "+" : ""}{diffRate.toFixed(1)}%)
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
