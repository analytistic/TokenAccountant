# Phase 3: UI/UX 重新设计

> 设计文档 — 实施计划由 writing-plans 另行生成

**Goal:** 将当前 MVP 风格前端（暗色主题、顶部导航、基础 Tailwind）改造为符合 v0.3 设计系统的专业 UI：浅色主题、侧边栏导航、卡片式仪表盘、SVG 图表、Provider 审计可视化、设置页面。

**Architecture:** 现有 React + TypeScript + Tailwind CSS 上叠加 CSS 变量设计系统（参考 HTML 原型 :root），保留 Tailwind 作为消费层。纯 SVG 内联绘图（复用原型 chart 渲染函数），逐步替换现有页面。后端新增聚合数据接口和 Provider 审计摘要字段。

**Tech Stack:** React 18 + TypeScript + Tailwind CSS + Vite + Tauri v2 (Rust IPC) + 纯 SVG（趋势图/柱状图/环形图）

**Spec ref:** [UI v0.3 需求文档](/docs/product/ui-v0.3-requirements.md) · [UI v0.3 Prototype](/docs/product/ui-v0.3-prototype.html)

---

## 一、设计决策

### 1. CSS 变量优先，Tailwind 消费

HTML 原型定义了完整的 CSS 变量体系（品牌色/语义色/中性色阶/I/C/O 颜色/字体/阴影/圆角/动画曲线）。这些变量是设计系统的源数据，Tailwind 通过 `theme.extend` 映射消费。

**理由：**
- HTML 原型已有一个完整、经过验证的设计系统，直接搬运 CSS 变量比重构为纯 Tailwind 语义更省时
- 变量可被 React inline style 和 Tailwind class 同时使用，灵活性高
- 与设计师使用的变量名一致，减少沟通成本

### 2. 图表：纯 SVG（复用原型代码）

页面上的三组图表（Token 趋势图、7 日柱状图、I/C/O 环形图）用纯 SVG 渲染，原型 `renderSingleChart` / `renderDashBarChart` 等函数已完整实现。

**理由：**
- 原型代码已完成且经过视觉调优，直接封装为 React 组件即可
- 纯 SVG 零依赖，不增加包体积
- 颜色/字体/间距与设计系统天然一致
- 未来如需 tooltip 等交互 → 封一层 recharts wrapper，侵入式改造不破坏现有逻辑

### 3. 页面架构：三页路由，RequestList 合并

当前三个页面（Dashboard、Providers、RequestList）→ v0.3 三个页面（Dashboard、Providers、Settings）。

RequestList 的审计列表功能合并到 Dashboard 的"当前请求审计"组件中（展示最新一条 + 可疑标记）。完整的历史请求列表在 v0.3 中不提供独立页面——如果用户需要，应在 Phase 4 中作为增强功能添加。

**理由：**
- v0.3 需求文档只定义了 3 个页面
- "当前请求审计"展示最新一条的 I/C/O 对比，信息密度高于列表
- Settings 是新增需求，必须进 Phase 3

### 4. 后端扩展但不重构

新增少量 Tauri 命令（`get_dashboard_summary`、`get_provider_audit_summary`、配置读写），在现有 Provider 模型上增加审计摘要字段（credibility、per_model 审计数据）。不动 proxy handler、tokenizer、diff 系统。

### 5. 左侧导航替代顶部标签栏

当前顶部标签栏（`App.tsx` 中的 `nav`）替换为固定 220px 左侧侧边栏，使用 `react-router-dom` v6 的 `<Routes>` / `<NavLink>`。

---

## 二、现有代码变化清单

### 2.1 前端新增文件

| 文件 | 职责 |
|------|------|
| `src/styles/design-tokens.css` | CSS 变量定义（从原型 `:root` 复制） |
| `src/styles/global.css` | Reset、滚动条、`@keyframes` 全局动画 |
| `src/components/Sidebar.tsx` | 左侧侧边栏导航（Brand + NavItem ×3 + ProxyToggle） |
| `src/components/Button.tsx` | 按钮组件（primary/secondary/danger/ghost, sm/lg） |
| `src/components/Card.tsx` | 通用卡片容器 |
| `src/components/Modal.tsx` | 模态框（遮罩 + 弹窗 + 动画） |
| `src/components/Badge.tsx` | 徽标（success/warning/danger/info/neutral + dot） |
| `src/components/Toggle.tsx` | 开关组件（受控，滑块动画） |
| `src/components/Input.tsx` | 输入框（标准 form-input 风格 + focus ring） |
| `src/components/EmptyState.tsx` | 空状态占位（图标 + 标题 + 描述 + CTA） |
| `src/components/Skeleton.tsx` | 加载骨架屏（shimmer 动画） |
| `src/components/Tooltip.tsx` | 悬浮提示 |
| `src/components/TrustRing.tsx` | 环形可信度评分（大 56px / 小 34px） |
| `src/components/TrendChart.tsx` | Token 趋势图组件（3 子图：Input/Cache/Output） |
| `src/components/BarChart7Day.tsx` | 7 日 Token 柱状图组件 |
| `src/components/TripleRing.tsx` | I/C/O 三重环形图组件 |
| `src/components/CurrentAudit.tsx` | 当前请求审计组件（通过/失败指示器 + 对比条） |
| `src/components/SessionTimer.tsx` | Session 计时器（播放/暂停） |
| `src/components/ProviderForm.tsx` | 添加/编辑 Provider 模态框表单 |
| `src/components/ProviderCard.tsx` | Provider 卡片（名称/URL/模型标签/可信度环/审计条） |
| `src/pages/DashboardPage.tsx` | 新 Dashboard 页 |
| `src/pages/ProvidersPage.tsx` | 新 Providers 页 |
| `src/pages/SettingsPage.tsx` | 新 Settings 页 |

