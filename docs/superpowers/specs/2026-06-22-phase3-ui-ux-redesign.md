# Phase 3: v0.3 UI/UX 重构

> **状态**: 实施中
> **设计日期**: 2026-06-22
> **版本**: v0.3
> **PRD 参考**: `docs/product/prd_v0.3_ui_ux_redesign.md`
> **实施计划**: `docs/superpowers/plans/2026-06-13-phase3a-foundation.md`, `docs/superpowers/plans/2026-06-13-phase3b-pages.md`

---

## Goal

将当前 MVP 风格的前端（顶部Tab导航、原始审计列表、无实时数据）重构为产品级 UI：设计系统、侧边栏导航、Dashboard 实时图表、供应商管理卡片、设置页面。

---

## 总体架构

```
┌──────────────────────────────────────────────────────────┐
│  Tauri v2 Desktop App                                    │
│                                                          │
│  ┌─────────────────┐    Tauri IPC (invoke + event)       │
│  │  React Frontend  │ ◄───────────────────────────────   │
│  │  (vite build)    │     commands: get_dashboard_data    │
│  │                  │     events:   audit-tick           │
│  │  ┌───────────┐   │                                    │
│  │  │ Sidebar   │   │                                    │
│  │  │ NavLink   │   │                                    │
│  │  ├───────────┤   │                                    │
│  │  │ Dashboard │   │                                    │
│  │  │ Providers │   │                                    │
│  │  │ Settings  │   │                                    │
│  │  │ Dev       │   │                                    │
│  │  └───────────┘   │                                    │
│  └────────┬─────────┘                                    │
│           │                                               │
│  ┌────────▼─────────────────────────────────────────┐    │
│  │  Rust 后端                                        │    │
│  │                                                   │    │
│  │  ┌──────────────────────┐  ┌──────────────────┐   │    │
│  │  │ Tauri Commands       │  │ Event Emitter     │   │    │
│  │  │ get_dashboard_data   │  │ audit-tick        │   │    │
│  │  │ list_providers (agg) │  │ (写SQLite后触发)   │   │    │
│  │  │ get_provider_detail  │  └──────────────────┘   │    │
│  │  │ get/save_app_config  │                          │    │
│  │  └──────────┬───────────┘                          │    │
│  │             │                                      │    │
│  │  ┌──────────▼───────────┐                          │    │
│  │  │ SQLite (rusqlite)    │                          │    │
│  │  │ audit_log (无schema变更)│                        │    │
│  │  │ SQL 聚合查询         │                          │    │
│  │  └──────────────────────┘                          │    │
│  └────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────┘
```

---

## 设计决策

### 1. 数据刷新模式：Event 驱动 + SQLite 回查

**决策**：后端写 SQLite 后 `emit("audit-tick")` 信号，前端收到后**重新调用 IPC 查 SQLite**，不使用 event 携带业务数据。

```
审计引擎写完 audit_log
    │
    ├─ emit("audit-tick")          ← 仅通知，不携带数据
    ▼
前端收到 event
    │
    ├─ invoke("get_dashboard_data") ← 重新查 SQLite
    ├─ shift/push 趋势队列（20条）
    ├─ 更新今日概览
    └─ 更新当前审计卡片
```

**理由：**
- SQLite 是唯一事实来源，避免 event 数据和 SQLite 不一致
- event 仅作为"有新数据"的信号，结构轻量，无版本兼容问题
- 前端可以按需获取自己想要的数据，不依赖 event payload 结构

**代价：** 每次 event 触发一次完整 SQL 聚合。SQLite 是单机数据库，聚合查询在万级数据量下仍然在毫秒级，可接受。

**延伸：Rust 的 event 机制**：当前使用 Tauri 的 `app_handle.emit()`，在 proxy handler 中持有 `AppHandle` 引用。Phase 2 已通过 `ProxyState` 注入 `AppHandle`，Phase 3 不做改动。

### 2. 单 BFF 命令 vs 多独立命令

**决策**：Dashboard 所有数据通过一个 `get_dashboard_data` 命令返回，而不是每个小组件独立调用。

