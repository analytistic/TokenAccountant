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

  const auditTotal = audit.input_audit + audit.cache_audit + audit.output_audit;
  const claimedTotal = audit.input_claimed + audit.cache_claimed + audit.output_claimed;
  const maxTotal = Math.max(auditTotal, claimedTotal, 1);

  // Build segmented bar: array of { pct, color }
  const auditSegs = ICO.map((ico) => ({
    pct: ((audit[`${ico.key}_audit` as keyof CurrentAuditType] as number) / maxTotal) * 100,
    color: ico.color,
  }));
  const claimedSegs = ICO.map((ico) => ({
    pct: ((audit[`${ico.key}_claimed` as keyof CurrentAuditType] as number) / maxTotal) * 100,
    color: ico.color,
  }));

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

      {/* Segmented bars: audit row + claimed row */}
      <div className="flex flex-col gap-2">
        {/* Audit row */}
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-semibold text-gray-500 w-7 text-right shrink-0">审计</span>
          <div className="flex-1 h-1.5 bg-gray-100 rounded-full relative flex overflow-hidden">
            {auditSegs.map((seg, i) => (
              <div
                key={i}
                className="h-full shrink-0"
                style={{ width: `${seg.pct}%`, backgroundColor: seg.color }}
              />
            ))}
          </div>
          <span className="text-[10px] font-semibold text-gray-600 w-11 text-right shrink-0 font-mono">
            {formatK(auditTotal)}
          </span>
        </div>

        {/* Claimed row */}
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-semibold text-gray-400 w-7 text-right shrink-0">声称</span>
          <div className="flex-1 h-1.5 bg-gray-100 rounded-full relative flex overflow-hidden">
            {claimedSegs.map((seg, i) => (
              <div
                key={i}
                className="h-full shrink-0 opacity-45"
                style={{ width: `${seg.pct}%`, backgroundColor: seg.color }}
              />
            ))}
          </div>
          <span className="text-[10px] font-semibold text-gray-500 w-11 text-right shrink-0 font-mono">
            {formatK(claimedTotal)}
          </span>
        </div>
      </div>

      {/* Separator + diff rows */}
      <div className="border-t border-gray-100 pt-2 flex flex-col gap-1.5">
        {ICO.map((ico) => {
          const claimed = audit[`${ico.key}_claimed` as keyof CurrentAuditType] as number;
          const real = audit[`${ico.key}_audit` as keyof CurrentAuditType] as number;
          const diff = (audit[`${ico.key}_diff` as keyof CurrentAuditType] as number);
          const diffRate = claimed > 0 ? ((claimed - real) / claimed) * 100 : 0;
          const color = Math.abs(diffRate) < 3 ? "var(--success)" : Math.abs(diffRate) < 5 ? "var(--warning)" : "var(--danger)";
          return (
            <div key={ico.key} className="flex items-center justify-between">
              <span className="flex items-center gap-1.5 text-[10px] text-gray-500">
                <span className="w-2 h-2 rounded-sm shrink-0" style={{ backgroundColor: ico.color }} />
                {ico.label} 差异
              </span>
              <span className="text-[10px] font-mono font-semibold" style={{ color }}>
                {diff >= 0 ? "+" : ""}{formatK(diff)} ({diffRate >= 0 ? "+" : ""}{diffRate.toFixed(1)}%)
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