### 2.2 前端修改文件

| 文件 | 变更 |
|------|------|
| `src/App.tsx` | 替换为 `BrowserRouter` + `Sidebar` + `<Routes>` 结构，删除顶部导航 |
| `src/main.tsx` | 引入 `design-tokens.css` / `global.css` |
| `src/styles.css` | 清空为 `@tailwind base/components/utilities` + 引入设计系统 |
| `tailwind.config.js` | `theme.extend.colors/fontFamily/borderRadius/boxShadow` 映射 CSS 变量 |

### 2.3 前端删除文件

| 文件 | 原因 |
|------|------|
| `src/pages/Dashboard.tsx` | 被 `DashboardPage.tsx` 替代 |
| `src/pages/Providers.tsx` | 被 `ProvidersPage.tsx` 替代 |
| `src/pages/RequestList.tsx` | 审计列表合并到 Dashboard 的 CurrentAudit 组件，DevPanel 移入设置页的开发者模式 |

### 2.4 后端修改文件

| 文件 | 变更 |
|------|------|
| `src-tauri/src/api/commands.rs` | 新增 `get_dashboard_summary`、`get_provider_audit_summary`、`get_app_config`、`save_app_config` |
| `src-tauri/src/provider/manager.rs` | Provider 模型增加 `credibility`、`audit_summary`、`per_model` 字段（聚合计算，非持久化） |
| `src-tauri/src/storage/database.rs` | 新增 `get_dashboard_summary`、`get_provider_audit_summary` 等聚合查询方法（不改表结构） |
| `src-tauri/src/lib.rs` | 注册新命令 |

> **关键**：数据库表**不需要改**。`audit_log` 表已有 `provider_id`、`model`、`claimed_*_tokens`、`real_*_tokens`、`is_suspicious` 等字段。credibility 和 audit_summary 通过 SQL 聚合查询从 audit_log 实时计算。

---

## 三、设计系统

### 3.1 CSS 变量（源数据）

从 HTML 原型 `:root` 块原样复制以下类别，存入 `src/styles/design-tokens.css`：

```
品牌色:  --brand: #4F46E5 系列
语义色:  --success / --warning / --danger / --info + 各自的 subtle/border
中性色阶: --gray-0 到 --gray-950 （12 阶）
I/C/O 色: Input=#60A5FA, Cache=#FB923C, Output=#34D399
字体:    --font-sans / --font-mono（SF Pro / SF Mono 为首选）
文字尺寸: --text-xs 到 --text-4xl（6 阶）
间距:    --space-1 到 --space-16（10 阶）
圆角:    --radius-sm/md/lg/xl
阴影:    --shadow-xs/sm/md/lg/xl/card/modal（7 阶）
动画:    --ease-out / --ease-in / --ease-spring
         --duration-fast/normal/slow
布局:    --sidebar-w: 220px
```

### 3.2 Tailwind 映射（消费层）

```js
// tailwind.config.js — 仅展示变化部分
module.exports = {
  theme: {
    extend: {
      colors: {
        brand: {
          DEFAULT: 'var(--brand)',
          light: 'var(--brand-light)',
          hover: 'var(--brand-hover)',
          subtle: 'var(--brand-subtle)',
          muted: 'var(--brand-muted)',
        },
        success: { DEFAULT: 'var(--success)', subtle: 'var(--success-subtle)', border: 'var(--success-border)' },
        warning: { DEFAULT: 'var(--warning)', subtle: 'var(--warning-subtle)', border: 'var(--warning-border)' },
        danger:  { DEFAULT: 'var(--danger)',  subtle: 'var(--danger-subtle)',  border: 'var(--danger-border)' },
        info:    { DEFAULT: 'var(--info)',    subtle: 'var(--info-subtle)',    border: 'var(--info-border)' },
        gray: {
          0: 'var(--gray-0)',   50: 'var(--gray-50)',
          100: 'var(--gray-100)', 200: 'var(--gray-200)',
          300: 'var(--gray-300)', 400: 'var(--gray-400)',
          500: 'var(--gray-500)', 600: 'var(--gray-600)',
          700: 'var(--gray-700)', 800: 'var(--gray-800)',
          900: 'var(--gray-900)', 950: 'var(--gray-950)',
        },
        ico: {
          input: '#60A5FA', cache: '#FB923C', output: '#34D399',
        },
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', "'SF Pro Display'", "'Segoe UI'", 'system-ui', 'sans-serif'],
        mono: ["'SF Mono'", "'JetBrains Mono'", "'Fira Code'", "'Cascadia Code'", 'monospace'],
      },
      borderRadius: {
        sm: 'var(--radius-sm)', md: 'var(--radius-md)',
        lg: 'var(--radius-lg)', xl: 'var(--radius-xl)',
      },
      boxShadow: {
        card: 'var(--shadow-card)', modal: 'var(--shadow-modal)',
      },
      transitionTimingFunction: {
        out: 'var(--ease-out)', spring: 'var(--ease-spring)',
      },
    },
  },
};
```

