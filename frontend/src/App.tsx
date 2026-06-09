import { useState } from "react";
import Dashboard from "./pages/Dashboard";
import Providers from "./pages/Providers";
import RequestList from "./pages/RequestList";

function App() {
  const [activeTab, setActiveTab] = useState("dashboard");

  return (
    <div className="min-h-screen bg-gray-900 text-gray-100">
      <nav className="border-b border-gray-700 px-6 py-3 flex items-center gap-6">
        <h1 className="text-xl font-bold">TokenAccountant</h1>
        <button onClick={() => setActiveTab("dashboard")}
          className={`${activeTab === "dashboard" ? "text-blue-400 border-b-2 border-blue-400" : "text-gray-400"} pb-1`}>
          仪表盘
        </button>
        <button onClick={() => setActiveTab("providers")}
          className={`${activeTab === "providers" ? "text-blue-400 border-b-2 border-blue-400" : "text-gray-400"} pb-1`}>
          Provider
        </button>
        <button onClick={() => setActiveTab("requests")}
          className={`${activeTab === "requests" ? "text-blue-400 border-b-2 border-blue-400" : "text-gray-400"} pb-1`}>
          请求
        </button>
      </nav>
      <main className="p-6">
        {activeTab === "dashboard" && <Dashboard />}
        {activeTab === "providers" && <Providers />}
        {activeTab === "requests" && <RequestList />}
      </main>
    </div>
  );
}

export default App;
