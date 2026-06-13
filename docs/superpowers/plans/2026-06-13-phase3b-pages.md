# Phase 3B: Sidebar + Three Pages — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the full v0.3 UI: sidebar navigation, Dashboard with SVG charts, Providers with audit bars, and Settings page.

**Architecture:** React router replaces top-nav with sidebar. Dashboard uses 1 BFF command `get_dashboard_data` and renders 6 sub-components with inline SVG charts. Providers page loads list, model filtering triggers per-provider detail fetch. Settings reads/writes config via IPC.

**Tech Stack:** React 18 + TypeScript + Tailwind CSS + react-router-dom v6 + Tauri IPC (invoke)

**Spec ref:** `docs/superpowers/specs/2026-06-09-phase3-ui-ux-redesign.md` §四、§六、§八

**Depends on:** Phase 3A (design tokens, components, backend commands)

---

### Task 1: App shell — Sidebar + Routing

**Files:**
- Create: `frontend/src/components/Sidebar.tsx`
- Modify: `frontend/src/App.tsx`

- [ ] **Step 1: Write Sidebar.tsx**

Create `frontend/src/components/Sidebar.tsx`:
```tsx
import { NavLink } from 'react-router-dom'
import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface ProxyStatus {
  running: boolean
  port: number
  requests_served: number
}

const navItems = [
  { to: '/', label: 'Dashboard', icon: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>' },
  { to: '/providers', label: 'Providers', icon: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 20h16"/><path d="M6 16l6-12 6 12"/></svg>' },
  { to: '/settings', label: 'Settings', icon: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>' },
]

export default function Sidebar() {
  const [proxy, setProxy] = useState<ProxyStatus>({ running: false, port: 0, requests_served: 0 })

  const fetchStatus = () => {
    invoke<ProxyStatus>('get_proxy_status').then(setProxy).catch(console.error)
  }

  useEffect(() => { fetchStatus(); const id = setInterval(fetchStatus, 10000); return () => clearInterval(id) }, [])

  const toggleProxy = () => {
    if (proxy.running) {
      invoke('stop_proxy').then(fetchStatus).catch(console.error)
    } else {
      invoke('start_proxy', { bindAddr: `0.0.0.0:${proxy.port || 8080}` }).then(fetchStatus).catch(console.error)
    }
  }

  return (
    <aside className="w-[220px] min-w-[220px] h-screen bg-white border-r border-gray-200 flex flex-col shrink-0 z-50">
      {/* Brand */}
      <div className="px-5 py-6 flex items-center gap-3 border-b border-gray-100">
        <div className="w-[34px] h-[34px] rounded-[10px] bg-gradient-to-br from-brand to-brand-light flex items-center justify-center text-white shrink-0">
          <svg width="26" height="26" viewBox="0 0 32 32" fill="none"><path d="M9 12C9 7 12 4 16 4C20 4 23 7 23 12C23 16 20 18 16 18C12 18 9 16 9 12Z" stroke="white" stroke-width="2" fill="none"/><path d="M10 6L8 2L12 5" stroke="white" stroke-width="2" fill="none"/><path d="M22 6L24 2L20 5" stroke="white" stroke-width="2" fill="none"/><circle cx="12.5" cy="12" r="3.5" stroke="white" stroke-width="1.8" fill="none"/><circle cx="19.5" cy="12" r="3.5" stroke="white" stroke-width="1.8" fill="none"/><circle cx="12.5" cy="12" r="1.8" fill="white"/><circle cx="19.5" cy="12" r="1.8" fill="white"/><path d="M16 14L14.5 17H17.5L16 14Z" fill="white"/><path d="M11 18C8 20 7 24 8 28H24C25 24 24 20 21 18" stroke="white" stroke-width="2" fill="none"/></svg>
        </div>
        <div>
          <div className="text-[15px] font-bold text-gray-900 tracking-tight leading-[1.2]">TokenAccountant</div>
          <div className="text-[10px] text-gray-400 font-medium">v0.3</div>
        </div>
      </div>

      {/* Nav */}
      <nav className="flex-1 p-3 flex flex-col gap-[2px] overflow-y-auto">
        {navItems.map(item => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.to === '/'}
            className={({ isActive }) =>
              `flex items-center gap-3 px-3 py-2 rounded-[6px] text-sm font-medium transition-all duration-150 ease-out relative whitespace-nowrap min-h-[36px] ${
                isActive
                  ? 'bg-[rgba(79,70,229,0.06)] text-brand before:absolute before:left-0 before:top-2 before:bottom-2 before:w-[3px] before:bg-brand before:rounded-r-[2px]'
                  : 'text-gray-500 hover:bg-gray-100 hover:text-gray-800'
              }`
            }
          >
            <span className="w-[18px] h-[18px] shrink-0 flex items-center justify-center" dangerouslySetInnerHTML={{ __html: item.icon }} />
            {item.label}
          </NavLink>
        ))}
      </nav>

      {/* Proxy toggle */}
      <div className="p-3 border-t border-gray-100">
        <button
          onClick={toggleProxy}
          className={`w-full flex items-center justify-center gap-2 px-3 py-2 rounded-[6px] text-sm font-bold transition-all duration-150 min-h-[36px] ${
            proxy.running
              ? 'bg-[#059669] text-white shadow-[0_1px_2px_rgba(5,150,105,0.2)] hover:bg-[#047857] hover:-translate-y-px hover:shadow-[0_4px_8px_rgba(5,150,105,0.25)]'
              : 'bg-gray-300 text-gray-600 hover:bg-gray-400 hover:shadow-[0_2px_4px_rgba(0,0,0,0.1)]'
          }`}
        >
          <span className={`w-[6px] h-[6px] rounded-full bg-current ${proxy.running ? 'animate-pulse' : ''}`} />
          {proxy.running ? '代理运行中' : '启动代理'}
        </button>
      </div>
    </aside>
  )
}
```

- [ ] **Step 2: Rewrite App.tsx with routing**

Replace `frontend/src/App.tsx`:
```tsx
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import Sidebar from './components/Sidebar'
import DashboardPage from './pages/DashboardPage'
import ProvidersPage from './pages/ProvidersPage'
import SettingsPage from './pages/SettingsPage'

export default function App() {
  return (
    <BrowserRouter>
      <div className="flex h-screen w-screen overflow-hidden">
        <Sidebar />
        <main className="flex-1 overflow-y-auto overflow-x-hidden min-w-0">
          <Routes>
            <Route path="/" element={<DashboardPage />} />
            <Route path="/providers" element={<ProvidersPage />} />
            <Route path="/settings" element={<SettingsPage />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  )
}
```

- [ ] **Step 3: Build check**

Run: `cd frontend && npx tsc --noEmit`
Expected: No errors. (May need to install react-router-dom types if not already: `npm install`)

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: add Sidebar nav with proxy toggle, restructure App with react-router"
```

---

### Task 2: Dashboard page skeleton + data fetching

**Files:**
- Create: `frontend/src/pages/DashboardPage.tsx`

- [ ] **Step 1: Write DashboardPage data hook**

Create `frontend/src/hooks/useDashboardData.ts`:
```tsx
import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

export interface DashboardData {
  proxy_status: { running: boolean; port: number; requests_served: number }
  today_summary: {
    total_requests: number; suspicious_requests: number
    input_diff_rate: number; cache_diff_rate: number; output_diff_rate: number
    input_diff_tokens: number; cache_diff_tokens: number; output_diff_tokens: number
    input_match_rate: number; cache_match_rate: number; output_match_rate: number
  }
  trend_data: Array<{
    idx: number
    input_claimed: number;  input_detected: number
    cache_claimed: number;  cache_detected: number
    output_claimed: number; output_detected: number
  }>
  provider_ranking: Array<{
    id: string; name: string; url: string; credibility: number
    input_diff_rate: number; cache_diff_rate: number; output_diff_rate: number
  }>
  current_audit: null | {
    audit_passed: boolean
    input_audit: number;  input_claimed: number;  input_diff: number
    cache_audit: number;  cache_claimed: number;  cache_diff: number
    output_audit: number; output_claimed: number; output_diff: number
  }
}