### 3.3 I/C/O 颜色全局编码

| Token 类 | 色值 | 使用场景 |
|----------|------|----------|
| Input | `#60A5FA` | Prefill 水平条蓝色段、趋势图蓝色线、小圆点 |
| Cache | `#FB923C` | Prefill 水平条橙色段、趋势图橙色线、小圆点 |
| Output | `#34D399` | Output 水平条、趋势图绿色线、小圆点 |

### 3.4 差异率颜色编码

| 范围 | 色值 | 语义 |
|------|------|------|
| < 5% | `#10B981` | 可信 |
| 5%–15% | `#D97706`| 需关注 |
| > 15% | `#DC2626` | 可疑 |

Provider 卡片内使用更严格阈值：≤1% / ≤5% / >5%。

### 3.5 Token 数量格式化

```typescript
function formatToken(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M'
  if (n >= 1_000)     return (n / 1_000).toFixed(1) + 'K'
  return n.toString()
}
```

---

## 四、布局结构

### 4.1 整体布局

```
┌───────────────────────────────────────────────────┐
│  ┌────────────┐  ┌─────────────────────────────┐  │
│  │  Sidebar   │  │  <Routes>                    │  │
│  │  (220px)   │  │                              │  │
│  │            │  │  ┌───────────────────────┐   │  │
│  │  🦉 Logo   │  │  │  PageHeader          │   │  │
│  │  v0.3      │  │  │  (标题 + 操作按钮)     │   │  │
│  │            │  │  ├───────────────────────┤   │  │
│  │  Dashboard │  │  │                       │   │  │
│  │  Providers │  │  │  页面内容区            │   │  │
│  │  Settings  │  │  │                       │   │  │
│  │            │  │  └───────────────────────┘   │  │
│  │  ────────  │  │                              │  │
│  │  ● 代理    │  │                              │  │
│  │    运行中  │  │                              │  │
│  └────────────┘  └─────────────────────────────┘  │
└───────────────────────────────────────────────────┘
```

### 4.2 路由表

| 路径 | 页面组件 | 侧边栏标签 |
|------|----------|-----------|
| `/` | `DashboardPage` | Dashboard |
| `/providers` | `ProvidersPage` | Providers |
| `/settings` | `SettingsPage` | Settings |

### 4.3 Dashboard 两栏 Grid

```
grid-template-columns: 1fr 300px
grid-template-rows:    auto auto auto

┌──────────────────────┬──────────────────┐
│  TrendChart          │  Right Split     │
│  (3 子趋势图)         │  ┌────────────┐  │
│  Input / Cache /     │  │ Today      │  │
│  Output              │  │ Overview   │  │
│                      │  │ (环形图)    │  │
│                      │  ├────────────┤  │
│                      │  │ Provider   │  │
│                      │  │ Ranking    │  │
│                      │  └────────────┘  │
├──────────────────────┼──────────────────┤
│  BarChart7Day        │  CurrentAudit    │
│  (近 7 日统计)        │  (当前请求审计)   │
├──────────────────────┼──────────────────┤
│                      │  SessionTimer    │
└──────────────────────┴──────────────────┘
```

---

## 五、核心组件规范

### 5.1 Sidebar

**Props**: 无（通过 `useLocation` 自行判断激活项）

```
┌────────────────────┐
│  ┌──┐              │
│  │🦉│              │
│  │  │ TokenAccount │  ← Brand (34px SVG + 名称 + v0.3)
│  └──┘ v0.3         │
├────────────────────┤
│  ▎ ■ Dashboard     │  ← 激活项：左 3px 竖线 + brand-subtle 背景
│  ▎ ▲ Providers     │
│  ▎ ⚙ Settings      │  ← 未激活：gray-500, hover 变深
│                    │
├────────────────────┤
│  ┌──────────────┐  │
│  │ ● 代理运行中   │  │  ← 绿色按钮（pulse-dot 动画）
│  │              │  │     灰色 + "启动代理" = 已停止
│  └──────────────┘  │
└────────────────────┘
```

- 导航项使用 `<NavLink>`，通过 `isActive` 控制 `active` 样式
- 代理状态调用 `get_proxy_status` 初始化，点击调用 `start_proxy` / `stop_proxy`
- 状态切换：绿色 + 脉冲圆点 ⇄ 灰色 + 静态圆点

### 5.2 Button

