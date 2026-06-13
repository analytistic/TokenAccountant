# CC Switch 架构与实现调研

> 调研日期：2026-06-09
> 项目版本：CC Switch v3.12.3 / cc-switch-cli
> 目的：为 TokenAccountant 架构设计提供参考

## 1. 项目概况

CC Switch 是一个跨平台的桌面应用（Tauri 2 + React），核心功能是为 Claude Code、Codex、Gemini CLI 等 AI 编程工具提供统一的 **Provider 配置管理** 和 **本地 API 代理**。

### 核心能力

| 能力 | 说明 |
|------|------|
| Provider 管理 | 50+ 预设 Provider，一键切换，支持拖拽排序 |
| 本地 API 代理 | Axum 反向代理，每个 App 独立端口，支持流式 |
| 自动故障转移 | 熔断器 + 健康检查 + 自动切换到备用 Provider |
| 格式转换 | 在 OpenAI ↔ Anthropic ↔ Gemini 格式之间转换 |
| 用量追踪 | 解析 API 响应中的 usage 字段，计算费用 |
| MCP/技能管理 | 统一管理 MCP 服务器、System Prompt、技能 |

### 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | React 18, TypeScript, Vite, TailwindCSS, TanStack Query |
| 桌面壳 | Tauri 2.8 |
| 后端 | Rust, Axum, tokio, serde |
| 存储 | SQLite (`~/.cc-switch/cc-switch.db`) + JSON (设备级配置) |

---

## 2. 架构分层

### 2.1 总体架构

```
┌────────────────────────────────────────────────────────────┐
│                    CC Switch App                           │
│  ┌─────────────┐    ┌──────────────┐    ┌───────────────┐ │
│  │ React 前端   │◄──►│ Tauri IPC    │◄──►│ Rust 后端      │ │
│  │ (GUI/TUI)    │    │ (commands)   │    │ (services)    │ │
│  └─────────────┘    └──────────────┘    └───────┬───────┘ │
│                                                  │         │
│  ┌───────────────────────────────────────────────┴───────┐ │
│  │              Proxy 子系统 (Axum)                       │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────────┐    │ │
│  │  │ Handler  │─►│ Forwarder│─►│ ResponseHandler   │    │ │
│  │  │ (路由分发)│  │ (转发+重试)│  │ (响应处理+日志)    │    │ │
│  │  └──────────┘  └──────────┘  └──────────────────┘    │ │
│  └───────────────────────────────────────────────────────┘ │
│                           │                                 │
│                           ▼                                 │
│                    ┌──────────────┐                         │
│                    │  上游 Provider│                         │
│                    │  (Anthropic/ │                         │
│                    │   OpenAI/    │                         │
│                    │   中转站等)   │                         │
│                    └──────────────┘                         │
└────────────────────────────────────────────────────────────┘
```

### 2.2 后端模块结构

```
src-tauri/src/
├── main.rs              # 入口
├── lib.rs               # 库入口
├── provider.rs           # Provider 数据模型 (settings_config 包含 ANTHROPIC_BASE_URL 等)
├── provider_defaults.rs  # Provider 预设
├── settings.rs           # CC Switch 自身设置 (~/.cc-switch/settings.json)
├── config.rs             # 通用配置读写工具 (read_json_file, write_json_file)
├── store.rs              # 应用状态 (AppState)
│
├── proxy/                # ★ 代理子系统 (核心)
│   ├── mod.rs
│   ├── server.rs         # Axum 服务器 (启动/停止/路由注册)
│   ├── handlers.rs       # HTTP 处理器 (按 API 格式分发)
│   ├── forwarder.rs      # 请求转发 (重试/熔断/超时)
│   ├── response_handler.rs # 响应处理 (流式/buffer 两种路径)
│   ├── types.rs          # 代理配置和状态类型
│   ├── sse.rs            # SSE 流式处理
│   ├── provider_router.rs # Provider 路由选择
│   ├── circuit_breaker.rs # 熔断器
│   ├── model_mapper.rs   # 模型映射
│   ├── body_filter.rs    # 请求体过滤
│   ├── cache_injector.rs # 缓存注入
│   ├── http_client.rs    # HTTP 客户端
│   ├── session.rs        # 会话管理
│   ├── handler_context.rs # Handler 上下文
│   ├── usage/            # 用量解析
│   │   ├── mod.rs
│   │   ├── parser.rs     # Token usage 解析 (Claude/OpenAI/Gemini)
│   │   ├── calculator.rs # 费用计算
│   │   └── logger.rs     # 用量日志
│   └── ... (format 转换相关模块)
│
├── services/              # 业务逻辑层
│   ├── config.rs          # ★ 配置同步 (写入 CLI 工具的 settings.json)
│   ├── provider.rs        # Provider 服务 (build_effective_live_snapshot)
│   ├── proxy.rs           # 代理服务
│   ├── env_manager.rs     # 环境变量管理
│   └── ...
│
├── commands/              # Tauri 命令处理器
├── database/              # SQLite 数据访问
├── daemon/                # 后台守护进程
└── session_manager/       # 会话管理
```