```rust
// 一个命令返回所有 Dashboard 数据
#[tauri::command]
async fn get_dashboard_data(state: ...) -> Result<DashboardData> {
    // 1 次事务中完成所有聚合
    DashboardData {
        today_summary:       ...  // 今日汇总 + 差异
        model_breakdown:     ...  // 今日各模型 I/C/O 汇总（饼图用）
        latest_trend_point:  ...  // 最新 1 条趋势点（前端自己 shift/push 入 20 队列）
        daily_breakdown:     ...  // 近 7 天每日 I/C/O 审计/声称汇总
        provider_ranking:    ...  // 按可信度排名的 Provider 列表
        current_audit:       ...  // 最新一条审计记录
    }
}
```

**注意**：趋势图数据不是后端返回 20 条，而是**后端只返回最新 1 条，前端维护 20 长度的队列**。详见下文"前端趋势队列"。

**理由：**
- 避免 N+1 IPC 调用（趋势图 + 今日概览 + 近 7 日 + 排名 + 当前审计 = 5 次调用变 1 次）
- SQL 聚合在单次事务中完成，保证数据一致性
- 前端状态管理简单：一个 `useDashboardData` hook 搞定
- 每次 event 触发一次批量查询，比多次查询负载更低

**代价：**
- 返回数据量略大，但 Tauri IPC 是进程内通信，开销可忽略
- 前端某个小组件想单独刷新时无法做到（当前场景没有这个需求）

### 3. 前端趋势队列

**决策**：前端维护 20 条固定长度的趋势数据队列。

```typescript
// 初始化全 0
const [queue, setQueue] = useState<TrendPoint[]>(Array(20).fill(zeroPoint))

// 收到 event 后：
const latest = await invoke("get_dashboard_data").then(d => d.trend_data[0])
setQueue(prev => [...prev.slice(1), latest])
```

**理由：**
- 趋势图需要展示连续变化，但 SQLite 只存最新一条就够了（不需要每次查 20 条历史）
- 队列避免了：重复查询历史数据 + 前端维护时间窗口状态
- 20 条对应大约 20 次请求，桌面端用户的典型使用场景

**边界情况：**
| 场景 | 行为 |
|------|------|
| app 启动 | 队列 20 个 0，图表显示直线 |
| 收到第一条 event | 最新数据 push 入队，图表出现曲线 |
| 不足 20 条数据 | 前端仍满 20 个 0 + N 个实际数据点，图表从 0 开始爬升 |
| 页面切换 | 队列保留（memory），不重置 |
| 停止代理 | 队列保留，不重置 |
| 重启 app | 队列自然清空，重新 20 个 0 |

### 4. Schema 不变

**决策**：不新增任何 SQLite 表或列。

**理由：**
- 所有 Dashboard 数据都可以通过 `SUM / GROUP BY / ORDER BY` 从 `audit_log` 聚合
- 新增 schema 需要 migration、兼容旧数据库、增加复杂度
- 聚合查询在本地 SQLite 上足够快

### 5. 代理启停与计时器整合在 Sidebar

**决策**：Sidebar 底部按钮是代理启动/停止的唯一入口，计时器整合在该按钮内。

```
代理已停止 → Sidebar 显示 "▶ 启动代理"
代理运行中 → Sidebar 显示 "⏸ 代理运行中 HH:MM:SS" → 秒级计时
```

**理由：**
- 避免在 Dashboard 额外放一个独立计时器按钮造成 UI 冗余
- PRD 明确要求"按钮和导航页的代理启动按钮是绑定的"
- 一个入口更清晰，状态只有 Sidebar 管理

### 6. Provider 切换自动停代理

**决策**：用户点击切换 Provider 时，如果代理正在运行，先自动停代理，再切换。

**理由：**
- 代理转发的是当前活跃 Provider 的请求
- 切换 Provider 后如果代理还在运行，新请求会转发到旧的 Provider
- 防止用户忘记停代理就直接切换造成混乱

### 7. 开发者模式作为条件路由

**决策**：Settings 页面的开发者模式开关控制导航栏是否显示 Dev 页面。

```typescript
// Sidebar 中：
{devMode && <NavLink to="/dev" />}
```

