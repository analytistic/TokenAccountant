use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResponseFingerprint {
    pub status: String,
    pub response_id: Option<String>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeaderFingerprint {
    pub status: String,
    pub request_id: Option<String>,
    pub issues: Vec<String>,
}

pub fn should_check_origin(model: &str) -> bool {
    let model = model.to_ascii_lowercase();
    model.starts_with("claude-")
        || model.starts_with("gpt-")
        || model.starts_with("chatgpt-")
        || model.starts_with("o1")
        || model.starts_with("o3")
        || model.starts_with("o4")
        || model.starts_with("codex-")
        || model.starts_with("text-embedding-")
}

pub fn not_applicable_response() -> ResponseFingerprint {
    ResponseFingerprint {
        status: "not_applicable".into(),
        response_id: None,
        issues: vec!["仅对 Anthropic Claude 与 OpenAI 模型启用来源指纹检测".into()],
    }
}

pub fn not_applicable_headers() -> HeaderFingerprint {
    HeaderFingerprint {
        status: "not_applicable".into(),
        request_id: None,
        issues: vec!["当前模型不属于来源指纹检测范围".into()],
    }
}

pub fn capture_headers(headers: &reqwest::header::HeaderMap) -> String {
    const ALLOWED: &[&str] = &[
        "server", "via", "x-powered-by", "x-request-id", "request-id",
        "openai-organization", "openai-processing-ms", "openai-version",
        "cf-ray", "cf-cache-status", "x-vercel-id", "x-served-by",
        "x-envoy-upstream-service-time", "x-amzn-requestid", "x-amz-cf-id",
        "retry-after", "x-cursor-proxy-prompt-truncated",
    ];

    let mut captured = serde_json::Map::new();
    for (name, value) in headers {
        let key = name.as_str().to_ascii_lowercase();
        let is_allowed = ALLOWED.contains(&key.as_str())
            || key.starts_with("x-ratelimit-")
            || key.starts_with("anthropic-ratelimit-")
            || key.starts_with("x-cursor-")
            || key.starts_with("x-bridge-")
            || key.starts_with("x-proxy-");
        if is_allowed {
            if let Ok(value) = value.to_str() {
                captured.insert(key, Value::String(value.chars().take(512).collect()));
            }
        }
    }
    Value::Object(captured).to_string()
}

pub fn analyze_headers(api_format: &str, raw_headers: &str) -> HeaderFingerprint {
    let headers = serde_json::from_str::<Value>(raw_headers)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    let get = |name: &str| headers.get(name).and_then(Value::as_str).map(str::to_owned);
    let request_id = get("x-request-id").or_else(|| get("request-id"));
    let names: Vec<&str> = headers.keys().map(String::as_str).collect();
    let mut issues = Vec::new();
    let mut suspicious = false;

    let bridge_headers: Vec<&str> = names.iter().copied().filter(|name| {
        name.starts_with("x-cursor-") || name.starts_with("x-bridge-") || name.starts_with("x-proxy-")
    }).collect();
    if !bridge_headers.is_empty() {
        issues.push(format!("响应泄漏桥接服务头：{}", bridge_headers.join(", ")));
        suspicious = true;
    }

    match api_format {
        "openai" => {
            if names.iter().any(|name| name.starts_with("anthropic-ratelimit-")) {
                issues.push("OpenAI 响应包含 Anthropic 专属限流头".into());
                suspicious = true;
            }
            if request_id.is_none() && !names.iter().any(|name| name.starts_with("openai-")) {
                issues.push("未观察到 OpenAI 请求追踪或元信息头".into());
            }
        }
        "anthropic" => {
            if names.iter().any(|name| name.starts_with("openai-")) {
                issues.push("Anthropic 响应包含 OpenAI 专属元信息头".into());
                suspicious = true;
            }
            if request_id.is_none() && !names.iter().any(|name| name.starts_with("anthropic-ratelimit-")) {
                issues.push("未观察到 Anthropic 请求追踪或限流头".into());
            }
        }
        _ => {
            if headers.is_empty() {
                issues.push("没有可用于识别的响应头".into());
            }
        }
    }

    if headers.contains_key("x-powered-by") {
        issues.push("检测到应用框架响应头，链路可能包含额外代理层".into());
    }
    if headers.contains_key("via") {
        issues.push("检测到 Via 响应头，链路包含代理节点".into());
    }

    let status = if suspicious {
        "suspicious"
    } else if issues.is_empty() {
        "consistent"
    } else {
        "nonstandard"
    };
    HeaderFingerprint { status: status.into(), request_id, issues }
}

pub fn analyze(api_format: &str, raw_response: &str) -> ResponseFingerprint {
    let Some(value) = first_response_json(raw_response) else {
        return ResponseFingerprint {
            status: "suspicious".into(),
            response_id: None,
            issues: vec!["响应不是可解析的 JSON 或 SSE".into()],
        };
    };

    match api_format {
        "openai" => analyze_openai(&value),
        "anthropic" => analyze_anthropic(&value),
        "gemini" => analyze_gemini(&value),
        _ => ResponseFingerprint {
            status: "unknown".into(),
            response_id: extract_id(&value),
            issues: vec!["请求协议未识别，无法匹配响应指纹".into()],
        },
    }
}

fn first_response_json(raw: &str) -> Option<Value> {
    if let Ok(value) = serde_json::from_str::<Value>(raw) {
        return Some(value);
    }

    raw.lines().find_map(|line| {
        let data = line.trim().strip_prefix("data:")?.trim();
        if data.is_empty() || data == "[DONE]" {
            return None;
        }
        serde_json::from_str::<Value>(data).ok()
    })
}

fn extract_id(value: &Value) -> Option<String> {
    value.get("id")
        .and_then(Value::as_str)
        .or_else(|| value.pointer("/message/id").and_then(Value::as_str))
        .map(str::to_owned)
}

fn analyze_openai(value: &Value) -> ResponseFingerprint {
    let response_id = extract_id(value);
    let object = value.get("object").and_then(Value::as_str);
    let event_type = value.get("type").and_then(Value::as_str);
    let mut issues = Vec::new();
    let mut suspicious = false;

    if matches!(event_type, Some(t) if t.starts_with("message_") || t.starts_with("content_block_")) {
        issues.push("OpenAI 请求返回了 Anthropic 事件结构".into());
        suspicious = true;
    }

    if !matches!(object, Some("chat.completion" | "chat.completion.chunk" | "text_completion" | "response")) {
        issues.push("缺少标准 OpenAI object 类型".into());
    }

    match response_id.as_deref() {
        Some(id) if id.starts_with("chatcmpl-") || id.starts_with("cmpl-") || id.starts_with("resp_") => {}
        Some(_) => issues.push("响应 ID 不符合常见 OpenAI 前缀".into()),
        None => {
            issues.push("响应缺少 ID".into());
            suspicious = true;
        }
    }

    finish(response_id, issues, suspicious)
}

fn analyze_anthropic(value: &Value) -> ResponseFingerprint {
    let response_id = extract_id(value);
    let event_type = value.get("type").and_then(Value::as_str);
    let mut issues = Vec::new();
    let mut suspicious = false;

    if value.get("object").is_some() && value.get("choices").is_some() {
        issues.push("Anthropic 请求返回了 OpenAI choices/object 结构".into());
        suspicious = true;
    }

    if !matches!(event_type, Some("message" | "message_start")) {
        issues.push("首个事件不是标准 Anthropic message/message_start".into());
    }

    match response_id.as_deref() {
        Some(id) if id.starts_with("msg_") => {}
        Some(_) => issues.push("响应 ID 不符合常见 Anthropic msg_ 前缀".into()),
        None => {
            issues.push("响应缺少 message ID".into());
            suspicious = true;
        }
    }

    finish(response_id, issues, suspicious)
}

fn analyze_gemini(value: &Value) -> ResponseFingerprint {
    let mut issues = Vec::new();
    let suspicious = value.get("choices").is_some() || value.get("content_block").is_some();
    if value.get("candidates").is_none() {
        issues.push("缺少标准 Gemini candidates 结构".into());
    }
    if suspicious {
        issues.push("Gemini 请求返回了其他协议的响应结构".into());
    }
    finish(extract_id(value), issues, suspicious)
}

fn finish(response_id: Option<String>, issues: Vec<String>, suspicious: bool) -> ResponseFingerprint {
    let status = if suspicious {
        "suspicious"
    } else if issues.is_empty() {
        "consistent"
    } else {
        "nonstandard"
    };
    ResponseFingerprint { status: status.into(), response_id, issues }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_openai_chat_completion() {
        let report = analyze("openai", r#"{"id":"chatcmpl-abc","object":"chat.completion","choices":[]}"#);
        assert_eq!(report.status, "consistent");
        assert_eq!(report.response_id.as_deref(), Some("chatcmpl-abc"));
    }

    #[test]
    fn flags_protocol_mismatch() {
        let report = analyze("openai", r#"data: {"type":"message_start","message":{"id":"msg_abc"}}

"#);
        assert_eq!(report.status, "suspicious");
        assert!(report.issues.iter().any(|issue| issue.contains("Anthropic")));
    }

    #[test]
    fn treats_unknown_id_prefix_as_nonstandard() {
        let report = analyze("openai", r#"{"id":"bridge-123","object":"chat.completion","choices":[]}"#);
        assert_eq!(report.status, "nonstandard");
    }

    #[test]
    fn recognizes_anthropic_stream_start() {
        let report = analyze("anthropic", r#"event: message_start
data: {"type":"message_start","message":{"id":"msg_abc"}}

"#);
        assert_eq!(report.status, "consistent");
    }

    #[test]
    fn recognizes_documented_openai_headers() {
        let report = analyze_headers("openai", r#"{"x-request-id":"req_123","openai-version":"2020-10-01","x-ratelimit-limit-requests":"500"}"#);
        assert_eq!(report.status, "consistent");
        assert_eq!(report.request_id.as_deref(), Some("req_123"));
    }

    #[test]
    fn flags_cursor_bridge_header() {
        let report = analyze_headers("openai", r#"{"x-cursor-proxy-prompt-truncated":"true"}"#);
        assert_eq!(report.status, "suspicious");
        assert!(report.issues[0].contains("桥接"));
    }

    #[test]
    fn treats_missing_vendor_headers_as_nonstandard() {
        let report = analyze_headers("openai", r#"{"server":"nginx"}"#);
        assert_eq!(report.status, "nonstandard");
    }

    #[test]
    fn limits_origin_checks_to_openai_and_claude_model_families() {
        assert!(should_check_origin("claude-opus-4-6"));
        assert!(should_check_origin("gpt-5.5"));
        assert!(should_check_origin("o4-mini"));
        assert!(!should_check_origin("deepseek-v4-flash"));
        assert!(!should_check_origin("gemini-3-pro"));
    }
}
