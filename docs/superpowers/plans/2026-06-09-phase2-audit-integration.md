# Phase 2: Audit Integration & SSE Streaming

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect Tokenizer, DiffComparator, CacheDetector into the proxy flow for automated request/response auditing.

**Architecture:** Request arrives → spawn parallel tasks: (1) forward to upstream and stream SSE response to client, (2) tokenize input and detect cache. When stream ends, tokenize output, compare with upstream usage, store in SQLite.

**Tech Stack:** Axum 0.7, reqwest 0.12, reuse existing `auditor/` modules. `http-body-util` and `futures-util` are already available transitively.

---

### Task 1: Create StreamForwarder

**Files:**
- Create: `src-tauri/src/proxy/stream_forwarder.rs`
- Modify: `src-tauri/src/proxy/mod.rs`

`StreamForwarder` reads upstream SSE chunks via `reqwest::Response::bytes_stream()`, forwards them to the client via an `mpsc` channel + `StreamBody`, and accumulates text for post-stream audit.

- [ ] **Step 1: Create stream_forwarder.rs**

```rust
use axum::body::{Body, boxed};
use axum::response::Response;
use bytes::Bytes;
use futures_util::StreamExt;
use http_body_util::StreamBody;
use reqwest::Response as UpstreamResponse;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;

pub struct StreamForwarder {
    pub accumulated_text: Arc<Mutex<String>>,
    pub stream_ended: Arc<AtomicBool>,
    pub on_chunk: Option<Arc<dyn Fn(&[u8]) + Send + Sync>>,
}

impl StreamForwarder {
    pub fn new() -> Self {
        StreamForwarder {
            accumulated_text: Arc::new(Mutex::new(String::new())),
            stream_ended: Arc::new(AtomicBool::new(false)),
            on_chunk: None,
        }
    }

    /// Set per-chunk callback (Pattern B entry point).
    pub fn with_on_chunk(mut self, cb: Arc<dyn Fn(&[u8]) + Send + Sync>) -> Self {
        self.on_chunk = Some(cb);
        self
    }

    /// Forward SSE response to client while accumulating text.
    pub async fn forward_stream(self, upstream_resp: UpstreamResponse) -> Response<Body> {
        let status = upstream_resp.status();
        let mut rb = Response::builder().status(status);
        for (k, v) in upstream_resp.headers().iter() {
            let name = k.as_str();
            if name.eq_ignore_ascii_case("transfer-encoding")
                || name.eq_ignore_ascii_case("content-encoding")
                || name.eq_ignore_ascii_case("connection")
            {
                continue;
            }
            rb = rb.header(name, v.as_bytes());
        }

        let (tx, rx) = mpsc::channel::<Bytes>(64);
        let acc = self.accumulated_text.clone();
        let ended = self.stream_ended.clone();
        let on_chunk = self.on_chunk.clone();

        tokio::spawn(async move {
            let mut stream = upstream_resp.bytes_stream();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        if let Ok(text) = std::str::from_utf8(&bytes) {
                            acc.lock().await.push_str(text);
                        }
                        if let Some(ref cb) = on_chunk {
                            cb(&bytes);
                        }
                        if tx.send(bytes).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Stream chunk error: {}", e);
                        break;
                    }
                }
            }
            ended.store(true, Ordering::SeqCst);
        });

        let body = boxed(StreamBody::new(ReceiverStream::new(rx).map(Ok::<_, std::convert::Infallible>)));
        rb.body(Body::new(body)).unwrap()
    }

    pub async fn get_text(&self) -> String {
        self.accumulated_text.lock().await.clone()
    }
}
```

- [ ] **Step 2: Add mod declaration in proxy/mod.rs**

```rust
pub mod stream_forwarder;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Clean compilation (test for boxed/StreamBody API compatibility — adjust as needed)

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/proxy/stream_forwarder.rs src-tauri/src/proxy/mod.rs
git commit -m "feat: add StreamForwarder for SSE streaming with text accumulation"
```

---

### Task 2: Extend ProxyState with audit components

**Files:**
- Modify: `src-tauri/src/proxy/server.rs`
- Modify: `src-tauri/src/lib.rs`

Add `TokenizerFactory`, `DiffComparator`, `CacheDetector`, and `Database` to `ProxyState` so handlers can access audit components.

- [ ] **Step 1: Extend ProxyState in server.rs**

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

Update `ProxyServer::new` to accept the extra components:

```rust
pub fn new(
    provider_manager: Arc<Mutex<crate::provider::manager::ProviderManager>>,
    tokenizer_factory: Arc<TokenizerFactory>,
    diff_comparator: Arc<DiffComparator>,
    cache_detector: Arc<Mutex<CacheDetector>>,
    db: Arc<Mutex<Database>>,
) -> Self {
    let state = ProxyState {
        provider_manager,
        status: Arc::new(Mutex::new(ProxyStatus { running: false, port: 8080, uptime_secs: 0, requests_served: 0 })),
        tokenizer_factory,
        diff_comparator,
        cache_detector,
        db,
    };
    ProxyServer { state }
}
```

- [ ] **Step 2: Initialize audit components in lib.rs**

In `run()`, after creating `provider_manager`:

```rust
// Initialize audit components
use std::sync::Arc as StdArc;
use crate::auditor::tokenizer::TokenizerFactory;
use crate::auditor::diff_comparator::DiffComparator;
use crate::auditor::cache_detector::CacheDetector;
use crate::auditor::tokenizers::gpt::GptTokenizer;

let mut tokenizer_factory = TokenizerFactory::new();
let gpt = StdArc::new(GptTokenizer::new());
tokenizer_factory.register("gpt".into(), gpt);

let diff_comparator = DiffComparator::new(cfg.audit.suspicion_threshold);
let cache_detector = CacheDetector::new();
```

Update the `ProxyServer::new` call in `run()`:

```rust
let server = ProxyServer::new(
    pm.clone(),
    StdArc::new(tokenizer_factory),
    StdArc::new(diff_comparator),
    StdArc::new(Mutex::new(cache_detector)),
    db.clone(),
);
```

**Also update `api/commands.rs`:** `start_proxy` creates a `ProxyServer::new(pm)`. Change it to pass the new fields. The `TauriState` needs to include the new audit components, or `start_proxy` can receive them from the existing state.

```rust
// In start_proxy (commands.rs)
let server = ProxyServer::new(
    state.provider_manager.clone(),
    /* need tokenizer_factory, diff_comparator, cache_detector, db here */
);
```

This means `TauriState` also needs to hold references to these new components. Add them:

```rust
pub struct TauriState {
    pub provider_manager: ...,
    pub proxy_server: ...,
    pub proxy_status: ...,
    pub proxy_handle: ...,
    pub original_env: ...,
    // New:
    pub tokenizer_factory: Arc<TokenizerFactory>,
    pub diff_comparator: Arc<DiffComparator>,
    pub cache_detector: Arc<Mutex<CacheDetector>>,
    pub db: Arc<Mutex<Database>>,
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Clean compilation

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/proxy/server.rs src-tauri/src/lib.rs
git commit -m "feat: extend ProxyState with audit components"
```

---

### Task 3: Audit Integration in claude.rs

**Files:**
- Modify: `src-tauri/src/proxy/handlers/claude.rs`

Insert audit into the proxy flow: request arrives → spawn parallel audit task → forward → stream response → when stream ends → diff → store.

- [ ] **Step 1: Add request audit (parallel with forwarding)**

Replace the current handler body. The pattern:

```rust
use crate::auditor::model_detector::{detect, extract_request_text, extract_usage, ApiFormat};
use crate::proxy::stream_forwarder::StreamForwarder;

async fn forward_with_audit(
    state: ProxyState,
    req: Request<Body>,
    _api_type: &str,
) -> Response<Body> {
    // --- 1. Read request body ---
    let body_bytes = axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await.unwrap_or_default();
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    // --- 2. Detect model and API format ---
    let path = req.uri().path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| "/v1/messages".to_string());
    let detected = detect(&path, &body_str);
    let tokenizer = state.tokenizer_factory.for_model(&detected.model);

    // --- 3. Spawn parallel audit task (input tokens + cache) ---
    let audit_state = state.clone();
    let audit_body = body_str.clone();
    let audit_detected_model = detected.model.clone();
    let audit_fmt = detected.api_format;
    let audit_handle = tokio::spawn(async move {
        let request_text = extract_request_text(&audit_body, audit_fmt);
        let (real_input, real_cached) = if let Some(ref t) = tokenizer {
            let ids = t.encode(&request_text);
            let (cached_hit, _) = audit_state.cache_detector.lock().await.detect(&ids);
            audit_state.cache_detector.lock().await.store(&ids);
            (t.count_tokens(&request_text) as i32, cached_hit as i32)
        } else {
            (0, 0)
        };
        (real_input, real_cached, audit_detected_model, request_text)
    });

    // --- 4. Build upstream URL and headers ---
    let provider_url = {
        let pm = state.provider_manager.lock().await;
        pm.get_active().await.ok().flatten()
            .map(|p| p.api_base_url)
            .unwrap_or_else(|| "https://api.anthropic.com".to_string())
    };
    let upstream_url = format!("{}{}", provider_url.trim_end_matches('/'), path);
    let client = reqwest::Client::new();
    let mut upstream_req = client.request(req.method().clone(), &upstream_url);
    for (k, v) in req.headers().iter() {
        if k.as_str().eq_ignore_ascii_case("host") { continue; }
        upstream_req = upstream_req.header(k, v);
    }
    let upstream_req = upstream_req.body(body_bytes).send().await;

    // --- 5. Handle response ---
    match upstream_req {
        Ok(resp) => {
            let is_streaming = resp.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.contains("text/event-stream"))
                .unwrap_or(false);

            if is_streaming {
                let forwarder = StreamForwarder::new();
                let response = forwarder.forward_stream(resp).await;

                // --- 6. Post-stream audit ---
                let full_text = forwarder.get_text().await;
                let (claimed_input, claimed_output, claimed_cached) =
                    extract_usage(&full_text, detected.api_format);

                let audit_result = audit_handle.await.unwrap_or((0, 0, String::new(), String::new()));
                let (real_input, real_cached, model_name, _req_text) = audit_result;

                let real_output = if let Some(ref t) = tokenizer {
                    // Extract assistant response text from accumulated SSE for token counting
                    t.count_tokens(&full_text) as i32
                } else { 0 };

                let record = state.diff_comparator.compare(
                    "", &model_name, detected.api_format.as_str(),
                    &body_str, &full_text,
                    claimed_input, claimed_output, claimed_cached,
                    real_input, real_output, real_cached,
                );
                state.db.lock().await.insert_audit_log(&record).ok();
                {
                    let mut s = state.status.lock().await;
                    s.requests_served += 1;
                }

                response
            } else {
                // Non-streaming: existing buffer path
                let status = resp.status();
                let resp_headers = resp.headers().clone();
                let resp_body = resp.bytes().await.unwrap_or_default();
                let mut response = Response::builder().status(status);
                for (k, v) in resp_headers.iter() {
                    let name = k.as_str();
                    if name.eq_ignore_ascii_case("transfer-encoding")
                        || name.eq_ignore_ascii_case("content-encoding")
                        || name.eq_ignore_ascii_case("connection")
                    { continue; }
                    response = response.header(name, v.as_bytes());
                }
                response.body(axum::body::Body::from(resp_body)).unwrap()
            }
        }
        Err(e) => {
            tracing::error!("Upstream request failed: {:?}", e);
            Response::builder()
                .status(502)
                .body(axum::body::Body::from(format!("Proxy error: {}", e)))
                .unwrap()
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Clean compilation (fix any type mismatches — `extract_usage` may need input as &str, etc.)

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/proxy/handlers/claude.rs
git commit -m "feat: add audit integration to Claude proxy handler"
```

---

### Task 4: Audit Integration in openai.rs

**Files:**
- Modify: `src-tauri/src/proxy/handlers/openai.rs`

Mirror the same audit pattern from Task 3 into `handle_openai`.

- [ ] **Step 1: Apply same audit pattern**

Copy the audit logic from claude.rs into openai.rs. The structure is identical:
1. Read body, detect model, spawn audit task
2. Build upstream request (skip host header)
3. If streaming → use StreamForwarder → post-stream audit
4. If non-streaming → buffer and return