| 变体 | `className` | 用途 |
|------|------------|------|
| Primary | `btn-primary` | 添加 Provider、保存 |
| Secondary | `btn-secondary` | 取消、次要操作 |
| Danger | `btn-danger` | 删除 Provider、清除数据 |
| Ghost | `btn-ghost` | 工具栏图标按钮（刷新/设置） |
| sm | `btn-sm` | 小号胶囊按钮（切换、标签） |
| lg | `btn-lg` | 大号按钮（底部添加 Provider） |

通用行为：`:active { transform: scale(0.98) }`，`:hover` 对应色值加深 + Y:-1px 上浮。

### 5.3 Card

```tsx
<Card>
  <CardHeader title="标题" action={<Button />} />
  <CardBody>内容</CardBody>
  <CardFooter>页脚</CardFooter>
</Card>
```

样式：`bg-gray-0 border border-gray-200 rounded-lg shadow-card`，hover 时 `shadow-md`。

### 5.4 Modal

- 遮罩：`fixed inset-0 bg-black/40 backdrop-blur-sm`，点击关闭
- 弹窗：`max-w-[640px] w-[calc(100%-48px)] max-h-[85vh]`，居中
- 动画：进入时 `scale(0.95) translateY(10px) → scale(1) translateY(0)`，250ms spring
- 内容：`ModalHeader`（标题 + 关闭按钮，sticky） + `ModalBody` + `ModalFooter`（按钮）

### 5.5 Toggle

受控组件：`<Toggle checked={value} onChange={setValue} />`

- 宽 44px × 高 24px，圆角 12px
- 开启：`--brand` 背景，knob 右移 20px（spring 动画）
- 关闭：`--gray-300` 背景，knob 左对齐

### 5.6 TrustRing

```tsx
<TrustRing value={96} size="sm" />
```

SVG 环形进度（周长用 `stroke-dasharray` / `stroke-dashoffset` 控制）。
- 大（56px）：评分醒目展示
- 小（34px）：列表内嵌使用
- 颜色：>=90 绿色 / >=70 黄色 / <70 红色

### 5.7 EmptyState

```tsx
<EmptyState
  icon="📊"
  title="暂无数据"
  description="启动代理后，审计数据将在此展示"
  action={<Button>启动代理</Button>}
/>
```

居中布局，pad: 64px 上下，max-width: 320px。

### 5.8 Skeleton

```tsx
<Skeleton className="h-4 w-24" />
```

渐变动画 `shimmer`（`linear-gradient(90deg, gray-100, gray-200, gray-100)`，`background-size: 200%`）。

---

## 六、页面详细设计

### 6.1 DashboardPage

#### 6.1.1 数据来源

| 数据 | 来源 | 备注 |
|------|------|------|
| 代理状态 | `get_proxy_status` | 已有命令 |
| 今日请求数/可疑数 | `get_dashboard_summary` | **新命令**，或前端从 `list_audit_logs` 按日期筛选 |
| I/C/O 差异率 | `get_dashboard_summary` | **新命令** |
| 趋势图数据 | `list_audit_logs` (最近 20 条) | 已有命令 |
| 7 日统计 | `get_dashboard_summary`（含 7 日聚合） | **新命令** |
| Provider 排名 | `list_providers`（按 credibility 排序） | **需扩展**，加入 credibility 字段 |
| 当前请求审计 | `list_audit_logs`（最新 1 条） | 已有命令 |

#### 6.1.2 TrendChart（左栏上）

- 3 个子图表垂直排列，每个高度约 95px
- 每个子图两条曲线：审计值（实线、不透明） + 声明值（半透明）
- 中间填充区域（`fill-opacity: 0.10`）
- 每个子图下方显示差异率 + 趋势箭头（↑/↓/→）
- 图例：审计值（基准）/ 声明值 / 差值区域
- SVG 画布 740×95，Catmull-Rom 插值

#### 6.1.3 BarChart7Day（左栏下）

- 7 天分组柱状图，每组 Input/Cache/Output 各两柱（审计 + 声明）
- Input+Cache = Prefill，用括号标注
- SVG 画布 800×200，`pad: { top:14 right:16 bottom:28 left:44 }`

#### 6.1.4 TodayOverview（右栏上）

- 请求次数 + 可疑请求数（可疑数标黄/红警示）
- 三重环形图：内圈 Input → 中圈 Cache → 外圈 Output，分别显示匹配率
- 右侧逐行列示 I/C/O 差异 token 数和差异率

#### 6.1.5 ProviderRanking（右栏中）

- 最多 5 个 Provider 排名
- 每行：排名编号（前三名奖牌色：金银铜） + 名称 + URL + 环形图评分 + I/C/O 小字差异率
- 点击切换活跃 Provider

#### 6.1.6 CurrentAudit（右栏下）

- 审计通过指示器（绿色勾 + "审计通过" / 红色叉 + "审计未通过"）
- 审计 Token 水平条（实色，I/C/O 三段）
- 声明 Token 水平条（半透明，I/C/O 三段）
- 下方：I/C/O 逐行列示差异 token 数

#### 6.1.7 SessionTimer（右下）

- 播放/暂停按钮，点击切换
- 运行中显示 `HH:MM:SS` 计时（`setInterval` 每秒更新）
- 纯前端功能，不与后端通信

### 6.2 ProvidersPage

