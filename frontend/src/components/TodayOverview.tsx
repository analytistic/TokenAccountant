import type { TodaySummary, ModelBreakdown } from "../types";

interface TodayOverviewProps {
  summary: TodaySummary;
  modelBreakdown: ModelBreakdown[];
}

function formatTokens(value: number) {
  const abs = Math.abs(value);
  if (abs >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (abs >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return String(Math.round(value));
}

export default function TodayOverview({ summary, modelBreakdown }: TodayOverviewProps) {
  const detected = modelBreakdown.reduce(
    (total, model) => total + model.input_total + model.cache_total + model.output_total,
    0,
  );
  const claimed = modelBreakdown.reduce(
    (total, model) =>
      total + model.input_claimed_total + model.cache_claimed_total + model.output_claimed_total,
    0,
  );
  const difference = claimed - detected;
  const differenceRate = claimed > 0 ? (difference / claimed) * 100 : 0;
  const differenceLabel = difference > 0 ? "多报 Token" : difference < 0 ? "少报 Token" : "Token 差异";
  const differenceColor =
    difference === 0 ? "text-success" : Math.abs(differenceRate) < 3 ? "text-warning" : "text-danger";

  const metrics = [
    { label: "今日请求", value: String(summary.total_requests), hint: `${summary.suspicious_requests} 个可疑` },
    { label: "Provider 声称", value: formatTokens(claimed), hint: "Input + Cache + Output" },
    { label: "本地检测", value: formatTokens(detected), hint: "本地 Tokenizer 结果" },
    {
      label: differenceLabel,
      value: `${difference > 0 ? "+" : ""}${formatTokens(difference)}`,
      hint: `${differenceRate > 0 ? "+" : ""}${differenceRate.toFixed(1)}%`,
      color: differenceColor,
    },
  ];

  return (
    <section className="card reconciliation-strip overflow-hidden">
      <div className="grid h-full grid-cols-4 divide-x divide-gray-100">
        {metrics.map((metric) => (
          <div key={metric.label} className="flex min-w-0 flex-col justify-center px-5 py-3">
            <span className="text-[10px] font-semibold uppercase tracking-wide text-gray-400">
              {metric.label}
            </span>
            <span className={`mt-0.5 truncate font-mono text-xl font-bold tabular-nums ${metric.color ?? "text-gray-900"}`}>
              {metric.value}
            </span>
            <span className={`mt-0.5 truncate text-[10px] ${metric.label === "今日请求" && summary.suspicious_requests > 0 ? "text-danger" : "text-gray-400"}`}>
              {metric.hint}
            </span>
          </div>
        ))}
      </div>
    </section>
  );
}
