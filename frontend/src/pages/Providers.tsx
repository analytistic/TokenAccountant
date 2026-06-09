import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Provider {
  id: string;
  name: string;
  provider_type: string;
  api_base_url: string;
  api_key: string;
  supported_models: string[];
  is_active: boolean;
}

export default function Providers() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ name: "", api_base_url: "", api_key: "", provider_type: "relay", supported_models: "" });

  const load = () => {
    invoke<Provider[]>("list_providers").then(setProviders).catch(console.error);
  };

  useEffect(load, []);

  const addProvider = async () => {
    await invoke("create_provider", {
      req: {
        name: form.name,
        provider_type: form.provider_type,
        api_base_url: form.api_base_url,
        api_key: form.api_key,
        supported_models: form.supported_models.split(",").map(s => s.trim()).filter(Boolean),
      },
    });
    setShowAdd(false);
    setForm({ name: "", api_base_url: "", api_key: "", provider_type: "relay", supported_models: "" });
    load();
  };

  const switchProvider = async (id: string) => {
    await invoke("switch_provider", { id });
    load();
  };

  const deleteProvider = async (id: string) => {
    if (!confirm("确定删除？")) return;
    await invoke("delete_provider", { id });
    load();
  };

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold">Provider 管理</h2>
        <button onClick={() => setShowAdd(true)} className="bg-green-600 hover:bg-green-700 px-4 py-2 rounded">添加</button>
      </div>

      {showAdd && (
        <div className="bg-gray-800 rounded-lg p-4 mb-6 space-y-3">
          <input placeholder="名称" value={form.name} onChange={e => setForm({ ...form, name: e.target.value })}
            className="w-full bg-gray-700 rounded px-3 py-2" />
          <input placeholder="API Base URL" value={form.api_base_url} onChange={e => setForm({ ...form, api_base_url: e.target.value })}
            className="w-full bg-gray-700 rounded px-3 py-2" />
          <input placeholder="API Key" value={form.api_key} onChange={e => setForm({ ...form, api_key: e.target.value })}
            className="w-full bg-gray-700 rounded px-3 py-2" type="password" />
          <input placeholder="模型列表 (逗号分隔)" value={form.supported_models} onChange={e => setForm({ ...form, supported_models: e.target.value })}
            className="w-full bg-gray-700 rounded px-3 py-2" />
          <div className="flex gap-2">
            <button onClick={addProvider} className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded">保存</button>
            <button onClick={() => setShowAdd(false)} className="bg-gray-600 hover:bg-gray-700 px-4 py-2 rounded">取消</button>
          </div>
        </div>
      )}

      <div className="space-y-3">
        {providers.map(p => (
          <div key={p.id} className={`bg-gray-800 rounded-lg p-4 border ${p.is_active ? 'border-blue-500' : 'border-gray-700'}`}>
            <div className="flex justify-between items-start">
              <div>
                <div className="font-bold text-lg">{p.name} {p.is_active && <span className="text-blue-400 text-sm">(当前)</span>}</div>
                <div className="text-gray-400 text-sm">{p.api_base_url}</div>
                <div className="text-gray-500 text-xs mt-1">模型: {p.supported_models.join(", ") || "-"}</div>
              </div>
              <div className="flex gap-2">
                {!p.is_active && <button onClick={() => switchProvider(p.id)} className="bg-blue-600 hover:bg-blue-700 px-3 py-1 rounded text-sm">切换</button>}
                <button onClick={() => deleteProvider(p.id)} className="bg-red-600 hover:bg-red-700 px-3 py-1 rounded text-sm">删除</button>
              </div>
            </div>
          </div>
        ))}
        {providers.length === 0 && <div className="text-gray-500 text-center py-8">暂无 Provider，点击"添加"开始配置</div>}
      </div>
    </div>
  );
}
