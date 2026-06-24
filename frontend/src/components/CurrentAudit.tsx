import type { CurrentAudit as CurrentAuditType } from "../types";

interface CurrentAuditProps {
  audit: CurrentAuditType | null;
}

const ICO = [
  { key: "input" as const, label: "Input", color: "var(--ico-input)" },
  { key: "cache" as const, label: "Cache", color: "var(--ico-cache)" },
  { key: "output" as const, label: "Output", color: "var(--ico-output)" },
];

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

  const auditVals = ICO.map((ico) => audit[`${ico.key}_audit` as keyof CurrentAuditType] as number);
  const claimedVals = ICO.map((ico) => audit[`${ico.key}_claimed` as keyof CurrentAuditType] as number);
  const auditTotal = auditVals.reduce((s, v) => s + v, 0);
  const claimedTotal = claimedVals.reduce((s, v) => s + v, 0);
  const maxTotal = Math.max(auditTotal, claimedTotal, 1);

  // Build segments: each ICO is a contiguous block in the bar
  // Audit = solid bar, claimed = lighter extension where claimed > audit
  let auditOffset = 0;
  let claimedOffset = 0;

  return (
    <div className="flex flex-col gap-2">
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

      {/* Single segmented bar: audit segments + claimed overhang */}
      <div className="flex items-center gap-2">
        <span className="text-[10px] font-semibold text-gray-500 w-8 shrink-0">审计</span>
        <div className="flex-1 h-4 bg-gray-100 relative">
          {/* Audit segments (solid) */}
          {(() => {
            auditOffset = 0;
            return ICO.map((ico, i) => {
              const w = (auditVals[i] / maxTotal) * 100;
              const el = (
                <div
                  key={`a-${ico.key}`}
                  className="absolute inset-y-0"
                  style={{ left: `${auditOffset}%`, width: `${w}%`, backgroundColor: ico.color }}
                />
              );
              auditOffset += w;
              return el;
            });
          })()}
          {/* Claimed overhang: only where claimed > audit */}
          {(() => {
            auditOffset = 0;
            claimedOffset = 0;
            return ICO.map((ico, i) => {
              const aW = (auditVals[i] / maxTotal) * 100;
              const cW = (claimedVals[i] / maxTotal) * 100;
              const overhang = Math.max(0, cW - aW);
              const el = overhang > 0 ? (
                <div
                  key={`c-${ico.key}`}
                  className="absolute inset-y-0"
                  style={{ left: `${auditOffset + aW}%`, width: `${overhang}%`, backgroundColor: ico.color, opacity: 0.3 }}
                />
              ) : null;
              auditOffset += aW;
              return el;
            });
          })()}
        </div>
        <span className="text-[10px] font-mono font-semibold text-gray-700 w-11 text-right shrink-0 tabular-nums">
          {formatK(auditTotal)}
        </span>
      </div>

      {/* Claimed total bar */}
      <div className="flex items-center gap-2">
        <span className="text-[10px] font-semibold text-gray-400 w-8 shrink-0">声称</span>
        <div className="flex-1 h-4 bg-gray-100 relative">
          {(() => {
            claimedOffset = 0;
            return ICO.map((ico, i) => {
              const w = (claimedVals[i] / maxTotal) * 100;
              const el = (
                <div
                  key={`cb-${ico.key}`}
                  className="absolute inset-y-0"
                  style={{ left: `${claimedOffset}%`, width: `${w}%`, backgroundColor: ico.color, opacity: 0.3 }}
                />
              );
              claimedOffset += w;
              return el;
            });
          })()}
        </div>
        <span className="text-[10px] font-mono font-semibold text-gray-500 w-11 text-right shrink-0 tabular-nums">
          {formatK(claimedTotal)}
        </span>
      </div>

      {/* Table */}
      <div className="flex items-start gap-2">
        <span className="w-8 shrink-0" />
        {ICO.map((ico, i) => {
          const a = auditVals[i];
          const c = claimedVals[i];
          const diff = c - a;
          const diffRate = c > 0 ? (diff / c) * 100 : 0;
          const diffColor = Math.abs(diffRate) < 3 ? "var(--success)" : Math.abs(diffRate) < 5 ? "var(--warning)" : "var(--danger)";
          return (
            <div key={ico.key} className="flex-1 flex flex-col min-w-0 text-[10px] font-mono">
              <div className="flex justify-between tabular-nums">
                <span className="flex items-center gap-1 text-gray-500">
                  <span className="w-1.5 h-1.5 rounded-sm shrink-0" style={{ backgroundColor: ico.color }} />
                  {ico.label}
                </span>
              </div>
              <div className="flex justify-between tabular-nums">
                <span className="text-gray-400">审计</span>
                <span className="text-gray-600">{formatK(a)}</span>
              </div>
              <div className="flex justify-between tabular-nums">
                <span className="text-gray-400">声称</span>
                <span className="text-gray-500">{formatK(c)}</span>
              </div>
              <div className="flex justify-between tabular-nums mt-0.5">
                <span className="text-gray-400">差异</span>
                <span className="font-semibold" style={{ color: diffColor }}>
                  {diff >= 0 ? "+" : ""}{formatK(diff)} ({diffRate >= 0 ? "+" : ""}{diffRate.toFixed(1)}%)
                </span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
