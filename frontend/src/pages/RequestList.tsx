import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface AuditRecord {
  id: number;
  timestamp: string;
  model: string;
  api_format: string;
  claimed_input_tokens: number;
  claimed_output_tokens: number;
  claimed_cached_tokens: number;
  real_input_tokens: number;
  real_output_tokens: number;
  detected_cached_tokens: number;
  input_diff: number;
  output_diff: number;
  cache_diff: number;
  is_suspicious: boolean;
  suspicion_reason: string;
}

export default function RequestList() {
  const [records, setRecords] = useState<AuditRecord[]>([]);

  const load = () => {
    invoke<AuditRecord[]>("list_audit_logs", { limit: 100, offset: 0 })
      .then(setRecords)
      .catch(console.error);
  };

  useEffect(load, []);

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold">请求列表</h2>
        <button onClick={load} className="bg-gray-700 hover:bg-gray-600 px-3 py-1 rounded text-sm">
          刷新
        </button>
      </div>
      <div className="space-y-3">
        {records.map(r => (
          <div key={r.id}
            className={`bg-gray-800 rounded-lg p-4 border ${
              r.is_suspicious ? 'border-red-500' : 'border-gray-700'
            }`}
          >
            <div className="flex justify-between items-start">
              <div>
                <span className="font-bold">{r.model}</span>
                <span className="text-gray-500 text-sm ml-2">({r.api_format})</span>
              </div>
              <span className={r.is_suspicious ? 'text-red-400' : 'text-green-400'}>
                {r.is_suspicious ? '可疑' : '正常'}
              </span>
            </div>
            <div className="grid grid-cols-3 gap-4 mt-2 text-sm">
              <div>
                <span className="text-gray-500">Input</span>
                <div>{r.claimed_input_tokens} (声称) vs {r.real_input_tokens} (实际)</div>
                {r.input_diff !== 0 && (
                  <div className={r.input_diff > 0 ? "text-yellow-400" : "text-blue-400"}>
                    差异: {r.input_diff > 0 ? "+" : ""}{r.input_diff}
                  </div>
                )}
              </div>
              <div>
                <span className="text-gray-500">Output</span>
                <div>{r.claimed_output_tokens} vs {r.real_output_tokens}</div>
                {r.output_diff !== 0 && (
                  <div className={r.output_diff > 0 ? "text-yellow-400" : "text-blue-400"}>
                    差异: {r.output_diff > 0 ? "+" : ""}{r.output_diff}
                  </div>
                )}
              </div>
              <div>
                <span className="text-gray-500">Cache</span>
                <div>{r.claimed_cached_tokens} vs {r.detected_cached_tokens}</div>
                {r.cache_diff !== 0 && <div className="text-yellow-400">差异: {r.cache_diff}</div>}
              </div>
            </div>
            {r.is_suspicious && (
              <div className="text-sm text-red-500 mt-2 bg-red-900/20 p-2 rounded">
                {r.suspicion_reason}
              </div>
            )}
            <div className="text-xs text-gray-600 mt-2">{r.timestamp}</div>
          </div>
        ))}
        {records.length === 0 && (
          <div className="text-gray-500 text-center py-12">暂无审计记录</div>
        )}
      </div>
    </div>
  );
}