#### 6.2.1 数据来源

| 数据 | 来源 |
|------|------|
| Provider 列表 | `list_providers`（扩展后含 credibility 和 audit_summary） |
| 新增 Provider | `create_provider` |
| 编辑 Provider | `update_provider` |
| 删除 Provider | `delete_provider`（前端确认对话框） |
| 切换活跃 | `switch_provider` |
| 模型筛选后的审计数据 | `get_provider_audit_summary(provider_id, model?)` **新命令** |

#### 6.2.2 ProviderCard

```
┌────────────────────────────────────────────┐
│  [● 官方] [● 活跃]                    ⋮    │  ← 类型徽标 + 活跃徽标 + 三点菜单
│  DeepSeek Official                         │  ← 名称（xl, bold）
│  https://api.deepseek.com (mono 小字)       │
│  [deepseek-chat] [deepseek-coder]          │  ← 可点击模型标签
│                       ┌────┐  ┌──────┐    │
│                       │ 96%│  │ 当前  │    │  ← 可信度环 + 切换按钮
│                       └────┘  └──────┘    │
│  ── Prefill ──                              │
│  声称  ████████████░░░░░░░░░  170K         │  ← 水平条（蓝=Input, 橙=Cache）
│  审计  ██████████░░░░░░░░░░░  167K         │
│       ● Input 3.1%  ● Cache 5.2%          │  ← 差异率
│  ── Output ──                               │
│  声称  ███████████████░░░░░░  89K          │
│  审计  ██████████████░░░░░░░  88K          │
│       ● Output 2.8%                        │
└────────────────────────────────────────────┘
```

交互：
- 点击卡片 → `switch_provider`，激活态顶部 3px Indigo 渐变线
- 点击模型标签 → 筛选该模型，标签高亮；再次点击取消筛选
- 部分 Provider 不提供该模型 → 显示"该 Provider 无 {model} 数据"空状态
- 三点菜单 → 编辑 / 清除数据 / 删除（危险操作需确认）

#### 6.2.3 ProviderForm（模态框）

字段：
| 字段 | 类型 | 说明 |
|------|------|------|
| Provider 名称 | 文本 | 可选，默认从 URL 提取 |
| API Endpoint | 文本 | 必填 |
| API Key | 密码（可切换显示） | 必填 |
| 模型列表 | 逗号分隔或标签输入 | 必填 |
| 设为活跃 | 开关 | 可选，默认关 |

### 6.3 SettingsPage

#### 6.3.1 数据来源

| 数据 | 来源 |
|------|------|
| 当前配置 | `get_app_config` **新命令** |
| 保存配置 | `save_app_config` **新命令** |

#### 6.3.2 设置分类

```
Settings
├── 基础设置
│   ├── 代理端口   数字输入（默认 8080）
│   └── 界面语言   下拉（简体中文 / English）
├── 代理设置
│   ├── 自动配置 Claude   开关
│   └── 启动时自动运行代理  开关
├── 审计设置
│   ├── 可疑阈值   下拉（3% / 5% / 10%）
│   └── 记录完整请求/响应  开关
├── 开发者模式 ← 可展开
│   ├── 启用开发者模式  开关（展开/收起下级）
│   ├── Render Inspector  开关
│   ├── Tokenizer 调试    开关
│   └── DevTrace 缓冲区   数字输入（默认 1000）
└── 关于
    ├── TokenAccountant v0.3.0
    ├── GitHub 链接按钮
    └── Changelog 按钮
```

每个设置项一行（`settings-row`）：左侧标签 + 描述，右侧控件。

开发者模式用 `dev-section` 包裹，默认 `display:none`，开启后 `.visible { display: block }`，带黄色警示横幅。

---

## 七、全局行为

### 7.1 页面切换动画

```css
@keyframes page-in {
  from { opacity: 0; transform: translateY(8px); }
  to   { opacity: 1; transform: translateY(0); }
}
/* duration: 250ms, ease-out */
```

### 7.2 代理状态轮询

页面加载时 + 每 10 秒调用 `get_proxy_status` 更新侧边栏按钮状态。

### 7.3 模态框交互

- 点击遮罩关闭
- 点击关闭按钮关闭
- 按 Escape 关闭
- 打开时 body scroll lock

### 7.4 响应式断点

| 断点 | 行为 |
|------|------|
| >1024px | 侧边栏 220px + 两栏 Dashboard |
| 768–1024px | 侧边栏折叠为 64px（仅图标），Dashboard 单栏 |
| <768px | 侧边栏隐藏（汉堡菜单），内容全宽 |

### 7.5 无障碍

- 图标按钮带 `aria-label`
- 图表 SVG 带 `role="img"` + `aria-label`
- `:focus-visible` 轮廓（2px solid brand）
- 动画遵循 `prefers-reduced-motion`

---

## 八、数据流：BFF 模式

### 8.1 原则

所有页面数据通过 **1 个命令 1 次 IPC** 获取。后端负责查 SQL + 整形，前端只负责渲染。不在前端做 `sort`/`filter`/`map` 聚合逻辑。