**理由：**
- 开发者面板不是普通用户需要的功能
- 条件渲染比隐藏页面更简单（不需要权限系统）
- Render Inspector 的精确文本已经在 Phase 2 实现了，Phase 3 只需要做界面接入

---

## 数据流

### 2.1 Event: `audit-tick`

| 字段 | 类型 | 说明 |
|------|------|------|
| 不携带业务数据 | - | 仅作为"有新审计记录"的信号 |

**触发时机**：每条审计记录写入 SQLite 后，由 proxy handler 调用 `app_handle.emit("audit-tick", ())`。

**前端处理**：
1. 收到 event → 调 `get_dashboard_data`
2. 用返回的 `latest_trend_point`（如果有值）shift/push 入 20 队列
3. 用返回的 `today_summary` 更新今日概览
4. 用返回的 `current_audit` 更新当前审计卡片
5. 用返回的 `provider_ranking` 更新排名

**注意**：Provider 页面不在 event 后刷新，只在用户进入页面 / 手动刷新时更新。

`daily_breakdown` 包含在 `get_dashboard_data` 中，event 后也会刷新——虽然数据变化慢（按天聚合），但 SQLite 查询开销极低，无必要单独拆一个命令。

### 2.2 Dashboard 完整刷新流程

```
用户打开 app
  │
  ├─ invoke("get_dashboard_data")
  │   └─ SQLite 聚合：今日汇总 + 最新趋势点 + 近 7 天 + 排名 + 最新审计
  │
  ├─ 趋势队列初始化为 20 个 0
  ├─ 今日概览 → 渲染数字 + 饼图
  ├─ 近 7 日柱状图 → 渲染每日柱子
  ├─ 当前审计 → 渲染最近一条
  └─ 排名 → 渲染 Provider 卡片列表

收到 audit-tick
  │
  ├─ invoke("get_dashboard_data")
  ├─ 趋势队列 shift/push（用 latest_trend_point）
  ├─ 今日概览重新渲染
  ├─ 近 7 日柱状图重新渲染（数据也包含在返回值中）
  ├─ 当前审计重新渲染
  └─ 排名重新渲染
```

---

## 接口契约

### 3.1 新增 Tauri Commands

#### `get_dashboard_data`

```rust
// 输入：无
// 返回：
struct DashboardData {
    proxy_status: ProxyStatus,              // { running, port, requests_served }
    today_summary: TodaySummary,            // 今日汇总（含差异）
    model_breakdown: Vec<ModelBreakdown>,   // 今日各模型的 I/C/O 审计/声称汇总（饼图用）
    latest_trend_point: Option<TrendPoint>, // 最新 1 条趋势点（前端 push 入 20 队列）
    daily_breakdown: Vec<DailyBreakdown>,   // 近 7 天每日汇总
    provider_ranking: Vec<ProviderRank>,    // 按可信度排序的前 N 个 Provider
    current_audit: Option<CurrentAudit>,    // 最新一条审计（或 null）
}

struct DailyBreakdown {
    date: String,                           // "YYYY-MM-DD"
    input_claimed: i64,  input_detected: i64,
    cache_claimed: i64,  cache_detected: i64,
    output_claimed: i64, output_detected: i64,
}

struct TodaySummary {
    total_requests: i64,                    // 今日总请求数
    suspicious_requests: i64,               // 今日可疑请求数
    input_diff_rate: f64,                   // Input 差异率 %
    cache_diff_rate: f64,                   // Cache 差异率 %
    output_diff_rate: f64,                  // Output 差异率 %
    input_diff_tokens: i64,                 // Input 差异 token 数
    cache_diff_tokens: i64,                // Cache 差异 token 数
    output_diff_tokens: i64,               // Output 差异 token 数
    input_match_rate: f64,                 // Input 匹配率 % (100 - diff_rate)
    cache_match_rate: f64,                 // Cache 匹配率 %
    output_match_rate: f64,                // Output 匹配率 %
}

struct ModelBreakdown {
    model: String,
    input_total: i64,                       // 该模型今日 Input 审计 SUM（所有请求）
    cache_total: i64,                       // 该模型今日 Cache 审计 SUM
    output_total: i64,                      // 该模型今日 Output 审计 SUM
    input_claimed_total: i64,               // 该模型今日 Input 声称 SUM
    cache_claimed_total: i64,
    output_claimed_total: i64,
}

struct TrendPoint {
    idx: i64,                               // audit_log.id
    input_claimed: i64,  input_detected: i64,
    cache_claimed: i64,  cache_detected: i64,
    output_claimed: i64, output_detected: i64,
}

struct ProviderRank {
    id: String,
    name: String,
    url: String,                            // api_base_url
    credibility: f64,                       // 0-100
    input_diff_rate: f64, cache_diff_rate: f64, output_diff_rate: f64,
}

struct CurrentAudit {
    audit_passed: bool,
    input_audit: i64,  input_claimed: i64,  input_diff: i64,
    cache_audit: i64,  cache_claimed: i64,  cache_diff: i64,
    output_audit: i64, output_claimed: i64, output_diff: i64,
}
```

