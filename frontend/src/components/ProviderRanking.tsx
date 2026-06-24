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
          className="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-gray-50 transition-colors text-left cursor-pointer w-full"
        >
          {/* Rank */}
          <span className={`w-5 text-center text-sm font-bold tabular-nums shrink-0 ${medalColor(i)}`}>
            {medalEmoji(i) || i + 1}
          </span>

          {/* Name + URL */}
          <div className="flex-1 min-w-0">
            <div className="text-sm font-medium text-gray-800 truncate">{p.name}</div>
            <div className="text-[10px] text-gray-400 truncate">{p.url}</div>
          </div>

          {/* Credibility + I/C/O diff — pushed to right */}
          <div className="flex items-center gap-3 shrink-0 ml-auto">
            <span className={`text-lg font-bold font-mono tabular-nums w-10 text-left ${credibilityColor(p.credibility)}`}>
              {Math.round(p.credibility)}%
            </span>
            <div className="flex flex-col gap-0.5 w-16">
              <span className="text-[9px] text-gray-500 font-mono tabular-nums">
                I {p.input_diff_rate >= 0 ? "+" : ""}{p.input_diff_rate.toFixed(1)}%
              </span>
              <span className="text-[9px] text-gray-500 font-mono tabular-nums">
                C {p.cache_diff_rate >= 0 ? "+" : ""}{p.cache_diff_rate.toFixed(1)}%
              </span>
              <span className="text-[9px] text-gray-500 font-mono tabular-nums">
                O {p.output_diff_rate >= 0 ? "+" : ""}{p.output_diff_rate.toFixed(1)}%
              </span>
            </div>
          </div>
        </button>
      ))}
    </div>
  );
}
