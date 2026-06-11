# Phase 2: Audit Integration & SSE Streaming

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect Tokenizer, DiffComparator, CacheDetector into the proxy flow for automated request/response auditing.

**Architecture:** Request arrives → spawn parallel tasks: (1) forward to upstream and stream SSE response to client, (2) tokenize input and detect cache. When stream ends, tokenize output, compare with upstream usage, store in SQLite.

**Tech Stack:** Axum 0.7, reqwest 0.12, reuse existing `auditor/` modules.

---

## 时间线

### Phase 2 — 初始实现 (2026-06-09)

| Task | 状态 | 说明 |
|------|------|------|
| Task 1: StreamForwarder | ✅ | SSE 转发 + 文本累加 |
| Task 2: ProxyState 扩展 | ✅ | 审计组件注入 |
| Task 3: Handler 审计集成 | ✅ | claude.rs + openai.rs |
| Task 4: 前端请求列表 | ✅ | RequestList.tsx |
| Task 5: B-Plan 扩展点 | ✅ | 骨架预留 |

原始 6 个 Task 的详细 step-by-step 代码见下方。

### Phase 2 — 验收修正 (2026-06-10)

| 修正 | 触发 | 说明 |
|------|------|------|
| NormalizedMessage 结构化 | `extract_request_text` 遗漏 content blocks | `message_converter.rs` + reasoning/tool_calls 字段 |
| DeepSeek DSML 对齐 | 对比 vLLM 源码发现 transition 标签错误 | `</think><think>` 修复，`render_assistant_output` |
| CacheDetector block hash | `DefaultHasher` 随机种子 | LCG 确定性 hash |
| 输出 token 计数 | `full_text` 是原始 SSE 文本 | `build_output()` 结构化解析 + `render_output()` |
| store 存储方式 | 拼 prompt_ids+output_ids 边界不匹配 | `Conversation` + `apply_chat_template` 重渲染 |
| extract_usage | SSE full_text 无法直接 JSON 解析 | `last_sse_json()` 提取最后一个 data: JSON |
| x-anthropic-billing-header | 发现 cch=xxxx 使系统 prompt 每次不同 | `extract_system_text` 过滤 |
| 硬编码配置 | PR review | 集中到 AppConfig + serde(default) |
| 死代码清理 | 发现未使用模块 | 删除 forwarder/response_handler/sse |
| 并发正确性验证 | debug 确认 | tokio multi-thread 并行 audit + 转发 |

---

## 当前状态 (2026-06-11)

### ✅ 已完成

- StreamForwarder SSE 解析（`thinking_delta` / `text_delta` / `input_json_delta` 分类累积）
- `build_output()` 产出结构化 NormalizedMessage
- NormalizedMessage: reasoning + tool_calls 独立字段
- `from_anthropic()` 多 block 类型分流（text / thinking / tool_use / tool_result）
- DeepSeek DSML 渲染（transition tokens, think tags、EOS）
- `render_assistant_output()`（standalone DSML，含 `<think>`）
- `Tokenzier` trait 新增 `render_output()`
- GptTokenizer/ClaudeTokenizer `apply_chat_template` + `render_output` 实现
- CacheDetector: block hash chain + LCG hash + LRU FIFO
- `store_combined()` — 接收完整渲染 prompt 存 hash
- `detect()` — 逐 block chain hash 查表
- Post-stream 存储: `conv + output_msg` → `apply_chat_template` → `store_combined()`
- `extract_usage()` SSE 修复 — `last_sse_json()` 提取
- `x-anthropic-billing-header` 过滤（cache-busting）
- 配置集中化 AppConfig（`max_cached_blocks`, `max_body_bytes`）
- 死代码清除
- `real_input = needs_prefill` — 匹配上游 API 语义
- `Cargo.toml` 依赖对齐

### ✅ 2026-06-11 新增完成