export function useDashboardData() {
  const [data, setData] = useState<DashboardData | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = () => {
    setLoading(true)
    setError(null)
    invoke<DashboardData>('get_dashboard_data')
      .then(setData)
      .catch(e => setError(e.toString()))
      .finally(() => setLoading(false))
  }

  useEffect(load, [])

  return { data, loading, error, refresh: load }
}
```

- [ ] **Step 2: Write DashboardPage skeleton**

Create `frontend/src/pages/DashboardPage.tsx`:
```tsx
import { useDashboardData } from '../hooks/useDashboardData'
import TrendChart from '../components/TrendChart'
import BarChart7Day from '../components/BarChart7Day'
import TodayOverview from '../components/TodayOverview'
import ProviderRanking from '../components/ProviderRanking'
import CurrentAudit from '../components/CurrentAudit'
import SessionTimer from '../components/SessionTimer'
// Button will be imported when used

export default function DashboardPage() {
  const { data, loading, error, refresh } = useDashboardData()

  return (
    <div className="p-10 px-12 max-w-[1200px] animate-[page-in_250ms_ease-out]">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <h1 className="text-[1.75rem] font-bold text-gray-900 tracking-tight leading-[1.2]">仪表盘</h1>
        <div className="flex gap-2 items-center">
          <button onClick={refresh} className="btn btn-ghost btn-sm" title="刷新">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
          </button>
        </div>
      </div>

      {/* Grid */}
      {loading ? (
        <DashboardSkeleton />
      ) : error ? (
        <div className="text-center py-16 text-gray-400">{error}</div>
      ) : data ? (
        <div className="grid grid-cols-[1fr_300px] gap-4">
          {/* Left top — Trend */}
          <div className="bg-white border border-gray-200 rounded-[16px] shadow-card overflow-hidden col-start-1 row-start-1 flex flex-col">
            <TrendChart data={data.trend_data} />
          </div>

          {/* Right split — today overview + ranking */}
          <div className="col-start-2 row-start-1 flex flex-col gap-4">
            <TodayOverview summary={data.today_summary} />
            <ProviderRanking providers={data.provider_ranking} />
          </div>

          {/* Left bottom — 7-day bar chart */}
          <div className="bg-white border border-gray-200 rounded-[16px] shadow-card overflow-hidden col-start-1 row-start-2">
            <BarChart7Day data={[]} /> {/* Will receive daily_breakdown from backend later */}
          </div>

          {/* Right bottom — current audit */}
          <div className="col-start-2 row-start-2">
            <CurrentAudit audit={data.current_audit} />
          </div>

          {/* Session timer */}
          <div className="col-start-2 row-start-3 flex justify-end">
            <SessionTimer />
          </div>
        </div>
      ) : null}
    </div>
  )
}

function DashboardSkeleton() {
  return (
    <div className="grid grid-cols-[1fr_300px] gap-4">
      <div className="bg-white border border-gray-200 rounded-[16px] p-6 space-y-4">
        {[1,2,3].map(i => <div key={i} className="h-20 skeleton rounded-[6px]" />)}
      </div>
      <div className="space-y-4">
        {[1,2].map(i => <div key={i} className="bg-white border border-gray-200 rounded-[16px] p-4 h-32 skeleton" />)}
      </div>
    </div>
  )
}
```

- [ ] **Step 3: Build check**

Run: `cd frontend && npx tsc --noEmit`
Expected: No errors (components not yet created, but imports will fail — that's OK, we add them next)

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: add DashboardPage skeleton with data hook and loading state"
```

---

### Task 3: Dashboard SVG chart components

**Files:**
- Create: `frontend/src/components/TrendChart.tsx`
- Create: `frontend/src/components/BarChart7Day.tsx`
- Create: `frontend/src/components/TripleRing.tsx`

- [ ] **Step 1: Write TrendChart.tsx**

Create `frontend/src/components/TrendChart.tsx`. Implements 3 sub-charts (Input/Cache/Output) as inline SVG. Each chart shows two lines (claimed=faint, detected=bold) with diff area fill. Copied from HTML prototype `renderSingleChart` function, wrapped as React component.

```tsx
interface TrendProps {
  data: Array<{
    idx: number
    input_claimed: number;  input_detected: number
    cache_claimed: number;  cache_detected: number
    output_claimed: number; output_detected: number
  }>
}

const COLORS = { input: '#60A5FA', cache: '#FB923C', output: '#34D399' }

function SubChart({ data, color, label, showXAxis }: {
  data: { c: number; d: number }[]
  color: string
  label: string
  showXAxis: boolean
}) {
  if (!data.length) return <div className="flex-1 flex items-center justify-center text-gray-300 text-xs">暂无数据</div>

  const W = 740, H = 95, pad = { top: 8, right: 16, bottom: showXAxis ? 22 : 6, left: 44 }
  const cW = W - pad.left - pad.right, cH = H - pad.top - pad.bottom
  const n = data.length
  const allVals = data.flatMap(d => [d.c, d.d])
  const yMax = Math.max(...allVals) * 1.18
  const X = (i: number) => pad.left + (i / (n - 1)) * cW
  const Y = (v: number) => pad.top + cH - ((v - 0) / (yMax - 0)) * cH
  const fmt = (v: number) => v >= 1e6 ? (v/1e6).toFixed(1)+'M' : v >= 1e3 ? (v/1e3).toFixed(0)+'K' : String(v)

  // Catmull-Rom to Bezier (same as prototype)
  function catmullToBezier(pts: [number,number][]) {
    if (pts.length < 2) return ''
    let d = `M${pts[0][0].toFixed(1)},${pts[0][1].toFixed(1)}`
    for (let i = 0; i < pts.length - 1; i++) {
      const p0 = pts[Math.max(0,i-1)], p1 = pts[i], p2 = pts[i+1], p3 = pts[Math.min(pts.length-1,i+2)]
      d += `C${(p1[0]+(p2[0]-p0[0])/6).toFixed(1)},${(p1[1]+(p2[1]-p0[1])/6).toFixed(1)} ${(p2[0]-(p3[0]-p1[0])/6).toFixed(1)},${(p2[1]-(p3[1]-p1[1])/6).toFixed(1)} ${p2[0].toFixed(1)},${p2[1].toFixed(1)}`
    }
    return d
  }

  const claimedPts: [number,number][] = data.map((d,i) => [X(i), Y(d.c)])
  const detectedPts: [number,number][] = data.map((d,i) => [X(i), Y(d.d)])

  // Area fill
  let areaD = `M${claimedPts[0][0].toFixed(1)},${claimedPts[0][1].toFixed(1)}`
  for (let i = 1; i < claimedPts.length; i++) areaD += `L${claimedPts[i][0].toFixed(1)},${claimedPts[i][1].toFixed(1)}`
  for (let i = detectedPts.length-1; i >= 0; i--) areaD += `L${detectedPts[i][0].toFixed(1)},${detectedPts[i][1].toFixed(1)}`
  areaD += 'Z'

  // Diff rate
  const latest = data[n-1]
  const diffRate = latest.c > 0 ? ((latest.c - latest.d) / latest.c * 100) : 0
  const diffColor = diffRate < 3 ? '#059669' : diffRate < 5 ? '#D97706' : '#DC2626'

  return (
    <div className="flex-1 min-h-0 flex flex-col">
      <div className="shrink-0 flex items-center h-[14px] px-[2px]">
        <span className="flex items-center gap-[3px] text-[9px] font-bold uppercase tracking-[0.02em]" style={{ color }}>
          <span className="inline-block w-[5px] h-[5px] rounded-[1.5px] shrink-0" style={{ background: color }} />
          {label}
        </span>
      </div>
      <svg viewBox={`0 0 ${W} ${H}`} className="w-full flex-1" role="img" aria-label={`${label} token trend`}>
        {/* Grid lines */}
        {[0,1,2,3].map(i => {
          const v = 0 + (yMax - 0) * (i/3)
          return <g key={i}>
            <line x1={pad.left} y1={Y(v)} x2={W-pad.right} y2={Y(v)} stroke="#F3F4F6" strokeWidth="1"/>
            <text x={pad.left-5} y={Y(v)+3} textAnchor="end" fill="#D1D5DB" fontSize="8" fontFamily="var(--font-mono)">{fmt(v)}</text>
          </g>
        })}
        <path d={areaD} fill={color} fillOpacity="0.10"/>
        <path d={catmullToBezier(claimedPts)} fill="none" stroke={color} strokeWidth="1.5" strokeOpacity="0.35" strokeLinecap="round"/>
        <path d={catmullToBezier(detectedPts)} fill="none" stroke={color} strokeWidth="2" strokeLinecap="round"/>
        {[claimedPts, detectedPts].map((pts, j) =>
          <circle key={j} cx={pts[n-1][0]} cy={pts[n-1][1]} r="2.5" fill={color} opacity={j===0 ? 0.4 : 1} />
        )}
        {showXAxis && <text x={(pad.left+(W-pad.right))/2} y={H-0} textAnchor="middle" fill="#9CA3AF" fontSize="8">请求序号</text>}
      </svg>
      <div className="shrink-0 h-5 flex items-center justify-end px-1 text-[12px] font-bold font-mono" style={{ color: diffColor }}>
        {diffRate.toFixed(1)}%
      </div>
    </div>
  )
}

export default function TrendChart({ data }: TrendProps) {
  if (!data.length) return <div className="p-4 text-center text-gray-400 text-sm">暂无趋势数据</div>

  const reversed = [...data].reverse()
  const series = [
    { key: 'input',  label: 'Input',  color: COLORS.input,  d: reversed.map(d => ({ c: d.input_claimed, d: d.input_detected })) },
    { key: 'cache',  label: 'Cache',  color: COLORS.cache,  d: reversed.map(d => ({ c: d.cache_claimed, d: d.cache_detected })) },
    { key: 'output', label: 'Output', color: COLORS.output, d: reversed.map(d => ({ c: d.output_claimed, d: d.output_detected })) },
  ]

  return (
    <>
      <div className="px-4 py-2 border-b border-gray-100 flex items-center gap-2">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
        <span className="text-[15px] font-bold text-gray-800 tracking-tight">Token 趋势</span>
      </div>
      <div className="flex-1 flex flex-col gap-[2px] p-2">
        {series.map(s => <SubChart key={s.key} data={s.d} color={s.color} label={s.label} showXAxis={s.key === 'output'} />)}
      </div>
      {/* Legend */}
      <div className="flex items-center gap-5 px-4 pb-3 text-[11px] text-gray-500">
        <span className="flex items-center gap-1"><span className="inline-block w-[14px] h-[2px] rounded-[2px] bg-current" />审计值（基准）</span>
        <span className="flex items-center gap-1"><span className="inline-block w-[14px] h-[2px] rounded-[2px] bg-current opacity-30" />声称值</span>
        <span className="flex items-center gap-1"><span className="inline-block w-[14px] h-2 rounded-[2px] bg-current opacity-10" />差值区域</span>
      </div>
    </>
  )
}
```

