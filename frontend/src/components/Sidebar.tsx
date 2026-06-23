import { useState, useEffect, useCallback } from "react";
import { NavLink } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export default function Sidebar() {
  const [proxyRunning, setProxyRunning] = useState(false);
  const [proxyPort, setProxyPort] = useState(8080);
  const [uptime, setUptime] = useState(0);
  const [showDev, setShowDev] = useState(false);

  // Load initial proxy status + dev mode
  useEffect(() => {
    (async () => {
      try {
        const status = await invoke<{ running: boolean; port: number; uptime_secs: number }>(
          "get_proxy_status"
        );
        console.log("[INVOKE] get_proxy_status →", status);
        setProxyRunning(status.running);
        setProxyPort(status.port);
        setUptime(status.uptime_secs);
      } catch (e) { console.error("[INVOKE] get_proxy_status failed", e); }
      try {
        const config = await invoke<{ dev_mode_enabled: boolean }>("get_app_config");
        console.log("[INVOKE] get_app_config →", config);
        setShowDev(config.dev_mode_enabled);
      } catch (e) { console.error("[INVOKE] get_app_config failed", e); }
    })();
  }, []);

  // Listen for dev-mode changes from Settings page
  useEffect(() => {
    const unlisten = listen<{ enabled: boolean }>("dev-mode-changed", (e) => {
      setShowDev(e.payload.enabled);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // Uptime timer
  useEffect(() => {
    if (!proxyRunning) {
      setUptime(0);
      return;
    }
    const timer = setInterval(() => setUptime((s) => s + 1), 1000);
    return () => clearInterval(timer);
  }, [proxyRunning]);

  const formatUptime = useCallback((seconds: number) => {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  }, []);

  const handleProxyToggle = async () => {
    try {
      if (proxyRunning) {
        await invoke("stop_proxy");
        setProxyRunning(false);
      } else {
        const port = await invoke<number>("start_proxy", {
          bindAddr: `0.0.0.0:${proxyPort}`,
        });
        setProxyPort(port);
        setProxyRunning(true);
      }
    } catch (e) {
      console.error("Proxy toggle failed:", e);
    }
  };

  const linkBase =
    "flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors duration-fast";

  return (
    <aside
      className="flex flex-col w-[var(--sidebar-w)] h-screen bg-gray-50 border-r border-gray-200 shrink-0 select-none"
    >
      {/* Brand */}
      <div className="px-4 pt-6 pb-4">
        <div className="flex items-center gap-2.5">
          <svg width="28" height="28" viewBox="0 0 28 28" fill="none">
            <rect width="28" height="28" rx="8" fill="var(--brand)" />
            <path d="M8 10l6-4 6 4v8l-6 4-6-4V10z" stroke="#fff" strokeWidth="1.5" fill="none" />
            <circle cx="14" cy="14" r="3" fill="#fff" />
          </svg>
          <div>
            <div className="text-sm font-bold text-gray-900 tracking-tight leading-tight">
              TokenAccountant
            </div>
            <div className="text-[10px] text-gray-400 tracking-wide">v0.3</div>
          </div>
        </div>
      </div>

      {/* Nav links */}
      <nav className="flex-1 flex flex-col gap-1 px-2">
        <NavLink
          to="/"
          end
          className={({ isActive }) =>
            `${linkBase} ${isActive ? "bg-brand-subtle text-brand" : "text-gray-600 hover:bg-gray-100"}`
          }
        >
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
            <rect x="2" y="2" width="7" height="7" rx="1.5" stroke="currentColor" strokeWidth="1.5" />
            <rect x="11" y="2" width="7" height="7" rx="1.5" stroke="currentColor" strokeWidth="1.5" />
            <rect x="2" y="11" width="7" height="7" rx="1.5" stroke="currentColor" strokeWidth="1.5" />
            <rect x="11" y="11" width="7" height="7" rx="1.5" stroke="currentColor" strokeWidth="1.5" />
          </svg>
          仪表盘
        </NavLink>

        <NavLink
          to="/providers"
          className={({ isActive }) =>
            `${linkBase} ${isActive ? "bg-brand-subtle text-brand" : "text-gray-600 hover:bg-gray-100"}`
          }
        >
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
            <rect x="2" y="3" width="16" height="10" rx="2" stroke="currentColor" strokeWidth="1.5" />
            <rect x="5" y="7" width="10" height="6" rx="1" stroke="currentColor" strokeWidth="1.5" />
            <line x1="10" y1="0" x2="10" y2="5" stroke="currentColor" strokeWidth="1.5" />
          </svg>
          Providers
        </NavLink>

        <NavLink
          to="/settings"
          className={({ isActive }) =>
            `${linkBase} ${isActive ? "bg-brand-subtle text-brand" : "text-gray-600 hover:bg-gray-100"}`
          }
        >
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
            <circle cx="10" cy="10" r="3" stroke="currentColor" strokeWidth="1.5" />
            <path
              d="M10 2v2M10 16v2M2 10h2M16 10h2M4.93 4.93l1.41 1.41M13.66 13.66l1.41 1.41M4.93 15.07l1.41-1.41M13.66 6.34l1.41-1.41"
              stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"
            />
          </svg>
          设置
        </NavLink>

        {showDev && (
          <NavLink
            to="/dev"
            className={({ isActive }) =>
              `${linkBase} ${isActive ? "bg-warning-subtle text-warning" : "text-gray-500 hover:bg-gray-100"}`
            }
          >
            <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
              <path d="M7 3l-4 7 4 7M13 3l4 7-4 7M12 2l-4 16" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
            开发者
          </NavLink>
        )}
      </nav>

      {/* Proxy button */}
      <div className="px-3 pb-4">
        <button
          onClick={handleProxyToggle}
          className={`w-full flex items-center gap-2 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-fast ease-out
            ${
              proxyRunning
                ? "bg-success-subtle text-success border border-success-border"
                : "bg-gray-100 text-gray-600 hover:bg-gray-200"
            }`}
        >
          <span
            className={`w-2 h-2 rounded-full ${proxyRunning ? "bg-success animate-pulse" : "bg-gray-400"}`}
          />
          {proxyRunning ? `代理运行中 ${formatUptime(uptime)}` : "▶ 启动代理"}
        </button>
      </div>
    </aside>
  );
}
