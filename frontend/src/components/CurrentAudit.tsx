import type { CurrentAudit as CurrentAuditType } from "../types";

interface CurrentAuditProps { audit: CurrentAuditType | null; }

function formatTokens(value: number) {
  const abs = Math.abs(value);
  return abs >= 1000 ? `${(value / 1000).toFixed(1)}K` : String(value);
}

export default function CurrentAudit({ audit }: CurrentAuditProps) {
  if (!audit) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-center">
        <span className="text-sm font-medium text-gray-400">等待第一笔审计</span>
        <span className="mt-1 text-[10px] text-gray-400">启动代理并发送请求后，这里会显示最新对账结果</span>
      </div>
    );
  }

  const rows = [
    { label: "Input", color: "var(--ico-input)", claimed: audit.input_claimed, detected: audit.input_audit },
    { label: "Cache", color: "var(--ico-cache)", claimed: audit.cache_claimed, detected: audit.cache_audit },
    { label: "Output", color: "var(--ico-output)", claimed: audit.output_claimed, detected: audit.output_audit },
  ];
  const hasOverreport = rows.some((row) => row.claimed > row.detected);
  const hasAnyDiff = rows.some((row) => row.claimed !== row.detected);
  const conclusion = hasOverreport
    ? "计费异常"
    : hasAnyDiff
      ? "存在差异"
      : "审计一致";
  const fingerprint = {
    consistent: { label: "响应指纹一致", className: "text-success", dot: "bg-success" },
    nonstandard: { label: "响应指纹非标准", className: "text-warning", dot: "bg-warning" },
    suspicious: { label: "响应指纹可疑", className: "text-danger", dot: "bg-danger" },
    unknown: { label: "响应指纹未识别", className: "text-gray-500", dot: "bg-gray-400" },
    not_applicable: { label: "响应指纹不适用", className: "text-gray-400", dot: "bg-gray-300" },
  }[audit.fingerprint_status] ?? { label: "响应指纹未识别", className: "text-gray-500", dot: "bg-gray-400" };
  const headerFingerprint = {
    consistent: { label: "HTTP 指纹一致", className: "text-success", dot: "bg-success" },
    nonstandard: { label: "HTTP 指纹非标准", className: "text-warning", dot: "bg-warning" },
    suspicious: { label: "HTTP 指纹可疑", className: "text-danger", dot: "bg-danger" },
    unknown: { label: "HTTP 指纹未识别", className: "text-gray-500", dot: "bg-gray-400" },
    not_applicable: { label: "HTTP 指纹不适用", className: "text-gray-400", dot: "bg-gray-300" },
  }[audit.header_fingerprint_status] ?? { label: "HTTP 指纹未识别", className: "text-gray-500", dot: "bg-gray-400" };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className={`mb-3 flex shrink-0 items-center gap-2 rounded-md px-3 py-2 ${hasOverreport ? "bg-danger-subtle text-danger" : "bg-success-subtle text-success"}`}>
        <span className="flex h-5 w-5 items-center justify-center rounded-full bg-white/70 text-xs font-bold">{hasOverreport ? "!" : "✓"}</span>
        <div className="min-w-0">
          <div className="truncate text-xs font-semibold">{conclusion}</div>
        </div>
      </div>

      <div className="mb-2 flex min-w-0 shrink-0 items-center justify-between gap-2 px-1 text-[9px]">
        <span className={`flex items-center gap-1.5 font-medium ${fingerprint.className}`}>
          <i className={`h-1.5 w-1.5 rounded-full ${fingerprint.dot}`} />
          {fingerprint.label}
        </span>
        <span
          className="min-w-0 truncate font-mono text-gray-400"
          title={[audit.response_id, ...audit.fingerprint_issues].filter(Boolean).join("\n")}
        >
          {audit.response_id ?? audit.fingerprint_issues[0] ?? "无响应 ID"}
        </span>
      </div>

      <div className="mb-2 flex min-w-0 shrink-0 items-center justify-between gap-2 px-1 text-[9px]">
        <span className={`flex items-center gap-1.5 font-medium ${headerFingerprint.className}`}>
          <i className={`h-1.5 w-1.5 rounded-full ${headerFingerprint.dot}`} />
          {headerFingerprint.label}
        </span>
        <span
          className="min-w-0 truncate font-mono text-gray-400"
          title={[audit.upstream_request_id, ...audit.header_fingerprint_issues].filter(Boolean).join("\n")}
        >
          {audit.upstream_request_id ?? audit.header_fingerprint_issues[0] ?? "无追踪 ID"}
        </span>
      </div>

      <div className="min-h-0 flex-1 overflow-hidden rounded-md border border-gray-100">
        <div className="grid grid-cols-[1fr_1fr_1fr_1.1fr] bg-gray-50 px-3 py-1.5 text-[9px] font-medium text-gray-400">
          <span>类型</span><span className="text-right">声称</span><span className="text-right">检测</span><span className="text-right">差异</span>
        </div>
        {rows.map((row) => {
          const diff = row.claimed - row.detected;
          return (
            <div key={row.label} className="grid grid-cols-[1fr_1fr_1fr_1.1fr] items-center border-t border-gray-100 px-3 py-2 text-[10px]">
              <span className="flex items-center gap-1.5 font-medium text-gray-600"><i className="h-2 w-2 rounded-sm" style={{ backgroundColor: row.color }} />{row.label}</span>
              <span className="text-right font-mono text-gray-500">{formatTokens(row.claimed)}</span>
              <span className="text-right font-mono text-gray-700">{formatTokens(row.detected)}</span>
              <span className={`text-right font-mono font-semibold ${diff > 0 ? "text-danger" : diff === 0 ? "text-success" : "text-gray-500"}`}>
                {diff > 0 ? "+" : ""}{formatTokens(diff)}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
