# Phase 3A: Design System + Base Components + Backend — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lay the foundation for Phase 3 frontend redesign: CSS design tokens, shared UI components (Button/Modal/Toggle/TrustRing), and backend aggregation commands.

**Architecture:** CSS variables from HTML prototype → Tailwind extends. React components with controlled props and SVG rendering. Rust backend adds SQL aggregation queries to existing `audit_log` table (no schema changes), exposed via 3 new Tauri commands.

**Tech Stack:** React 18 + TypeScript + Tailwind CSS + Vite + Tauri v2 (Rust IPC) + rusqlite

**Spec ref:** `docs/superpowers/specs/2026-06-09-phase3-ui-ux-redesign.md` §三、§五、§九

---

### Task 1: Create CSS design tokens and Tailwind config

**Files:**
- Create: `frontend/src/styles/design-tokens.css`
- Create: `frontend/src/styles/global.css`
- Modify: `frontend/src/styles.css`
- Modify: `frontend/tailwind.config.js`
- Modify: `frontend/src/main.tsx`

- [ ] **Step 1: Write design-tokens.css**

Create `frontend/src/styles/design-tokens.css`:
```css
:root {
  /* Brand */
  --brand: #4F46E5;
  --brand-light: #6366F1;
  --brand-hover: #4338CA;
  --brand-subtle: rgba(79,70,229,0.06);
  --brand-muted: rgba(79,70,229,0.12);
  --brand-glow: 0 0 0 3px rgba(79,70,229,0.15);

  /* Semantic */
  --success: #059669;
  --success-subtle: #ECFDF5;
  --success-border: #A7F3D0;
  --warning: #D97706;
  --warning-subtle: #FFFBEB;
  --warning-border: #FDE68A;
  --danger: #DC2626;
  --danger-subtle: #FEF2F2;
  --danger-border: #FECACA;
  --info: #2563EB;
  --info-subtle: #EFF6FF;
  --info-border: #BFDBFE;

  /* Gray scale (12 steps) */
  --gray-0: #FFFFFF;
  --gray-50: #F9FAFB;
  --gray-100: #F3F4F6;
  --gray-200: #E5E7EB;
  --gray-300: #D1D5DB;
  --gray-400: #9CA3AF;
  --gray-500: #6B7280;
  --gray-600: #4B5563;
  --gray-700: #374151;
  --gray-800: #1F2937;
  --gray-900: #111827;
  --gray-950: #030712;

  /* I/C/O colors */
  --ico-input: #60A5FA;
  --ico-cache: #FB923C;
  --ico-output: #34D399;

  /* Typography */
  --font-sans: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', system-ui, sans-serif;
  --font-mono: 'SF Mono', 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
  --text-xs: 0.75rem;
  --text-sm: 0.8125rem;
  --text-base: 0.875rem;
  --text-lg: 1rem;
  --text-xl: 1.125rem;
  --text-2xl: 1.375rem;
  --text-3xl: 1.75rem;
  --text-4xl: 2.25rem;
  --leading-tight: 1.2;
  --leading-normal: 1.5;
  --tracking-tight: -0.02em;
  --tracking-normal: -0.01em;
  --tracking-wide: 0.02em;

  /* Spacing */
  --space-1: 0.25rem;  --space-2: 0.5rem;  --space-3: 0.75rem;
  --space-4: 1rem;     --space-5: 1.25rem; --space-6: 1.5rem;
  --space-8: 2rem;     --space-10: 2.5rem; --space-12: 3rem; --space-16: 4rem;

  /* Borders */
  --radius-sm: 6px;  --radius-md: 10px;  --radius-lg: 16px;  --radius-xl: 24px;

  /* Shadows */
  --shadow-xs: 0 1px 2px rgba(0,0,0,0.04);
  --shadow-sm: 0 1px 3px rgba(0,0,0,0.06), 0 1px 2px rgba(0,0,0,0.04);
  --shadow-md: 0 4px 6px -1px rgba(0,0,0,0.06), 0 2px 4px -2px rgba(0,0,0,0.05);
  --shadow-lg: 0 10px 15px -3px rgba(0,0,0,0.08), 0 4px 6px -4px rgba(0,0,0,0.04);
  --shadow-xl: 0 20px 25px -5px rgba(0,0,0,0.08), 0 8px 10px -6px rgba(0,0,0,0.04);
  --shadow-card: 0 1px 3px rgba(0,0,0,0.04), 0 1px 2px rgba(0,0,0,0.03);
  --shadow-modal: 0 25px 50px -12px rgba(0,0,0,0.2);

  /* Transitions */
  --ease-out: cubic-bezier(0.16, 1, 0.3, 1);
  --ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);
  --duration-fast: 150ms;
  --duration-normal: 250ms;
  --duration-slow: 400ms;

  /* Layout */
  --sidebar-w: 220px;
}
```