---

## 3. ★ settings.json 修改机制 (关键)

这是 TokenAccountant 最需要参考的部分。CC Switch 通过修改 CLI 工具的配置文件，使其将 API 流量指向本地代理。

### 3.1 修改哪些文件

| CLI 工具 | 配置文件路径 | 配置方式 |
|----------|------------|----------|
| Claude Code | `~/.claude/settings.json` | 写入 `env` 字段 (ANTHROPIC_BASE_URL) |
| Codex | `~/.codex/config.toml` | 写入 TOML 格式配置 |
| Gemini CLI | `~/.gemini/.env` | 写入环境变量 |

### 3.2 Claude Code settings.json 修改流程

核心代码在 `services/config.rs` 的 `sync_claude_live()` 中：

```
sync_claude_live(provider):
  1. 解析配置路径: get_claude_settings_path() → ~/.claude/settings.json
  2. 创建父目录: create_dir_all
  3. 获取通用配置片段: config.common_config_snippets.claude
  4. 构建有效快照: ProviderService::build_effective_live_snapshot(AppType::Claude, provider, snippet)
  5. 写入文件: write_json_file(settings_path, effective_snapshot)
  6. 回读验证: read_json_file(settings_path)
  7. 标准化存储: normalize_settings_config_for_storage
```

**生成的 settings.json 结构**：

```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://localhost:{proxy_port}",
    "ANTHROPIC_AUTH_TOKEN": "sk-ant-...",
    "ANTHROPIC_COOKIES": "..."
  }
}
```

### 3.3 关键设计点

1. **写入即生效**：Claude Code 启动时读取一次 settings.json，修改后需要重启
2. **原子写入**：CC Switch 使用临时文件 + rename 模式，防止写损坏
3. **回读验证**：写入后立即读取，确保文件可解析
4. **通用配置片段**：允许多个 Provider 共享公共配置（如代理地址模板）
5. **安全权限**：Unix 上设置 `0o600` 文件权限保护 API Key
6. **多设备隔离**：设备级配置存本地 JSON，不与云端同步的 DB 冲突

### 3.4 vs Codex / Gemini

- **Codex**：调用 `codex_config::write_codex_provider_live_*`，写入 TOML 格式的 `config.toml`
- **Gemini**：调用 `ProviderService::write_gemini_live_force`，写入 `.env` 文件

---

## 4. Proxy 子系统详解

### 4.1 服务器架构

`ProxyServer` 是代理的核心，状态通过 `Arc<RwLock<ProxyServerState>>` 共享：

```rust
struct ProxyServerState {
    db: Database,
    config: ProxyConfig,            // 监听地址/端口
    status: ProxyStatus,            // 运行时状态 (连接数/token数/成功率)
    current_providers: HashMap<AppType, (ProviderId, ProviderName)>,
    provider_router: ProviderRouter, // 熔断器 + Provider 选择
}
```

### 4.2 路由分发

Axum 路由注册：

| 端点 | App 类型 | Handler |
|------|----------|---------|
| `/v1/messages`, `/claude/v1/messages` | Claude | `handle_messages` |
| `/v1/chat/completions`, `/chat/completions` | OpenAI/Codex | `handle_chat_completions` |
| `/v1/responses`, `/responses` | Codex Responses API | `handle_responses` |
| `/v1beta/*path`, `/gemini/v1beta/*path` | Gemini | `handle_gemini` |

### 4.3 Handler 处理流程

所有 handler 遵循 **三段式模式**：

