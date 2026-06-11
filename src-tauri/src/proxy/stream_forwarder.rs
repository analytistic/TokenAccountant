use axum::body::Body;
use axum::response::Response;
use bytes::Bytes;
use futures_util::StreamExt;
use serde_json::Value;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;

use crate::auditor::message_converter::{NormalizedMessage, ToolCall};

/// Accumulator for streaming tool call arguments (partial JSON fragments).
#[derive(Debug, Clone)]
struct ToolCallAccum {
    name: String,
    fragments: Vec<String>,
}

impl ToolCallAccum {
    fn to_tool_call(&self) -> ToolCall {
        ToolCall {
            name: self.name.clone(),
            arguments: self.fragments.concat(),
        }
    }
}

pub struct StreamForwarder {
    pub stream_ended: Arc<AtomicBool>,
    pub on_chunk: Option<Arc<dyn Fn(&[u8]) + Send + Sync>>,
    // Raw accumulation (for usage extraction from the last event)
    raw_text: Arc<Mutex<String>>,
    // Structured accumulation
    thinking_parts: Arc<Mutex<Vec<String>>>,
    text_parts: Arc<Mutex<Vec<String>>>,
    tool_calls: Arc<Mutex<Vec<ToolCallAccum>>>,
}

impl StreamForwarder {
    pub fn new() -> Self {
        StreamForwarder {
            raw_text: Arc::new(Mutex::new(String::new())),
            thinking_parts: Arc::new(Mutex::new(Vec::new())),
            text_parts: Arc::new(Mutex::new(Vec::new())),
            tool_calls: Arc::new(Mutex::new(Vec::new())),
            stream_ended: Arc::new(AtomicBool::new(false)),
            on_chunk: None,
        }
    }

    pub fn with_on_chunk(mut self, cb: Arc<dyn Fn(&[u8]) + Send + Sync>) -> Self {
        self.on_chunk = Some(cb);
        self
    }

    /// Forward SSE response while parsing events into structured accumulators.
    pub async fn forward_stream(&self, upstream_resp: reqwest::Response) -> Response<Body> {
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
        let raw = self.raw_text.clone();
        let thinking = self.thinking_parts.clone();
        let text = self.text_parts.clone();
        let tool_calls = self.tool_calls.clone();
        let ended = self.stream_ended.clone();
        let on_chunk = self.on_chunk.clone();

        tokio::spawn(async move {
            let mut stream = upstream_resp.bytes_stream();
            while let Some(item) = stream.next().await {
                match item {
                    Ok(chunk) => {
                        // 1. Accumulate raw text (for usage extraction)
                        let chunk_str = if let Ok(s) = std::str::from_utf8(&chunk) {
                            s.to_string()
                        } else {
                            String::from_utf8_lossy(&chunk).to_string()
                        };
                        raw.lock().await.push_str(&chunk_str);

                        // 2. Parse SSE event and route to structured accumulators
                        Self::parse_and_route(&chunk_str, &thinking, &text, &tool_calls).await;

                        // 3. Forward chunk to client
                        if let Some(ref cb) = on_chunk {
                            cb(&chunk);
                        }
                        if tx.send(chunk).await.is_err() {
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

        let body = Body::from_stream(ReceiverStream::new(rx).map(|b| Ok::<_, axum::Error>(b)));
        rb.body(body).expect("valid response builder after setting status and headers")
    }

    /// Parse a single SSE event chunk and route deltas to accumulators.
    async fn parse_and_route(
        chunk: &str,
        thinking: &Mutex<Vec<String>>,
        text: &Mutex<Vec<String>>,
        tool_calls: &Mutex<Vec<ToolCallAccum>>,
    ) {
        // Process ALL `data:` lines in the chunk (may contain multiple SSE events)
        for line in chunk.lines() {
            let Some(json_str) = line.strip_prefix("data: ") else { continue };
            let Ok(val) = serde_json::from_str::<Value>(json_str) else { continue };

            let typ = val.get("type").and_then(|t| t.as_str());
            match typ {
            Some("content_block_start") => {
                // Check if the block is a tool_use
                if let Some(block) = val.get("content_block") {
                    if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                        let name = block
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string();
                        tool_calls.lock().await.push(ToolCallAccum {
                            name,
                            fragments: Vec::new(),
                        });
                    }
                }
            }
            Some("content_block_delta") => {
                let delta = val.get("delta");
                let delta_type = delta.and_then(|d| d.get("type")).and_then(|t| t.as_str());
                match delta_type {
                    Some("thinking_delta") => {
                        if let Some(t) = delta.and_then(|d| d.get("thinking")).and_then(|v| v.as_str()) {
                            thinking.lock().await.push(t.to_string());
                        }
                    }
                    Some("text_delta") => {
                        if let Some(t) = delta.and_then(|d| d.get("text")).and_then(|v| v.as_str()) {
                            text.lock().await.push(t.to_string());
                        }
                    }
                    Some("input_json_delta") => {
                        if let Some(j) = delta.and_then(|d| d.get("partial_json")).and_then(|v| v.as_str()) {
                            let mut tc = tool_calls.lock().await;
                            if let Some(last) = tc.last_mut() {
                                last.fragments.push(j.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        } // end for line in chunk.lines()
    }

    /// Build a NormalizedMessage from the accumulated structured parts.
    /// Call after stream has ended.
    pub async fn build_output(&self) -> NormalizedMessage {
        let thinking = {
            let parts = self.thinking_parts.lock().await;
            if parts.is_empty() {
                None
            } else {
                Some(parts.concat())
            }
        };

        let content = {
            let parts = self.text_parts.lock().await;
            parts.concat()
        };

        let tool_calls = {
            let tcs = self.tool_calls.lock().await;
            if tcs.is_empty() {
                None
            } else {
                Some(tcs.iter().map(|tc| tc.to_tool_call()).collect())
            }
        };

        NormalizedMessage {
            role: "assistant".into(),
            content_parts: vec![crate::auditor::message_converter::ContentPart::Text(content)],
            reasoning: thinking,
            tool_calls,
            tool_call_id: None,
        }
    }

    /// Get raw accumulated text (for usage extraction from the last event).
    pub async fn get_raw_text(&self) -> String {
        self.raw_text.lock().await.clone()
    }

    /// Backward compatibility: returns raw accumulated text.
    pub async fn get_text(&self) -> String {
        self.get_raw_text().await
    }
}
