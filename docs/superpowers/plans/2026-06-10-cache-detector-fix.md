# CacheDetector: 完整实现

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现正确的 token counting 和 cache hit 估算，包括 input/output token 计数、block hash chain 缓存、SSE 结构化解析、系统 prompt 过滤。

**Tech Stack:** Rust, tiktoken, existing auditor modules.

---

## 实际完成

### ✅ Task 1: Deterministic block hash chain

**Files:**
- `src-tauri/src/auditor/cache_detector.rs`

将 `Vec<Vec<u32>>` 前缀匹配改为 `HashSet<u64>` block hash chain，模拟 vLLM APC。

Key 决策：
- 用 **LCG 线性同余 hash**（非 `DefaultHasher`，因其每次随机种子导致不可重现）
- `BLOCK_SIZE = 16`，不满 16 的 partial block 不缓存
- `INIT_HASH = 0`
- LRU FIFO 淘汰（`VecDeque<u64>`），上限来自 `AppConfig.audit.max_cached_blocks`
- `insert_block()` — 插入并触发淘汰
- `detect(token_ids)` — 逐 block 算 chain hash 查表，返回 `(matched, total - matched)`
- `store_combined(token_ids)` — 接收完整渲染 prompt，切 block 存 hash
- `store()` — 已删除（不再使用，全部由 `store_combined` 处理）

### ✅ Task 2: SSE 结构化解析

**Files:**
- `src-tauri/src/proxy/stream_forwarder.rs`

将 `get_text()`（原始 SSE 文本累加，用于 `extract_usage`）改为结构化累积：
- `thinking_parts` — `thinking_delta` 内容
- `text_parts` — `text_delta` 内容
- `tool_calls` — `content_block_start(tool_use)` + `input_json_delta` 碎片

新增：
- `build_output() → NormalizedMessage` — stream 结束后组装结构化消息
- 修复: 遍历 chunk 中**所有** `data:` 行（原 `.find()` 只取第一个，漏掉合并的多事件 chunk）

### ✅ Task 3: DSML 输出渲染

**Files:**
- `src-tauri/src/auditor/tokenizers/deepseek.rs`
- `src-tauri/src/auditor/tokenizers/gpt.rs`
- `src-tauri/src/auditor/tokenizers/claude.rs`
- `src-tauri/src/auditor/tokenizer.rs`

- 提取 `render_assistant_output()` — 独立对外函数，带 `<think>` 标签
- 提取 `render_assistant_output_internal()` — encode_messages 内部使用，无 `<think>`（来自 transition）
- `Tokenizer` trait 新增 `render_output()` 方法
- DeepSeek: DSML 格式（`<think>reasoning</think>content\n\n<DSML>...`)
- GPT: `[thinking][/thinking]` 兜底格式
- Claude: `unimplemented!()` 占位

### ✅ Task 4: Post-stream 存储重构

**Files:**
- `src-tauri/src/proxy/handlers/claude.rs`
- `src-tauri/src/proxy/handlers/openai.rs`

原方案（已放弃）: 拼 `prompt_ids + output_ids` 存 → 边界 `<think>`/`</think>` 不匹配导致 chain 断

现方案: `audit_handle` 返回 `Conversation`，post-stream 追加 `output_msg` 后**重新 `apply_chat_template`**，存完整渲染 prompt 的 block hashes

修正:
- `output_msg` 去掉 reasoning 再存（匹配客户端实际发送格式，thinking block 通常不包含在请求体中）
- `real_input` = `needs_prefill` = `total - cached`（匹配上游 API 的 `input_tokens` 语义）
- 不再调用 `store()`（pre-stream 只 detect 不 store）

### ✅ Task 5: extract_usage 修复

**Files:**
- `src-tauri/src/auditor/model_detector.rs`

SSE 流中 usage 信息在最后一个 `message_delta` 事件的 `data:` JSON 中，但 `full_text` 是整个 SSE 拼接体。新增 `last_sse_json()` 函数，从 SSE 事件流中提取最后一个包含 `usage` 字段的 `data:` JSON。

### ✅ Task 6: 硬编码配置集中化

**Files:**
- `src-tauri/src/config/app_config.rs`
- `src-tauri/src/auditor/cache_detector.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/proxy/server.rs`
- `src-tauri/src/api/commands.rs`

将原本分散的硬编码值移到 `AppConfig`：

| 旧硬编码 | 新配置项 |
|---------|---------|
| `MAX_CACHED_BLOCKS = 100_000` | `AuditConfig.max_cached_blocks` |
| `handlers: 10 * 1024 * 1024` | `ServerConfig.max_body_bytes` |

使用 `#[serde(default)]` 保证旧配置向后兼容。

### ✅ Task 7: 过滤 cache-busting header

**Files:**
- `src-tauri/src/auditor/message_converter.rs`

在 `extract_system_text` 中跳过 `x-anthropic-billing-header` 开头的 text block。这是 Claude Code 每次请求携带的唯一标识头，导致系统 prompt 每次都变、prefix cache 永远无法命中。

---

## 待办

### 🔲 Task A: 验证 input/output 拼接一致性

确保 `store_combined` 存储的内容和 `detect` 读取的内容完全一致：
- [ ] 验证 `apply_chat_template(conv + output_msg_stripped)` 的输出是否是下个请求 prompt 的精确前缀
- [ ] 验证 `output_msg`（从 SSE 解析）和客户端请求体中 assistant 消息的格式一致
- [ ] 处理 `signature_delta` 等被忽略的 SSE 事件类型
- [ ] 验证非 DeepSeek 模型（GPT）的 output token 计数

### 🔲 Task B: 改进 cache hit 精度

- [ ] vLLM block hash chain 精确模拟（当前 LCG 仅是估算）
- [ ] TTL 淘汰替代 FIFO LRU（当前基于数量，后续换成基于时间）
- [ ] 多请求并发时 cache state 共享的正确性验证

### ✅ Task C: 清理

- [x] 删除不再使用的 `forwarder.rs`、`response_handler.rs`、`sse.rs`
- [x] 删除 `extract_request_text`（已废弃）
- [x] 删除 debug tracing log（`CACHE_HIT`、`CACHE_STORE`、`CACHE_BLOCK2`）
