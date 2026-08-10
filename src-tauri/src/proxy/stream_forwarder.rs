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

#[derive(Default)]
struct SseLineBuffer {
    bytes: Vec<u8>,
}

impl SseLineBuffer {
    fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        self.bytes.extend_from_slice(chunk);
        let mut lines = Vec::new();
        while let Some(newline) = self.bytes.iter().position(|byte| *byte == b'\n') {
            let raw: Vec<u8> = self.bytes.drain(..=newline).collect();
            lines.push(String::from_utf8_lossy(&raw)
                .trim_end_matches(['\r', '\n']).to_string());
        }
        lines
    }

    fn finish(&mut self) -> Option<String> {
        if self.bytes.is_empty() { None }
        else { Some(String::from_utf8_lossy(&std::mem::take(&mut self.bytes)).to_string()) }
    }
}

/// Accumulator for streaming tool call arguments (partial JSON fragments).
#[derive(Debug, Clone)]
struct ToolCallAccum {
    id: Option<String>,
    name: String,
    fragments: Vec<String>,
}

impl ToolCallAccum {
    fn to_tool_call(&self) -> ToolCall {
        ToolCall {
            id: self.id.clone(),
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
            let mut line_buffer = SseLineBuffer::default();
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

                        // 2. Buffer partial network chunks until complete SSE lines exist.
                        for line in line_buffer.push(&chunk) {
                            Self::parse_and_route_line(&line, &thinking, &text, &tool_calls).await;
                        }

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
            if let Some(line) = line_buffer.finish() {
                Self::parse_and_route_line(&line, &thinking, &text, &tool_calls).await;
            }
            ended.store(true, Ordering::SeqCst);
        });

        let body = Body::from_stream(ReceiverStream::new(rx).map(|b| Ok::<_, axum::Error>(b)));
        rb.body(body).expect("valid response builder after setting status and headers")
    }

    async fn parse_and_route_line(
        line: &str,
        thinking: &Mutex<Vec<String>>,
        text: &Mutex<Vec<String>>,
        tool_calls: &Mutex<Vec<ToolCallAccum>>,
    ) {
        let Some(json_str) = line.strip_prefix("data:").map(str::trim_start) else { return };
        if json_str == "[DONE]" { return; }
        let Ok(val) = serde_json::from_str::<Value>(json_str) else { return };

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
                            id: block.get("id").and_then(|id| id.as_str()).map(str::to_owned),
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
            _ => Self::route_openai_delta(&val, thinking, text, tool_calls).await,
        }
    }

    async fn route_openai_delta(
        value: &Value,
        thinking: &Mutex<Vec<String>>,
        text: &Mutex<Vec<String>>,
        tool_calls: &Mutex<Vec<ToolCallAccum>>,
    ) {
        let Some(delta) = value.get("choices").and_then(Value::as_array)
            .and_then(|choices| choices.first()).and_then(|choice| choice.get("delta")) else { return };

        if let Some(part) = delta.get("reasoning_content").or_else(|| delta.get("reasoning"))
            .and_then(Value::as_str) {
            thinking.lock().await.push(part.to_string());
        }
        if let Some(part) = delta.get("content").and_then(Value::as_str) {
            text.lock().await.push(part.to_string());
        }
        if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
            let mut accumulators = tool_calls.lock().await;
            for call in calls {
                let index = call.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                while accumulators.len() <= index {
                    accumulators.push(ToolCallAccum { id: None, name: String::new(), fragments: Vec::new() });
                }
                let accumulator = &mut accumulators[index];
                if let Some(id) = call.get("id").and_then(Value::as_str) {
                    accumulator.id = Some(id.to_string());
                }
                if let Some(function) = call.get("function") {
                    if let Some(name) = function.get("name").and_then(Value::as_str) {
                        accumulator.name.push_str(name);
                    }
                    if let Some(arguments) = function.get("arguments").and_then(Value::as_str) {
                        accumulator.fragments.push(arguments.to_string());
                    }
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use super::SseLineBuffer;

    #[test]
    fn buffers_sse_lines_split_across_network_chunks() {
        let mut buffer = SseLineBuffer::default();
        assert!(buffer.push(b"data: {\"type\":\"content_bl").is_empty());
        let lines = buffer.push(b"ock_delta\"}\r\n\r\n");
        assert_eq!(lines, vec!["data: {\"type\":\"content_block_delta\"}", ""]);
        assert!(buffer.finish().is_none());
    }

    #[test]
    fn preserves_utf8_split_across_chunks() {
        let mut buffer = SseLineBuffer::default();
        let bytes = "data: 中文\n".as_bytes();
        assert!(buffer.push(&bytes[..8]).is_empty());
        assert_eq!(buffer.push(&bytes[8..]), vec!["data: 中文"]);
    }
}
