# Phase 2: 审计接入与 SSE 流式转发

> 设计文档 — 实施计划由 writing-plans 另行生成

**Goal:** 将 Tokenizer、DiffComparator、CacheDetector 接入代理流程，实现请求/响应的自动化审计，并在前端展示审计结果

**Architecture:** 在现有 Axum Handler 中插入审计步骤，采用"转发与审计异步并行"模式（A 方案）。请求到达后立即并行执行：线程 1 转发请求并逐 chunk 转发 SSE 响应、线程 2 后台算 input tokens 和缓存检测。响应流结束后算 output tokens，对比差异，结果存 SQLite 并展示在前端。预留"边转发边审计"的扩展点（B 方案）。

---

## 设计决策

### 1. 审计模式：A 方案（转发与审计异步并行）

转发和审计在两个异步任务中并行执行，互不等待。流结束后合并结果做 diff 对比。

**选择理由：**
- 不影响流式体验（Claude Code 实时看到 thinking token）
- 不增加转发延迟（审计在后台进行）
- 流结束后算总账更精确（tiktoken 对整个文本一次性 encode）
- 对于显著虚报（>5%），A 方案完全够用

**流程：**
```
请求到达
  ├─ 线程 1: 立即转发 → SSE 流式转发（逐 chunk）→ Claude Code
  │                                            ↓
  │                                      累加文本
  │                                            ↓
  │                                 流结束 + 线程 2 完成
  │
  ├─ 线程 2: 算 input tokens（后台，与转发并行）
  │          CacheDetector.detect/store
  │
  ▼
  Diff 对比 → 存 SQLite → 前端展示
```

**B 方案（边转发边审计）** 预留在 `StreamForwarder` 的回调接口中，未来可以通过逐 chunk encode 实现实时 token 计算。

### 2. SSE 流式转发

现在用 `resp.bytes().await` 等全部收完，要改为逐 chunk 转发。

- 使用 `reqwest::Response::bytes_stream()` 获取异步流
- 每收到一个 chunk，立即用 Axum 的 `Body::from_stream` 转发
- **不等待流结束**，实时透传
- 同时将 chunk 文本累加到内存缓冲区，供流结束后审计使用
- 在累加文本中提取 `usage` 字段（中转站声称值）

### 3. 审计数据流

```
请求体
  ├─ detect(model) → 选择 tokenizer
  ├─ extract_request_text → 本地 encode → real_input_tokens
  ├─ encode → CacheDetector.detect → detected_cached_tokens
  └─ CacheDetector.store → 缓存本次结果

响应体（累加文本）
  ├─ extract_usage → claimed_input / claimed_output / claimed_cached
  ├─ count_tokens → real_output_tokens
  └─ DiffComparator.compare → AuditRecord → SQLite.insert_audit_log
```

### 4. 组件变化

**新增文件：**
- `proxy/stream_forwarder.rs` — SSE 流式转发 + 文本累加，预留 B 方案回调接口

**修改文件：**
- `proxy/handlers/claude.rs` — 插入审计步骤，使用 StreamForwarder
- `proxy/handlers/openai.rs` — 同上
- `proxy/server.rs` — ProxyState 增加 tokenizer_factory 等审计组件
- `lib.rs` — 初始化审计组件并注入 TauriState / ProxyState
- `api/commands.rs` — 新增 list_audit_logs IPC 命令
- `frontend/src/pages/RequestList.tsx` — 替换占位符，展示审计记录
- `frontend/src/App.tsx` — 更新导入

**不修改：**
- `auditor/tokenizer.rs`, `auditor/tokenizers/gpt.rs` — 已有代码直接复用
- `auditor/diff_comparator.rs` — 已有代码直接复用
- `auditor/cache_detector.rs` — 已有代码直接复用
- `auditor/model_detector.rs` — 已有代码直接复用

### 5. ProxyState 扩展

当前 `ProxyState` 只有 `provider_manager` 和 `status`。需要增加：

```rust
pub struct ProxyState {
    pub provider_manager: Arc<Mutex<ProviderManager>>,
    pub status: Arc<Mutex<ProxyStatus>>,
    pub tokenizer_factory: Arc<TokenizerFactory>,
    pub diff_comparator: Arc<DiffComparator>,
    pub cache_detector: Arc<Mutex<CacheDetector>>,
    pub db: Arc<Mutex<Database>>,  // 写入审计日志
}
```

### 6. 前端新页面

**请求列表页** 替换现有的占位符：
- 卡片列表，每条显示：模型名、input tokens（声称 vs 实际）、output tokens（声称 vs 实际）、可疑标记
- 可疑请求红色边框，正常请求灰色
- 空状态显示"暂无审计记录"
- 数据来源：通过 Tauri IPC 调用 `list_audit_logs`

**不做：**
- 图表、时间线（留到 Phase 3）
- 请求详情弹窗（留到 Phase 3）

### 7. 不做项

- **非流式响应处理**：目前不支持也不做（Claude Code 始终使用流式）
- **Gemini 支持**：Phase 3
- **缓存持久化**：CacheDetector 保持纯内存，跨会话缓存留到 Phase 3
- **Provider 可信度评分**：留到 Phase 3
- **熔断/故障转移**：不是审计工具的核心

---

## Scope

### Phase 2 包含

1. SSE 流式转发（逐 chunk）
2. 请求审计（input tokens + 缓存检测）
3. 响应审计（output tokens + diff 对比）
4. 审计记录存入 SQLite
5. 前端请求列表页
6. B 方案扩展接口预留

