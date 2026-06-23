import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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
  const unlistenRef = useRef<UnlistenFn | null>(null);

  const fetchData = useCallback(async (isEvent: boolean) => {
    console.log("[useDashboardData] fetchData called, isEvent:", isEvent);
    try {
      const data = await invoke<DashboardData>("get_dashboard_data");
      console.log("[useDashboardData] got data, latest_trend_point:", data.latest_trend_point);

      setDashboardData((prev) => {
        const isFirstRender = !prev;
        return {
          ...data,
          current_audit: isFirstRender ? null : data.current_audit,
        };
      });

      if (isEvent && data.latest_trend_point) {
        setTrendQueue((prev) => {
          const lastIdx = prev[prev.length - 1]?.idx ?? 0;
          if (data.latest_trend_point!.idx > lastIdx) {
            console.log("[useDashboardData] pushing trend point:", data.latest_trend_point!.idx);
            return [...prev.slice(1), data.latest_trend_point!];
          }
          console.log("[useDashboardData] skipping duplicate trend point, idx:", data.latest_trend_point!.idx);
          return prev;
        });
      }

      setError(null);
    } catch (e) {
      console.error("[useDashboardData] fetchData error:", e);
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    console.log("[useDashboardData] useEffect mount");
    let cancelled = false;

    // Initial load
    fetchData(false);

    // Set up event listener
    (async () => {
      try {
        const unlisten = await listen("audit-tick", () => {
          console.log("[useDashboardData] audit-tick event received!");
          if (!cancelled) fetchData(true);
        });
        console.log("[useDashboardData] audit-tick listener registered");
        if (!cancelled) {
          unlistenRef.current = unlisten;
        } else {
          unlisten();
        }
      } catch (e) {
        console.error("[useDashboardData] listen setup error:", e);
      }
    })();

    return () => {
      console.log("[useDashboardData] useEffect cleanup");
      cancelled = true;
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
    };
  }, [fetchData]);

  const refresh = useCallback(() => {
    console.log("[useDashboardData] manual refresh");
    fetchData(true);
  }, [fetchData]);

  return { dashboardData, trendQueue, loading, error, refresh };
}
