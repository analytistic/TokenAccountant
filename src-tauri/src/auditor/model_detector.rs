use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiFormat {
    OpenAI,
    Anthropic,
    Gemini,
    Unknown,
}

impl ApiFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiFormat::OpenAI => "openai",
            ApiFormat::Anthropic => "anthropic",
            ApiFormat::Gemini => "gemini",
            ApiFormat::Unknown => "unknown",
        }
    }
}

pub struct DetectionResult {
    pub model: String,
    pub api_format: ApiFormat,
}

/// Detect model and API format from request path and body
pub fn detect(request_path: &str, body: &str) -> DetectionResult {
    // First try URL path
    if request_path.contains("/v1/messages") {
        if let Ok(body_val) = serde_json::from_str::<Value>(body) {
            if let Some(model) = body_val.get("model").and_then(|m| m.as_str()) {
                return DetectionResult {
                    model: model.to_string(),
                    api_format: ApiFormat::Anthropic,
                };
            }
        }
        return DetectionResult { model: "claude-unknown".into(), api_format: ApiFormat::Anthropic };
    }

    if request_path.contains("/v1beta") || request_path.contains("/gemini") {
        return DetectionResult { model: "gemini-unknown".into(), api_format: ApiFormat::Gemini };
    }

    // Default: try OpenAI format
    if let Ok(body_val) = serde_json::from_str::<Value>(body) {
        if let Some(model) = body_val.get("model").and_then(|m| m.as_str()) {
            let api_format = if model.starts_with("claude-") { ApiFormat::Anthropic }
                             else if model.starts_with("gemini-") { ApiFormat::Gemini }
                             else { ApiFormat::OpenAI };
            return DetectionResult { model: model.to_string(), api_format };
        }
    }

    DetectionResult { model: "unknown".into(), api_format: ApiFormat::Unknown }
}

/// Extract text from request body for tokenization
pub fn extract_request_text(body: &str, format: ApiFormat) -> String {
    let Ok(body_val) = serde_json::from_str::<Value>(body) else { return String::new() };
    let mut text = String::new();

    match format {
        ApiFormat::OpenAI | ApiFormat::Anthropic => {
            if let Some(messages) = body_val.get("messages").and_then(|m| m.as_array()) {
                for msg in messages {
                    if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                        text.push_str(content);
                        text.push('\n');
                    }
                }
            }
        }
        ApiFormat::Gemini => {
            if let Some(contents) = body_val.get("contents").and_then(|c| c.as_array()) {
                for content in contents {
                    if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                        for part in parts {
                            if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
                                text.push_str(t);
                                text.push('\n');
                            }
                        }
                    }
                }
            }
        }
        ApiFormat::Unknown => {}
    }
    text
}

/// Extract usage from response body
pub fn extract_usage(response_body: &str, format: ApiFormat) -> (i32, i32, i32) {
    let Ok(body_val) = serde_json::from_str::<Value>(response_body) else {
        return (0, 0, 0);
    };

    match format {
        ApiFormat::Anthropic => {
            let usage = body_val.get("usage");
            let input = usage.and_then(|u| u.get("input_tokens")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let output = usage.and_then(|u| u.get("output_tokens")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let cached = usage.and_then(|u| u.get("cache_read_input_tokens")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            (input, output, cached)
        }
        ApiFormat::OpenAI => {
            let usage = body_val.get("usage");
            let input = usage.and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let output = usage.and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let cached = usage.and_then(|u| u.get("prompt_tokens_details"))
                .and_then(|d| d.get("cached_tokens")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            (input, output, cached)
        }
        ApiFormat::Gemini => {
            let usage = body_val.get("usageMetadata");
            let input = usage.and_then(|u| u.get("promptTokenCount")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let output = usage.and_then(|u| u.get("candidatesTokenCount")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let cached = usage.and_then(|u| u.get("cachedContentTokenCount")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            (input, output, cached)
        }
        ApiFormat::Unknown => (0, 0, 0),
    }
}