**SQL 聚合逻辑**：

- **今日汇总**：`SELECT SUM(...), COUNT(...) FROM audit_log WHERE timestamp >= date('now')`
- **趋势数据**：`SELECT id, claimed/real I/C/O FROM audit_log ORDER BY id DESC LIMIT 1`
- **今日模型分布**：`SELECT model, SUM(real_input_tokens), SUM(real_output_tokens), SUM(detected_cached_tokens), SUM(claimed_input_tokens), SUM(claimed_output_tokens), SUM(claimed_cached_tokens) FROM audit_log WHERE timestamp >= date('now') GROUP BY model`
- **近 7 天统计**：`SELECT date(timestamp), SUM(...) FROM audit_log GROUP BY date(timestamp) ORDER BY date DESC LIMIT 7`
- **Provider 排名**：`list_providers_with_stats()` 聚合每个 Provider 的 avg_diff → 算 credibility
- **最新审计**：`SELECT ... FROM audit_log ORDER BY id DESC LIMIT 1`

#### `get_provider_detail`

```rust
// 输入：
provider_id: String,
model: String,

// 返回：
AuditSummary  // 该 Provider+Model 的 I/C/O 审计SUM/声称SUM（点模型标签时过滤用）
```

#### `get_app_config`

```rust
// 输入：无
// 返回：
struct AppConfigPayload {
    proxy_port: u16,
    language: String,                       // "zh-CN" | "en"
    auto_configure_claude: bool,
    auto_start_proxy: bool,
    suspicion_threshold: f64,
    record_full_body: bool,
    dev_mode_enabled: bool,
    dev_trace_buffer_size: u32,
}
```

#### `save_app_config`

```rust
// 输入：
config: AppConfigPayload

// 返回：无
```

### 3.2 修改的 Tauri Commands

#### `list_providers`（改为返回聚合数据）

```rust
// 原：返回 providers 表原始数据
// 改：返回 providers + 审计聚合数据

struct Provider {
    // ... 原有字段 ...

    // 新增聚合字段（Option，前端可以判断是否有审计数据）
    credibility: Option<f64>,
    total_requests: Option<i64>,
    suspicious_requests: Option<i64>,
    audit_summary: Option<AuditSummary>,
}

struct AuditSummary {
    input_claimed: i64,  input_detected: i64,
    cache_claimed: i64,  cache_detected: i64,
    output_claimed: i64, output_detected: i64,
}
```

### 3.3 Event

| Event 名 | 方向 | 触发时机 | 携带数据 |
|----------|------|----------|----------|
| `audit-tick` | 后端 → 前端 | 每条审计记录写入 SQLite 后 | `()`（无业务数据） |

---

## 组件职责

### 前端组件树

```
App (BrowserRouter + Routes)
├── Sidebar
│   ├── 品牌区 (Logo + "TokenAccountant v0.3")
│   ├── NavLinks (仪表盘 / Providers / Settings / [Dev])
│   └── 代理按钮 (启动/停止 + 计时器)
│
├── DashboardPage
│   ├── RefreshButton
│   ├── TrendChart
│   │   └── SubChart × 3 (Input / Cache / Output)
│   ├── BarChart7Day
│   ├── TodayOverview
│   │   └── ModelPieChart（模型分布饼图，径向双层扇区）
│   ├── ProviderRanking
│   │   └── ProviderRankItem × N
│   └── CurrentAudit
│
├── ProvidersPage
│   ├── ProviderCard × N
│   │   ├── TrustRing
│   │   ├── AuditBars (Prefill + Output 条形对比)
│   │   └── 模型标签按钮
│   └── ProviderForm (Modal)
│       └── Toggle
│
├── SettingsPage
│   └── SettingsSection × 4 (基础/代理/审计/开发者模式)
│
└── DevPage (条件渲染，开发者模式开启后出现)
```

