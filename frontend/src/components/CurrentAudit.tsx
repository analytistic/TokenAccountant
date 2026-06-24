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

  // Per-column scale: each ICO type gets its own max(claimed, audit)
  const perMax = ICO.map((ico) => {
    const a = audit[`${ico.key}_audit` as keyof CurrentAuditType] as number;
    const c = audit[`${ico.key}_claimed` as keyof CurrentAuditType] as number;
    return Math.max(a, c, 1);
  });

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

      {/* Header row: I C O labels */}
      <div className="flex items-center gap-2">
        <span className="w-8 shrink-0" />
        {ICO.map((ico) => (
          <span key={ico.key} className="flex-1 flex items-center gap-1.5 min-w-0">
            <span className="w-2 h-2 rounded-sm shrink-0" style={{ backgroundColor: ico.color }} />
            <span className="text-[10px] font-medium text-gray-500">{ico.label}</span>
          </span>
        ))}
      </div>

      {/* Audit row */}
      <div className="flex items-center gap-2">
        <span className="text-[10px] font-semibold text-gray-500 w-8 shrink-0">审计</span>
        {ICO.map((ico, i) => {
          const val = audit[`${ico.key}_audit` as keyof CurrentAuditType] as number;
          return (
            <div key={ico.key} className="flex-1 flex items-center gap-1.5 min-w-0">
              <div className="flex-1 h-2.5 bg-gray-100 relative">
                <div
                  className="absolute inset-y-0 left-0"
                  style={{ width: `${(val / perMax[i]) * 100}%`, backgroundColor: ico.color, opacity: 0.85 }}
                />
              </div>
              <span className="text-[10px] font-mono text-gray-600 w-11 shrink-0 text-right tabular-nums">
                {formatK(val)}
              </span>
            </div>
          );
        })}
      </div>

      {/* Claimed row */}
      <div className="flex items-center gap-2">
        <span className="text-[10px] font-semibold text-gray-400 w-8 shrink-0">声称</span>
        {ICO.map((ico, i) => {
          const val = audit[`${ico.key}_claimed` as keyof CurrentAuditType] as number;
          return (
            <div key={ico.key} className="flex-1 flex items-center gap-1.5 min-w-0">
              <div className="flex-1 h-2.5 bg-gray-100 relative">
                <div
                  className="absolute inset-y-0 left-0"
                  style={{ width: `${(val / perMax[i]) * 100}%`, backgroundColor: ico.color, opacity: 0.4 }}
                />
              </div>
              <span className="text-[10px] font-mono text-gray-500 w-11 shrink-0 text-right tabular-nums">
                {formatK(val)}
              </span>
            </div>
          );
        })}
      </div>

      {/* Diff row */}
      <div className="flex items-center gap-2">
        <span className="w-8 shrink-0" />
        {ICO.map((ico, i) => {
          const a = audit[`${ico.key}_audit` as keyof CurrentAuditType] as number;
          const c = audit[`${ico.key}_claimed` as keyof CurrentAuditType] as number;
          const diff = c - a;
          const diffRate = c > 0 ? (diff / c) * 100 : 0;
          const diffColor = Math.abs(diffRate) < 3 ? "var(--success)" : Math.abs(diffRate) < 5 ? "var(--warning)" : "var(--danger)";
          return (
            <span key={ico.key} className="flex-1 text-[10px] font-mono font-semibold text-right min-w-0" style={{ color: diffColor }}>
              {diff >= 0 ? "+" : ""}{formatK(diff)} ({diffRate >= 0 ? "+" : ""}{diffRate.toFixed(1)}%)
            </span>
          );
        })}
      </div>
    </div>
  );
}