| 改动 | 说明 |
|------|------|
| **message_converter 重写** | `ContentPart` 枚举（Text / ImageUrl / ToolReference）、`content_parts: Vec<ContentPart>` 替代 `content: String`、`tool_call_id` 字段、处理所有系统消息（不限于第一个）、image block → data URI、redacted_thinking 静默跳过、tool_result 子结构（text + image + tool_reference） |
| **vLLM `deepseek_v4_encoding.py` 完全对齐** | per-parameter DSML tool call 编码、`content_blocks` 格式的 `merge_tool_messages`、`sort_tool_results_by_call_order`、`_drop_thinking_messages`、`reasoning_effort='max'` 前缀、developer role / response_format / wo_eos 支持 |
| **`TemplateParams` 参数化** | `thinking_mode`、`drop_thinking`、`add_default_bos_token`、`reasoning_effort` 通过 `apply_chat_template_with()` 传入，`extract_template_params()` 从请求体提取 |
| **Render Inspector DevPanel** | `DevTraceBuffer` 环形缓冲区、`list_dev_traces` / `clear_dev_traces` IPC、`DevPanel.tsx` 组件、开发者模式开关、深色主题左右布局 |
| **Store 保留 reasoning** | 去掉 `store_msg.reasoning = None`，Claude Code 回传完整 thinking blocks，store 必须保留以匹配 detect 前缀 |

### 🔲 待办

| ID | 内容 | 优先级 |
|----|------|--------|
| **A** | Cache hit 精度改进（TTL 淘汰 / vLLM 精确 block hash 模拟） | 中 |
| **B** | 完善单元测试（message_converter / deepseek tokenizer） | 中 |
| **C** | 非 Claude Code client 兼容性验证 | 低 |

---

---

## Task D — Rendering Inspector（开发模式测试模块）

> ⚠️ **DEPRECATED — 被 `docs/superpowers/plans/2026-06-11-render-inspector.md` 取代。**
> 旧方案为纯 `#[cfg(test)]` 终端工具，新方案改为运行时捕获 + Tauri IPC + 前端 DevPanel，与 spec §8 一致。
> 保留此 Task 作为历史记录，不删除。

**目标：** 纯开发工具，通过 `cargo test -- --nocapture` 运行。暴露每一轮对话中 detect 路径和 store 路径的中间状态，让开发者肉眼验证 `from_anthropic_body` → `apply_chat_template` → block hash chain 的一致性。

**不是产品功能：**
- 没有 Tauri IPC
- 没有前端页面
- 不写入数据库
- 纯 `#[cfg(test)]`，release build 不编译

### 设计

#### 数据结构

```rust
/// 一轮对话的完整展平结果（detect 和 store 分别有一份）
struct RenderTrace {
    turn: usize,
    phase: Phase,    // Detect | Store
    /// from_anthropic_body 产出的 NormalizedMessage 列表（打印用）
    messages: Vec<NormalizedMessage>,
    /// apply_chat_template 后的完整文本
    rendered: String,
    /// token IDs
    token_ids: Vec<u32>,
    /// block hash chain（每 16 token 一个 hash）
    block_hashes: Vec<u64>,
    /// detect 结果（仅 Detect 阶段有）
    cached_tokens: u32,
    remaining_tokens: u32,
    /// output_msg（仅 Store 阶段有）
    output_msg: Option<NormalizedMessage>,
}
```

#### 输入

模拟的多轮对话 JSON 文件（hardcoded 或从 `tests/fixtures/` 加载），每轮包含：

```json
[
  {
    "turn": 1,
    "request": { /* Anthropic Messages API 格式的完整 request body */ },
    "stream_sse": "...",  /* 模拟的 SSE 响应文本 */
    "next_turn_messages": [ /* 下一轮客户端回传的 messages 数组 */ ]
  }
]
```

#### 输出格式（`println!` → `--nocapture` 可见）

