import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ProviderWithStats, AuditSummary } from "../types";
import ProviderCard from "../components/ProviderCard";
import ProviderForm from "../components/ProviderForm";
import Button from "../components/Button";
import Modal from "../components/Modal";

export default function ProvidersPage() {
  const [providers, setProviders] = useState<ProviderWithStats[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedModel, setSelectedModel] = useState<string | null>(null);
  const [modelDetails, setModelDetails] = useState<Record<string, AuditSummary>>({});
  const [formOpen, setFormOpen] = useState(false);
  const [editProvider, setEditProvider] = useState<ProviderWithStats | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<ProviderWithStats | null>(null);

  const loadProviders = useCallback(async () => {
    try {
      const list = await invoke<ProviderWithStats[]>("list_providers");
      setProviders(list);
    } catch (e) {
      console.error("Failed to load providers:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadProviders();
  }, [loadProviders]);

  // When model is selected, fetch detail per provider
  useEffect(() => {
    if (!selectedModel) {
      setModelDetails({});
      return;
    }
    (async () => {
      const details: Record<string, AuditSummary> = {};
      for (const p of providers) {
        try {
          const d = await invoke<AuditSummary>("get_provider_detail", {
            providerId: p.id,
            model: selectedModel,
          });
          details[p.id] = d;
        } catch (_) {
          // provider doesn't have this model
        }
      }
      setModelDetails(details);
    })();
  }, [selectedModel, providers]);

  const handleDelete = async () => {
    if (!deleteTarget) return;
    try {
      await invoke("delete_provider", { id: deleteTarget.id });
      setDeleteTarget(null);
      loadProviders();
    } catch (e) {
      console.error("Delete failed:", e);
    }
  };

  const handleEdit = (p: ProviderWithStats) => {
    setEditProvider(p);
    setFormOpen(true);
  };

  const handleAdd = () => {
    setEditProvider(null);
    setFormOpen(true);
  };

  // Build provider data for rendering: use model-specific detail if selected
  const getDisplayProvider = (p: ProviderWithStats): ProviderWithStats => {
    if (selectedModel && modelDetails[p.id]) {
      return { ...p, audit_summary: modelDetails[p.id] };
    }
    return p;
  };

  return (
    <div className="h-full overflow-y-auto">
      <div className="p-8 max-w-5xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-[28px] font-bold text-gray-900 tracking-tight leading-tight">
              Providers
            </h1>
            <p className="text-sm text-gray-400 mt-1">管理 API Provider 配置与审计统计</p>
          </div>
          <Button onClick={handleAdd}>添加 Provider</Button>
        </div>

        {/* Loading */}
        {loading && (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 animate-pulse">
            {[1, 2].map((i) => (
              <div key={i} className="h-64 bg-gray-100 rounded-lg" />
            ))}
          </div>
        )}

        {/* Empty */}
        {!loading && providers.length === 0 && (
          <div className="text-center py-20">
            <p className="text-gray-400 mb-4">暂无 Provider</p>
            <Button variant="secondary" onClick={handleAdd}>添加第一个 Provider</Button>
          </div>
        )}

        {/* Card grid */}
        {!loading && providers.length > 0 && (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {providers.map((p) => (
              <ProviderCard
                key={p.id}
                provider={getDisplayProvider(p)}
                selectedModel={selectedModel}
                onSelectModel={setSelectedModel}
                onEdit={handleEdit}
                onDelete={setDeleteTarget}
                onRefresh={loadProviders}
              />
            ))}
          </div>
        )}

        {/* Add/Edit Form Modal */}
        <ProviderForm
          provider={editProvider}
          open={formOpen}
          onClose={() => setFormOpen(false)}
          onSaved={loadProviders}
        />

        {/* Delete confirmation */}
        <Modal
          open={deleteTarget !== null}
          onClose={() => setDeleteTarget(null)}
          title="删除 Provider"
          footer={
            <>
              <Button variant="secondary" onClick={() => setDeleteTarget(null)}>取消</Button>
              <Button variant="danger" onClick={handleDelete}>删除</Button>
            </>
          }
        >
          <p className="text-sm text-gray-600">
            确定删除 Provider 「{deleteTarget?.name}」？此操作不可撤销，关联的审计记录将保留。
          </p>
        </Modal>
      </div>
    </div>
  );
}