- [ ] **Step 2: Write triple-ring component**

Create `frontend/src/components/TripleRing.tsx`:
```tsx
interface TripleRingProps {
  inputRate: number   // 0-100 match rate
  cacheRate: number
  outputRate: number
}

export default function TripleRing({ inputRate, cacheRate, outputRate }: TripleRingProps) {
  const rings = [
    { rate: outputRate, r: 36, color: '#34D399', stroke: 5 },
    { rate: cacheRate, r: 28, color: '#FB923C', stroke: 5 },
    { rate: inputRate, r: 20, color: '#60A5FA', stroke: 5 },
  ]

  const circ = (r: number) => 2 * Math.PI * r

  return (
    <div className="relative w-[76px] h-[76px] shrink-0">
      <svg width="76" height="76" viewBox="0 0 80 80">
        {rings.map((ring, i) => (
          <g key={i}>
            <circle cx="40" cy="40" r={ring.r} fill="none" stroke="#F3F4F6" strokeWidth={ring.stroke} opacity={0.5} />
            <circle cx="40" cy="40" r={ring.r} fill="none" stroke={ring.color} strokeWidth={ring.stroke}
              strokeDasharray={circ(ring.r)} strokeDashoffset={circ(ring.r) - (ring.rate / 100) * circ(ring.r)}
              strokeLinecap="round" transform="rotate(-90 40 40)"
              style={{ transition: 'stroke-dashoffset 0.8s ease' }}
            />
          </g>
        ))}
      </svg>
    </div>
  )
}
```

- [ ] **Step 3: Build check**

Run: `cd frontend && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: add TrendChart SVG and TripleRing SVG components"
```

---

### Task 4: Dashboard remaining sub-components

**Files:**
- Create: `frontend/src/components/TodayOverview.tsx`
- Create: `frontend/src/components/ProviderRanking.tsx`
- Create: `frontend/src/components/CurrentAudit.tsx`
- Create: `frontend/src/components/SessionTimer.tsx`
- Create: `frontend/src/components/BarChart7Day.tsx`

- [ ] **Step 1: Write TodayOverview.tsx**

```tsx
import TripleRing from './TripleRing'

interface Props {
  summary: {
    total_requests: number
    suspicious_requests: number
    input_diff_rate: number
    cache_diff_rate: number
    output_diff_rate: number
    input_diff_tokens: number
    cache_diff_tokens: number
    output_diff_tokens: number
    input_match_rate: number
    cache_match_rate: number
    output_match_rate: number
  }
}

function diffColor(rate: number) {
  if (rate < 5) return 'text-[#059669]'
  if (rate < 15) return 'text-[#D97706]'
  return 'text-[#DC2626]'
}

export default function TodayOverview({ summary }: Props) {
  const items = [
    { label: 'Input', rate: summary.input_diff_rate, diff: summary.input_diff_tokens, match: summary.input_match_rate, color: '#60A5FA' },
    { label: 'Cache', rate: summary.cache_diff_rate, diff: summary.cache_diff_tokens, match: summary.cache_match_rate, color: '#FB923C' },
    { label: 'Output', rate: summary.output_diff_rate, diff: summary.output_diff_tokens, match: summary.output_match_rate, color: '#34D399' },
  ]

  return (
    <div className="bg-white border border-gray-200 rounded-[16px] shadow-card overflow-hidden">
      <div className="px-4 py-2.5 border-b border-gray-100 flex items-center gap-1.5">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="4" width="18" height="16" rx="2"/><line x1="7" y1="9" x2="17" y2="9"/><line x1="7" y1="13" x2="13" y2="13"/></svg>
        <span className="text-[12px] font-semibold text-gray-700">今日概览</span>
      </div>
      <div className="px-4 py-3 flex flex-col gap-0">
        <div className="flex justify-between items-center py-1.5 border-b border-gray-100">
          <span className="text-[12px] text-gray-500">请求次数</span>
          <span className="text-[14px] font-bold text-gray-900 tabular-nums">{summary.total_requests.toLocaleString()}</span>
        </div>
        <div className="flex justify-between items-center py-1.5 border-b border-gray-100">
          <span className="text-[12px] text-gray-500">可疑请求</span>
          <span className={`text-[14px] font-bold tabular-nums ${summary.suspicious_requests > 0 ? 'text-[#D97706]' : 'text-gray-900'}`}>
            {summary.suspicious_requests.toLocaleString()}
          </span>
        </div>
        {/* Triple ring + diffs */}
        <div className="flex items-center gap-4 py-2.5">
          <TripleRing inputRate={summary.input_match_rate} cacheRate={summary.cache_match_rate} outputRate={summary.output_match_rate} />
          <div className="flex flex-col gap-1.5 flex-1">
            {items.map(item => (
              <div key={item.label} className="flex items-center justify-between min-w-[100px]">
                <span className="flex items-center gap-1.5 text-[11px] text-gray-500">
                  <span className="w-[6px] h-[6px] rounded-full inline-block shrink-0" style={{ background: item.color }} />
                  {item.label}
                </span>
                <span className={`text-[12px] font-mono font-semibold ${diffColor(item.rate)}`}>
                  {item.diff > 0 ? '+' : ''}{item.diff.toLocaleString()}
                  <span className="text-gray-400 text-[10px] ml-1">{item.rate.toFixed(1)}%</span>
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Write ProviderRanking.tsx**

```tsx
interface Props {
  providers: Array<{
    id: string; name: string; url: string; credibility: number
    input_diff_rate: number; cache_diff_rate: number; output_diff_rate: number
  }>
}

const medalColors = ['#F59E0B', '#9CA3AF', '#B45309']

