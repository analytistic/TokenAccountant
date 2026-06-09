import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ProxyStatus {
  running: boolean;
  port: number;
  uptime_secs: number;
  requests_served: number;
}

export default function Dashboard() {
  const [status, setStatus] = useState<ProxyStatus | null>(null);

  useEffect(() => {
    invoke<ProxyStatus>("get_proxy_status").then(setStatus).catch(console.error);
  }, []);

  return (
    <div>
      <h2 className="text-2xl font-bold mb-6">仪表盘</h2>
      <div className="grid grid-cols-3 gap-4 mb-8">
        <div className="bg-gray-800 rounded-lg p-4">
          <div className="text-gray-400 text-sm">代理状态</div>
          <div className={status?.running ? "text-green-400" : "text-red-400"}>
            {status?.running ? "运行中" : "已停止"}
          </div>
        </div>
        <div className="bg-gray-800 rounded-lg p-4">
          <div className="text-gray-400 text-sm">监听端口</div>
          <div className="text-xl">{status?.port || "-"}</div>
        </div>
        <div className="bg-gray-800 rounded-lg p-4">
          <div className="text-gray-400 text-sm">已处理请求</div>
          <div className="text-xl">{status?.requests_served || 0}</div>
        </div>
      </div>
      <button
        onClick={() => invoke("start_proxy", { bindAddr: "0.0.0.0:8080" })}
        className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded"
      >
        启动代理
      </button>
    </div>
  );
}
