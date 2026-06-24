import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ProviderWithStats } from "../types";
import Modal from "./Modal";
import Toggle from "./Toggle";
import Button from "./Button";

interface ProviderFormProps {
  provider: ProviderWithStats | null; // null = create
  open: boolean;
  onClose: () => void;
  onSaved: () => void;
}

export default function ProviderForm({ provider, open, onClose, onSaved }: ProviderFormProps) {
  const isEdit = provider !== null;
  const [name, setName] = useState("");
  const [endpoint, setEndpoint] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [models, setModels] = useState("");
  const [isActive, setIsActive] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (provider) {
      setName(provider.name);
      setEndpoint(provider.api_base_url);
      setApiKey(provider.api_key);
      setModels(provider.supported_models.join(", "));
      setIsActive(provider.is_active);
    } else {
      setName("");
      setEndpoint("");
      setApiKey("");
      setModels("");
      setIsActive(false);
    }
  }, [provider, open]);

  const canSave = endpoint.trim().length > 0;

  const handleSave = async () => {
    if (!canSave) return;
    setSaving(true);
    try {
      const modelList = models
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);

      if (isEdit) {
        await invoke("update_provider", {
          id: provider!.id,
          req: {
            name: name || undefined,
            provider_type: "relay",
            api_base_url: endpoint,
            api_key: apiKey,
            supported_models: modelList,
          },
        });
      } else {
        await invoke("create_provider", {
          req: {
            name,
            provider_type: "relay",
            api_base_url: endpoint,
            api_key: apiKey,
            supported_models: modelList,
          },
        });
        if (isActive) {
          // set as active after creation — need to find and switch
          // switch_provider will be called after the provider list refreshes
        }
      }
      onSaved();
      onClose();
    } catch (e) {
      console.error("Save provider failed:", e);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={isEdit ? "编辑 Provider" : "添加 Provider"}
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>取消</Button>
          <Button onClick={handleSave} disabled={!canSave || saving}>
            {saving ? "保存中..." : "保存"}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        {/* Name */}
        <div className="flex flex-col gap-1">
          <label className="text-xs font-medium text-gray-600">Provider 名称</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="可选"
            className="px-3 py-2 text-sm border border-gray-200 rounded-lg focus:outline-none focus:border-brand"
          />
        </div>

        {/* Endpoint */}
        <div className="flex flex-col gap-1">
          <label className="text-xs font-medium text-gray-600">API Endpoint <span className="text-danger">*</span></label>
          <input
            type="text"
            value={endpoint}
            onChange={(e) => setEndpoint(e.target.value)}
            placeholder="https://api.openai.com"
            className="px-3 py-2 text-sm border border-gray-200 rounded-lg focus:outline-none focus:border-brand"
          />
        </div>

        {/* API Key */}
        <div className="flex flex-col gap-1">
          <label className="text-xs font-medium text-gray-600">API Key</label>
          <div className="relative">
            <input
              type={showKey ? "text" : "password"}
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="sk-..."
              className="w-full px-3 py-2 pr-10 text-sm border border-gray-200 rounded-lg focus:outline-none focus:border-brand"
            />
            <button
              type="button"
              onClick={() => setShowKey(!showKey)}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-xs text-gray-400 hover:text-gray-600"
            >
              {showKey ? "隐藏" : "显示"}
            </button>
          </div>
        </div>

        {/* Models */}
        <div className="flex flex-col gap-1">
          <label className="text-xs font-medium text-gray-600">模型列表（逗号分隔）</label>
          <input
            type="text"
            value={models}
            onChange={(e) => setModels(e.target.value)}
            placeholder="gpt-4o, gpt-4-turbo"
            className="px-3 py-2 text-sm border border-gray-200 rounded-lg focus:outline-none focus:border-brand"
          />
        </div>

        {/* Active toggle (create only) */}
        {!isEdit && (
          <div className="flex items-center justify-between">
            <label className="text-xs font-medium text-gray-600">设为活跃 Provider</label>
            <Toggle checked={isActive} onChange={setIsActive} />
          </div>
        )}
      </div>
    </Modal>
  );
}