export default function ProviderRanking({ providers }: Props) {
  return (
    <div className="bg-white border border-gray-200 rounded-[16px] shadow-card overflow-hidden">
      <div className="px-4 py-2.5 border-b border-gray-100 flex items-center gap-1.5">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 20V10"/><path d="M18 20V4"/><path d="M6 20v-4"/></svg>
        <span className="text-[12px] font-semibold text-gray-700">Provider 可信度</span>
      </div>
      <div className="px-3.5 py-2.5 flex flex-col gap-2.5">
        {providers.length === 0 && <div className="text-center text-gray-400 text-xs py-4">暂无 Provider 数据</div>}
        {providers.map((p, i) => (
          <div key={p.id} className="flex items-start gap-2.5 p-2.5 bg-gray-50 rounded-[8px]">
            <span
              className={`w-[22px] h-[22px] rounded-[6px] flex items-center justify-center text-[12px] font-bold shrink-0 mt-[1px] text-white ${
                i < 3 ? '' : 'bg-gray-100 text-gray-400'
              }`}
              style={i < 3 ? { background: `linear-gradient(135deg, ${medalColors[i]}, ${medalColors[i]}dd)` } : {}}
            >
              {i + 1}
            </span>
            <div className="flex-1 min-w-0">
              <div className="flex items-baseline justify-between gap-2 mb-0.5">
                <span className="text-[13px] font-semibold text-gray-800 truncate">{p.name}</span>
                <span className="text-[13px] font-bold text-[#059669] font-mono shrink-0">{p.credibility.toFixed(0)}%</span>
              </div>
              <div className="text-[10px] text-gray-400 mb-1 truncate">{p.url}</div>
              <div className="flex gap-2.5 text-[10px] text-gray-500">
                <span><span className="text-[#60A5FA] font-semibold">I</span> {p.input_diff_rate.toFixed(1)}%</span>
                <span><span className="text-[#FB923C] font-semibold">C</span> {p.cache_diff_rate.toFixed(1)}%</span>
                <span><span className="text-[#34D399] font-semibold">O</span> {p.output_diff_rate.toFixed(1)}%</span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
```

- [ ] **Step 3: Write CurrentAudit.tsx**

```tsx
interface Props {
  audit: null | {
    audit_passed: boolean
    input_audit: number;  input_claimed: number;  input_diff: number
    cache_audit: number;  cache_claimed: number;  cache_diff: number
    output_audit: number; output_claimed: number; output_diff: number
  }
}

function fmt(n: number) {
  if (n >= 1e6) return (n/1e6).toFixed(1)+'M'
  if (n >= 1e3) return (n/1e3).toFixed(1)+'K'
  return String(n)
}

export default function CurrentAudit({ audit }: Props) {
  if (!audit) return (
    <div className="bg-white border border-gray-200 rounded-[16px] shadow-card p-4 text-center text-gray-400 text-sm">
      暂无审计记录
    </div>
  )

  const bars = [
    { label: 'Input', audit: audit.input_audit, claimed: audit.input_claimed, diff: audit.input_diff, color: '#60A5FA' },
    { label: 'Cache', audit: audit.cache_audit, claimed: audit.cache_claimed, diff: audit.cache_diff, color: '#FB923C' },
    { label: 'Output', audit: audit.output_audit, claimed: audit.output_claimed, diff: audit.output_diff, color: '#34D399' },
  ]
  const maxVal = Math.max(...bars.map(b => Math.max(b.audit, b.claimed)), 1)

  return (
    <div className="bg-white border border-gray-200 rounded-[16px] shadow-card overflow-hidden">
      <div className="px-4 py-2.5 border-b border-gray-100 flex items-center justify-between">
        <div className="flex items-center gap-1.5">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M9 12l2 2 4-4"/><circle cx="12" cy="12" r="10"/></svg>
          <span className="text-[12px] font-semibold text-gray-700">当前请求审计</span>
        </div>
      </div>
      <div className="px-4 py-2">
        <div className="flex items-center gap-2 mb-2">
          <div className={`w-7 h-7 rounded-full flex items-center justify-center shrink-0 ${audit.audit_passed ? 'bg-[#ECFDF5]' : 'bg-[#FEF2F2]'}`}>
            {audit.audit_passed
              ? <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#10B981" strokeWidth="3"><polyline points="20 6 9 17 4 12"/></svg>
              : <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#DC2626" strokeWidth="3"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            }
          </div>
          <span className="text-[11px] text-gray-500 font-medium">{audit.audit_passed ? '审计通过' : '审计未通过'}</span>
        </div>
        {/* Bars */}
        {['audit', 'claimed'].map(type => (
          <div key={type} className="mb-1.5">
            <div className="text-[10px] text-gray-400 font-mono mb-[3px]">{type === 'audit' ? '审计 Token' : '声明 Token'}</div>
            <div className="flex h-2 rounded-[4px] overflow-hidden gap-[1px]">
              {bars.map(bar => (
                <div
                  key={bar.label}
                  className="h-full transition-all"
                  style={{
                    flex: (type === 'audit' ? bar.audit : bar.claimed),
                    background: bar.color,
                    opacity: type === 'audit' ? 1 : 0.45,
                  }}
                  title={`${bar.label}: ${fmt(type === 'audit' ? bar.audit : bar.claimed)}`}
                />
              ))}
            </div>
            <div className="flex justify-between text-[9px] text-gray-400 font-mono mt-[2px]">
              {bars.map(bar => (
                <span key={bar.label} style={{ color: bar.color, opacity: type === 'audit' ? 1 : 0.7 }}>
                  {fmt(type === 'audit' ? bar.audit : bar.claimed)}
                </span>
              ))}
            </div>
          </div>
        ))}
      </div>
      {/* Diff values */}
      <div className="px-4 py-2 border-t border-gray-100">
        {bars.map(bar => (
          <div key={bar.label} className="flex items-center justify-between py-1">
            <span className="flex items-center gap-1.5 text-[11px] text-gray-600">
              <span className="w-[6px] h-[6px] rounded-full inline-block shrink-0" style={{ background: bar.color }} />
              {bar.label}
            </span>
            <span className={`text-[12px] font-mono font-semibold ${bar.diff > 0 ? 'text-[#DC2626]' : 'text-[#059669]'}`}>
              {bar.diff > 0 ? '+' : ''}{fmt(bar.diff)}
            </span>
          </div>
        ))}
      </div>
    </div>
  )
}
```

- [ ] **Step 4: Write SessionTimer.tsx**

```tsx
import { useState, useRef, useCallback, useEffect } from 'react'

export default function SessionTimer() {
  const [running, setRunning] = useState(false)
  const [seconds, setSeconds] = useState(0)
  const intervalRef = useRef<ReturnType<typeof setInterval>>()

  const toggle = useCallback(() => {
    if (running) {
      clearInterval(intervalRef.current)
      intervalRef.current = undefined
    } else {
      intervalRef.current = setInterval(() => setSeconds(s => s + 1), 1000)
    }
    setRunning(r => !r)
  }, [running])

  useEffect(() => () => clearInterval(intervalRef.current), [])

  const h = String(Math.floor(seconds / 3600)).padStart(2, '0')
  const m = String(Math.floor((seconds % 3600) / 60)).padStart(2, '0')
  const s = String(seconds % 60).padStart(2, '0')

  return (
    <button
      onClick={toggle}
      className={`inline-flex items-center justify-center gap-2 w-11 h-11 bg-gray-100 border border-gray-200 rounded-[14px] cursor-pointer text-gray-500 transition-all duration-200 select-none active:scale-95 ${
        running ? 'w-auto px-3.5' : ''
      }`}
    >
      {running ? (
        <>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/></svg>
          <span className="text-[13px] font-semibold font-mono tracking-[0.04em] text-gray-700">{h}:{m}:{s}</span>
        </>
      ) : (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
      )}
    </button>
  )
}
```

- [ ] **Step 5: Write BarChart7Day.tsx** (simplified SVG bar chart)

```tsx
// BarChart7Day is a simplified SVG bar chart showing daily breakdown
// For v0.3, it uses mock/placeholder data since the backend daily_breakdown
// may not be available in the first iteration
export default function BarChart7Day({ data }: { data: any[] }) {
  return (
    <>
      <div className="px-4 py-2 border-b border-gray-100 flex items-center gap-1.5">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg>
        <span className="text-[12px] font-semibold text-gray-700">近 7 日 Token 统计</span>
        <span className="ml-auto text-[10px] font-semibold text-brand bg-[rgba(79,70,229,0.08)] px-2 py-0.5 rounded-[8px]">当前 Provider</span>
      </div>
      <div className="h-[160px] flex items-center justify-center text-gray-300 text-sm">
        数据加载中...
      </div>
    </>
  )
}
```

- [ ] **Step 6: Build check**

Run: `cd frontend && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 7: Commit**

```bash
git add -A && git commit -m "feat: add TodayOverview, ProviderRanking, CurrentAudit, SessionTimer, BarChart7Day"
```

---

### Task 5: Providers page

**Files:**
- Create: `frontend/src/pages/ProvidersPage.tsx`
- Create: `frontend/src/components/ProviderCard.tsx`
- Create: `frontend/src/components/ProviderForm.tsx`

- [ ] **Step 1: Write ProvidersPage.tsx**

```tsx
import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import ProviderCard from '../components/ProviderCard'
import ProviderForm from '../components/ProviderForm'
import Button from '../components/Button'

interface Provider {
  id: string; name: string; provider_type: string; api_base_url: string
  api_key: string | null; supported_models: string[]; is_active: boolean
  credibility: number | null; total_requests: number | null
  suspicious_requests: number | null
  audit_summary: { input_claimed: number; input_detected: number; cache_claimed: number; cache_detected: number; output_claimed: number; output_detected: number } | null
}

export default function ProvidersPage() {
  const [providers, setProviders] = useState<Provider[]>([])
  const [showForm, setShowForm] = useState(false)
  const [editProvider, setEditProvider] = useState<Provider | null>(null)
  const [selectedModel, setSelectedModel] = useState<string | null>(null)

  const load = () => {
    invoke<Provider[]>('list_providers').then(setProviders).catch(console.error)
  }

  useEffect(load, [])

  const switchProvider = (id: string) => {
    invoke('switch_provider', { id }).then(load).catch(console.error)
  }

  const deleteProvider = (id: string) => {
    if (!confirm('确定删除该 Provider？此操作不可撤销。')) return
    invoke('delete_provider', { id }).then(load).catch(console.error)
  }

  return (
    <div className="p-10 px-12 max-w-[1200px] animate-[page-in_250ms_ease-out]">
      <div className="flex items-start justify-between mb-8">
        <div>
          <h1 className="text-[1.75rem] font-bold text-gray-900 tracking-tight leading-[1.2]">Providers</h1>
          <p className="text-sm text-gray-400">管理 AI Provider · 点击卡片切换活跃 Provider</p>
        </div>
        <Button onClick={() => { setEditProvider(null); setShowForm(true) }}>
          添加 Provider
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-4">
        {providers.map(p => (
          <ProviderCard
            key={p.id}
            provider={p}
            selectedModel={selectedModel}
            onSelectModel={setSelectedModel}
            onSwitch={switchProvider}
            onEdit={() => { setEditProvider(p); setShowForm(true) }}
            onDelete={deleteProvider}
          />
        ))}
        {providers.length === 0 && (
          <div className="text-center py-16 text-gray-400">暂无 Provider</div>
        )}
      </div>

      <div className="text-center py-8">
        <Button variant="secondary" size="lg" onClick={() => { setEditProvider(null); setShowForm(true) }}>
          添加 Provider
        </Button>
      </div>

      {showForm && (
        <ProviderForm
          provider={editProvider}
          onClose={() => { setShowForm(false); setEditProvider(null) }}
          onSaved={load}
        />
      )}
    </div>
  )
}
```

- [ ] **Step 2: Write ProviderCard.tsx**

```tsx
import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import TrustRing from './TrustRing'
import Button from './Button'

interface Provider {
  id: string; name: string; provider_type: string; api_base_url: string
  supported_models: string[]; is_active: boolean
  credibility: number | null
  audit_summary: { input_claimed: number; input_detected: number; cache_claimed: number; cache_detected: number; output_claimed: number; output_detected: number } | null
}

interface Props {
  provider: Provider
  selectedModel: string | null
  onSelectModel: (model: string | null) => void
  onSwitch: (id: string) => void
  onEdit: () => void
  onDelete: (id: string) => void
}

type AuditData = {
  input_claimed: number; input_detected: number
  cache_claimed: number; cache_detected: number
  output_claimed: number; output_detected: number
}

function computeDiffRates(a: AuditData) {
  const iDiff = a.input_claimed > 0 ? ((a.input_claimed - a.input_detected) / a.input_claimed * 100) : 0
  const cDiff = a.cache_claimed > 0 ? ((a.cache_claimed - a.cache_detected) / a.cache_claimed * 100) : 0
  const oDiff = a.output_claimed > 0 ? ((a.output_claimed - a.output_detected) / a.output_claimed * 100) : 0
  return { iDiff, cDiff, oDiff }
}

function fmt(n: number) {
  if (n >= 1e6) return (n/1e6).toFixed(1)+'M'
  if (n >= 1e3) return (n/1e3).toFixed(0)+'K'
  return String(n)
}

function DiffPct({ rate }: { rate: number }) {
  const cls = rate <= 1 ? 'text-[#059669]' : rate <= 5 ? 'text-[#D97706]' : 'text-[#DC2626]'
  return <span className={`${cls} font-semibold font-mono tabular-nums`}>{rate.toFixed(1)}%</span>
}

function AuditBars({ data }: { data: AuditData }) {
  const { iDiff, cDiff, oDiff } = computeDiffRates(data)
  const prefillTotal = Math.max(data.input_claimed + data.cache_claimed, data.input_detected + data.cache_detected)
  const outTotal = Math.max(data.output_claimed, data.output_detected)

  const BarRow = ({ label, input, cache, output, total }: { label: string; input: number; cache: number; output: number; total: number }) => (
    <div className="flex items-center gap-2 mb-[3px]">
      <span className="text-[10px] font-semibold text-gray-900 w-7 shrink-0 text-right">{label}</span>
      <div className="flex-1 h-1 bg-gray-100 relative overflow-hidden">
        <div className="absolute inset-y-0 left-0 bg-[#60A5FA] transition-all" style={{ width: `${(input / Math.max(total, 1)) * 100}%` }} />
        <div className="absolute inset-y-0 right-0 bg-[#FB923C] transition-all" style={{ width: `${(cache / Math.max(total, 1)) * 100}%` }} />
        <div className="absolute inset-y-0 left-0 bg-[#34D399] transition-all" style={{ width: `${(output / Math.max(total, 1)) * 100}%`, display: label === '声称' && output > 0 ? 'block' : 'none' }} />
      </div>
      <span className="text-[10px] font-semibold text-gray-600 w-11 shrink-0 text-right tabular-nums">{fmt(input + cache + output)}</span>
    </div>
  )

  return (
    <div className="mt-5">
      <div className="text-[10px] font-bold uppercase tracking-wide flex items-center gap-2 mb-1.5" style={{ color: '#60A5FA' }}>Prefill</div>
      <BarRow label="声称" input={data.input_claimed} cache={data.cache_claimed} output={0} total={prefillTotal} />
      <BarRow label="审计" input={data.input_detected} cache={data.cache_detected} output={0} total={prefillTotal} />
      <div className="flex items-center gap-4 ml-7 mt-1.5 text-[10px] font-semibold tabular-nums">
        <span className="flex items-center gap-1"><span className="w-2.5 h-2.5 rounded-full bg-[#60A5FA] shrink-0" />Input <DiffPct rate={iDiff} /></span>
        <span className="flex items-center gap-1"><span className="w-2.5 h-2.5 rounded-full bg-[#FB923C] shrink-0" />Cache <DiffPct rate={cDiff} /></span>
      </div>

      <div className="text-[10px] font-bold uppercase tracking-wide flex items-center gap-2 mt-3 mb-1.5" style={{ color: '#34D399' }}>Output</div>
      <BarRow label="声称" input={0} cache={0} output={data.output_claimed} total={outTotal} />
      <BarRow label="审计" input={0} cache={0} output={data.output_detected} total={outTotal} />
      <div className="flex items-center gap-4 ml-7 mt-1.5 text-[10px] font-semibold tabular-nums">
        <span className="flex items-center gap-1"><span className="w-2.5 h-2.5 rounded-full bg-[#34D399] shrink-0" />Output <DiffPct rate={oDiff} /></span>
      </div>
    </div>
  )
}

export default function ProviderCard({ provider, selectedModel, onSelectModel, onSwitch, onEdit, onDelete }: Props) {
  const [menuOpen, setMenuOpen] = useState(false)

  return (
    <div
      className={`bg-white border rounded-[16px] p-6 shadow-card transition-all duration-150 hover:shadow-md hover:-translate-y-px relative overflow-hidden ${
        provider.is_active ? 'border-brand-light shadow-[0_0_0_3px_rgba(79,70,229,0.08)] before:absolute before:top-0 before:left-0 before:right-0 before:h-[3px] before:bg-gradient-to-r before:from-brand before:to-brand-light' : 'border-gray-200'
      }`}
    >
      {/* Card menu */}
      <div className="absolute top-3 right-3">
        <button onClick={() => setMenuOpen(!menuOpen)} className="w-7 h-7 inline-flex items-center justify-center rounded-[6px] text-gray-400 hover:bg-gray-100 hover:text-gray-600 border border-transparent hover:border-gray-200 transition-all">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="5" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="12" cy="19" r="1.5"/></svg>
        </button>
        {menuOpen && (
          <>
            <div className="fixed inset-0 z-40" onClick={() => setMenuOpen(false)} />
            <div className="absolute top-full right-0 mt-1 min-w-[160px] bg-white border border-gray-200 rounded-[10px] shadow-lg z-50 p-1 animate-[fade-in_150ms_ease-out]">
              <button onClick={() => { onEdit(); setMenuOpen(false) }} className="flex items-center gap-2 w-full px-2.5 py-1.5 rounded-[6px] text-sm font-medium text-gray-700 hover:bg-gray-100 transition-all">编辑</button>
              <button onClick={() => { onDelete(provider.id); setMenuOpen(false) }} className="flex items-center gap-2 w-full px-2.5 py-1.5 rounded-[6px] text-sm font-medium text-[#DC2626] hover:bg-[#FEF2F2] transition-all">删除 Provider</button>
            </div>
          </>
        )}
      </div>

      {/* Header */}
      <div className="flex items-start justify-between gap-4 mb-5">
        <div className="flex-1 min-w-0">
          <div className="text-xl font-bold text-gray-900 tracking-tight mb-0.5">{provider.name}</div>
          <div className="text-xs text-gray-400 font-mono mb-2 break-all">{provider.api_base_url}</div>
          <div className="flex items-center gap-2 flex-wrap">
            <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-[10px] text-[10px] font-semibold ${
              provider.provider_type === '官方' ? 'bg-[rgba(79,70,229,0.08)] text-brand' : 'bg-gray-100 text-gray-500'
            }`}>
              <span className="w-[6px] h-[6px] rounded-full bg-current" />
              {provider.provider_type === '官方' ? '官方' : '中转'}
            </span>
            {provider.is_active && (
              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-[10px] text-[10px] font-semibold bg-[#ECFDF5] text-[#059669]">
                <span className="w-[6px] h-[6px] rounded-full bg-current" />活跃
              </span>
            )}
          </div>
          <div className="mt-2 flex flex-wrap gap-1">
            {provider.supported_models.map(m => (
              <button
                key={m}
                onClick={() => onSelectModel(selectedModel === m ? null : m)}
                className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-[10px] text-[10px] font-semibold cursor-pointer border transition-all select-none whitespace-nowrap ${
                  selectedModel === m
                    ? 'bg-[rgba(79,70,229,0.06)] text-brand border-brand shadow-[0_0_0_3px_rgba(79,70,229,0.15)]'
                    : 'bg-gray-100 text-gray-600 border-transparent hover:bg-[rgba(79,70,229,0.06)] hover:text-brand hover:border-[rgba(79,70,229,0.12)]'
                }`}
              >
                {m}
              </button>
            ))}
          </div>
        </div>
        <div className="flex flex-col items-center gap-1 shrink-0">
          <div className="h-11 flex items-center justify-center">
            <TrustRing value={provider.credibility ?? 0} size="md" />
          </div>
          <div className="text-[10px] text-gray-400">可信度</div>
        </div>
      </div>

      {/* Audit bars */}
      {provider.audit_summary ? (
        <AuditBars data={provider.audit_summary} />
      ) : (
        <div className="text-center py-6 text-gray-400 text-sm">暂无审计数据</div>
      )}
    </div>
  )
}
```

- [ ] **Step 3: Write ProviderForm.tsx**

```tsx
import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import Modal from './Modal'
import Button from './Button'
import Toggle from './Toggle'

interface Provider {
  id: string; name: string; provider_type: string; api_base_url: string
  api_key: string | null; supported_models: string[]; is_active: boolean
}

interface Props {
  provider: Provider | null     // null = add mode, non-null = edit mode
  onClose: () => void
  onSaved: () => void
}

export default function ProviderForm({ provider, onClose, onSaved }: Props) {
  const isEdit = provider !== null
  const [name, setName] = useState(provider?.name ?? '')
  const [url, setUrl] = useState(provider?.api_base_url ?? '')
  const [key, setKey] = useState(provider?.api_key ?? '')
  const [models, setModels] = useState(provider?.supported_models.join(', ') ?? '')
  const [isActive, setIsActive] = useState(provider?.is_active ?? false)
  const [saving, setSaving] = useState(false)

  const handleSave = async () => {
    setSaving(true)
    try {
      const modelList = models.split(',').map(s => s.trim()).filter(Boolean)
      if (isEdit) {
        await invoke('update_provider', {
          id: provider.id,
          req: { name, api_base_url: url, api_key: key, provider_type: 'relay', supported_models: modelList },
        })
      } else {
        await invoke('create_provider', {
          req: { name, api_base_url: url, api_key: key, provider_type: url.includes('anthropic') ? 'official' : 'relay', supported_models: modelList },
        })
        if (isActive) {
          // Created provider needs to be set active separately
          const created = await invoke<Provider[]>('list_providers')
          if (created.length > 0) await invoke('switch_provider', { id: created[0].id })
        }
      }
      onSaved()
      onClose()
    } catch (e) {
      console.error(e)
    } finally {
      setSaving(false)
    }
  }

  return (
    <Modal
      open={true}
      onClose={onClose}
      title={isEdit ? '编辑 Provider' : '添加 Provider'}
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>取消</Button>
          <Button onClick={handleSave} disabled={saving}>{saving ? '保存中...' : '保存'}</Button>
        </>
      }
    >
      <div className="space-y-4">
        <div className="flex flex-col gap-1.5">
          <label className="text-sm font-semibold text-gray-700">Provider 名称</label>
          <input className="px-2.5 py-2 border border-gray-300 rounded-[6px] text-sm bg-white text-gray-900 outline-none transition-all hover:border-gray-400 focus:border-brand focus:shadow-[0_0_0_3px_rgba(79,70,229,0.15)]" value={name} onChange={e => setName(e.target.value)} placeholder="可选，默认从 URL 提取" />
        </div>
        <div className="flex flex-col gap-1.5">
          <label className="text-sm font-semibold text-gray-700">API Endpoint</label>
          <input className="px-2.5 py-2 border border-gray-300 rounded-[6px] text-sm bg-white text-gray-900 outline-none transition-all hover:border-gray-400 focus:border-brand focus:shadow-[0_0_0_3px_rgba(79,70,229,0.15)]" value={url} onChange={e => setUrl(e.target.value)} placeholder="https://api.openai.com/v1/chat/completions" />
        </div>
        <div className="flex flex-col gap-1.5">
          <label className="text-sm font-semibold text-gray-700">API Key</label>
          <input type="password" className="px-2.5 py-2 border border-gray-300 rounded-[6px] text-sm bg-white text-gray-900 outline-none transition-all hover:border-gray-400 focus:border-brand focus:shadow-[0_0_0_3px_rgba(79,70,229,0.15)]" value={key} onChange={e => setKey(e.target.value)} placeholder="sk-..." />
        </div>
        <div className="flex flex-col gap-1.5">
          <label className="text-sm font-semibold text-gray-700">模型列表</label>
          <input className="px-2.5 py-2 border border-gray-300 rounded-[6px] text-sm bg-white text-gray-900 outline-none transition-all hover:border-gray-400 focus:border-brand focus:shadow-[0_0_0_3px_rgba(79,70,229,0.15)]" value={models} onChange={e => setModels(e.target.value)} placeholder="gpt-4o, deepseek-chat（逗号分隔）" />
          <span className="text-xs text-gray-400">多个模型用逗号分隔</span>
        </div>
        {!isEdit && (
          <div className="flex items-center justify-between pt-2">
            <div>
              <div className="text-sm font-semibold text-gray-800">设为活跃 Provider</div>
              <div className="text-xs text-gray-400">创建后立即切换为此 Provider</div>
            </div>
            <Toggle checked={isActive} onChange={setIsActive} />
          </div>
        )}
      </div>
    </Modal>
  )
}
```

- [ ] **Step 4: Build check**

Run: `cd frontend && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: add Providers page with card grid, audit bars, and modal form"
```

---

### Task 6: Settings page

**Files:**
- Create: `frontend/src/pages/SettingsPage.tsx`

- [ ] **Step 1: Write SettingsPage.tsx**

```tsx
import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import Button from '../components/Button'
import Toggle from '../components/Toggle'

interface AppConfig {
  proxy_port: number
  language: string
  auto_configure_claude: boolean
  auto_start_proxy: boolean
  suspicion_threshold: number
  record_full_body: boolean
  dev_mode_enabled: boolean
  dev_trace_buffer_size: number
}

export default function SettingsPage() {
  const [config, setConfig] = useState<AppConfig | null>(null)
  const [devMode, setDevMode] = useState(false)
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    invoke<AppConfig>('get_app_config').then(c => {
      setConfig(c)
      setDevMode(c.dev_mode_enabled)
    }).catch(console.error)
  }, [])

  const update = (partial: Partial<AppConfig>) => {
    if (config) setConfig({ ...config, ...partial })
  }

  const save = async () => {
    if (!config) return
    await invoke('save_app_config', { config: { ...config, dev_mode_enabled: devMode } })
    setSaved(true)
    setTimeout(() => setSaved(false), 2000)
  }

  if (!config) return <div className="p-10 px-12 animate-[page-in_250ms_ease-out]"><div className="h-12 w-48 skeleton rounded-[6px]" /></div>

  return (
    <div className="p-10 px-12 max-w-[1200px] animate-[page-in_250ms_ease-out]">
      <div className="mb-8">
        <h1 className="text-[1.75rem] font-bold text-gray-900 tracking-tight leading-[1.2]">Settings</h1>
        <p className="text-sm text-gray-400">配置应用偏好与开发者选项</p>
      </div>

      {/* Basic */}
      <SettingsSection title="基础设置" desc="代理端口与语言偏好">
        <SettingsRow label="代理端口" desc="TokenAccountant 代理监听的本地端口号">
          <input type="number" className="px-2.5 py-2 border border-gray-300 rounded-[6px] text-sm bg-white text-gray-900 outline-none w-[100px]" value={config.proxy_port} onChange={e => update({ proxy_port: parseInt(e.target.value) || 8080 })} />
        </SettingsRow>
        <SettingsRow label="界面语言" desc="应用界面的显示语言">
          <select className="px-2.5 py-2 border border-gray-300 rounded-[6px] text-sm bg-white text-gray-900 outline-none w-[140px]" value={config.language} onChange={e => update({ language: e.target.value })}>
            <option>简体中文</option>
            <option>English</option>
          </select>
        </SettingsRow>
      </SettingsSection>

      {/* Proxy */}
      <SettingsSection title="代理设置" desc="自动配置与代理行为">
        <SettingsRow label="自动配置 Claude" desc="启动时自动修改 Claude 代理配置">
          <Toggle checked={config.auto_configure_claude} onChange={v => update({ auto_configure_claude: v })} />
        </SettingsRow>
        <SettingsRow label="启动时自动运行代理" desc="应用启动后自动开始代理服务">
          <Toggle checked={config.auto_start_proxy} onChange={v => update({ auto_start_proxy: v })} />
        </SettingsRow>
      </SettingsSection>

      {/* Audit */}
      <SettingsSection title="审计设置" desc="可疑检测阈值与记录策略">
        <SettingsRow label="可疑阈值" desc="差异率超过此值标记为可疑">
          <select className="px-2.5 py-2 border border-gray-300 rounded-[6px] text-sm bg-white text-gray-900 outline-none w-[120px]" value={config.suspicion_threshold} onChange={e => update({ suspicion_threshold: parseFloat(e.target.value) })}>
            <option value={0.03}>3%</option>
            <option value={0.05}>5%</option>
            <option value={0.10}>10%</option>
          </select>
        </SettingsRow>
        <SettingsRow label="记录完整请求/响应" desc="保存完整 Request/Response Body 用于调试">
          <Toggle checked={config.record_full_body} onChange={v => update({ record_full_body: v })} />
        </SettingsRow>
      </SettingsSection>

      {/* Dev Mode */}
      <div className="bg-white border border-gray-200 rounded-[16px] shadow-card mb-4">
        <div className="px-6 py-5 border-b border-gray-100">
          <div className="text-base font-semibold text-gray-900">开发者模式</div>
          <div className="text-xs text-gray-400 mt-0.5">开启后将显示高级调试工具</div>
        </div>
        <div className="px-6 py-5">
          <SettingsRow label="启用开发者模式" desc="显示 Render Inspector、Tokenizer 调试等工具">
            <Toggle checked={devMode} onChange={setDevMode} />
          </SettingsRow>
        </div>
        {devMode && (
          <div className="px-6 py-5 border-t border-[#FDE68A] bg-[#FFFBEB] rounded-b-[16px]">
            <div className="flex items-start gap-3 p-3 bg-[rgba(217,119,6,0.1)] rounded-[6px] mb-4 text-xs text-[#D97706] font-medium">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="shrink-0 mt-px"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
              以下功能仅限开发者使用。修改这些设置可能导致审计数据异常。
            </div>
            <SettingsRow label="Render Inspector" desc="实时查看 API 渲染中间状态">
              <Toggle checked={false} onChange={() => {}} />
            </SettingsRow>
            <SettingsRow label="Tokenizer 调试" desc="查看分词结果与编码详情">
              <Toggle checked={false} onChange={() => {}} />
            </SettingsRow>
            <SettingsRow label="DevTrace 缓冲区大小" desc="调试追踪缓冲区最大条目数">
              <input type="number" className="px-2.5 py-2 border border-gray-300 rounded-[6px] text-sm bg-white text-gray-900 outline-none w-[100px]" value={config.dev_trace_buffer_size} onChange={e => update({ dev_trace_buffer_size: parseInt(e.target.value) || 100 })} />
            </SettingsRow>
          </div>
        )}
      </div>

      {/* About */}
      <div className="bg-white border border-gray-200 rounded-[16px] shadow-card mb-4">
        <div className="px-6 py-5 border-b border-gray-100">
          <div className="text-base font-semibold text-gray-900">关于</div>
        </div>
        <div className="px-6 py-5">
          <div className="flex items-center gap-6">
            <div className="text-sm text-gray-700">
              <div className="mb-1"><strong>TokenAccountant</strong> v0.3.0</div>
              <div className="text-xs text-gray-400">Token 透明化审计工具 · 让每一笔 AI 消费都清晰可见</div>
            </div>
            <div className="ml-auto flex gap-2">
              <Button variant="secondary" size="sm" onClick={() => window.open('https://github.com')}>GitHub</Button>
              <Button variant="ghost" size="sm">Changelog</Button>
            </div>
          </div>
        </div>
      </div>

      <div className="py-4">
        <Button onClick={save}>{saved ? '已保存 ✓' : '保存设置'}</Button>
      </div>
    </div>
  )
}

// Helper components

function SettingsSection({ title, desc, children }: { title: string; desc: string; children: React.ReactNode }) {
  return (
    <div className="bg-white border border-gray-200 rounded-[16px] shadow-card mb-4">
      <div className="px-6 py-5 border-b border-gray-100">
        <div className="text-base font-semibold text-gray-900">{title}</div>
        <div className="text-xs text-gray-400 mt-0.5">{desc}</div>
      </div>
      <div className="px-6 py-5">{children}</div>
    </div>
  )
}

function SettingsRow({ label, desc, children }: { label: string; desc: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between py-3 gap-6 [&+&]:border-t [&+&]:border-gray-100">
      <div className="flex-1">
        <div className="text-sm font-semibold text-gray-800 mb-0.5">{label}</div>
        <div className="text-xs text-gray-400">{desc}</div>
      </div>
      <div className="shrink-0 flex items-center gap-3">{children}</div>
    </div>
  )
}
```

- [ ] **Step 2: Build check**

Run: `cd frontend && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: add Settings page with 4 sections and dev mode toggle"
```

---

### Task 7: Cleanup — remove old pages

**Files:**
- Delete: `frontend/src/pages/Dashboard.tsx`
- Delete: `frontend/src/pages/Providers.tsx`
- Delete: `frontend/src/pages/RequestList.tsx`
- Delete: `frontend/src/components/DevPanel.tsx`

- [ ] **Step 1: Remove old page files**

```bash
rm frontend/src/pages/Dashboard.tsx frontend/src/pages/Providers.tsx frontend/src/pages/RequestList.tsx frontend/src/components/DevPanel.tsx
```

- [ ] **Step 2: Update main.tsx to ensure clean imports**

Verify `frontend/src/main.tsx` no longer references removed files. The current `main.tsx` should only import `App` and render it.

- [ ] **Step 3: Full build check**

Run: `cd frontend && npx tsc --noEmit && npx vite build 2>&1 | tail -5`
Expected: Build succeeds without errors.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "chore: remove old MVP pages (Dashboard, Providers, RequestList, DevPanel)"
```

---

### Task 8: Design polish pass — font, motion, texture, trust glow

**Files:**
- Modify: `frontend/src/main.tsx`
- Modify: `frontend/src/components/Sidebar.tsx`
- Modify: `frontend/src/components/TrustRing.tsx`
- Modify: `frontend/src/pages/DashboardPage.tsx`
- Modify: `frontend/src/styles/design-tokens.css`

This task applies the `frontend-design` skill feedback:
- Typography: distinctive brand font → `Clash Display` via `@fontsource`
- Motion: staggered card reveal instead of uniform fade-in
- Texture: subtle noise grain on card backgrounds
- Trust ring: glow effect to make credibility scores visually pop

- [ ] **Step 1: Install distinctive font**

Run: `cd frontend && npm install @fontsource/clash-display`

This installs Clash Display (a distinctive geometric sans-serif) for use in brand and page titles.

- [ ] **Step 2: Import font in main.tsx**

In `frontend/src/main.tsx`, add before the App import:
```tsx
import '@fontsource/clash-display/700.css'  // Bold weight for headings
import '@fontsource/clash-display/600.css'  // Semibold for subheadings
```

- [ ] **Step 3: Apply brand font to Sidebar brand and page titles**

In `frontend/src/components/Sidebar.tsx`, change the brand name to use Clash Display:
```tsx
// Replace the sidebar-brand-text class
className="text-[15px] font-bold text-gray-900 tracking-tight leading-[1.2]"
// → add Clash Display:
className="text-[15px] font-bold text-gray-900 tracking-tight leading-[1.2] font-['Clash_Display']"
```

Also add `font-['Clash_Display']` to the page title in `DashboardPage.tsx` and `ProvidersPage.tsx`:
```tsx
// Before
<h1 className="text-[1.75rem] font-bold text-gray-900 tracking-tight leading-[1.2]">仪表盘</h1>
// After
<h1 className="text-[1.75rem] font-bold text-gray-900 tracking-tight leading-[1.2] font-['Clash_Display']">仪表盘</h1>
```

- [ ] **Step 4: Add staggered reveal animation to Dashboard cards**

In `frontend/src/pages/DashboardPage.tsx`, update the grid to add staggered delays:

```tsx
// Replace the grid div with:
<div className="grid grid-cols-[1fr_300px] gap-4">
  {/* Left top — delay 0ms */}
  <div
    className="bg-white border border-gray-200 rounded-[16px] shadow-card overflow-hidden col-start-1 row-start-1 flex flex-col opacity-0 animate-[fade-slide-up_400ms_ease-out_forwards]"
    style={{ animationDelay: '0ms' }}
  >
    <TrendChart data={data.trend_data} />
  </div>

  {/* Right split — delay 100ms */}
  <div className="col-start-2 row-start-1 flex flex-col gap-4 opacity-0 animate-[fade-slide-up_400ms_ease-out_forwards]" style={{ animationDelay: '100ms' }}>
    <TodayOverview summary={data.today_summary} />
    <ProviderRanking providers={data.provider_ranking} />
  </div>

  {/* Left bottom — delay 200ms */}
  <div className="bg-white border border-gray-200 rounded-[16px] shadow-card overflow-hidden col-start-1 row-start-2 opacity-0 animate-[fade-slide-up_400ms_ease-out_forwards]" style={{ animationDelay: '200ms' }}>
    <BarChart7Day data={[]} />
  </div>

  {/* Right bottom — delay 250ms */}
  <div className="col-start-2 row-start-2 opacity-0 animate-[fade-slide-up_400ms_ease-out_forwards]" style={{ animationDelay: '250ms' }}>
    <CurrentAudit audit={data.current_audit} />
  </div>

  {/* Session — delay 300ms */}
  <div className="col-start-2 row-start-3 flex justify-end opacity-0 animate-[fade-slide-up_400ms_ease-out_forwards]" style={{ animationDelay: '300ms' }}>
    <SessionTimer />
  </div>
</div>
```

Add the `fade-slide-up` keyframe in `global.css` or inline in `design-tokens.css`:
```css
@keyframes fade-slide-up {
  from { opacity: 0; transform: translateY(12px); }
  to   { opacity: 1; transform: translateY(0); }
}
```

- [ ] **Step 5: Add subtle noise texture to card backgrounds**

In `frontend/src/styles/design-tokens.css`, add a noise SVG as a CSS background utility:

```css
/* Subtle noise texture for depth — applied to cards */
.bg-noise {
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.015'/%3E%3C/svg%3E");
  background-repeat: repeat;
  background-size: 128px 128px;
}
```

Then add `bg-noise` class to card wrappers in:
- `DashboardPage.tsx` — the grid card wrappers (add `bg-noise` to each card's className)
- `ProvidersPage.tsx` — the provider card container (add `bg-noise` to the card div)
- `SettingsPage.tsx` — the section containers

Example for one card div:
```tsx
<div className="bg-white bg-noise border border-gray-200 rounded-[16px] shadow-card ...">
```

- [ ] **Step 6: Add glow effect to TrustRing on hover**

In `frontend/src/components/TrustRing.tsx`, wrap the SVG in a container with hover glow:

```tsx
export default function TrustRing({ value, size = 'md' }: TrustRingProps) {
  // ... existing code stays the same ...

  return (
    <div className={`relative shrink-0 group ${size === 'sm' ? 'w-[34px] h-[34px]' : 'w-[56px] h-[56px]'}`}>
      <svg viewBox={`0 0 ${w} ${w}`} className="w-full h-full -rotate-90 drop-shadow-[0_0_6px_rgba(0,0,0,0.06)] group-hover:drop-shadow-[0_0_12px_var(--glow-color)] transition-all duration-300">
        {/* ... rest stays the same ... */}
      </svg>
      <span className={`absolute inset-0 flex items-center justify-center text-[${size === 'sm' ? '11px' : '16px'}] font-extrabold leading-none tracking-tight ${cls} group-hover:scale-110 transition-transform duration-200`}>
        {value}
      </span>
    </div>
  )
}
```

Also set the glow color variable dynamically based on value (add inside the component before the JSX):
```tsx
const glowColor = value >= 90 ? 'rgba(5,150,105,0.35)' : value >= 70 ? 'rgba(217,119,6,0.35)' : 'rgba(220,38,38,0.35)'
```

- [ ] **Step 7: Build check**

Run: `cd frontend && npx tsc --noEmit && npx vite build 2>&1 | tail -5`
Expected: Build succeeds (font may need `"skipLibCheck": true` or vite config to handle).

- [ ] **Step 8: Commit**

```bash
git add -A && git commit -m "feat: design polish - Clash Display font, staggered card reveal, noise texture, trust ring glow"
```