import type { TodaySummary, ModelBreakdown } from "../types";
import ModelPieChart from "./ModelPieChart";

interface TodayOverviewProps {
  summary: TodaySummary;
  modelBreakdown: ModelBreakdown[];
}

function diffDisplay(rate: number, tokens: number) {
  const color =
    Math.abs(rate) < 3 ? "text-success" : Math.abs(rate) < 5 ? "text-warning" : "text-danger";
  const sign = rate >= 0 ? "+" : "";
  const symbol = rate >= 0 ? "↑" : "↓";
  return (
    <span className={`text-xs font-mono font-semibold ${color}`}>
      {sign}{rate.toFixed(1)}% {symbol}{Math.abs(tokens)}
    </span>
  );
}

export default function TodayOverview({ summary, modelBreakdown }: TodayOverviewProps) {
  return (
    <div className="card flex flex-col shrink-0">
      {/* Stats row */}
      <div className="flex items-center gap-6 px-4 py-3 border-b border-gray-100">
        <div className="flex flex-col">
          <span className="text-2xl font-bold text-gray-900 tabular-nums">
            {summary.total_requests}
          </span>
          <span className="text-[11px] text-gray-400">请求数</span>
        </div>
        <div className="flex flex-col">
          <span className={`text-2xl font-bold tabular-nums ${summary.suspicious_requests > 0 ? "text-danger" : "text-gray-900"}`}>
            {summary.suspicious_requests}
          </span>
          <span className="text-[11px] text-gray-400">可疑请求</span>
        </div>
      </div>

      {/* Body: left = I/C/O rates, right = pie chart */}
      <div className="flex items-center gap-4 px-4 py-3">
        {/* Left: diff rates */}
        <div className="flex flex-col gap-2 flex-1 min-w-0">
          <div className="flex items-center justify-between">
            <span className="flex items-center gap-1.5 text-xs text-gray-500">
              <span className="w-2 h-2 rounded-sm bg-ico-input" /> Input
            </span>
            {diffDisplay(summary.input_diff_rate, summary.input_diff_tokens)}
          </div>
          <div className="flex items-center justify-between">
            <span className="flex items-center gap-1.5 text-xs text-gray-500">
              <span className="w-2 h-2 rounded-sm bg-ico-cache" /> Cache
            </span>
            {diffDisplay(summary.cache_diff_rate, summary.cache_diff_tokens)}
          </div>
          <div className="flex items-center justify-between">
            <span className="flex items-center gap-1.5 text-xs text-gray-500">
              <span className="w-2 h-2 rounded-sm bg-ico-output" /> Output
            </span>
            {diffDisplay(summary.output_diff_rate, summary.output_diff_tokens)}
          </div>
        </div>

        {/* Right: pie chart */}
        <div className="shrink-0">
          <ModelPieChart data={modelBreakdown} />
        </div>
      </div>
    </div>
  );
}
