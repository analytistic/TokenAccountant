import useDashboardData from "../hooks/useDashboardData";
import TrendChart from "../components/TrendChart";
import BarChart7Day from "../components/BarChart7Day";
import TodayOverview from "../components/TodayOverview";
import ProviderRanking from "../components/ProviderRanking";
import CurrentAudit from "../components/CurrentAudit";

export default function DashboardPage() {
  const { dashboardData, trendQueue, loading, error, refresh } = useDashboardData();

  if (loading) {
    return (
      <div className="dashboard-scroll">
        <div className="dashboard-shell gap-3 animate-pulse">
          <div className="h-9 w-32 shrink-0 rounded bg-gray-200" />
          <div className="h-24 shrink-0 rounded-lg bg-gray-100" />
          <div className="grid min-h-0 flex-1 grid-cols-[2fr_1fr] gap-3">
            {[1, 2, 3, 4].map((item) => <div key={item} className="rounded-lg bg-gray-100" />)}
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <p className="mb-2 font-semibold text-danger">审计数据加载失败</p>
          <p className="mb-4 text-sm text-gray-500">{error}</p>
          <button onClick={refresh} className="rounded-lg bg-brand px-4 py-2 text-sm font-medium text-white hover:bg-brand-hover">重新加载</button>
        </div>
      </div>
    );
  }

  if (!dashboardData) return <div className="flex h-full items-center justify-center text-gray-400">等待审计数据...</div>;

  const { today_summary, model_breakdown, daily_breakdown, provider_ranking, current_audit } = dashboardData;

  return (
    <div className="dashboard-scroll">
      <div className="dashboard-shell">
        <header className="mb-4 flex shrink-0 items-center justify-between">
          <div>
            <h1 className="text-[26px] font-bold leading-tight tracking-tight text-gray-900">今日审计</h1>
            <p className="mt-0.5 text-[11px] text-gray-400">核对 Provider 声称值与本地 Tokenizer 检测值</p>
          </div>
          <button onClick={refresh} className="flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-medium text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-700">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="23 4 23 10 17 10" /><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" /></svg>
            刷新
          </button>
        </header>

        <div className="reconciliation-grid">
          <TodayOverview summary={today_summary} modelBreakdown={model_breakdown} />

          <section className="card reconciliation-trend flex min-h-0 min-w-0 flex-col">
            <div className="card-header"><span className="card-title">最近请求差异</span><span className="text-[9px] text-gray-400">最近 20 次请求</span></div>
            <div className="card-body flex min-h-0 flex-1 flex-col"><TrendChart data={trendQueue} /></div>
          </section>

          <section className="card reconciliation-audit flex min-h-0 min-w-0 flex-col overflow-hidden">
            <div className="card-header"><span className="card-title">最新审计</span></div>
            <div className="card-body min-h-0 flex-1 overflow-hidden"><CurrentAudit audit={current_audit} /></div>
          </section>

          <section className="card reconciliation-history flex min-h-0 min-w-0 flex-col">
            <div className="card-header"><span className="card-title">近 7 日净差异</span></div>
            <div className="card-body min-h-0 flex-1 overflow-hidden"><BarChart7Day data={daily_breakdown} /></div>
          </section>

          <section className="card provider-ranking-card reconciliation-providers flex min-h-0 min-w-0 flex-col overflow-hidden">
            <div className="card-header"><span className="card-title">Provider 对比</span><span className="text-[9px] text-gray-400">点击切换</span></div>
            <div className="card-body provider-ranking-body min-h-0 flex-1 overflow-hidden"><ProviderRanking providers={provider_ranking} onSwitch={refresh} /></div>
          </section>
        </div>
      </div>
    </div>
  );
}
