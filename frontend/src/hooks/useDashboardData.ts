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

  const fetchData = useCallback(async (isEvent: boolean) => {
    try {
      const data = await invoke<DashboardData>("get_dashboard_data");

      setDashboardData((prev) => {
        // Initial load: strip current_audit (prev is null on first call)
        const isFirstRender = !prev;
        return {
          ...data,
          current_audit: isFirstRender ? null : data.current_audit,
        };
      });

      // Only push trend point on events or manual refresh
      if (isEvent && data.latest_trend_point) {
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
    // Initial load — no trend push
    fetchData(false);

    // Listen for audit-tick events — push trend point
    const setup = async () => {
      const unlisten = await listen("audit-tick", () => {
        fetchData(true);
      });
      return unlisten;
    };
    const unlistenPromise = setup();

    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, [fetchData]);

  const refresh = useCallback(() => {
    fetchData(true);
  }, [fetchData]);

  return { dashboardData, trendQueue, loading, error, refresh };
}