```
页面 mount
  └─ invoke("get_dashboard_data")          ──▶ { proxyStatus, todaySummary, trendData,
       │                                            ranking, currentAudit }
       │                                  ──▶ 前端直接 setState，不用二次加工
       └─ 数据太长？不可能。后端知道前端要什么
```

**教训**：不要把前端当 SQL 客户端。`list_audit_logs` + `list_providers` 各自返回一堆前端不需要的字段，再在前端 `.map().filter().sort()`——这是 REST 时代的坏习惯。Tauri IPC 没有缓存层，调一次就是一次序列化 + 跨进程开销，更应该一接口拿完。

### 8.2 Dashboard 数据流：单命令 get_dashboard_data

```
DashboardPage mount
  └─ invoke("get_dashboard_data")
       │
       ▼
    后端 database.rs 执行 5 条 SQL（同 1 个事务）
       │
       ├─ 代理状态（现有 get_proxy_status 逻辑）
       ├─ 今日汇总：SELECT SUM/CASE WHEN  FROM audit_log WHERE timestamp > datetime('-1 day')
       ├─ 近 20 条趋势：SELECT id, claimed_*_tokens, real_*_tokens FROM audit_log ORDER BY id DESC LIMIT 20
       ├─ Provider 排名 top 5：
       │    SELECT id, name, api_base_url, credibility, ... FROM providers
       │    JOIN (SELECT provider_id, AVG(input_diff), COUNT(*) FROM audit_log GROUP BY provider_id)
       │    ORDER BY credibility DESC LIMIT 5
       └─ 最新 1 条审计：SELECT * FROM audit_log ORDER BY id DESC LIMIT 1
       │
       ▼
    返回 DashboardData（前端直接 setState）
       └─ proxyStatus:   { running, port }
       └─ todaySummary:  { totalRequests, suspiciousRequests, inputDiffRate, cacheDiffRate, outputDiffRate }
       └─ trendData:     [{ idx, input: {c,d}, cache: {c,d}, output: {c,d} }]
       └─ ranking:       [{ id, name, url, credibility, inputDiff, cacheDiff, outputDiff }]
       └─ currentAudit:  { auditPassed, input: {audit,claimed,diff}, cache: {...}, output: {...} }
```

前端拿到后直接拆 props 传给子组件，**不做任何计算**。

### 8.3 Providers 数据流：惰性按需加载

```
ProvidersPage mount
  ├─ invoke("list_providers")          ──▶ [{id, name, url, type, models[], isActive,
  │                                              credibility, totalRequests, suspiciousRequests,
  │                                              auditSummary: {i_claimed, i_detected, ...}}]
  │                                    ──▶ 每个 ProviderCard 展示汇总数据
  │                                        注意：per_model 数组不在此返回
  │
  └─ 用户点击模型标签
       └─ invoke("get_provider_detail", { providerId, model? })
            ──▶ { input: {claimed,detected}, cache: {claimed,detected},
                   output: {claimed,detected}, credibility, diffRates }
            ──▶ 更新当前卡片的审计条 + 差异率
            ──▶ 其他 Provider 同步调用 get_provider_detail 获取同模型数据
```

**要点**：
- `list_providers` 不返回 `per_model` 数组——那是所有模型 × 所有 Provider 的笛卡尔积数据，前端可能永远用不上
- 用户点击模型标签后，才按需调 `get_provider_detail` 获取该模型的审计数据
- 所有选中了同一模型的 Provider 可以并行请求

### 8.4 Settings 数据流：读写分离

```
SettingsPage mount
  └─ invoke("get_app_config")
       ──▶ { proxyPort, language, autoConfigureClaude, autoStartProxy, suspicionThreshold, ... }
       ──▶ 设置表单 defaultValue

用户修改 → 点击"保存设置"
  └─ invoke("save_app_config", { config })
       ──▶ 成功/失败反馈
```

---

## 九、后端新增需求

### 9.1 数据库：不新增表，只加聚合查询

现有 `audit_log` 表足以支撑所有 v0.3 需求：

```sql
audit_log (
  id, timestamp, provider_id, model, api_format,
  claimed_input_tokens, claimed_output_tokens, claimed_cached_tokens,
  real_input_tokens, real_output_tokens, detected_cached_tokens,
  input_diff, output_diff, cache_diff,
  is_suspicious, suspicion_reason,
  request_preview, response_preview
)
```

**不改表结构**。所有新增数据通过 `SELECT SUM/GROUP BY` 从 `audit_log` 聚合产生。

### 9.2 新 Tauri 命令（3 条）

| 命令 | 用途 | 调用时机 |
|------|------|----------|
| `get_dashboard_data` | Dashboard 完整数据（1 次 IPC 拿完） | Dashboard mount |
| `get_provider_detail` | 单个 Provider 的模型级审计数据 | 用户点击模型标签时按需调用 |
| `get_app_config` | 读取配置 | Settings mount |
| `save_app_config` | 保存配置 | Settings 点击保存 |

### 9.3 get_dashboard_data（核心命令）

