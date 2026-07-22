import { useEffect, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import Button from "../components/Button";
import Toggle from "../components/Toggle";

interface AppConfig {
  proxy_port: number;
  language: string;
  auto_start_proxy: boolean;
  dev_mode_enabled: boolean;
  dev_trace_buffer_size: number;
}

const DEFAULT_CONFIG: AppConfig = {
  proxy_port: 8080,
  language: "zh-CN",
  auto_start_proxy: false,
  dev_mode_enabled: false,
  dev_trace_buffer_size: 100,
};

function SettingRow({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-8 py-4 first:pt-0 last:pb-0">
      <div className="min-w-0">
        <div className="text-sm font-medium text-gray-800">{title}</div>
        <p className="mt-1 text-xs leading-relaxed text-gray-400">{description}</p>
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section>
      <h2 className="mb-3 text-xs font-semibold uppercase tracking-wide text-gray-400">{title}</h2>
      <div className="divide-y divide-gray-100 rounded-lg border border-gray-200 bg-white p-5 shadow-card">
        {children}
      </div>
    </section>
  );
}

export default function SettingsPage() {
  const [config, setConfig] = useState<AppConfig>(DEFAULT_CONFIG);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    invoke<AppConfig>("get_app_config")
      .then((value) => {
        if (active) setConfig(value);
      })
      .catch((reason) => {
        if (active) setError(`设置加载失败：${String(reason)}`);
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, []);

  const update = <K extends keyof AppConfig>(key: K, value: AppConfig[K]) => {
    setConfig((current) => ({ ...current, [key]: value }));
    setSaved(false);
    setError(null);
  };

  const updateDevMode = (enabled: boolean) => {
    update("dev_mode_enabled", enabled);
    emit("dev-mode-changed", { enabled }).catch((reason) => {
      setError(`开发者模式切换失败：${String(reason)}`);
    });
  };

  const portValid = Number.isInteger(config.proxy_port) && config.proxy_port >= 1 && config.proxy_port <= 65535;
  const bufferValid =
    Number.isInteger(config.dev_trace_buffer_size) &&
    config.dev_trace_buffer_size >= 1 &&
    config.dev_trace_buffer_size <= 10000;

  const handleSave = async () => {
    if (!portValid || !bufferValid) return;
    setSaving(true);
    setSaved(false);
    setError(null);
    try {
      await invoke("save_app_config", { config });
      await emit("dev-mode-changed", { enabled: config.dev_mode_enabled });
      await emit("app-config-changed", config);
      setSaved(true);
      window.setTimeout(() => setSaved(false), 2000);
    } catch (reason) {
      setError(`保存失败：${String(reason)}`);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="h-full overflow-y-auto">
        <div className="mx-auto max-w-3xl animate-pulse p-8">
          <div className="mb-8 h-9 w-28 rounded bg-gray-200" />
          {[1, 2, 3].map((item) => (
            <div key={item} className="mb-6 h-36 rounded-lg bg-gray-100" />
          ))}
        </div>
      </div>
    );
  }

  const inputClass =
    "w-40 rounded-md border border-gray-200 bg-white px-3 py-2 text-sm text-gray-800 outline-none transition-shadow focus:border-brand focus:ring-[3px] focus:ring-brand-muted";

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-3xl p-8 pb-12">
        <div className="mb-7 flex items-end justify-between">
          <div>
            <h1 className="text-[28px] font-bold leading-tight tracking-tight text-gray-900">设置</h1>
            <p className="mt-1 text-sm text-gray-400">配置代理、界面与开发者工具</p>
          </div>
          <Button onClick={handleSave} disabled={saving || !portValid || !bufferValid}>
            {saving ? "保存中…" : saved ? "已保存 ✓" : "保存设置"}
          </Button>
        </div>

        {error && (
          <div role="alert" className="mb-5 rounded-md border border-danger-border bg-danger-subtle px-4 py-3 text-sm text-danger">
            {error}
          </div>
        )}

        <div className="space-y-6">
          <Section title="基础设置">
            <SettingRow title="代理端口" description="本地代理服务监听的端口，修改后需重新启动代理。">
              <div>
                <input
                  className={`${inputClass} ${!portValid ? "border-danger focus:border-danger focus:ring-danger-subtle" : ""}`}
                  type="number"
                  min={1}
                  max={65535}
                  value={config.proxy_port}
                  aria-invalid={!portValid}
                  onChange={(event) => update("proxy_port", Number(event.target.value))}
                />
                {!portValid && <p className="mt-1 text-xs text-danger">请输入 1–65535</p>}
              </div>
            </SettingRow>
            <SettingRow title="界面语言" description="选择应用界面使用的语言。">
              <select
                className={inputClass}
                value={config.language}
                onChange={(event) => update("language", event.target.value)}
              >
                <option value="zh-CN">简体中文</option>
                <option value="en-US">English</option>
              </select>
            </SettingRow>
          </Section>

          <Section title="代理设置">
            <SettingRow title="启动时自动运行代理" description="打开 TokenAccountant 后自动启动本地代理服务。">
              <Toggle
                checked={config.auto_start_proxy}
                onChange={(value) => update("auto_start_proxy", value)}
              />
            </SettingRow>
          </Section>

          <Section title="开发者模式">
            <SettingRow title="启用开发者模式" description="在侧边栏显示开发者工具，用于检查审计过程中的渲染文本。">
              <Toggle
                checked={config.dev_mode_enabled}
                onChange={updateDevMode}
              />
            </SettingRow>
            {config.dev_mode_enabled && (
              <SettingRow title="DevTrace 缓冲区大小" description="保留的调试记录数量，允许范围为 1–10000。">
                <div>
                  <input
                    className={`${inputClass} ${!bufferValid ? "border-danger focus:border-danger focus:ring-danger-subtle" : ""}`}
                    type="number"
                    min={1}
                    max={10000}
                    value={config.dev_trace_buffer_size}
                    aria-invalid={!bufferValid}
                    onChange={(event) => update("dev_trace_buffer_size", Number(event.target.value))}
                  />
                  {!bufferValid && <p className="mt-1 text-xs text-danger">请输入 1–10000</p>}
                </div>
              </SettingRow>
            )}
          </Section>

          <Section title="关于">
            <SettingRow title="TokenAccountant" description="Token 透明化审计工具 · v0.3">
              <div className="flex items-center gap-2">
                <a
                  className="rounded-md px-3 py-2 text-xs font-medium text-brand hover:bg-brand-subtle"
                  href="https://github.com/analytistic/TokenAccountant"
                  target="_blank"
                  rel="noreferrer"
                >
                  GitHub ↗
                </a>
                <a
                  className="rounded-md px-3 py-2 text-xs font-medium text-gray-600 hover:bg-gray-100"
                  href="https://github.com/analytistic/TokenAccountant/blob/main/docs/product/changelog.md"
                  target="_blank"
                  rel="noreferrer"
                >
                  Changelog ↗
                </a>
              </div>
            </SettingRow>
          </Section>
        </div>
      </div>
    </div>
  );
}
