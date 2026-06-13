# TokenAccountant 架构设计

> 参考实现调研：[CC Switch 架构调研](./cc-switch-research.md)

## 项目目标

构建一个开源工具，帮助用户检测 API 中转站是否虚报 token 用量。工具作为本地 HTTP 反向代理，整合 **Provider 管理**（类似 CC Switch）和 **Token 审计** 两大能力——用户可管理多个上游中转站配置、一键切换激活，同时工具独立计算 token 消耗并与中转站声称的值做对比，通过 Web 面板呈现差异。

**与 CC Switch 的核心差异**：CC Switch 是 Provider 配置管理工具，token 追踪是被动的（读取 API 返回的 usage）。TokenAccountant 将 Provider 管理与 **主动审计** 相结合——本地分词计算、缓存命中检测、差异对比，让用户不仅能切换 Provider，还能验证 Provider 是否可信。

---

## 核心需求

1. **Provider 管理**：支持管理多个上游中转站配置（名称、URL、API Key、模型映射），一键切换激活
2. **本地反向代理**：启动后修改 CLI 工具配置，将 API 流量路由到本地代理，并根据当前激活的 Provider 转发到对应中转站
3. **多模型支持**：自动识别模型类型，使用对应的分词器
4. **独立 Token 计算**：本地计算 prefill（input）和 decode（output）的 token 数
5. **缓存命中检测**：通过 Token IDs 最长前缀匹配，独立检测缓存命中情况
6. **差异对比**：本地计算 vs 中转站声称，标记异常
7. **Web 监控面板**：实时查看统计数据和差异报告，同时管理 Provider 配置
8. **持久化**：SQLite 存储 Provider 配置、审计记录、缓存状态，支持跨会话数据持久

---

## 总体架构

