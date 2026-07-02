# TokenAccountant

Token 透明化审计工具 —— 代理你的 AI API 请求，实时对比 Provider 声称的 Token 数与本地 Tokenizer 计算的真实值，让每一笔 AI 消费都清晰可见。

## 快速开始

```bash
# 安装依赖
cd frontend && npm install

# 启动开发环境
cd .. && cargo tauri dev
```

## 技术栈

- **前端**：React 18 + TypeScript + Tailwind CSS + Vite
- **后端**：Rust (Tauri v2) + SQLite (rusqlite) + Axum
- **图表**：内联 SVG（无第三方图表库依赖）
- **IPC**：Tauri invoke / event（audit-tick 事件驱动刷新）

## 功能

- **Dashboard**：趋势图、近 7 日统计、今日概览（含模型分布饼图）、Provider 可信度排名、实时审计卡片
- **Providers 管理**：添加/编辑/删除 Provider，一键切换，审计统计条（Prefill + Output）
- **代理审计**：拦截 Claude Code / OpenAI API 请求，对比 claimed vs detected token 数
- **Settings**：代理端口、语言、开发者模式
- **Dev Mode**：开发者调试页面（detect/store 文本对比）

## 架构

```
Claude Code / App
    │
    ▼
TokenAccountant Proxy (Axum)
    │
    ├─ 转发请求 → Provider API
    ├─ Tokenizer 本地计算 detected tokens
    ├─ 对比 claimed vs detected → 写入 SQLite
    └─ emit("audit-tick") → 前端实时刷新
```