The only differences are:
- Default path: `/v1/chat/completions`
- Default upstream: `https://api.openai.com`

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Clean compilation

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/proxy/handlers/openai.rs
git commit -m "feat: add audit integration to OpenAI proxy handler"
```

---

### Task 5: Frontend Request List Page

**Files:**
- Modify: `frontend/src/App.tsx`
- Create: `frontend/src/pages/RequestList.tsx` (replace placeholder)
- Modify: `src-tauri/src/api/commands.rs` (add list_audit_logs IPC command)
- Modify: `src-tauri/src/lib.rs` (register new command)

- [ ] **Step 1: Add list_audit_logs to ProviderManager**

`ProviderManager` already wraps `db` (Arc<Mutex<Database>>). Add a passthrough method:

```rust
// In provider/manager.rs
pub async fn list_audit_logs(&self, limit: i64, offset: i64, suspicious_only: bool) -> Result<Vec<crate::auditor::diff_comparator::AuditRecord>> {
    let db = self.db.lock().await;
    db.list_audit_logs(limit, offset, suspicious_only)
}
```

- [ ] **Step 2: Add list_audit_logs IPC command in commands.rs**

```rust
#[tauri::command]
pub async fn list_audit_logs(
    state: State<'_, TauriState>,
    limit: i64,
    offset: i64,
) -> Result<Vec<crate::auditor::diff_comparator::AuditRecord>, String> {
    state.provider_manager.lock().await
        .list_audit_logs(limit, offset, false)
        .await
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 3: Register list_audit_logs in lib.rs**

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    api::commands::list_audit_logs,
])
```

- [ ] **Step 3: Create RequestList.tsx**

```tsx
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface AuditRecord {
  id: number;
  timestamp: string;
  model: string;
  api_format: string;
  claimed_input_tokens: number;
  claimed_output_tokens: number;
  claimed_cached_tokens: number;
  real_input_tokens: number;
  real_output_tokens: number;
  detected_cached_tokens: number;
  input_diff: number;
  output_diff: number;
  cache_diff: number;
  is_suspicious: boolean;
  suspicion_reason: string;
}