### Phase 2 不包含（留到 Phase 3）

1. 边转发边审计（B 方案）
2. 图表、时间线
3. 请求详情弹窗
4. 缓存持久化
5. Provider 可信度评分
6. Gemini 支持
7. 非流式响应处理

---

## 8. Rendering Inspector 开发者面板

> 开发模式工具。纯函数 + DevTraceBuffer，不耦合 proxy handler 核心逻辑。

### 目标

验证 store 阶段拼接 prefill + decode 后的渲染结果，与下一次 detect 阶段 client 回传的对话历史的渲染结果，在 transition token 边界处保持一致——从而保证 block hash chain 前缀匹配，prefix cache 生效。

### 问题背景

Proxy 的 store/detect 流程：

```
Turn N:
  detect:  request body → from_anthropic_body → conv → render → encode → cache.detect
  store:   conv + output_msg → render → encode → cache.store_combined
                                    ↑
                            拼接 prefill 和 decode 后的完整对话

Turn N+1:
  detect:  request body(含 Turn N 的 assistant msg)
           → from_anthropic_body → conv → render → encode → cache.detect
                                    ↑
                     client 回传的对话历史，应该与 store 的前缀一致
```

关键验证点：Turn N store 的渲染结果（conv + assistant output，含 transition token 如 `<｜Assistant｜>`, `<think>`, `</think>`, `<｜end▁of▁sentence｜>` 等）必须是 Turn N+1 detect 渲染结果的前缀。

如果 store 拼接 prefill 和 decode 时边界处理有误（如 transition token 重复、缺失、顺序错乱），会导致 block hash chain 在前几个 block 之后断掉，prefix cache 命中率为 0。

### 架构

proxy handler 中捕获，App 内开发者面板展示：

```
proxy handler（开发者模式开启时）:
  detect:  body → from_anthropic_body → apply_chat_template → detect_text
  store:   conv + output_msg → apply_chat_template → store_text
                ↓
         写入 DevTraceBuffer（环形缓冲区，最多 100 轮）
                ↓
IPC: list_dev_traces() → Vec<DevTrace { turn, model, detect_text, store_text }>
                ↓
         前端 DevPanel 组件展示
```

`inspect_detect` / `inspect_store` 是与 proxy handler 解耦的纯函数，只在 handler 中被调用，不感知 Tauri、IPC、前端。

### 使用方式

入口暂定设置页加一个"开发者模式"开关。

打开后，请求列表页右侧出现一个开发者面板，展示当前选中请求的 detect 和 store 展平文本：

```
┌──────────────────────────────────┬────────────────────────────┐
│                                  │  开发者面板                │
│         请求列表                  │  ┌─ Store 展平 ────────┐  │
│                                  │  │ <｜begin▁of▁sentence│  │
│  请求 1  ← 选中                  │  │ ｜>You are...        │  │
│  请求 2                          │  │ <｜User｜>Hello!      │  │
│  请求 3                          │  │ ...                  │  │
│                                  │  └──────────────────────┘  │
│                                  │  ┌─ Detect 展平 ──────┐  │
│                                  │  │ <｜begin▁of▁sentence│  │
│                                  │  ｜>You are...          │  │
│                                  │  │ <｜User｜>Hello!      │  │
│                                  │  │ ...                  │  │
│                                  │  └──────────────────────┘  │
└──────────────────────────────────┴────────────────────────────┘
```

展示的内容只有 `apply_chat_template` 产出的纯文本，无额外 JSON 或 metadata。

### 文本展示要求

- 每个文本框独立滚动（文本可能很长）
- Cmd+/- 调整字号
- 系统等宽字体（SF Mono 或等宽 fallback）
- 设计简洁，不需要额外 UI 库

### 验证方式

不加断言。开发者肉眼对比两份文本在 transition token 边界处（`<｜Assistant｜>`、`<think>`、`</think>`、`<｜end▁of▁sentence｜>` 等）是否吻合。

### 文件（后端相关）

- 新建 `src-tauri/src/auditor/render_inspector.rs` — DevTrace、DevTraceBuffer、inspect_detect、inspect_store
- 修改 `src-tauri/src/proxy/handlers/claude.rs` — 开发者模式时调用 inspect_detect / inspect_store
- 修改 `src-tauri/src/proxy/handlers/openai.rs` — 同上
- 修改 `src-tauri/src/proxy/server.rs` — ProxyState 增加 DevTraceBuffer
- 修改 `src-tauri/src/api/commands.rs` — 新增 list_dev_traces IPC
- 修改 `src-tauri/src/lib.rs` — 初始化 DevTraceBuffer 并注入

### 前端组件

新增 `DevPanel` 组件，放在请求列表页右侧。

- 选中不同请求时显示对应的 trace
- 每个 trace 展示两个文本框：Store 展平、Detect 展平
- 带 model 标签
- 纯文本，无高亮
- 独立滚动容器
- Cmd+/- 缩放字号

---

## 验收标准

1. 用 Claude Code 发消息后，SQLite 中有对应的审计记录
2. 审计记录包含正确的 input/output token 数（与 DeepSeek 返回的接近，差异在预期范围内）
3. 前端请求列表页显示审计记录，可疑请求有标记
4. 流式转发体验与直连 DeepSeek 一致（无额外延迟）
5. 前端 UI 无法展示时显示空状态
