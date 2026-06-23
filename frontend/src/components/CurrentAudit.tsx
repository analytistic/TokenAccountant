import type { CurrentAudit as CurrentAuditType } from "../types";

interface CurrentAuditProps {
  audit: CurrentAuditType | null;
}

interface AuditBarProps {
  label: string;
  color: string;
  auditVal: number;
  claimedVal: number;
  diff: number;
}

function AuditBar({ label, color, auditVal, claimedVal, diff }: AuditBarProps) {
  const max = Math.max(auditVal, claimedVal, 1);
  const auditPct = (auditVal / max) * 100;
  const claimedPct = (claimedVal / max) * 100;
  const diffSign = diff >= 0 ? "+" : "";

  return (
    <div className="flex items-center gap-2">
      <span className="text-[10px] text-gray-500 w-10 shrink-0">{label}</span>
      <div className="flex-1 relative h-5">
        {/* Audit bar (solid) */}
        <div
          className="absolute top-0.5 h-4 rounded-sm"
          style={{ width: `${auditPct}%`, backgroundColor: color, opacity: 0.8 }}
        />
        {/* Claimed outline (dashed) */}
        <div
          className="absolute top-0.5 h-4 rounded-sm border border-dashed"
          style={{
            width: `${claimedPct}%`,
            borderColor: color,
            opacity: 0.6,
            backgroundColor: "transparent",
          }}
        />
      </div>
      <span className="text-[10px] font-mono text-gray-500 w-14 text-right tabular-nums">
        {diffSign}{diff}
      </span>
    </div>
  );
}

export default function CurrentAudit({ audit }: CurrentAuditProps) {
  if (!audit) {
    return (
      <div className="flex items-center justify-center h-full text-sm text-gray-400">
        等待新请求...
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-3">
      {/* Pass/fail indicator */}
      <div className="flex items-center gap-2">
        <span
          className={`w-2.5 h-2.5 rounded-full ${audit.audit_passed ? "bg-success" : "bg-danger"}`}
        />
        <span className={`text-sm font-semibold ${audit.audit_passed ? "text-success" : "text-danger"}`}>
          {audit.audit_passed ? "审计通过" : "审计未通过"}
        </span>
      </div>

      {/* I/C/O bars */}
      <div className="flex flex-col gap-2">
        <AuditBar
          label="Input"
          color="var(--ico-input)"
          auditVal={audit.input_audit}
          claimedVal={audit.input_claimed}
          diff={audit.input_diff}
        />
        <AuditBar
          label="Cache"
          color="var(--ico-cache)"
          auditVal={audit.cache_audit}
          claimedVal={audit.cache_claimed}
          diff={audit.cache_diff}
        />
        <AuditBar
          label="Output"
          color="var(--ico-output)"
          auditVal={audit.output_audit}
          claimedVal={audit.output_claimed}
          diff={audit.output_diff}
        />
      </div>
    </div>
  );
}