```
┌──────────────────────────────────────────────────────────────────────┐
│                         TokenAccountant                              │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    CLI 入口                                    │   │
│  │  tokenaccountant start     → 启动代理 + Web 服务               │   │
│  │  tokenaccountant provider  → 管理上游中转站配置                 │   │
│  │  tokenaccountant status    → 查看运行状态                       │   │
│  └───────────┬──────────────────────────────────────────────────┘   │
│              │                                                       │
│  ┌───────────▼──────────────────────────────────────────────────┐   │
│  │              Provider 管理 (ProviderManager)                   │   │
│  │  ┌──────────────────┐  ┌──────────────────────────────────┐   │   │
│  │  │ 多上游配置        │  │ 一键切换 + 健康检查               │   │   │
│  │  │ (URL/API Key/模型)│  │ (类比 CC Switch 的切换能力)      │   │   │
│  │  └────────┬─────────┘  └────────┬─────────────────────────┘   │   │
│  │           │                      │                              │   │
│  │  ┌────────▼──────────────────────▼──────────────────────────┐   │   │
│  │  │              ConfigManager                                │   │   │
│  │  │  ┌──────────────────┐  ┌────────────────────────────┐  │   │   │
│  │  │  │ settings.json    │  │ config.toml                │  │   │   │
│  │  │  │ (修改 CLI 配置)   │  │ (TokenAccountant 自身配置)  │  │   │   │
│  │  │  └──────────────────┘  └────────────────────────────┘  │   │   │
│  │  └─────────────────────────────────────────────────────────┘   │   │
│  └──────────────────────────┬─────────────────────────────────────┘   │
│                             │                                         │
│  ┌──────────────────────────▼─────────────────────────────────────┐   │
│  │              Proxy 子系统 (Axum)                                │   │
│  │                                                                │   │
│  │  ┌───────────────┐    ┌───────────────┐    ┌───────────────┐  │   │
│  │  │ 请求拦截 Handler├───►│ Provider路由  ├───►│ 转发 Forwarder│  │   │
│  │  │ (/v1/*)       │    │ (当前激活)    │    │ (透传+流式)   │  │   │
│  │  └───────┬───────┘    └───────────────┘    └───────┬───────┘  │   │
│  │          │                                          │           │   │
│  │  ┌───────▼──────────────────────────────────────────▼───────┐   │   │
│  │  │               响应处理 ResponseHandler                      │   │   │
│  │  │  ┌───────────┐  ┌──────────────────┐  ┌──────────────┐   │   │   │
│  │  │  │ 流式处理   │  │ 非流式处理        │  │ 触发审计     │   │   │   │
│  │  │  │ (SSE)     │  │ (JSON)           │  │ (异步)       │   │   │   │
│  │  │  └───────────┘  └──────────────────┘  └──────────────┘   │   │   │
│  │  └──────────────────────────┬───────────────────────────────┘   │   │
│  └─────────────────────────────┼───────────────────────────────────┘   │
│                                │                                        │
│  ┌─────────────────────────────▼──────────────────────────────────┐   │
│  │                    Token 审计引擎                               │   │
│  │                                                               │   │
│  │  ┌────────────────┐  ┌─────────────────┐  ┌──────────────┐   │   │
│  │  │ Tokenizer       │  │ Cache Detector   │  │Provider审计  │   │   │
│  │  │ (本地分词计算)   │  │ (前缀匹配+LRU)   │  │(可信度评分)  │   │   │
│  │  └────────┬───────┘  └────────┬────────┘  └──────────────┘   │   │
│  │           │                   │                                │   │
│  │  ┌────────▼───────────────────▼──────────────────────────┐   │   │
│  │  │              DiffComparator                            │   │   │
│  │  │  本地算的 vs 中转站声称的 → 差异 → Provider 可信度评估    │   │   │
│  │  └────────────────────────┬──────────────────────────────┘   │   │
│  └───────────────────────────┼──────────────────────────────────┘   │
│                               │                                       │
│  ┌───────────────────────────▼──────────────────────────────────┐   │
│  │                    数据存储 (SQLite)                           │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │   │
│  │  │ providers    │  │ audit_log    │  │ cache_state  │       │   │
│  │  │ (上游配置)   │  │ (审计记录)    │  │ (缓存持久化)  │       │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘       │   │
│  └───────────────────────────┼──────────────────────────────────┘   │
│                               │                                       │
│  ┌───────────────────────────▼──────────────────────────────────┐   │
│  │                     Web 管理面板                              │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │   │
│  │  │ 仪表盘       │  │ Provider管理  │  │ 请求详情         │   │   │
│  │  │ (差异概览)   │  │ (切换/配置)   │  │ (单请求审计)     │   │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│                              │                                       │
│                              ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │         上游 Provider (中转站 / 直连 API)                      │   │
│  │  中转站A · 中转站B · 官方API · 自定义                          │   │
│  └──────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 核心模块设计

### 1. Provider 管理 (ProviderManager)

#### 1.1 功能定位

TokenAccountant 提供类似 CC Switch 的 Provider 管理能力，但这一切换会联动审计系统——切换到不同 Provider 后，代理自动路由到对应中转站，审计数据按 Provider 隔离记录。

**与 CC Switch 的区别**：
- CC Switch 的切换影响的是"哪个 Provider 被写入 CLI 配置"
- TokenAccountant 的切换影响的是"代理往哪转发" + "审计数据归到哪个 Provider"

#### 1.2 数据模型

```rust
struct Provider {
    id: String,                    // 唯一标识
    name: String,                  // 显示名称
    provider_type: ProviderType,   // 官方 / 中转站 / 自定义
    api_base_url: String,          // 中转站 URL (如 https://api.example-relay.com)
    api_key: String,               // API Key
    supported_models: Vec<String>, // 支持的模型列表
    proxy_port: u16,               // 分配端口 (每个 Provider 可独立)
    created_at: DateTime,
    updated_at: DateTime,
}

