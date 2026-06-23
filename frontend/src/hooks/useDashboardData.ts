import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { DashboardData, TrendPoint } from "../types";
import { ZERO_TREND_POINT } from "../types";

const QUEUE_SIZE = 20;

export default function useDashboardData() {
  const [dashboardData, setDashboardData] = useState<DashboardData | null>(null);
  const [trendQueue, setTrendQueue] = useState<TrendPoint[]>(
    Array.from({ length: QUEUE_SIZE }, () => ({ ...ZERO_TREND_POINT }))
  );
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    try {
      const data = await invoke<DashboardData>("get_dashboard_data");
      // Strip current_audit on initial load (plan: null until first event)
      setDashboardData((prev) => {
        if (!prev && data.current_audit) {
          // Initial load: keep current_audit null, wait for event
          data.current_audit = null;
        }
        return data;
      });
      // Shift/push trend queue
      if (data.latest_trend_point) {
        setTrendQueue((prev) => [...prev.slice(1), data.latest_trend_point!]);
      }
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    // Initial load
    fetchData();

    // Listen for audit-tick events
    const setup = async () => {
      const unlisten = await listen("audit-tick", () => {
        fetchData();
      });
      return unlisten;
    };
    const unlistenPromise = setup();

    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, [fetchData]);

  return { dashboardData, trendQueue, loading, error, refresh: fetchData };
}