- [ ] **Step 2: Write global.css**

Create `frontend/src/styles/global.css`:
```css
*, *::before, *::after {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-rendering: optimizeLegibility;
}

body {
  font-family: var(--font-sans);
  font-size: var(--text-base);
  line-height: var(--leading-normal);
  color: var(--gray-800);
  background: var(--gray-50);
}

::-webkit-scrollbar { width: 5px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--gray-300); border-radius: 10px; }

@keyframes page-in {
  from { opacity: 0; transform: translateY(8px); }
  to   { opacity: 1; transform: translateY(0); }
}

@keyframes modal-in {
  from { opacity: 0; transform: scale(0.95) translateY(10px); }
  to   { opacity: 1; transform: scale(1) translateY(0); }
}

@keyframes shimmer {
  0%   { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

- [ ] **Step 3: Update styles.css**

Replace `frontend/src/styles.css`:
```css
@import './styles/design-tokens.css';
@import './styles/global.css';
@tailwind base;
@tailwind components;
@tailwind utilities;
```

- [ ] **Step 4: Update tailwind.config.js**

Add to `frontend/tailwind.config.js`:
```js
module.exports = {
  content: ["./src/**/*.{ts,tsx}"],
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
        ico: { input: '#60A5FA', cache: '#FB923C', output: '#34D399' },
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
  plugins: [],
};
```

- [ ] **Step 5: Update main.tsx to import styles**

Verify `frontend/src/main.tsx` imports `./styles.css` (it should already). If not, add:
```tsx
import './styles.css'
```

- [ ] **Step 6: Build check**

Run: `cd frontend && npx tsc --noEmit && npx vite build 2>&1 | tail -5`
Expected: No errors, build succeeds.

- [ ] **Step 7: Commit**

```bash
git add -A && git commit -m "feat: add design tokens, global styles, tailwind config for v0.3 UI"
```

---

### Task 2: Button component

**Files:**
- Create: `frontend/src/components/Button.tsx`

- [ ] **Step 1: Write Button.tsx**

Create `frontend/src/components/Button.tsx`:
```tsx
import { type ButtonHTMLAttributes } from 'react'

type Variant = 'primary' | 'secondary' | 'danger' | 'ghost'
type Size = 'sm' | 'md' | 'lg'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant
  size?: Size
}

const variantStyles: Record<Variant, string> = {
  primary:   'bg-brand text-white shadow-[0_1px_2px_rgba(79,70,229,0.2)] hover:bg-brand-hover hover:shadow-[0_4px_8px_rgba(79,70,229,0.25)] hover:-translate-y-px',
  secondary: 'bg-gray-100 text-gray-700 hover:bg-gray-200 hover:text-gray-900',
  danger:    'bg-danger-subtle text-danger hover:bg-danger hover:text-white',
  ghost:     'text-gray-500 hover:bg-gray-100 hover:text-gray-800',
}

const sizeStyles: Record<Size, string> = {
  sm: 'px-3 py-1 text-xs min-h-[28px] rounded-[20px]',
  md: 'px-4 py-2 text-sm min-h-[36px] rounded-[6px]',
  lg: 'px-6 py-3 text-base min-h-[44px] rounded-[10px]',
}

