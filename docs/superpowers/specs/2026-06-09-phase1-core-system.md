# Phase 1: 核心系统

**Goal:** 构建一个可用的 Tauri 2 桌面应用，管理 API Provider、代理请求、本地分词计算 token 用量

**Architecture:** Tauri 2 桌面壳 + React 前端 + Rust 后端 Axum 反向代理，SQLite 存储 Provider 配置和审计日志

---

## 核心设计

### Provider 管理

多上游中转站配置管理（名称、URL、API Key、模型列表），支持一键切换激活。切换影响代理转发目标和审计数据隔离。

### 反向代理

本地 Axum HTTP 服务器监听 `0.0.0.0:8080`，接收 Claude Code 请求并转发到当前激活的 Provider。请求原样透传，不做格式转换。

### 本地分词

使用 tiktoken-rs（cl100k_base）对 GPT / Qwen / DeepSeek 模型做精确分词。Claude 和 Gemini 使用近似计算（Phase 2 或 3 改进）。

### 差异对比

本地计算的 token 数与 API 返回的 usage 字段做比较，差异超过阈值（默认 5%）标记为可疑。

### 配置管理

启动代理时修改 `~/.claude/settings.json` 的 `ANTHROPIC_BASE_URL` 指向本地代理，停止时恢复原始值。保留用户已有的模型映射等配置。

### 数据存储

SQLite 存储 Provider 配置、审计记录、缓存状态。数据库文件在 `~/.tokenaccountant/data.db`，用户卸载后数据保留。

### Web 面板

Tauri 桌面应用内的 React 面板：仪表盘（代理状态）、Provider 管理（CRUD + 切换）、请求列表（占位符）。