struct ProviderConfig {
    current_provider_id: String,   // 当前激活的 Provider
    providers: Vec<Provider>,
}
```

#### 1.3 切换流程

```
用户切换 Provider
  │
  ├─ 1. 更新 SQLite providers 表 (激活状态)
  ├─ 2. 更新内存中当前 Provider 状态
  ├─ 3. 写入 CLI settings.json (ANTHROPIC_BASE_URL 指向本地代理)
  │     ├─ Claude: ~/.claude/settings.json
  │     ├─ Codex: ~/.codex/config.toml
  │     └─ 由用户选择目标 CLI 工具
  ├─ 4. 提示用户重启 CLI 工具
  └─ 5. 后续请求自动路由到新的 Provider
```

### 2. 配置管理 (ConfigManager)

#### 2.1 修改 CLI 配置文件

参考 CC Switch 的实现，通过修改 CLI 工具的配置文件将 API 流量指向本地代理：

**Claude Code** — `~/.claude/settings.json`：
```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://localhost:${proxy_port}",
    "ANTHROPIC_AUTH_TOKEN": "${provider.api_key}"
  }
}
```

**流程**：
1. 用户添加/切换 Provider 时触发配置写入
2. 写入使用 **原子写入** 模式：临时文件 → write → rename
3. 写入后 **回读验证**，确保文件格式正确
4. 提示用户重启 CLI 工具使配置生效
5. 退出时恢复原始 settings.json（可选，安全退出）

**注意**：Claude Code 启动时只读一次配置，修改后需要重启。TokenAccountant 不提供自动重启，只提示用户。

#### 2.2 TokenAccountant 自身配置

`~/.tokenaccountant/config.toml`：

```toml
[server]
bind_addr = "0.0.0.0:8080"
timeout_seconds = 120

[audit]
suspicion_threshold = 0.05  # 差异 > 5% 标记可疑
cache_ttl_hours = 168        # 缓存数据保留 1 周

[web]
listen_addr = "127.0.0.1:9090"
```

---

### 3. HTTP 反向代理 (Proxy 子系统)

#### 3.1 架构

参考 CC Switch 的三层结构：

```
Handler (路由分发)
  │
  ▼
Forwarder (请求转发 + 响应接收)
  │
  ▼