### 组件职责划分

| 组件 | 数据来源 | 状态管理 | 子组件 |
|------|----------|----------|--------|
| `Sidebar` | `get_proxy_status`（启动时一次性读取）+ `await start/stop` 结果直接设置 | `proxyStatus` 本地state | - |
| `DashboardPage` | `useDashboardData` hook | `data/loading/error` 三个 state | 全部 Dashboard 子组件 |
| `TrendChart` | `props.trend_data` | 无（纯渲染） | `SubChart` |
| `BarChart7Day` | `props.data` | 无（纯渲染） | - |
| `TodayOverview` | `props.summary` + `props.modelBreakdown` | 无（纯渲染） | `ModelPieChart` |
| `ProviderRanking` | `props.providers` | 无（纯渲染） | - |
| `CurrentAudit` | `props.audit` | 无（纯渲染） | - |
| `ProvidersPage` | `invoke("list_providers")` | `providers/showForm/editProvider` | `ProviderCard` / `ProviderForm` |
| `ProviderCard` | `props` | `menuOpen` 本地 state | `TrustRing` / `AuditBars` |
| `ProviderForm` | `props.provider` (编辑时) | 表单字段 state | `Modal` / `Toggle` |
| `SettingsPage` | `get_app_config` | `config/devMode/saved` | `Toggle` / `SettingsSection` |

**状态管理原则**：
- 无全局状态管理库（不需要 Redux/Zustand）
- 每个页面自己通过 hook + invoke 管理数据
- Sidebar 的代理状态通过 `await start/stop` 返回结果直接设置，**不轮询**（PRD 要求）
- Dashboard 的 event 订阅在 `DashboardPage` 的 `useEffect` 中做

### 后端职责（新增/修改）

| 文件 | 修改内容 |
|------|----------|
| `storage/database.rs` | 新增 `get_dashboard_data()`、`list_providers_with_stats()`、`get_provider_detail()` |
| `api/commands.rs` | 新增 `get_dashboard_data`、`get_provider_detail`、`get_app_config`、`save_app_config`；修改 `list_providers` |
| `lib.rs` | 注册新 commands |
| `Cargo.toml` | 无需新增依赖（所有聚合用已有 rusqlite） |

**不改动**：
- `proxy/server.rs`（ProxyState 已在 Phase 2 注入 AppHandle）
- `proxy/handlers/*`（event 已在 Phase 2 触发）
- `auditor/*`（审计逻辑不变）
- SQLite schema（不新增表/列）

---

## 聚合查询逻辑

### 可信度评分算法

```
差异率 = (声称值 - 审计值) / 声称值 × 100%
多报率 = max(0, 差异率)    // 少报不扣分

综合差异率 = (input多报率 × 10 + cache多报率 × 2 + output多报率 × 5) / 17
可信度     = max(0, 100 - 综合差异率)
```

**说明**：
- 只惩罚多报（声称 > 审计），少报不扣分
- I weight 10（Input 是主体，虚报成本最高），O weight 5，C weight 2
- 最终值 clamp 到 [0, 100]（下限 0，上限不封顶但 100 封顶）

**举例**：
| 场景 | I 多报率 | C 多报率 | O 多报率 | 综合差异率 | 可信度 |
|------|---------|---------|---------|-----------|--------|
| 完全一致 | 0% | 0% | 0% | 0 | 100 |
| Input 多报 5% | 5% | 0% | 0% | 2.94 | 97 |
| 全面多报 | 5% | 3% | 4% | 5.12 | 95 |
| 少报不扣 | -5% | 0% | 2% | 0.59 (C,O只有2%多报) | 99 |

### 今日差异率计算

