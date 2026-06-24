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

        {/* I/C/O diff rates */}
        <div className="px-4 py-3 flex flex-col gap-2">
          <div className="flex items-center justify-between">
            <span className="flex items-center gap-1.5 text-[11px] text-gray-500">
              <span className="w-2 h-2 rounded-sm bg-ico-input" /> Input 差异
            </span>
            <span className={`text-[11px] font-mono font-semibold w-20 text-right ${Math.abs(summary.input_diff_rate) < 3 ? "text-success" : Math.abs(summary.input_diff_rate) < 5 ? "text-warning" : "text-danger"}`}>
              {summary.input_diff_rate >= 0 ? "+" : ""}{summary.input_diff_rate.toFixed(1)}%
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="flex items-center gap-1.5 text-[11px] text-gray-500">
              <span className="w-2 h-2 rounded-sm bg-ico-cache" /> Cache 差异
            </span>
            <span className={`text-[11px] font-mono font-semibold w-20 text-right ${Math.abs(summary.cache_diff_rate) < 3 ? "text-success" : Math.abs(summary.cache_diff_rate) < 5 ? "text-warning" : "text-danger"}`}>
              {summary.cache_diff_rate >= 0 ? "+" : ""}{summary.cache_diff_rate.toFixed(1)}%
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="flex items-center gap-1.5 text-[11px] text-gray-500">
              <span className="w-2 h-2 rounded-sm bg-ico-output" /> Output 差异
            </span>
            <span className={`text-[11px] font-mono font-semibold w-20 text-right ${Math.abs(summary.output_diff_rate) < 3 ? "text-success" : Math.abs(summary.output_diff_rate) < 5 ? "text-warning" : "text-danger"}`}>
              {summary.output_diff_rate >= 0 ? "+" : ""}{summary.output_diff_rate.toFixed(1)}%
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