ResponseHandler (响应处理 + 触发审计)
```

#### 3.2 路由分发

| 端点 | API 格式 | Handler |
|------|----------|---------|
| `/v1/messages` | Anthropic Claude | `handle_claude` |
| `/v1/chat/completions` | OpenAI 兼容 | `handle_openai` |
| `/v1beta/*` | Gemini | `handle_gemini` |

**模型识别**：从请求体 `model` 字段判断具体模型（gpt-4、claude-3、qwen等）。

#### 3.3 请求处理流程

```
请求进入
  │
  ├─ 1. 提取请求体 (messages/content/parts)
  ├─ 2. 识别模型 (model 字段)
  ├─ 3. 选择分词器 (TokenizerFactory)
  ├─ 4. 计算 input tokens (本地分词)
  ├─ 5. 检测缓存命中 (CacheDetector)
  ├─ 6. 转发请求到中转站
  │
  ▼
响应返回
  │
  ├─ 流式 (SSE): 
  │   ├─ 逐 chunk 转发
  │   ├─ 实时累加 output token 计数
  │   └─ 监听 message_delta / completion 事件提取 usage
  │
  ├─ 非流式 (JSON):
  │   ├─ 完整响应 buffer
  │   ├─ 提取 output text → 本地计算 output tokens
  │   └─ 提取 usage 字段 (中转站声称值)
  │
  ▼
差异对比
  ├─ claimed_input vs real_input
  ├─ claimed_output vs real_output
  ├─ claimed_cached vs detected_cached
  └─ 差异 > 阈值 → 标记可疑
```

#### 3.4 关键实现要点

- **异步并行**：分词计算和 API 调用并行进行，不增加延迟
- **流式透传**：使用 Axum 的 `StreamBody` 逐 chunk 转发，同时从中提取 token 信息
- **错误处理**：中转站返回错误时，透传错误信息，不做额外处理
- **超时控制**：可配置的请求超时，默认 120 秒

**关于流式响应处理**：

Claude API 的流式响应通过 SSE 事件传输：

| 事件 | 用途 | Token 提取 |
|------|------|-----------|
| `message_start` | 消息开始 | 含 `message.usage.input_tokens`, `cache_read_input_tokens` |
| `content_block_start` | 内容块开始 | - |
| `content_block_delta` | 内容增量 | `delta.text` → 累加 output tokens |
| `message_delta` | 消息完成 | 含 `usage.output_tokens` |
| `message_stop` | 消息结束 | - |

处理策略：
- Prefill（input）阶段：在 `message_start` 或请求转发前，用本地分词器计算
- Decode（output）阶段：累加 `content_block_delta` 中的 `delta.text`，本地分词计算
- 同时从中转站的 `usage` 提取声称值，后续做对比

---

### 4. 模型识别器 (ModelDetector)

#### 4.1 识别规则

从请求体 `model` 字段判断：

| model 前缀 | 模型 | API 格式 | Tokenizer |
|-----------|------|----------|-----------|
| `gpt-`, `text-` | OpenAI GPT | OpenAI | `tiktoken-rs` (cl100k_base) |
| `claude-` | Anthropic Claude | Anthropic | Claude 近似 |
| `gemini-` | Google Gemini | Gemini | Claude 近似 (字符数/4) |
| `qwen-` | 通义千问 | OpenAI | `tiktoken-rs` (cl100k_base) |
| `deepseek-` | DeepSeek | OpenAI | `tiktoken-rs` (cl100k_base) |
| `glm-` | GLM | OpenAI | `tokenizers` (待调研) |
| `abab` | MiniMax | OpenAI | 待调研 |
| `mimo` | MiMo | OpenAI | 待调研 |

> API 格式判断从 URL 路径来判断，而非从 request body。Claude 路径为 `/v1/messages`，OpenAI 兼容为 `/v1/chat/completions`。

#### 4.2 待调研项

- MiniMax、MiMo 的分词器是否公开
- GLM 的分词器 Rust 实现情况
- 是否有其他模型需要支持

---

### 5. Token 计算引擎 (Tokenizer)

#### 5.1 模块化设计

```rust
// Tokenizer trait
pub trait Tokenizer: Send + Sync {
    fn encode(&self, text: &str) -> Vec<u32>;
    fn decode(&self, ids: &[u32]) -> String;
    fn count_tokens(&self, text: &str) -> u32;
}
```

#### 5.2 分词器策略

| 模型 | 实现方式 | 精度 |
|------|----------|------|
| GPT / Qwen / DeepSeek | `tiktoken-rs` (cl100k_base) | 精确 |
| Claude | 近似 (字符数/ratio) | 近似 |
| Gemini | 近似 (字符数/4) | 近似 |
| GLM / MiniMax / MiMo | `tokenizers` (HuggingFace) 或 Python 脚本 | 待调研 |

**Claude 分词器说明**：Claude 的分词器未公开（Anthropic 未提供开源实现）。TokenAccountant 对 Claude 使用近似计算：
- 英文字符：约 4 字符/token
- 中文字符：约 1.5 字符/token
- 混合内容：按比例估算

**精度影响**：Claude 近似计算可能导致对比差异中包含分词误差。这在设计上需要接受——我们的目标是检测 **显著虚报**（>5%），近似分词带来的小误差可以接受。

---

### 6. 缓存检测器 (CacheDetector)

#### 6.1 工作原理

模仿 vLLM 的 Prefix Caching，通过 Token IDs 的最长前缀匹配来检测缓存命中：

```
请求 1 (首次): [A, B, C, D, E]  ← 全部未命中，缓存
请求 2 (复用): [A, B, C, F, G]  ← A,B,C 命中缓存，F,G 未命中
请求 3 (复用): [A, B, C, D, E]  ← 全部命中
```

#### 6.2 数据结构

```
Token IDs → 分块 (Block Size = 16) → 哈希链
                                           
Block 1: [A1..A16]  ←── hash_1──┐
Block 2: [B1..B16]  ←── hash_2──┼── Trie 链: hash = f(parent_hash, tokens)
Block 3: [C1..C16]  ←── hash_3──┘
```

- **块存储**：预分配块池，每块 16 个 Token IDs
- **哈希计算**：`hash = SHA256(parent_hash || tokens || extra_hash)`
- **LRU 驱逐**：Free Queue 链表 + 过期时间
- **持久化**：支持将缓存状态序列化到 SQLite，跨会话复用

#### 6.3 检测流程

```
请求进入
  │
  ├─ 1. 将 input Token IDs 分块 (每块 16)
  ├─ 2. 逐块检查 hash 是否在缓存中
  │     ├─ 命中 → cached_tokens += block_size
  │     └─ 未命中 → 分配新块 → 加入缓存
  ├─ 3. 统计: cached_tokens 和 uncached_tokens
  │
  ▼
和中转站声称的 cache_read_input_tokens 对比
```

---

### 7. 差异对比器 (DiffComparator)

#### 7.1 对比维度

| 维度 | 本地计算值 | 中转站声称值 | 差异计算公式 |
|------|-----------|-------------|-------------|
| Prefill tokens | `real_input` | `input_tokens` | `input_diff = claimed - real` |
| Decode tokens | `real_output` | `output_tokens` | `output_diff = claimed - real` |
| 缓存命中 | `detected_cached` | `cache_read_input_tokens` | `cache_diff = claimed - detected` |

#### 7.2 可疑判定

- 差异 > 5%（可配置）→ 标记为可疑
- 差异类型：多收、少收、缓存虚报
- **Provider 可信度评分**：基于历史差异数据，给每个 Provider 一个长期可信度评分

#### 7.3 SQLite 存储结构

```sql
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    model TEXT NOT NULL,
    api_format TEXT NOT NULL,
    request_text TEXT,
    response_text TEXT,
    
    -- 中转站声称值
    claimed_input_tokens INTEGER,
    claimed_output_tokens INTEGER,
    claimed_cached_tokens INTEGER,
    
    -- 本地计算值
    real_input_tokens INTEGER,
    real_output_tokens INTEGER,
    detected_cached_tokens INTEGER,
    
    -- 差异
    input_diff INTEGER,
    output_diff INTEGER,
    cache_diff INTEGER,
    
    -- 审计结果
    is_suspicious BOOLEAN,
    suspicion_reason TEXT,
    
    -- 原始响应（用于复核）
    raw_response TEXT
);
```

---

### 8. Web 监控面板

**技术选型**：React + TypeScript + Tailwind CSS，通过 Axum 后端提供 API。

#### 8.1 核心页面

**仪表盘**：
- Prefill / Decode 两个面板，显示本地计算 vs 中转站声称的曲线对比
- 时间窗口选择（1h / 6h / 24h / 7d）
- 柱状图显示每日汇总
- 可疑请求列表

**Provider 管理**：
- 添加上游中转站配置
- 一键切换激活
- 各 Provider 的可信度评分

**请求详情**：
- 单条请求的完整审计信息
- 本地 Token IDs 分块展示
- 缓存命中可视化

#### 8.2 API 端点

| 端点 | 用途 |
|------|------|
| `GET /api/stats/summary` | 统计数据概览 |
| `GET /api/stats/timeline?range=1h` | 时间线数据 |
| `GET /api/requests?page=1&suspicious_only=true` | 请求列表 |
| `GET /api/requests/:id` | 单条请求详情 |
| `GET /api/models` | 已识别的模型列表 |

---

### 9. 项目结构

```
tokenaccountant/
├── Cargo.toml                    # Rust 依赖 (含 Tauri)
├── tauri.conf.json               # Tauri 配置
├── config.toml                   # 默认配置文件
│
├── src-tauri/src/                # Rust 后端 (Tauri 约定目录)
│   ├── main.rs                   # Tauri 入口
│   ├── lib.rs                    # 库入口 (Tauri setup)
│   ├── cli.rs                    # CLI 命令解析 (clap)
│   │
│   ├── provider/
│   │   ├── mod.rs                # Provider 管理入口
│   │   ├── manager.rs            # CRUD + 切换逻辑
│   │   ├── types.rs              # 数据模型
│   │   └── defaults.rs           # 预设模板
│   │
│   ├── config/
│   │   ├── mod.rs                # 配置管理
│   │   ├── cli_config.rs         # CLI settings.json 读写
│   │   └── app_config.rs         # 自身配置读写
│   │
│   ├── proxy/
│   │   ├── mod.rs                # 代理模块入口
│   │   ├── server.rs             # Axum 服务器
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   ├── claude.rs         # Claude API handler
│   │   │   ├── openai.rs         # OpenAI 兼容 handler
│   │   │   └── gemini.rs         # Gemini handler
│   │   ├── forwarder.rs          # 请求转发
│   │   ├── response_handler.rs   # 响应处理 + 触发审计
│   │   ├── sse.rs                # SSE 流式处理
│   │   └── types.rs              # 请求/响应类型
│   │
│   ├── auditor/
│   │   ├── mod.rs                # 审计引擎入口
│   │   ├── model_detector.rs     # 模型识别
│   │   ├── tokenizer.rs          # Tokenizer trait + 工厂
│   │   ├── tokenizers/
│   │   │   ├── mod.rs
│   │   │   ├── gpt.rs            # GPT (tiktoken-rs)
│   │   │   ├── qwen.rs           # Qwen (tiktoken-rs)
│   │   │   ├── deepseek.rs       # DeepSeek (tiktoken-rs)
│   │   │   ├── claude.rs         # Claude (近似)
│   │   │   └── fallback.rs       # 其他 (通用近似)
│   │   ├── cache_detector.rs     # 缓存检测 (前缀匹配 + LRU)
│   │   └── diff_comparator.rs    # 差异对比
│   │
│   ├── storage/
│   │   ├── mod.rs                # 存储模块入口
│   │   ├── database.rs           # SQLite 操作
│   │   └── migration.rs          # 数据库迁移
│   │
│   └── api/                      # Tauri commands + HTTP API
│       ├── mod.rs                # API 入口
│       ├── commands.rs           # Tauri IPC 命令
│       ├── stats.rs              # 统计 API
│       └── requests.rs           # 请求查询 API
│
├── frontend/                     # React 前端源码
│   ├── src/
│   │   ├── App.tsx
│   │   ├── pages/
│   │   │   ├── Dashboard.tsx
│   │   │   ├── RequestDetail.tsx
│   │   │   └── Settings.tsx
│   │   └── components/
│   │       ├── StatsChart.tsx
│   │       ├── RequestTable.tsx
│   │       └── DiffIndicator.tsx
│   ├── index.html
│   └── package.json
│
└── tests/
    ├── integration/
    │   ├── proxy_test.rs
    │   └── auditor_test.rs
    └── fixtures/
        └── sample_payloads.json
```

---

## 技术栈

| 层级 | 技术选型 | 理由 |
|------|----------|------|
| **桌面壳** | Tauri 2 | 跨平台桌面应用，系统托盘常驻 |
| **后端框架** | Axum (Rust) | 高性能，异步支持好，适合代理场景 |
| **HTTP 代理** | Axum + tokio | 异步，支持流式响应 |
| **CLI 框架** | clap | Rust 生态标准 CLI 库 |
| **前端框架** | React + TypeScript + Tailwind | Tauri 生态标准前端方案 |
| **数据库** | SQLite (rusqlite) | 轻量级，无需额外服务 |
| **分词器** | `tiktoken-rs` / `tokenizers` | 主流模型分词支持 |
| **HTTP 客户端** | reqwest | 异步 HTTP 转发 |
| **序列化** | serde / serde_json | Rust 标准序列化方案 |

---

## 实现优先级

### Phase 1: MVP

- [x] 项目骨架搭建
- [ ] CLI 入口 (`tokenaccountant start`, `status`, `provider`, `config`)
- [ ] ProviderManager: 多上游 CRUD + 切换激活
- [ ] ConfigManager: CLI `settings.json` 读写 + `config.toml`
- [ ] Proxy 基础: Axum 服务器启动，请求透传（不分流）
- [ ] ModelDetector: 从 `model` 字段识别模型
- [ ] Tokenizer: GPT / Qwen / DeepSeek (tiktoken-rs) + Claude 近似
- [ ] DiffComparator: 基础对比逻辑
- [ ] SQLite 存储: providers 表 + audit_log 表 + 基础 CRUD
- [ ] Web 面板: Provider 管理 + 仪表盘（差异显示）

### Phase 2: 完整功能

- [ ] 流式响应支持 (SSE 处理)
- [ ] CacheDetector: 前缀匹配 + LRU 驱逐
- [ ] 完整 Web 面板（图表、时间线、筛选、Provider 切换）
- [ ] 多 API 格式支持（Gemini 等）
- [ ] 持久化缓存状态（跨会话）
- [ ] Provider 可信度评分
- [ ] 可疑请求告警

### Phase 3: 增强

- [ ] Gemini / Claude 精确分词（如果有开源实现）
- [ ] 多语言分词器支持 (Python 脚本桥接)
- [ ] Provider 预设模板库（类似 CC Switch 的 50+ 预设）
- [ ] 导出报告 (CSV/JSON)
- [ ] Docker 部署
- [ ] 更多模型支持

---

## 关键设计决策

### 1. Provider 管理的范围

TokenAccountant 提供完整的 Provider 管理能力，包括多上游配置、一键切换、健康检查。但与 CC Switch 的区别在于：
- CC Switch 的 Provider 管理覆盖 50+ 预设，重点是"有多少种方式连到 AI 服务"
- TokenAccountant 的 Provider 管理重点是"管理用户正在使用的几个中转站"，每个 Provider 的审计数据独立记录和评分

Phase 1 实现基础 CRUD + 切换，Phase 3 再补充预设模板库。**不做**格式转换（OpenAI↔Anthropic）、熔断故障转移等功能——这些是 Provider 切换工具（CC Switch）的范畴，不是审计工具的核心。

### 2. 透传不转换

与 CC Switch 不同，TokenAccountant 不做格式转换。请求和响应原样透传，只在必要时提取 token 信息。这降低了复杂度，减少了引入新 bug 的可能。

### 3. Claude 分词精度

Claude 分词器不公开，TokenAccountant 使用近似计算。这意味着对于 Claude 模型，我们的审计结果包含分词误差。设计上接受这一点——目标是检测显著虚报（>5%），近似分词的小误差可接受。

### 4. 流式处理的权衡

流式响应 token 统计有两种策略：
- **实时累加**：在 SSE chunk 到达时，逐块累加 `delta.text` 长度，请求结束时统一算 token 数
- **等待完整**：buffer 所有 chunk，请求结束后完整计算

选择 **实时累加 + 结束时算总账** 的策略，既不影响流式体验，又能拿到精确值。

### 5. 缓存检测的持久化

缓存检测依赖历史请求数据。跨会话场景需要持久化：
- 每次请求后将缓存块信息写入 SQLite
- 启动时从 SQLite 加载缓存状态
- 过期缓存自动清理
