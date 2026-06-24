import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ProviderWithStats } from "../types";
import TrustRing from "./TrustRing";

interface ProviderCardProps {
  provider: ProviderWithStats;
  selectedModel: string | null;
  onSelectModel: (model: string | null) => void;
  onEdit: (provider: ProviderWithStats) => void;
  onDelete: (provider: ProviderWithStats) => void;
  onRefresh: () => void;
}

function formatK(v: number) {
  return v >= 1000 ? `${(v / 1000).toFixed(1)}K` : String(v);
}

export default function ProviderCard({
  provider,
  selectedModel,
  onSelectModel,
  onEdit,
  onDelete,
  onRefresh,
}: ProviderCardProps) {
  const [menuOpen, setMenuOpen] = useState(false);

  const handleSwitch = async () => {
    try {
      await invoke("switch_provider", { id: provider.id });
      onRefresh();
    } catch (e) {
      console.error("Switch failed:", e);
    }
  };

  const credibility = provider.credibility ?? 100;
  const summary = provider.audit_summary;

  // Prefill = Input + Cache
  const prefillAudit = (summary?.input_detected ?? 0) + (summary?.cache_detected ?? 0);
  const prefillClaimed = (summary?.input_claimed ?? 0) + (summary?.cache_claimed ?? 0);
  const prefillMax = Math.max(prefillAudit, prefillClaimed, 1);
  const outputAudit = summary?.output_detected ?? 0;
  const outputClaimed = summary?.output_claimed ?? 0;
  const outputMax = Math.max(outputAudit, outputClaimed, 1);

  return (
    <div
      className={`card flex flex-col cursor-pointer transition-all duration-fast
        ${provider.is_active ? "ring-2 ring-brand ring-offset-1" : "hover:shadow-md"}`}
      onClick={handleSwitch}
    >
      {/* Header */}
      <div className="flex items-start justify-between px-4 pt-4 pb-2">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-sm font-bold text-gray-900 truncate">{provider.name}</span>
            {provider.is_active && (
              <span className="text-[9px] font-semibold bg-brand-subtle text-brand px-1.5 py-0.5 rounded">
                活跃
              </span>
            )}
          </div>
          <div className="text-[10px] text-gray-400 truncate mt-0.5">{provider.api_base_url}</div>
        </div>

        <div className="flex items-center gap-2 shrink-0">
          <TrustRing value={credibility} size="sm" />
          {/* Three-dot menu */}
          <div className="relative" onClick={(e) => e.stopPropagation()}>
            <button
              onClick={() => setMenuOpen(!menuOpen)}
              className="w-6 h-6 flex items-center justify-center rounded hover:bg-gray-100 text-gray-400"
            >
              ⋮
            </button>
            {menuOpen && (
              <div className="absolute right-0 top-7 bg-white border border-gray-200 rounded-lg shadow-lg py-1 z-10 min-w-[100px]">
                <button
                  className="w-full text-left px-3 py-1.5 text-xs text-gray-600 hover:bg-gray-50"
                  onClick={() => { setMenuOpen(false); onEdit(provider); }}
                >
                  编辑
                </button>
                <button
                  className="w-full text-left px-3 py-1.5 text-xs text-danger hover:bg-danger-subtle"
                  onClick={() => { setMenuOpen(false); onDelete(provider); }}
                >
                  删除 Provider
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Model tags */}
      {provider.supported_models.length > 0 && (
        <div className="flex flex-wrap gap-1 px-4 pb-3" onClick={(e) => e.stopPropagation()}>
          {provider.supported_models.map((m) => (
            <button
              key={m}
              onClick={() => onSelectModel(selectedModel === m ? null : m)}
              className={`text-[10px] px-2 py-0.5 rounded-full transition-colors
                ${selectedModel === m
                  ? "bg-brand text-white"
                  : "bg-gray-100 text-gray-500 hover:bg-gray-200"}`}
            >
              {m}
            </button>
          ))}
        </div>
      )}

      {/* Audit bars — Prefill + Output sections per prototype */}
      {summary ? (
        <div className="px-4 pb-4 space-y-3" onClick={(e) => e.stopPropagation()}>
          {/* Prefill section */}
          <div>
            <div className="text-[10px] font-bold uppercase tracking-wider mb-1.5" style={{ color: "var(--ico-input)" }}>
              Prefill
            </div>
            {/* 声称 */}
            <div className="flex items-center gap-2 mb-1">
              <span className="text-[10px] text-gray-400 w-7 text-right shrink-0">声称</span>
              <div className="flex-1 h-1 bg-gray-100 rounded-sm relative overflow-hidden">
                <div className="absolute top-0 h-full rounded-sm" style={{
                  left: 0, width: `${((summary.input_claimed ?? 0) / prefillMax) * 100}%`,
                  backgroundColor: "var(--ico-input)", opacity: 0.45,
                }} />
                <div className="absolute top-0 h-full rounded-sm" style={{
                  right: 0, width: `${((summary.cache_claimed ?? 0) / prefillMax) * 100}%`,
                  backgroundColor: "var(--ico-cache)", opacity: 0.45,
                }} />
              </div>
              <span className="text-[10px] font-mono text-gray-500 w-11 text-right shrink-0">{formatK(prefillClaimed)}</span>
            </div>
            {/* 审计 */}
            <div className="flex items-center gap-2">
              <span className="text-[10px] text-gray-500 w-7 text-right shrink-0">审计</span>
              <div className="flex-1 h-1 bg-gray-100 rounded-sm relative overflow-hidden">
                <div className="absolute top-0 h-full rounded-sm" style={{
                  left: 0, width: `${((summary.input_detected ?? 0) / prefillMax) * 100}%`,
                  backgroundColor: "var(--ico-input)",
                }} />
                <div className="absolute top-0 h-full rounded-sm" style={{
                  right: 0, width: `${((summary.cache_detected ?? 0) / prefillMax) * 100}%`,
                  backgroundColor: "var(--ico-cache)",
                }} />
              </div>
              <span className="text-[10px] font-mono text-gray-600 w-11 text-right shrink-0">{formatK(prefillAudit)}</span>
            </div>
          </div>

          {/* Output section */}
          <div>
            <div className="text-[10px] font-bold uppercase tracking-wider mb-1.5" style={{ color: "var(--ico-output)" }}>
              Output
            </div>
            <div className="flex items-center gap-2 mb-1">
              <span className="text-[10px] text-gray-400 w-7 text-right shrink-0">声称</span>
              <div className="flex-1 h-1 bg-gray-100 rounded-sm relative overflow-hidden">
                <div className="absolute top-0 h-full rounded-sm" style={{
                  left: 0, width: `${(outputClaimed / outputMax) * 100}%`,
                  backgroundColor: "var(--ico-output)", opacity: 0.45,
                }} />
              </div>
              <span className="text-[10px] font-mono text-gray-500 w-11 text-right shrink-0">{formatK(outputClaimed)}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-[10px] text-gray-500 w-7 text-right shrink-0">审计</span>
              <div className="flex-1 h-1 bg-gray-100 rounded-sm relative overflow-hidden">
                <div className="absolute top-0 h-full rounded-sm" style={{
                  left: 0, width: `${(outputAudit / outputMax) * 100}%`,
                  backgroundColor: "var(--ico-output)",
                }} />
              </div>
              <span className="text-[10px] font-mono text-gray-600 w-11 text-right shrink-0">{formatK(outputAudit)}</span>
            </div>
          </div>
        </div>
      ) : (
        <div className="px-4 pb-4 text-[10px] text-gray-400">暂无审计数据</div>
      )}
    </div>
  );
}
