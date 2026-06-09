# Phase 2: 审计接入与 SSE 流式转发

> 设计文档 — 实施计划由 writing-plans 另行生成

**Goal:** 将 Tokenizer、DiffComparator、CacheDetector 接入代理流程，实现请求/响应的自动化审计，并在前端展示审计结果

**Architecture:** 在现有 Axum Handler 中插入审计步骤，采用"先转发后审计"模式（A 方案）。请求到达时先算 input tokens 和缓存命中，然后转发。响应使用 SSE 流式逐块转发并累加文本，流结束后算 output tokens，对比差异，结果存 SQLite 并展示在前端。预留"边转发边审计"的扩展点（B 方案）。

---

## 设计决策

### 1. 审计模式：A 方案（先转发后审计）

**选择理由：**
- 不影响流式体验（Claude Code 实时看到 thinking token）
- 流结束后算总账更精确（tiktoken 对整个文本一次性 encode）
- 复杂度低，容易验证
- 对于显著虚报（>5%），A 方案完全够用

**流程：**
```
请求到达 → 算 input tokens → 检测缓存 → 转发请求
                                            ↓
                              SSE 流式转发（逐 chunk）→ Claude Code
                              ↓
                              累加文本
                              ↓
                    流结束 → 算 output tokens
                     ↓
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

## 验收标准

1. 用 Claude Code 发消息后，SQLite 中有对应的审计记录
2. 审计记录包含正确的 input/output token 数（与 DeepSeek 返回的接近，差异在预期范围内）
3. 前端请求列表页显示审计记录，可疑请求有标记
4. 流式转发体验与直连 DeepSeek 一致（无额外延迟）
5. 前端 UI 无法展示时显示空状态