```
Input 差异率 = (SUM(claimed_input) - SUM(real_input)) / SUM(claimed_input) × 100%
Cache 差异率 = (SUM(claimed_cache) - SUM(detected_cache)) / SUM(claimed_cache) × 100%
Output 差异率 = (SUM(claimed_output) - SUM(real_output)) / SUM(claimed_output) × 100%
```

### 近 7 日统计范围

```
未启动代理时：SELECT ... WHERE timestamp >= date('now', '-7 days')
                （所有 Provider 聚合）
启动代理后：  SELECT ... WHERE timestamp >= date('now', '-7 days')
                AND provider_id = :active_provider_id
```

---

## Scope

### Phase 3 包含

1. **设计系统**：CSS design tokens + Tailwind 扩展 + 全局样式
2. **共享组件库**：Button / Modal / Toggle / TrustRing
3. **导航框架**：React Router + 侧边栏（取代顶部 Tab 导航）
4. **Dashboard 页面**：趋势图 + 近 7 日柱状图 + 今日概览（含饼图）+ 可信度排名 + 当前审计 + 计时器
5. **Providers 页面**：卡片网格 + 审计条形图 + 模型过滤 + 表单模态框
6. **Settings 页面**：4 组设置项 + 开发者模式开关
7. **后端聚合查询**：`get_dashboard_data` + `list_providers_with_stats` + `get_provider_detail` + `get/save_app_config`
8. **Event 驱动更新**：`audit-tick` → Dashboard 刷新
9. **旧代码清理**：删除旧的 Dashboard / Providers / RequestList / DevPanel
10. **UI 润色**：品牌字体、卡片入场动画、噪声纹理、可信度光晕

### Phase 3 不包含（留到后续）

1. ❌ Gemini / 更多 Provider 支持
2. ❌ 缓存跨会话持久化
3. ❌ Provider 预设模板库（50+ 预设）
4. ❌ 导出报告（CSV/JSON）
5. ❌ Docker 部署
6. ❌ 多语言完善（仅中/英二选）
7. ❌ 请求详情弹窗（PRD 中列为 Phase 3 但当前 scope 不包含）
8. ❌ Edge 端/多用户/团队管理

---

## 验收标准

1. **设计系统**：Tailwind 构建通过，design-tokens.css 所有变量在浏览器中生效
2. **导航**：点击 Sidebar 导航项切换页面，URL 同步更新，激活态正确
3. **Dashboard 数据**：
   - 打开 app 后显示数据（或空状态）
   - 新请求到达后 event → 趋势图更新
   - 今日概览数字正确
   - 饼图各模型扇形比例正确
   - Provider 排名按可信度排序
   - 当前审计显示最新一条
4. **Dashboard 空状态**：无审计记录时显示"暂无数据"，无报错
5. **Dashboard 加载态**：数据加载中显示骨架屏
6. **Dashboard 错误态**：后端报错时显示错误信息
7. **Providers 页面**：卡片列表渲染、CRUD 正常、模型标签过滤正常
8. **Settings**：读/写配置正常，开发者模式开关动态显示 Dev 页面
9. **Proxy 启停**：Sidebar 按钮启停代理，计时器跟随状态
10. **Provider 切换**：自动停代理
11. **旧页面已删除**：`Dashboard.tsx`、`Providers.tsx`、`RequestList.tsx`、`DevPanel.tsx` 不再存在
12. **构建通过**：`cd frontend && npx tsc --noEmit && npx vite build` 无报错

---

## 非功能约束

| 约束 | 要求 |
|------|------|
| 技术栈 | React 18 + TypeScript + Tailwind CSS + Vite + Tauri v2 |
| CSS 方案 | CSS variables + Tailwind utility（无 CSS-in-JS） |
| 图表 | 内联 SVG（无 Chart.js / D3 依赖） |
| 路由 | react-router-dom v6 |
| IPC | @tauri-apps/api v2（invoke / listen） |
| 后端 | Rust + rusqlite + serde |
| Schema | 不新增表/列 |
| 字体 | 系统默认字体栈（无外部字体依赖，除非显式安装） |
| 无障碍 | 颜色对比度 > 4.5:1，键盘操作支持 |
| 布局 | Dashboard 在 1024×768 以上全屏展示，无滚动条 |