export default function RequestList() {
  const [records, setRecords] = useState<AuditRecord[]>([]);

  const load = () => {
    invoke<AuditRecord[]>("list_audit_logs", { limit: 100, offset: 0 })
      .then(setRecords)
      .catch(console.error);
  };

  useEffect(load, []);

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold">请求列表</h2>
        <button onClick={load} className="bg-gray-700 hover:bg-gray-600 px-3 py-1 rounded text-sm">
          刷新
        </button>
      </div>
      <div className="space-y-3">
        {records.map(r => (
          <div key={r.id}
            className={`bg-gray-800 rounded-lg p-4 border ${
              r.is_suspicious ? 'border-red-500' : 'border-gray-700'
            }`}
          >
            <div className="flex justify-between items-start">
              <div>
                <span className="font-bold">{r.model}</span>
                <span className="text-gray-500 text-sm ml-2">({r.api_format})</span>
              </div>
              <span className={r.is_suspicious ? 'text-red-400' : 'text-green-400'}>
                {r.is_suspicious ? '可疑' : '正常'}
              </span>
            </div>
            <div className="grid grid-cols-3 gap-4 mt-2 text-sm">
              <div>
                <span className="text-gray-500">Input</span>
                <div>{r.claimed_input_tokens} (声称) vs {r.real_input_tokens} (实际)</div>
                {r.input_diff !== 0 && (
                  <div className={r.input_diff > 0 ? "text-yellow-400" : "text-blue-400"}>
                    差异: {r.input_diff > 0 ? "+" : ""}{r.input_diff}
                  </div>
                )}
              </div>
              <div>
                <span className="text-gray-500">Output</span>
                <div>{r.claimed_output_tokens} vs {r.real_output_tokens}</div>
                {r.output_diff !== 0 && (
                  <div className={r.output_diff > 0 ? "text-yellow-400" : "text-blue-400"}>
                    差异: {r.output_diff > 0 ? "+" : ""}{r.output_diff}
                  </div>
                )}
              </div>
              <div>
                <span className="text-gray-500">Cache</span>
                <div>{r.claimed_cached_tokens} vs {r.detected_cached_tokens}</div>
                {r.cache_diff !== 0 && <div className="text-yellow-400">差异: {r.cache_diff}</div>}
              </div>
            </div>
            {r.is_suspicious && (
              <div className="text-sm text-red-500 mt-2 bg-red-900/20 p-2 rounded">
                {r.suspicion_reason}
              </div>
            )}
            <div className="text-xs text-gray-600 mt-2">{r.timestamp}</div>
          </div>
        ))}
        {records.length === 0 && (
          <div className="text-gray-500 text-center py-12">暂无审计记录</div>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Update App.tsx**

Replace the placeholder import with the real component:

```tsx
import RequestList from "./pages/RequestList";
```

- [ ] **Step 5: Verify compilation**

Run: `cargo check && cd frontend && npx tsc --noEmit`
Expected: Both Rust and TypeScript compile cleanly

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/api/commands.rs src-tauri/src/lib.rs src-tauri/src/provider/manager.rs frontend/src/pages/RequestList.tsx frontend/src/App.tsx
git commit -m "feat: add audit log list IPC and frontend request list page"
```

---

### Task 6: B-Plan Extension Point

**Files:**
- Modify: `src-tauri/src/proxy/stream_forwarder.rs`

The `StreamForwarder` already has `on_chunk` callback support. This task adds documentation and a test to ensure the interface works.

- [ ] **Step 1: Add documentation for Pattern B**

In `stream_forwarder.rs`, add a doc comment:

```rust
// Pattern B (future, not yet implemented):
// To enable real-time per-chunk token counting, use with_on_chunk():
//
//   let forwarder = StreamForwarder::new()
//       .with_on_chunk(Arc::new(|bytes: &[u8]| {
//           // Count tokens in this chunk, update running total, etc.
//       }));
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/proxy/stream_forwarder.rs
git commit -m "docs: add Pattern B extension point documentation"
```

---

## Self-Review

### Spec Coverage

| Spec Requirement | Task |
|---|---|
| SSE streaming (chunk-by-chunk) | Task 1: StreamForwarder |
| Request audit (input tokens + cache) | Task 3: spawn audit task |
| Response audit (output tokens + diff) | Task 3: post-stream audit |
| Store audit record in SQLite | Task 3: insert_audit_log |
| Frontend request list | Task 5 |
| B-Plan extension point | Task 6 |
| Non-streaming handling | Task 3: existing buffer path preserved |
| Extend ProxyState | Task 2 |

### Placeholder Check
- No "TBD", "TODO", or "implement later" in code blocks
- Each step has complete code or clear instructions
- No references to undefined functions or types

### Type Consistency
- `ProxyState` fields match spec design
- `StreamForwarder::forward_stream` returns `Response<Body>` consistent with existing handlers
- `AuditRecord` type from `diff_comparator.rs` is used in IPC command return type
- `extract_usage` returns `(i32, i32, i32)` matching existing signature

---

## Post-Implementation Findings

### Issue 1: `extract_request_text` 遗漏大量 input 内容

**发现时间:** 2026-06-10, Debug/验收阶段

**问题:** `extract_request_text` 只提取了 `messages[].content` 中 `string` 类型的文本，导致 `real_input_tokens` 严重偏低。遗漏的内容包括：

| 遗漏内容 | 原因 |
|---|---|
| 外层 `system` 字段（3 条 text block） | 未读取 |
| `messages[].content` 中 `tool_use` block | 仅处理 `as_str()`，数组内容跳过 |
| `messages[].content` 中 `tool_result` block | 同上 |
| `messages[].content` 中 `thinking` block | 同上 |
| `tools` 定义（26 个工具） | 未读取 |

**修复方案:**
1. 新建 `auditor/message_converter.rs`，负责将 Anthropic API 格式转换为标准化的 `Vec<NormalizedMessage>`
2. 按 `system → tools → messages[0..n]` 顺序组织
3. `extract_request_text` 被移除，调用方改用 `message_converter::from_anthropic_body()` + `tokenizer.apply_chat_template()`

### Issue 2: Chat template 归属不清晰

**问题:** 原来的 `extract_request_text` 直接在格式转换层拼接文本，但 chat template（模型特有分隔符、特殊 token）应属于 tokenizer 层职责。

**修正:**
- `Tokenizer` trait 新增 `apply_chat_template()` 方法
- GptTokenizer 实现简单文本拼接，ClaudeTokenizer 保留占位
- 未来 DeepSeek `encoding_dsv4` 接入时只需在该 tokenizer 里实现 DSML 格式的 template

### Issue 3: CacheDetector 低估 cache hit（缺失 output token 存储）

**发现时间:** 2026-06-10 Debug/验收阶段

**问题:** `cache_detector.store(&ids)` 只存了请求 prompt 的 token IDs，output token 从未存入。但 vLLM 的 KV Cache 同时缓存 prompt 和 decode 产出的 output tokens。

**修复:** 按 block hash chain 方案重写 CacheDetector，post-stream 调 `store_combined(prompt_ids, output_ids)`。见 `2026-06-10-cache-detector-fix.md`。

### Issue 4: Post-stream 轮询等待流结束

**发现时间:** 2026-06-10 Debug/验收阶段

**问题:** post-stream 使用 `while !stream_ended.load(...) { sleep(50ms).await }` 轮询等待流结束。这是忙等，延迟粗糙（50ms 粒度），增加 CPU 浪费。

**修复方向:** 改用 `tokio::sync::Notify` 事件通知替代轮询。简化为：`forward_stream` 完成后 `notify.notified()` 唤醒等待的 post-stream task。