```rust
// api/commands.rs

#[derive(Serialize)]
pub struct DashboardData {
    pub proxy_status: ProxyStatus,
    pub today_summary: TodaySummary,
    pub trend_data: Vec<TrendPoint>,
    pub provider_ranking: Vec<ProviderRankEntry>,
    pub current_audit: Option<CurrentAuditEntry>,
}

#[derive(Serialize)]
pub struct TodaySummary {
    pub total_requests: i64,
    pub suspicious_requests: i64,
    pub input_diff_rate: f64,      // 差异率百分比
    pub cache_diff_rate: f64,
    pub output_diff_rate: f64,
    pub input_diff_tokens: i64,    // 差异绝对值
    pub cache_diff_tokens: i64,
    pub output_diff_tokens: i64,
    pub input_match_rate: f64,     // 匹配率 0-100（环形图用）
    pub cache_match_rate: f64,
    pub output_match_rate: f64,
}

// 趋势图用——只含 ID + token 数，不含 request_preview 等大字段
#[derive(Serialize)]
pub struct TrendPoint {
    pub idx: i64,
    pub input_claimed: i64,  pub input_detected: i64,
    pub cache_claimed: i64,  pub cache_detected: i64,
    pub output_claimed: i64, pub output_detected: i64,
}

#[derive(Serialize)]
pub struct ProviderRankEntry {
    pub id: String,
    pub name: String,
    pub url: String,
    pub credibility: f64,
    pub input_diff_rate: f64,
    pub cache_diff_rate: f64,
    pub output_diff_rate: f64,
}

#[derive(Serialize)]
pub struct CurrentAuditEntry {
    pub audit_passed: bool,
    pub input_audit: i64,  pub input_claimed: i64,  pub input_diff: i64,
    pub cache_audit: i64,  pub cache_claimed: i64,  pub cache_diff: i64,
    pub output_audit: i64, pub output_claimed: i64, pub output_diff: i64,
}

#[tauri::command]
pub async fn get_dashboard_data(
    state: State<'_, TauriState>,
) -> Result<DashboardData, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // 1) 代理状态（复用现有逻辑）
    let proxy_status = { /* get_proxy_status 内部实现 */ };

    // 2) 今日汇总（1 条 SQL）
    let today = db.query_row(
        "SELECT
            COUNT(*),
            SUM(CASE WHEN is_suspicious=1 THEN 1 ELSE 0 END),
            COALESCE(SUM(claimed_input_tokens - real_input_tokens), 0),
            COALESCE(SUM(claimed_cached_tokens - detected_cached_tokens), 0),
            COALESCE(SUM(claimed_output_tokens - real_output_tokens), 0)
        FROM audit_log WHERE timestamp >= datetime('now', '-1 day')", ...)?;

    // 3) 趋势数据（只取 token 字段，不要 request_preview）
    let trend = db.query_rows(
        "SELECT id, claimed_input_tokens, real_input_tokens,
                claimed_cached_tokens, detected_cached_tokens,
                claimed_output_tokens, real_output_tokens
         FROM audit_log ORDER BY id DESC LIMIT 20", ...)?;

    // 4) Provider 排名 top 5
    let ranking = db.query_rows(
        "SELECT p.id, p.name, p.api_base_url,
                COALESCE(AVG(a.input_diff), 0) as avg_in,
                COALESCE(AVG(a.cache_diff), 0) as avg_ca,
                COALESCE(AVG(a.output_diff), 0) as avg_out
         FROM providers p
         LEFT JOIN audit_log a ON a.provider_id = p.id
         GROUP BY p.id
         HAVING COUNT(a.id) > 0
         ORDER BY avg_in + avg_ca + avg_out ASC
         LIMIT 5", ...)?;

    // 5) 最新 1 条审计记录
    let latest = db.query_row(
        "SELECT real_input_tokens, claimed_input_tokens,
                detected_cached_tokens, claimed_cached_tokens,
                real_output_tokens, claimed_output_tokens,
                input_diff, cache_diff, output_diff, is_suspicious
         FROM audit_log ORDER BY id DESC LIMIT 1", ...)?;

    Ok(DashboardData { proxy_status, today_summary, trend_data, provider_ranking, current_audit })
}
```

### 9.4 get_provider_detail（按需加载）

```rust
#[derive(Serialize)]
pub struct ProviderDetail {
    pub provider_id: String,
    pub model: String,
    pub credibility: f64,
    pub input_claimed: i64,  pub input_detected: i64,
    pub cache_claimed: i64,  pub cache_detected: i64,
    pub output_claimed: i64, pub output_detected: i64,
    pub input_diff_rate: f64,
    pub cache_diff_rate: f64,
    pub output_diff_rate: f64,
}

#[tauri::command]
pub async fn get_provider_detail(
    state: State<'_, TauriState>,
    provider_id: String,
    model: String,
) -> Result<ProviderDetail, String>
```

调用时机：用户点击 Provider 卡片上的模型标签。前端收集当前页所有选中了该模型的 Provider，并行调用 `get_provider_detail`。

### 9.5 list_providers 扩展