```
╔══════════════════════════════════════════════════════════════╗
║  Turn 1 · Detect                                           ║
╚══════════════════════════════════════════════════════════════╝

── Messages (from_anthropic_body) ────────────────────────────
  [system] "You are a helpful assistant."
  [user]   "Hello!"

── Rendered · DeepSeek (42 tokens) ──────────────────────────
  │ <｜begin▁of▁sentence｜>You are a helpful assistant.
  │ <｜User｜>Hello!
  │ <｜Assistant｜><think>
  ───────────────────────────────────────────────────────────

── Block Hash Chain ─────────────────────────────────────────
  Block 0: 0000000000000000 → a1b2c3d4e5f6a7b8  [MISS]
  Block 1: a1b2c3d4e5f6a7b8 → b2c3d4e5f6a7b8c9  [MISS]
  Block 2: b2c3d4e5f6a7b8c9 → c3d4e5f6a7b8c9d0  [MISS]
  Cache result: 0 cached, 42 remaining

──────────────────────────────────────────────────────────────
╔══════════════════════════════════════════════════════════════╗
║  Turn 1 · Store                                            ║
╚══════════════════════════════════════════════════════════════╝

── Output (from stream) ─────────────────────────────────────
  assistant (thinking)
    reasoning="Let me think about this..."
    content="Hello! How can I assist you today?"

── Messages (conv + output) ─────────────────────────────────
  [system]   "You are a helpful assistant."
  [user]     "Hello!"
  [assistant] reasoning="Let me think...", content="Hello! How can I..."

── Rendered · DeepSeek (78 tokens) ──────────────────────────
  │ <｜begin▁of▁sentence｜>You are a helpful assistant.
  │ <｜User｜>Hello!
  │ <｜Assistant｜><think>Let me think about this...
  │ </think>Hello! How can I assist you today?
  │ <｜end▁of▁sentence｜>
  ───────────────────────────────────────────────────────────

── Block Hash Chain ─────────────────────────────────────────
  Block 0: 0000000000000000 → a1b2c3d4e5f6a7b8  [STORE]
  Block 1: a1b2c3d4e5f6a7b8 → b2c3d4e5f6a7b8c9  [STORE]
  Block 2: b2c3d4e5f6a7b8c9 → c3d4e5f6a7b8c9d0  [STORE]
  Block 3: c3d4e5f6a7b8c9d0 → d4e5f6a7b8c9d0e1  [STORE]
  Block 4: d4e5f6a7b8c9d0e1 → e5f6a7b8c9d0e1f2  [STORE]

── Prefix Match Check ──────────────────────────────────────
  detect blocks:   [a1b2, b2c3, c3d4]
  stored blocks:   [a1b2, b2c3, c3d4, d4e5, e5f6]
  prefix match:    ✓ (3/3 detect blocks match stored prefix)
```

#### 验证逻辑（断言）

```
Turn 1 Store 的 block hash chain
  ⊆ Turn 1+2 Detect 的 block hash chain 前缀    // store 结果是下次 detect 的前缀
```

### 实现方案

**文件结构：**
- `src-tauri/src/auditor/render_inspector.rs` — RenderTrace 数据结构 + `inspect_detect()` / `inspect_store()` 函数 + `#[cfg(test)]` 测试函数
- 不需要 fixtures 目录（初期 hardcode 多轮对话 JSON string）

**关键函数签名：**

```rust
#[cfg(test)]
pub(crate) mod render_inspector {
    /// inspect detect 阶段：body → messages → rendered → encode → detect
    pub fn inspect_detect(
        tokenizer: &dyn Tokenizer,
        cache: &CacheDetector,
        body_str: &str,
        model: &str,
    ) -> RenderTrace { ... }

    /// inspect store 阶段：conv + output_msg → messages → rendered → encode → store
    pub fn inspect_store(
        tokenizer: &dyn Tokenizer,
        cache: &CacheDetector,
        conv: &Conversation,
        output_msg: &NormalizedMessage,
    ) -> RenderTrace { ... }

    /// 多轮对话模拟：按顺序依次 inspect detect → store → detect → store → ...
    pub fn simulate_conversation(scenarios: &[Scenario]) -> Vec<RenderTrace> { ... }
}
```

### 测试场景（初始）

| # | 模型 | 场景 | 验证点 |
|---|------|------|--------|
| 1 | DeepSeek | 简单对话（无 thinking, 无 tools） | 基础 prefix match |
| 2 | DeepSeek | 带 reasoning 的 assistant 回复 | reasoning 在 store/detect 一致 |
| 3 | DeepSeek | 多轮对话（3 turns） | 连续 prefix match |
| 4 | GPT | 简单对话 | GPT tokenizer 的 prefix match |
| 5 | Claude | 简单对话（tokenizer unimplemented） | graceful skip |

---

## 原始实现 (2026-06-09 Plan, 6 Tasks)

以下是 Phase 2 初始实现的 6 个 Task，step-by-step 代码保留供参考。

### Task 1: Create StreamForwarder

**Files:**
- Create: `src-tauri/src/proxy/stream_forwarder.rs`
- Modify: `src-tauri/src/proxy/mod.rs`

`StreamForwarder` reads upstream SSE chunks via `reqwest::Response::bytes_stream()`, forwards them to the client via an `mpsc` channel + `StreamBody`, and accumulates text for post-stream audit.

- [x] **Step 1: Create stream_forwarder.rs**

```rust
// 初始实现详见 git history: f880c7a
```

- [x] **Step 2: Add mod declaration in proxy/mod.rs**

```rust
pub mod stream_forwarder;
```

- [x] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Clean compilation

- [x] **Step 4: Commit**

### Task 2: Extend ProxyState with audit components

