import type { CurrentAudit as CurrentAuditType } from "../types";

interface CurrentAuditProps {
  audit: CurrentAuditType | null;
}

const ICO = [
  { key: "input" as const, label: "Input", color: "var(--ico-input)" },
  { key: "cache" as const, label: "Cache", color: "var(--ico-cache)" },
  { key: "output" as const, label: "Output", color: "var(--ico-output)" },
];

const CX = 50, CY = 50, R_AUDIT = 18;

function formatK(v: number) {
  return v >= 1000 ? `${(v / 1000).toFixed(1)}K` : String(v);
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
  const maxR = Math.max(R_AUDIT, ...ICO.map((_, i) => R_AUDIT * Math.max(1, claimedVals[i] / Math.max(1, auditVals[i]))));

  const pt = (r: number, deg: number) => ({
    x: CX + r * Math.cos((deg * Math.PI) / 180),
    y: CY + r * Math.sin((deg * Math.PI) / 180),
  });

  return (
    <div className="flex items-center gap-3">
      {/* Pie SVG */}
      <svg className="w-[90px] h-[90px] shrink-0" viewBox="0 0 100 100">
        <g transform={`rotate(-90 ${CX} ${CY})`}>
          {(() => {
            let startAngle = 0;
            return ICO.map((ico, i) => {
              const pct = auditTotal > 0 ? auditVals[i] / auditTotal : 0;
              const endAngle = startAngle + pct * 360;
              const R_claimed = Math.max(R_AUDIT, R_AUDIT * (claimedVals[i] / Math.max(1, auditVals[i])));
              const la = pct > 0.5 ? 1 : 0;

              const sa = pt(R_claimed, startAngle);
              const ea = pt(R_claimed, endAngle);
              const sa_a = pt(R_AUDIT, startAngle);
              const ea_a = pt(R_AUDIT, endAngle);

              const result = (
                <g key={ico.key}>
                  {/* Claimed layer (translucent, outer) */}
                  <path
                    d={`M${CX},${CY} L${sa.x.toFixed(1)},${sa.y.toFixed(1)} A${R_claimed.toFixed(1)},${R_claimed.toFixed(1)} 0 ${la},1 ${ea.x.toFixed(1)},${ea.y.toFixed(1)} Z`}
                    fill={ico.color} fillOpacity={0.25}
                  />
                  {/* Audit layer (solid, inner) */}
                  <path
                    d={`M${CX},${CY} L${sa_a.x.toFixed(1)},${sa_a.y.toFixed(1)} A${R_AUDIT},${R_AUDIT} 0 ${la},1 ${ea_a.x.toFixed(1)},${ea_a.y.toFixed(1)} Z`}
                    fill={ico.color} fillOpacity={0.85}
                  />
                  {/* Separator line */}
                  <line x1={CX} y1={CY}
                    x2={pt(maxR, startAngle).x.toFixed(1)} y2={pt(maxR, startAngle).y.toFixed(1)}
                    stroke="white" strokeWidth="0.8"
                  />
                </g>
              );
              startAngle = endAngle;
              return result;
            });
          })()}
        </g>
        <circle cx={CX} cy={CY} r={6} fill="white" />
      </svg>

      {/* Legend + diff on the right */}
      <div className="flex flex-col gap-1.5 flex-1 min-w-0">
        {ICO.map((ico, i) => {
          const claimed = claimedVals[i];
          const real = auditVals[i];
          const diff = claimed - real;
          const diffRate = claimed > 0 ? (diff / claimed) * 100 : 0;
          const diffColor = Math.abs(diffRate) < 3 ? "var(--success)" : Math.abs(diffRate) < 5 ? "var(--warning)" : "var(--danger)";
          return (
            <div key={ico.key} className="flex items-center gap-1.5">
              <span className="w-2.5 h-2.5 rounded-sm shrink-0" style={{ backgroundColor: ico.color }} />
              <span className="text-[11px] text-gray-700 font-medium flex-1">{ico.label}</span>
              <div className="flex flex-col items-end shrink-0">
                <span className="text-[10px] font-mono text-gray-500 leading-tight">审计 {formatK(real)}</span>
                <span className="text-[10px] font-mono text-gray-400 leading-tight">声称 {formatK(claimed)}</span>
              </div>
              <span className="text-[10px] font-mono font-semibold w-16 text-right" style={{ color: diffColor }}>
                {diff >= 0 ? "+" : ""}{formatK(diff)} ({diffRate >= 0 ? "+" : ""}{diffRate.toFixed(1)}%)
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
