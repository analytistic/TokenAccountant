import { invoke } from "@tauri-apps/api/core";
import type { ProviderRank } from "../types";

interface ProviderRankingProps {
  providers: ProviderRank[];
  onSwitch: () => void; // callback to refresh data after switch
}

function medalColor(idx: number) {
  if (idx === 0) return "text-yellow-500";
  if (idx === 1) return "text-gray-300";
  if (idx === 2) return "text-amber-600";
  return "text-gray-300";
}

function medalEmoji(idx: number) {
  if (idx === 0) return "🥇";
  if (idx === 1) return "🥈";
  if (idx === 2) return "🥉";
  return "";
}

function credibilityColor(v: number) {
  if (v >= 90) return "text-success";
  if (v >= 70) return "text-warning";
  return "text-danger";
}

export default function ProviderRanking({ providers, onSwitch }: ProviderRankingProps) {
  const handleSwitch = async (id: string) => {
    try {
      await invoke("switch_provider", { id });
      onSwitch();
    } catch (e) {
      console.error("Switch provider failed:", e);
    }
  };

  if (!providers.length) {
    return (
      <div className="flex items-center justify-center py-6 text-sm text-gray-400">
        暂无 Provider 数据
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto flex flex-col gap-1">
      {providers.map((p, i) => (
        <button
          key={p.id}
          onClick={() => handleSwitch(p.id)}
          className="provider-rank-row flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-gray-50 transition-colors text-left cursor-pointer w-full justify-between overflow-hidden"
        >
          {/* Left: rank + name+url */}
          <div className="flex items-center gap-2 min-w-0">
            <span className={`w-5 text-center text-sm font-bold tabular-nums shrink-0 ${medalColor(i)}`}>
              {medalEmoji(i) || i + 1}
            </span>
            <div className="min-w-0">
              <div className="text-sm font-medium text-gray-800 truncate">{p.name}</div>
              <div className="text-[10px] text-gray-400 truncate">{p.url}</div>
            </div>
          </div>

          {/* Right: credibility + compact discrepancy summary */}
          <div className="provider-rank-metrics flex items-center gap-3 shrink-0">
            <span className={`text-lg font-bold font-mono tabular-nums text-right ${credibilityColor(p.credibility)}`}>
              {Math.round(p.credibility)}%
            </span>
            <div className="provider-rank-diffs flex w-[74px] flex-col overflow-hidden text-right">
              <span className="text-[9px] text-gray-400">最大多报</span>
              <span className="whitespace-nowrap font-mono text-[10px] font-semibold tabular-nums text-gray-600">
                {(() => {
                  const overreport = Math.max(0, p.input_diff_rate, p.cache_diff_rate, p.output_diff_rate);
                  return `+${overreport.toFixed(1)}%`;
                })()}
              </span>
            </div>
          </div>
        </button>
      ))}
    </div>
  );
}
