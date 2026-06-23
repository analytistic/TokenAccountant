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

  // Shared scale: max of all audit + claimed values
  const allVals = ICO.flatMap((ico) => [
    audit[`${ico.key}_audit` as keyof CurrentAuditType] as number,
    audit[`${ico.key}_claimed` as keyof CurrentAuditType] as number,
  ]);
  const maxVal = Math.max(...allVals, 1);

  return (
    <div className="flex flex-col gap-2.5">
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

      {/* I/C/O rows: each has audit bar + claimed bar */}
      {ICO.map((ico) => {
        const auditVal = audit[`${ico.key}_audit` as keyof CurrentAuditType] as number;
        const claimedVal = audit[`${ico.key}_claimed` as keyof CurrentAuditType] as number;
        const diff = claimedVal - auditVal;
        const diffRate = claimedVal > 0 ? (diff / claimedVal) * 100 : 0;
        const diffColor = Math.abs(diffRate) < 3 ? "var(--success)" : Math.abs(diffRate) < 5 ? "var(--warning)" : "var(--danger)";
        const auditPct = (auditVal / maxVal) * 100;
        const claimedPct = (claimedVal / maxVal) * 100;

        return (
          <div key={ico.key} className="flex flex-col gap-0.5">
            {/* Label + audit bar */}
            <div className="flex items-center gap-2">
              <span className="flex items-center gap-1.5 w-14 shrink-0">
                <span className="w-2 h-2 rounded-sm shrink-0" style={{ backgroundColor: ico.color }} />
                <span className="text-[10px] font-medium text-gray-600">{ico.label}</span>
              </span>
              <div className="flex-1 h-3 bg-gray-100 rounded-sm relative">
                <div
                  className="absolute inset-y-0 left-0 rounded-sm"
                  style={{ width: `${auditPct}%`, backgroundColor: ico.color, opacity: 0.8 }}
                />
              </div>
              <span className="text-[10px] font-mono text-gray-600 w-11 text-right shrink-0 tabular-nums">
                {formatK(auditVal)}
              </span>
              <span className="text-[10px] text-gray-300 w-7 text-right shrink-0">审计</span>
            </div>

            {/* Claimed bar */}
            <div className="flex items-center gap-2">
              <span className="w-14 shrink-0" />
              <div className="flex-1 h-3 bg-gray-100 rounded-sm relative">
                <div
                  className="absolute inset-y-0 left-0 rounded-sm border border-dashed"
                  style={{
                    width: `${claimedPct}%`,
                    borderColor: ico.color,
                    opacity: 0.6,
                    backgroundColor: "transparent",
                  }}
                />
              </div>
              <span className="text-[10px] font-mono text-gray-500 w-11 text-right shrink-0 tabular-nums">
                {formatK(claimedVal)}
              </span>
              <span className="text-[10px] text-gray-300 w-7 text-right shrink-0">声称</span>
            </div>

            {/* Diff */}
            <div className="flex items-center gap-2">
              <span className="w-14 shrink-0" />
              <span className="text-[10px] font-mono font-semibold" style={{ color: diffColor }}>
                {diff >= 0 ? "+" : ""}{formatK(diff)} ({diffRate >= 0 ? "+" : ""}{diffRate.toFixed(1)}%)
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