export default function Button({
  variant = 'primary',
  size = 'md',
  className = '',
  children,
  ...props
}: ButtonProps) {
  return (
    <button
      className={`inline-flex items-center justify-center gap-2 font-semibold transition-all duration-150 ease-out active:scale-[0.98] whitespace-nowrap relative overflow-hidden ${variantStyles[variant]} ${sizeStyles[size]} ${className}`}
      {...props}
    >
      {children}
    </button>
  )
}
```

- [ ] **Step 2: Build check**

Run: `cd frontend && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: add Button component with 4 variants"
```

---

### Task 3: Modal component

**Files:**
- Create: `frontend/src/components/Modal.tsx`

- [ ] **Step 1: Write Modal.tsx**

Create `frontend/src/components/Modal.tsx`:
```tsx
import { useEffect, useCallback, type ReactNode } from 'react'

interface ModalProps {
  open: boolean
  onClose: () => void
  title: string
  children: ReactNode
  footer?: ReactNode
}

export default function Modal({ open, onClose, title, children, footer }: ModalProps) {
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Escape') onClose()
  }, [onClose])

  useEffect(() => {
    if (open) {
      document.addEventListener('keydown', handleKeyDown)
      document.body.style.overflow = 'hidden'
    }
    return () => {
      document.removeEventListener('keydown', handleKeyDown)
      document.body.style.overflow = ''
    }
  }, [open, handleKeyDown])

  if (!open) return null

  return (
    <div
      className="fixed inset-0 bg-black/40 backdrop-blur-sm z-[1000] flex items-center justify-center animate-[fade-in_150ms_ease-out]"
      onClick={(e) => { if (e.target === e.currentTarget) onClose() }}
    >
      <div
        className="bg-white rounded-xl shadow-modal max-w-[640px] w-[calc(100%-48px)] max-h-[85vh] overflow-y-auto animate-[modal-in_250ms_ease-out]"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="px-6 py-5 border-b border-gray-100 flex items-center justify-between sticky top-0 bg-white z-10 rounded-t-xl">
          <h2 className="text-xl font-bold text-gray-900 tracking-tight">{title}</h2>
          <button
            onClick={onClose}
            className="w-8 h-8 rounded-[6px] flex items-center justify-center text-gray-400 hover:bg-gray-100 hover:text-gray-800 transition-all duration-150"
            aria-label="关闭"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>

        {/* Body */}
        <div className="px-6 py-6">
          {children}
        </div>

        {/* Footer */}
        {footer && (
          <div className="px-6 py-4 border-t border-gray-100 bg-gray-50 flex justify-end gap-3 rounded-b-xl">
            {footer}
          </div>
        )}
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Build check**

Run: `cd frontend && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: add Modal component with overlay, animation, keyboard support"
```

---

### Task 4: Toggle component

**Files:**
- Create: `frontend/src/components/Toggle.tsx`

- [ ] **Step 1: Write Toggle.tsx**

Create `frontend/src/components/Toggle.tsx`:
```tsx
interface ToggleProps {
  checked: boolean
  onChange: (checked: boolean) => void
}

export default function Toggle({ checked, onChange }: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className={`relative w-11 h-6 rounded-full transition-all duration-150 ease-out shrink-0 ${
        checked ? 'bg-brand' : 'bg-gray-300'
      }`}
    >
      <span
        className={`block w-5 h-5 bg-white rounded-full shadow-[0_1px_3px_rgba(0,0,0,0.2)] transition-all duration-150 ease-[cubic-bezier(0.34,1.56,0.64,1)] ${
          checked ? 'translate-x-[22px]' : 'translate-x-[2px]'
        }`}
      />
    </button>
  )
}
```

- [ ] **Step 2: Build check**

Run: `cd frontend && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: add Toggle component with spring animation"
```

---

### Task 5: TrustRing SVG component

**Files:**
- Create: `frontend/src/components/TrustRing.tsx`

- [ ] **Step 1: Write TrustRing.tsx**

Create `frontend/src/components/TrustRing.tsx`:
```tsx
interface TrustRingProps {
  value: number   // 0-100
  size?: 'sm' | 'md'
}

export default function TrustRing({ value, size = 'md' }: TrustRingProps) {
  const w = size === 'sm' ? 34 : 56
  const strokeWidth = size === 'sm' ? 3 : 4
  const r = (w - strokeWidth) / 2
  const circ = 2 * Math.PI * r
  const offset = circ - (value / 100) * circ
  const color = value >= 90 ? '#059669' : value >= 70 ? '#D97706' : '#DC2626'
  const cls = value >= 90 ? 'text-[#059669]' : value >= 70 ? 'text-[#D97706]' : 'text-[#DC2626]'

  return (
    <div className={`relative shrink-0 ${size === 'sm' ? 'w-[34px] h-[34px]' : 'w-[56px] h-[56px]'}`}>
      <svg viewBox={`0 0 ${w} ${w}`} className="w-full h-full -rotate-90">
        <circle cx={w / 2} cy={w / 2} r={r} fill="none" stroke="#E5E7EB" strokeWidth={strokeWidth} />
        <circle
          cx={w / 2} cy={w / 2} r={r} fill="none" stroke={color}
          strokeWidth={strokeWidth} strokeLinecap="round"
          strokeDasharray={circ} strokeDashoffset={offset}
          style={{ transition: 'stroke-dashoffset 0.5s ease' }}
        />
      </svg>
      <span className={`absolute inset-0 flex items-center justify-center text-[${size === 'sm' ? '11px' : '16px'}] font-extrabold leading-none tracking-tight ${cls}`}>
        {value}
      </span>
    </div>
  )
}
```

- [ ] **Step 2: Build check**

Run: `cd frontend && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: add TrustRing SVG component with color coding"
```

---

### Task 6: Backend — add SQL aggregation queries

**Files:**
- Modify: `src-tauri/src/storage/database.rs`
- Modify: `src-tauri/src/provider/types.rs` (or `provider/manager.rs`)

- [ ] **Step 1: Add AuditSummary types to provider types**

Add to `src-tauri/src/provider/types.rs`:
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct AuditSummary {
    pub input_claimed: i64,
    pub input_detected: i64,
    pub cache_claimed: i64,
    pub cache_detected: i64,
    pub output_claimed: i64,
    pub output_detected: i64,
}
```

- [ ] **Step 2: Add credibility and audit_summary fields to Provider**

In `src-tauri/src/provider/types.rs`, add to existing `Provider` struct:
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Provider {
    // ... existing fields unchanged ...
    
    // Phase 3: aggregated from audit_log, not stored in DB
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

- [ ] **Step 3: Add list_providers_with_stats and get_provider_detail methods**

Add to `src-tauri/src/storage/database.rs`:

```rust
use crate::provider::types::{AuditSummary, PerModelAudit};

impl Database {
    /// Provider 列表，每条附带审计汇总数据
    pub fn list_providers_with_stats(&self) -> Result<Vec<crate::provider::types::Provider>> {
        let mut providers = self.list_providers()?;
        for p in &mut providers {
            let stats = self.conn.query_row(
                "SELECT
                    COUNT(*) as total,
                    SUM(CASE WHEN is_suspicious=1 THEN 1 ELSE 0 END) as susp,
                    COALESCE(AVG(claimed_input_tokens - real_input_tokens), 0) as avg_in_diff,
                    COALESCE(AVG(claimed_cached_tokens - detected_cached_tokens), 0) as avg_ca_diff,
                    COALESCE(AVG(claimed_output_tokens - real_output_tokens), 0) as avg_out_diff,
                    COALESCE(SUM(claimed_input_tokens), 0),
                    COALESCE(SUM(real_input_tokens), 0),
                    COALESCE(SUM(claimed_cached_tokens), 0),
                    COALESCE(SUM(detected_cached_tokens), 0),
                    COALESCE(SUM(claimed_output_tokens), 0),
                    COALESCE(SUM(real_output_tokens), 0)
                FROM audit_log WHERE provider_id=?1",
                rusqlite::params![p.id],
                |row| {
                    let avg_in: f64 = row.get(3)?;
                    let avg_ca: f64 = row.get(4)?;
                    let avg_out: f64 = row.get(5)?;
                    let avg_all = (avg_in.abs() + avg_ca.abs() + avg_out.abs()) / 3.0;
                    let cred = (100.0 - avg_all * 10.0).clamp(0.0, 100.0);
                    Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, cred, AuditSummary {
                        input_claimed: row.get(6)?,  input_detected: row.get(7)?,
                        cache_claimed: row.get(8)?,  cache_detected: row.get(9)?,
                        output_claimed: row.get(10)?, output_detected: row.get(11)?,
                    }))
                },
            ).unwrap_or((0, 0, 0.0, AuditSummary::default()));

            p.total_requests = Some(stats.0);
            p.suspicious_requests = Some(stats.1);
            p.credibility = Some(stats.2);
            p.audit_summary = Some(stats.3);
        }
        Ok(providers)
    }

    /// 单个 Provider 按模型的审计数据（用户在 Providers 页点模型标签时调）
    pub fn get_provider_detail(
        &self, provider_id: &str, model: &str
    ) -> Result<Option<CurrentAuditEntry>> {
        // 返回该 Provider+Model 的最新审计记录作为详情
        let mut stmt = self.conn.prepare(
            "SELECT real_input_tokens, claimed_input_tokens,
                    detected_cached_tokens, claimed_cached_tokens,
                    real_output_tokens, claimed_output_tokens,
                    input_diff, cache_diff, output_diff, is_suspicious
             FROM audit_log WHERE provider_id=?1 AND model=?2 ORDER BY id DESC LIMIT 1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![provider_id, model], |row| {
            Ok(CurrentAuditEntry {
                audit_passed: row.get::<_, i32>(9)? == 0,
                input_audit: row.get(0)?,  input_claimed: row.get(1)?,  input_diff: row.get(6)?,
                cache_audit: row.get(2)?,  cache_claimed: row.get(3)?,  cache_diff: row.get(7)?,
                output_audit: row.get(4)?, output_claimed: row.get(5)?, output_diff: row.get(8)?,
            })
        })?;
        match rows.next() {
            Some(Ok(r)) => Ok(Some(r)),
            _ => Ok(None),
        }
    }

    /// Dashboard 聚合数据（1 次事务查完）
    pub fn get_dashboard_data(&self) -> Result<DashboardData> {
        use std::time::Instant;

        let proxy_status = crate::proxy::types::ProxyStatus {
            running: false,
            port: 0,
            uptime_secs: 0,
            requests_served: self.conn
                .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get::<_, i64>(0))
                .unwrap_or(0) as u64,
        };

        // 今日汇总
        let today = self.conn.query_row(
            "SELECT
                COUNT(*),
                SUM(CASE WHEN is_suspicious=1 THEN 1 ELSE 0 END),
                COALESCE(SUM(claimed_input_tokens), 0),
                COALESCE(SUM(real_input_tokens), 0),
                COALESCE(SUM(claimed_cached_tokens), 0),
                COALESCE(SUM(detected_cached_tokens), 0),
                COALESCE(SUM(claimed_output_tokens), 0),
                COALESCE(SUM(real_output_tokens), 0)
            FROM audit_log WHERE timestamp >= datetime('now', '-1 day')",
            [], |row| {
                let tot: i64 = row.get(0)?;
                let susp: i64 = row.get(1)?;
                let ci: i64 = row.get(2)?; let di: i64 = row.get(3)?;
                let cc: i64 = row.get(4)?; let dc: i64 = row.get(5)?;
                let co: i64 = row.get(6)?; let d_o: i64 = row.get(7)?;
                let i_rate = if ci > 0 { (ci - di) as f64 / ci as f64 * 100.0 } else { 0.0 };
                let c_rate = if cc > 0 { (cc - dc) as f64 / cc as f64 * 100.0 } else { 0.0 };
                let o_rate = if co > 0 { (co - d_o) as f64 / co as f64 * 100.0 } else { 0.0 };
                Ok(TodaySummary {
                    total_requests: tot, suspicious_requests: susp,
                    input_diff_rate: i_rate, cache_diff_rate: c_rate, output_diff_rate: o_rate,
                    input_diff_tokens: ci - di, cache_diff_tokens: cc - dc, output_diff_tokens: co - d_o,
                    input_match_rate: (100.0 - i_rate).max(0.0),
                    cache_match_rate: (100.0 - c_rate).max(0.0),
                    output_match_rate: (100.0 - o_rate).max(0.0),
                })
            },
        ).unwrap_or_default();

        // 近 20 条趋势数据（仅 token 字段，不含 request_preview 大字段）
        let mut trend_stmt = self.conn.prepare(
            "SELECT id, claimed_input_tokens, real_input_tokens,
                    claimed_cached_tokens, detected_cached_tokens,
                    claimed_output_tokens, real_output_tokens
             FROM audit_log ORDER BY id DESC LIMIT 20"
        )?;
        let trend: Vec<TrendPoint> = trend_stmt.query_map([], |row| {
            Ok(TrendPoint {
                idx: row.get(0)?,
                input_claimed: row.get(1)?, input_detected: row.get(2)?,
                cache_claimed: row.get(3)?, cache_detected: row.get(4)?,
                output_claimed: row.get(5)?, output_detected: row.get(6)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        // Provider 排名 top 5
        let ranking = self.list_providers_with_stats()?
            .into_iter()
            .filter(|p| p.credibility.unwrap_or(0.0) > 0.0)
            .take(5)
            .map(|p| ProviderRankEntry {
                id: p.id, name: p.name, url: p.api_base_url,
                credibility: p.credibility.unwrap_or(0.0),
                input_diff_rate: 0.0, cache_diff_rate: 0.0, output_diff_rate: 0.0,
            })
            .collect();

        // 最新一条审计
        let latest = self.conn.query_row(
            "SELECT real_input_tokens, claimed_input_tokens,
                    detected_cached_tokens, claimed_cached_tokens,
                    real_output_tokens, claimed_output_tokens,
                    input_diff, cache_diff, output_diff, is_suspicious
             FROM audit_log ORDER BY id DESC LIMIT 1",
            [], |row| {
                Ok(CurrentAuditEntry {
                    audit_passed: row.get::<_, i32>(9)? == 0,
                    input_audit: row.get(0)?,  input_claimed: row.get(1)?,  input_diff: row.get(6)?,
                    cache_audit: row.get(2)?,  cache_claimed: row.get(3)?,  cache_diff: row.get(7)?,
                    output_audit: row.get(4)?, output_claimed: row.get(5)?, output_diff: row.get(8)?,
                })
            },
        ).ok();

        Ok(DashboardData {
            proxy_status,
            today_summary: today,
            trend_data: trend,
            provider_ranking: ranking,
            current_audit: latest,
        })
    }
}
```

- [ ] **Step 4: Define DashboardData types**

Add to `src-tauri/src/api/commands.rs` (or a new `src-tauri/src/api/types.rs`):
```rust
#[derive(Debug, serde::Serialize, Default)]
pub struct TodaySummary {
    pub total_requests: i64,
    pub suspicious_requests: i64,
    pub input_diff_rate: f64, pub cache_diff_rate: f64, pub output_diff_rate: f64,
    pub input_diff_tokens: i64, pub cache_diff_tokens: i64, pub output_diff_tokens: i64,
    pub input_match_rate: f64, pub cache_match_rate: f64, pub output_match_rate: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct TrendPoint {
    pub idx: i64,
    pub input_claimed: i64,  pub input_detected: i64,
    pub cache_claimed: i64,  pub cache_detected: i64,
    pub output_claimed: i64, pub output_detected: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct ProviderRankEntry {
    pub id: String, pub name: String, pub url: String,
    pub credibility: f64,
    pub input_diff_rate: f64, pub cache_diff_rate: f64, pub output_diff_rate: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct CurrentAuditEntry {
    pub audit_passed: bool,
    pub input_audit: i64,  pub input_claimed: i64,  pub input_diff: i64,
    pub cache_audit: i64,  pub cache_claimed: i64,  pub cache_diff: i64,
    pub output_audit: i64, pub output_claimed: i64, pub output_diff: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct DashboardData {
    pub proxy_status: crate::proxy::types::ProxyStatus,
    pub today_summary: TodaySummary,
    pub trend_data: Vec<TrendPoint>,
    pub provider_ranking: Vec<ProviderRankEntry>,
    pub current_audit: Option<CurrentAuditEntry>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct AppConfigPayload {
    pub proxy_port: u16,
    pub language: String,
    pub auto_configure_claude: bool,
    pub auto_start_proxy: bool,
    pub suspicion_threshold: f64,
    pub record_full_body: bool,
    pub dev_mode_enabled: bool,
    pub dev_trace_buffer_size: u32,
}
```

- [ ] **Step 5: Add new Tauri commands**

Add to `src-tauri/src/api/commands.rs`:
```rust
#[tauri::command]
pub async fn get_dashboard_data(
    state: State<'_, TauriState>,
) -> Result<DashboardData, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_dashboard_data().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_provider_detail(
    state: State<'_, TauriState>,
    provider_id: String,
    model: String,
) -> Result<Option<AuditRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_provider_detail(&provider_id, &model).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_config(
    state: State<'_, TauriState>,
) -> Result<AppConfigPayload, String> {
    let cfg = &state.config;
    Ok(AppConfigPayload {
        proxy_port: cfg.server.bind_addr.split(':').last()
            .and_then(|s| s.parse().ok()).unwrap_or(8080),
        language: "zh-CN".into(),
        auto_configure_claude: true,
        auto_start_proxy: false,
        suspicion_threshold: cfg.audit.suspicion_threshold,
        record_full_body: false,
        dev_mode_enabled: false,
        dev_trace_buffer_size: 100,
    })
}

#[tauri::command]
pub async fn save_app_config(
    state: State<'_, TauriState>,
    config: AppConfigPayload,
) -> Result<(), String> {
    let mut cfg = state.config.clone();
    cfg.server.bind_addr = format!("0.0.0.0:{}", config.proxy_port);
    cfg.audit.suspicion_threshold = config.suspicion_threshold;
    crate::config::app_config::save_config(&cfg).map_err(|e| e.to_string())
}
```

- [ ] **Step 6: Register new commands in lib.rs**

In `src-tauri/src/lib.rs`, add to `.invoke_handler(tauri::generate_handler![...])`:
```rust
crate::api::commands::get_dashboard_data,
crate::api::commands::get_provider_detail,
crate::api::commands::get_app_config,
crate::api::commands::save_app_config,
```

- [ ] **Step 7: Change list_providers to return stats**

In `src-tauri/src/api/commands.rs`, modify `list_providers`:
```rust
pub async fn list_providers(state: State<'_, TauriState>) -> Result<Vec<Provider>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_providers_with_stats().map_err(|e| e.to_string())
}
```

- [ ] **Step 8: Build check**

Run: `cd src-tauri && cargo check 2>&1 | tail -15`
Expected: `Compiling tokenaccountant v0.1.0 ... Finished`

- [ ] **Step 9: Commit**

```bash
git add -A && git commit -m "feat: add dashboard aggregation queries, provider stats, config commands"
```
