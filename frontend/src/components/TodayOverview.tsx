import type { TodaySummary, ModelBreakdown } from "../types";
import ModelPieChart from "./ModelPieChart";

interface TodayOverviewProps {
  summary: TodaySummary;
  modelBreakdown: ModelBreakdown[];
}

export default function TodayOverview({ summary, modelBreakdown }: TodayOverviewProps) {
  return (
    <div className="card flex flex-col flex-1 min-h-0">
      {/* Header */}
      <div className="card-header">
        <span className="card-title">今日概览</span>
      </div>

      <div className="flex-1 overflow-y-auto">
        {/* Stats + Pie row */}
        <div className="flex items-start gap-4 px-4 py-3 border-b border-gray-100">
          {/* Stats: 可疑/总数 */}
          <div className="flex flex-col shrink-0">
            <span className="text-lg font-bold text-gray-900 tabular-nums">
              <span className={summary.suspicious_requests > 0 ? "text-danger" : ""}>
                {summary.suspicious_requests}
              </span>
              <span className="text-gray-400 font-normal">/</span>
              {summary.total_requests}
            </span>
            <span className="text-[10px] text-gray-400">可疑 / 请求数</span>
          </div>
          {/* Pie + legend on the right */}
          <div className="flex-1 min-w-0">
            <ModelPieChart data={modelBreakdown} />
          </div>
        </div>

        {/* I/C/O diff: two-column layout */}
        <div className="px-4 py-3">
          {/* Header */}
          <div className="flex items-center text-[10px] text-gray-400 mb-1.5">
            <span className="w-14"></span>
            <span className="flex-1 text-right">差异数</span>
            <span className="w-16 text-right">差异率</span>
          </div>
          {(["input", "cache", "output"] as const).map((key) => {
            const rate = summary[`${key}_diff_rate`] as number;
            const tokens = summary[`${key}_diff_tokens`] as number;
            const bg = key === "input" ? "bg-ico-input" : key === "cache" ? "bg-ico-cache" : "bg-ico-output";
            const label = key === "input" ? "Input" : key === "cache" ? "Cache" : "Output";
            const c = Math.abs(rate) < 3 ? "text-success" : Math.abs(rate) < 5 ? "text-warning" : "text-danger";
            const fmtTk = tokens >= 1000 ? `${(tokens / 1000).toFixed(1)}K` : String(tokens);
            return (
              <div key={key} className="flex items-center py-0.5">
                <span className="flex items-center gap-1.5 w-14">
                  <span className={`w-2 h-2 rounded-sm shrink-0 ${bg}`} />
                  <span className="text-[11px] text-gray-500">{label}</span>
                </span>
                <span className="flex-1 text-[11px] font-mono text-right tabular-nums text-gray-700">
                  {rate >= 0 ? "+" : ""}{fmtTk}
                </span>
                <span className={`w-16 text-[11px] font-mono font-semibold text-right tabular-nums ${c}`}>
                  {rate >= 0 ? "+" : ""}{rate.toFixed(1)}%
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