**Files:**
- Modify: `src-tauri/src/proxy/server.rs`
- Modify: `src-tauri/src/lib.rs`

- [x] **Step 1: Extend ProxyState in server.rs**

```rust
use crate::auditor::tokenizer::TokenizerFactory;
use crate::auditor::diff_comparator::DiffComparator;
use crate::auditor::cache_detector::CacheDetector;
use crate::storage::database::Database;

#[derive(Clone)]
pub struct ProxyState {
    pub provider_manager: Arc<Mutex<crate::provider::manager::ProviderManager>>,
    pub status: Arc<Mutex<ProxyStatus>>,
    pub tokenizer_factory: Arc<TokenizerFactory>,
    pub diff_comparator: Arc<DiffComparator>,
    pub cache_detector: Arc<Mutex<CacheDetector>>,
    pub db: Arc<Mutex<Database>>,
}
```

- [x] **Step 2: Initialize audit components in lib.rs**

- [x] **Step 3: Commit**

### Task 3: Connect audit into proxy handlers

**Files:**
- Create: `src-tauri/src/proxy/handlers/claude.rs`
- Create: `src-tauri/src/proxy/handlers/openai.rs`

- [x] **Step 1: Create claude.rs with audit flow**

```rust
// Audit flow:
// 1. tokio::spawn(audit_handle) — encode + detect + store (parallel)
// 2. forward upstream request, stream response to client
// 3. tokio::spawn(post-stream) — wait stream end → extract_usage → real_output → compare → insert
```

- [x] **Step 2: Create openai.rs with same pattern**

- [x] **Step 3: Add handlers module**

- [x] **Step 4: Register routes in server.rs**

- [x] **Step 5: Implement fallback handler for API routes**

- [x] **Step 6: Compile and verify**

### Task 4: Add audit log IPC and frontend list page

**Files:**
- Modify: `src-tauri/src/api/commands.rs`
- Modify: `frontend/src/pages/RequestList.tsx`

- [x] **Step 1: Add IPC command `list_audit_logs`**

- [x] **Step 2: Update frontend RequestList page**

- [x] **Step 3: Compile**

### Task 5: B-Plan extension point

- [x] **Step 1: Add `on_chunk` callback to StreamForwarder**

---

## 验收发现 (2026-06-10 Debug)

### Issue 1: `extract_request_text` 遗漏大量 input 内容

**发现:** `extract_request_text` 只提取 `messages[].content` 中 `string` 类型的文本，遗漏 system 字段、tool_use/thinking/tool_result blocks、tools 定义。

**修复:**
1. 新建 `auditor/message_converter.rs`，标准化 `Vec<NormalizedMessage>`
2. `extract_request_text` 被移除，改用 `message_converter::from_anthropic_body()` + `t.apply_chat_template()`

### Issue 2: Chat template 归属不清晰

**修复:** `Tokenizer` trait 新增 `apply_chat_template()` 方法，模型相关分隔符从格式转换层移到 tokenizer 层。

### Issue 3: CacheDetector 低估 cache hit

**发现:** `store()` 只存 prompt token IDs，但 vLLM KV cache 同时缓存 prompt 和 decode 产出的 output tokens。

**修复:** Block hash chain 重写 CacheDetector，post-stream 调用 `store_combined()`。

### Issue 4: Post-stream 轮询等待流结束

**发现:** `while !stream_ended { sleep(50ms) }` 忙等。

**修复:** 保持当前实现（后续改用 `tokio::sync::Notify`）。

### Issue 5: Transition token 不匹配

**发现:** encode_messages 中 `</think>` 写死在 user 分支，assistant 又加 `<think>` → `</think><think>`。且 `store_combined` 拼 prompt_ids+output_ids 边界不匹配。

**修复:**
- 按 vLLM 架构重构：transition 属于 user→assistant，assistant 只渲染 `reasoning</think>`
- `store_combined` 改用 `Conversation + output_msg` 重新 `apply_chat_template`

### Issue 6: DefaultHasher 随机种子

**发现:** `DefaultHasher::new()` 每次随机种子，相同输入不同 hash → cache 命中率 0。

**修复:** 改用 LCG 确定性 hash（`wrapping_mul + wrapping_add`）。

### Issue 7: x-anthropic-billing-header

**发现:** Claude Code 每次请求携带唯一 `cch=xxxxx`，使系统 prompt 每次都变。

**修复:** `extract_system_text` 过滤该 header。

### Issue 8: 硬编码配置

**修复:** 集中到 `AppConfig`，`#[serde(default)]` 向后兼容。
