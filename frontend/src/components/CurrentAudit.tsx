import type { CurrentAudit as CurrentAuditType } from "../types";

interface CurrentAuditProps {
  audit: CurrentAuditType | null;
}

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

  const prefillAudit = audit.input_audit + audit.cache_audit;
  const prefillClaimed = audit.input_claimed + audit.cache_claimed;
  const prefillMax = Math.max(prefillAudit, prefillClaimed, 1);
  const outputMax = Math.max(audit.output_audit, audit.output_claimed, 1);

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

      {/* ===== Prefill section (Input + Cache) ===== */}
      <div className="flex flex-col gap-1.5">
        <span className="text-[10px] font-bold uppercase tracking-wider" style={{ color: "var(--ico-input)" }}>
          Prefill
        </span>

        {/* 声称 row */}
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-semibold text-gray-400 w-7 text-right shrink-0">声称</span>
          <div className="flex-1 h-1 bg-gray-100 rounded-sm relative overflow-hidden">
            <div
              className="absolute top-0 h-full rounded-sm"
              style={{ left: 0, width: `${(audit.input_claimed / prefillMax) * 100}%`, backgroundColor: "var(--ico-input)", opacity: 0.45 }}
            />
            <div
              className="absolute top-0 h-full rounded-sm"
              style={{ right: 0, width: `${(audit.cache_claimed / prefillMax) * 100}%`, backgroundColor: "var(--ico-cache)", opacity: 0.45 }}
            />
          </div>
          <span className="text-[10px] font-semibold text-gray-500 w-11 text-right shrink-0 font-mono tabular-nums">
            {formatK(prefillClaimed)}
          </span>
        </div>

        {/* 审计 row */}
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-semibold text-gray-500 w-7 text-right shrink-0">审计</span>
          <div className="flex-1 h-1 bg-gray-100 rounded-sm relative overflow-hidden">
            <div
              className="absolute top-0 h-full rounded-sm"
              style={{ left: 0, width: `${(audit.input_audit / prefillMax) * 100}%`, backgroundColor: "var(--ico-input)" }}
            />
            <div
              className="absolute top-0 h-full rounded-sm"
              style={{ right: 0, width: `${(audit.cache_audit / prefillMax) * 100}%`, backgroundColor: "var(--ico-cache)" }}
            />
          </div>
          <span className="text-[10px] font-semibold text-gray-600 w-11 text-right shrink-0 font-mono tabular-nums">
            {formatK(prefillAudit)}
          </span>
        </div>

        {/* Diff row */}
        <div className="flex items-center gap-4">
          {(["input", "cache"] as const).map((k) => {
            const a = audit[`${k}_audit`] as number;
            const c = audit[`${k}_claimed`] as number;
            const diff = c - a;
            const rate = c > 0 ? (diff / c) * 100 : 0;
            const color = Math.abs(rate) < 3 ? "var(--success)" : Math.abs(rate) < 5 ? "var(--warning)" : "var(--danger)";
            const dot = k === "input" ? "var(--ico-input)" : "var(--ico-cache)";
            return (
              <span key={k} className="flex items-center gap-1 text-[10px]">
                <span className="w-1.5 h-1.5 rounded-sm shrink-0" style={{ backgroundColor: dot }} />
                <span className="text-gray-500">{k === "input" ? "Input" : "Cache"}</span>
                <span className="font-mono font-semibold" style={{ color }}>
                  {diff >= 0 ? "+" : ""}{formatK(diff)}
                </span>
              </span>
            );
          })}
        </div>
      </div>

      {/* ===== Output section ===== */}
      <div className="flex flex-col gap-1.5">
        <span className="text-[10px] font-bold uppercase tracking-wider" style={{ color: "var(--ico-output)" }}>
          Output
        </span>

        {/* 声称 row */}
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-semibold text-gray-400 w-7 text-right shrink-0">声称</span>
          <div className="flex-1 h-1 bg-gray-100 rounded-sm relative overflow-hidden">
            <div
              className="absolute top-0 h-full rounded-sm"
              style={{ left: 0, width: `${(audit.output_claimed / outputMax) * 100}%`, backgroundColor: "var(--ico-output)", opacity: 0.45 }}
            />
          </div>
          <span className="text-[10px] font-semibold text-gray-500 w-11 text-right shrink-0 font-mono tabular-nums">
            {formatK(audit.output_claimed)}
          </span>
        </div>

        {/* 审计 row */}
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-semibold text-gray-500 w-7 text-right shrink-0">审计</span>
          <div className="flex-1 h-1 bg-gray-100 rounded-sm relative overflow-hidden">
            <div
              className="absolute top-0 h-full rounded-sm"
              style={{ left: 0, width: `${(audit.output_audit / outputMax) * 100}%`, backgroundColor: "var(--ico-output)" }}
            />
          </div>
          <span className="text-[10px] font-semibold text-gray-600 w-11 text-right shrink-0 font-mono tabular-nums">
            {formatK(audit.output_audit)}
          </span>
        </div>

        {/* Diff row */}
        <div className="flex items-center gap-1 text-[10px]">
          <span className="w-1.5 h-1.5 rounded-sm shrink-0" style={{ backgroundColor: "var(--ico-output)" }} />
          <span className="text-gray-500">Output</span>
          <span className="font-mono font-semibold ml-1" style={{ color: (() => {
            const c = audit.output_claimed;
            const a = audit.output_audit;
            const diff = c - a;
            const rate = c > 0 ? (diff / c) * 100 : 0;
            return Math.abs(rate) < 3 ? "var(--success)" : Math.abs(rate) < 5 ? "var(--warning)" : "var(--danger)";
          })() }}>
            {(() => {
              const c = audit.output_claimed;
              const a = audit.output_audit;
              const diff = c - a;
              return `${diff >= 0 ? "+" : ""}${formatK(diff)}`;
            })()}
          </span>
        </div>
      </div>
    </div>
  );
}