```
Phase 1: 上下文加载
  HandlerContext::load(&state, app_type, &headers, &body)
  → 确定 provider、应用 optimizer、解析 model

Phase 2: 转发决策
  request_is_streaming()?  → 流式路径 / buffer 路径

Phase 3: 响应转换
  流式: forward_response_detailed → 格式转换 → SSE 流
  buffer: forward_buffered_response_detailed → 格式转换 → JSON
```

### 4.4 流式处理

SSE (Server-Sent Events) 是代理的核心挑战。CC Switch 的做法：

1. **`StreamLogCollector`**：实时收集 SSE chunk，按 `\n\n` 分割事件
2. **流式转发**：使用 `async_stream` 逐 chunk 转发，不等待完整响应
3. **完成检测**：监控特定事件（如 `message_delta`、`response.completed`）判断流结束
4. **Token 提取**：从 `message_start` 提取 input tokens，从 `message_delta` 提取 output tokens

### 4.5 熔断器

`circuit_breaker.rs` 实现了标准的熔断模式：

- **Closed**：正常转发，记录失败次数
- **Open**：失败超过阈值，直接拒绝请求
- **Half-Open**：超时后允许试探请求，成功则恢复

---

## 5. Usage 追踪

### 5.1 Token 数据流

```
API 响应到达
      │
      ▼
response_handler.rs 接收 (流式或 buffer)
      │
      ▼
parser.rs 解析 usage 字段
  ├─ from_claude_response / from_claude_stream_events
  ├─ from_codex_response_auto / from_codex_stream_events_auto
  └─ from_gemini_response / from_gemini_stream_chunks
      │
      ▼
TokenUsage { input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens }
      │
      ▼
calculator.rs 计算费用 (按 model 定价)
      │
      ▼
记录到 DB (usage 表)
```

### 5.2 支持的 API 格式

**Claude (非流式)**
```json
{
  "usage": {
    "input_tokens": 100,
    "output_tokens": 50,
    "cache_read_input_tokens": 80,
    "cache_creation_input_tokens": 20
  }
}
```

**Claude (流式)**
- `message_start` 事件 → input tokens + cache tokens
- `message_delta` 事件 → output tokens

**OpenAI**
```json
{
  "usage": {
    "prompt_tokens": 100,
    "completion_tokens": 50,
    "prompt_tokens_details": { "cached_tokens": 80 }
  }
}
```

**Gemini**
```json
{
  "usageMetadata": {
    "promptTokenCount": 100,
    "candidatesTokenCount": 50,
    "cachedContentTokenCount": 80
  }
}
```

### 5.3 局限性

CC Switch 的 token 追踪是 **被动** 的——直接读取 API 返回的 `usage` 字段，不进行独立计算。这意味着：
- 如果中转站虚报 token 数，CC Switch 无法检测
- 它信任上游返回的 token 计数

这正是 **TokenAccountant 要解决的差异化问题**。

---

## 6. 与 TokenAccountant 的对比

| 维度 | CC Switch | TokenAccountant |
|------|-----------|-----------------|
| **核心定位** | Provider 配置管理 + 代理 | Token 用量审计 + 检测 |
| **Token 计算** | 被动读取 API 返回的 usage | 主动本地计算，对比差值 |
| **缓存检测** | 无（直接读 cache_read 字段） | 独立前缀匹配检测 |
| **Provider 管理** | 50+ 预设，一键切换 | 多上游 CRUD + 一键切换（联动审计） |
| **格式转换** | 支持（OpenAI↔Anthropic） | 不需要（透传不转换） |
| **熔断/故障转移** | 完整实现 | 不需要 |
| **Web 面板** | 完整 Tauri 桌面应用 | Tauri 桌面应用（React 面板） |
| **配置存储** | SQLite DB + JSON | SQLite（providers + 审计 + 缓存状态） |
| **代理端口** | 每个 App 独立端口 | 单端口代理 |

---

## 7. 值得 TokenAccountant 借鉴的模式

1. **settings.json 修改模式**：原子写入 + 回读验证 + 安全权限
2. **Axum 反向代理架构**：三层结构（Handler → Forwarder → ResponseHandler）
3. **SSE 流式处理**：StreamLogCollector 设计模式
4. **API 格式识别**：从 URL 路径和请求体字段双重判断
5. **模块化组织**：proxy/usage/parser 与 calculator 分离
6. **App 类型泛化**：用枚举统一管理不同 API 格式