```rust
#[derive(Serialize)]
pub struct ProviderWithStats {
    // 现有字段
    pub id: String, pub name: String, pub provider_type: String,
    pub api_base_url: String, pub api_key: Option<String>,
    pub supported_models: Vec<String>, pub is_active: bool,
    // 新增汇总字段（不含 per_model）
    pub credibility: f64,
    pub total_requests: i64,
    pub suspicious_requests: i64,
    pub audit_summary: AuditSummary,
}
```

区别 v1：
- **有** `credibility` + `audit_summary`（总体的 I/C/O 汇总）
- **没有** `per_model: Vec<PerModelAudit>`（不提前返回笛卡尔积数据）

### 9.6 get_app_config / save_app_config

```rust
#[derive(Deserialize, Serialize, Clone)]
pub struct AppConfigPayload {
    pub proxy_port: u16,
    pub language: String,                  // "zh-CN" | "en"
    pub auto_configure_claude: bool,
    pub auto_start_proxy: bool,
    pub suspicion_threshold: f64,
    pub record_full_body: bool,
    pub dev_mode_enabled: bool,
    pub dev_trace_buffer_size: u32,
}
```

### 9.7 Provider 模型扩展

```rust
// provider/types.rs — 现有 Provider struct 追加 4 个可选字段

#[derive(Serialize, Clone, Default)]
pub struct AuditSummary {
    pub input_claimed: i64,  pub input_detected: i64,
    pub cache_claimed: i64,  pub cache_detected: i64,
    pub output_claimed: i64, pub output_detected: i64,
}
```

```rust
#[derive(Serialize)]
pub struct Provider {
    // ... 现有字段不变
    // 新增（由 list_providers 查询时填充，不从数据库读取）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credibility: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_requests: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspicious_requests: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_summary: Option<AuditSummary>,
}
```

> 用 `Option` 保证现有 `list_providers` 调用方不报错，新调用方通过 `list_providers_with_stats` 获取填充后的值。

### 9.8 可信度计算

```rust
fn compute_credibility(input_diff_rate: f64, cache_diff_rate: f64, output_diff_rate: f64) -> f64 {
    let avg = (input_diff_rate.abs() + cache_diff_rate.abs() + output_diff_rate.abs()) / 3.0;
    let score = 100.0 - avg * 10.0;  // 每 1% 平均差异扣 10 分
    score.clamp(0.0, 100.0)
}
```

---

## 十、Phase 3 Scope

### Phase 3 包含

1. CSS 设计系统搭建（变量 + Tailwind 映射）
2. 共享组件库（Button / Card / Modal / Toggle / Badge / Input / EmptyState / Skeleton / Tooltip）
3. Sidebar 导航（路由切换 + 代理状态按钮）
4. Dashboard 页重构（两栏布局 + 6 个子组件 + SVG 图表）
5. Providers 页重构（卡片网格 + 审计分段条 + 模型筛选 + 模态框表单）
6. Settings 页新增（4 个设置分类 + 开发者模式面板）
7. 后端新增 Dashboard/Provider 聚合查询命令
8. 后端扩展 Provider 数据模型
9. 后端新增配置读写命令
10. RequestList 页面移除

### Phase 3 不包含（留到后续）

1. **recharts 交互增强** — tooltip、缩放、数据刷选（等用户反馈再决定是否引入）
2. **响应式侧边栏折叠** — 768px 断点下的折叠动画和汉堡菜单
3. **数据导出** — CSV/JSON 导出审计记录
4. **多语言 i18n** — 设置中的语言切换需要 i18n 框架支持
5. **请求详情弹窗** — 点击审计记录展开完整请求/响应
6. **主题切换** — 暗色/浅色切换（设计系统已预留变量，但实现放在后续）

---

## 十一、验收标准

1. 页面切换使用侧边栏导航，当前页有视觉高亮，切换时有淡入动画
2. Dashboard 展示 Token 趋势图（3 条曲线 + 差异区域）、7 日柱状图、今日概览（环形图 + 差异率）、Provider 排名、当前请求审计、Session 计时器
3. 趋势图三条线（I/C/O）各自区分，审计值实线 vs 声明值虚线，差异区域半透明
4. Providers 页以卡片网格展示，每个卡片含名称/URL/模型标签/可信度环/审计分段条
5. 点击模型标签筛选数据，不支持的 Provider 显示空状态
6. 添加/编辑 Provider 在模态框中进行，保存后刷新列表
7. Settings 页各设置项正常展示，开关切换有视觉反馈
8. 代理状态按钮实时反映运行状态，可切换
9. 差异率颜色编码正确（<5% 绿 / 5-15% 黄 / >15% 红）
10. Provider 审计分段条正确展示声称 vs 审计的 I/C/O 对比和差异率

---

## 参考

- [Phase 1 Spec](/docs/superpowers/specs/2026-06-09-phase1-core-system.md) — 核心系统架构
- [Phase 2 Spec](/docs/superpowers/specs/2026-06-09-phase2-audit-integration.md) — 审计接入，§Scope 列出了留到 Phase 3 的项
- [UI v0.3 需求文档](/docs/product/ui-v0.3-requirements.md) — 产品需求
- [UI v0.3 HTML Prototype](/docs/product/ui-v0.3-prototype.html) — 完整设计系统和组件原型
