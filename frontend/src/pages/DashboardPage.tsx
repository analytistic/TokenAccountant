import useDashboardData from "../hooks/useDashboardData";
import TrendChart from "../components/TrendChart";
import BarChart7Day from "../components/BarChart7Day";
import TodayOverview from "../components/TodayOverview";
import ProviderRanking from "../components/ProviderRanking";
import CurrentAudit from "../components/CurrentAudit";

export default function DashboardPage() {
  const { dashboardData, trendQueue, loading, error, refresh } = useDashboardData();

  // ---- Loading Skeleton ----
  if (loading) {
    return (
      <div className="h-full flex flex-col p-[24px_32px] gap-3 animate-pulse">
        <div className="flex items-center justify-between shrink-0">
          <div className="h-8 w-32 bg-gray-200 rounded" />
          <div className="h-8 w-20 bg-gray-200 rounded" />
        </div>
        <div className="flex-1 grid grid-cols-[2fr_1fr] grid-rows-[2fr_1fr] gap-3 min-h-0">
          <div className="bg-gray-100 rounded-lg" />
          <div className="bg-gray-100 rounded-lg" />
          <div className="bg-gray-100 rounded-lg" />
          <div className="bg-gray-100 rounded-lg" />
        </div>
      </div>
    );
  }

  // ---- Error ----
  if (error) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center">
          <p className="text-danger font-semibold mb-2">加载失败</p>
          <p className="text-sm text-gray-500 mb-4">{error}</p>
          <button
            onClick={refresh}
            className="px-4 py-2 bg-brand text-white rounded-lg text-sm font-medium hover:bg-brand-hover transition-colors"
          >
            重试
          </button>
        </div>
      </div>
    );
  }

  // ---- No data yet (first load, before any audit events) ----
  if (!dashboardData) {
    return (
      <div className="h-full flex items-center justify-center">
        <p className="text-gray-400">等待数据...</p>
      </div>
    );
  }

  const { today_summary, model_breakdown, daily_breakdown, provider_ranking, current_audit } =
    dashboardData;

  return (
    <div className="h-full flex flex-col p-[24px_32px]">
      {/* Header */}
      <div className="flex items-center justify-between mb-4 shrink-0">
        <h1 className="text-[28px] font-bold text-gray-900 tracking-tight leading-tight">
          仪表盘
        </h1>
        <button
          onClick={refresh}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <polyline points="23 4 23 10 17 10" />
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
          </svg>
          刷新
        </button>
      </div>

      {/* Grid */}
      <div
        className="flex-1 grid gap-3 min-h-0"
        style={{
          gridTemplateColumns: "2fr 1fr",
          gridTemplateRows: "2fr 1fr",
        }}
      >
        {/* Left top: TrendChart */}
        <div
          className="card flex flex-col min-h-0 min-w-0"
          style={{ gridColumn: 1, gridRow: 1 }}
        >
          <div className="card-header">
            <span className="card-title">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
              </svg>
              趋势图
            </span>
          </div>
          <div className="card-body flex flex-col flex-1 min-h-0">
            <TrendChart data={trendQueue} />
          </div>
        </div>

        {/* Left bottom: BarChart7Day */}
        <div
          className="card flex flex-col min-h-0"
          style={{ gridColumn: 1, gridRow: 2 }}
        >
          <div className="card-header">
            <span className="card-title">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="3" y="3" width="18" height="18" rx="2" />
                <line x1="3" y1="9" x2="21" y2="9" />
                <line x1="9" y1="21" x2="9" y2="9" />
              </svg>
              近 7 日统计
            </span>
          </div>
          <div className="card-body flex-1 min-h-0">
            <BarChart7Day data={daily_breakdown} />
          </div>
        </div>

        {/* Right top: TodayOverview + ProviderRanking */}
        <div
          className="flex flex-col gap-3 min-h-0"
          style={{ gridColumn: 2, gridRow: 1 }}
        >
          <TodayOverview summary={today_summary} modelBreakdown={model_breakdown} />
          <div className="card flex flex-col flex-1 min-h-0">
            <div className="card-header">
              <span className="card-title">Provider 排名</span>
            </div>
            <div className="card-body flex-1 min-h-0">
              <ProviderRanking
                providers={provider_ranking}
                onSwitch={refresh}
              />
            </div>
          </div>
        </div>

        {/* Right bottom: CurrentAudit */}
        <div
          className="card flex flex-col min-h-0"
          style={{ gridColumn: 2, gridRow: 2 }}
        >
          <div className="card-header">
            <span className="card-title">当前审计</span>
          </div>
          <div className="card-body flex-1 min-h-0">
            <CurrentAudit audit={current_audit} />
          </div>
        </div>
      </div>
    </div>
  );
}
