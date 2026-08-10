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

fn response_json_values(body: &str) -> Vec<Value> {
    if let Ok(value) = serde_json::from_str(body.trim()) {
        return vec![value];
    }

    body.lines().filter_map(|line| {
        let data = line.strip_prefix("data:")?.trim_start();
        if data == "[DONE]" || data.is_empty() { return None; }
        serde_json::from_str(data).ok()
    }).collect()
}

/// Extract usage from response body (SSE stream or plain JSON).
pub fn extract_usage(response_body: &str, format: ApiFormat) -> (i32, i32, i32) {
    let values = response_json_values(response_body);

    match format {
        ApiFormat::Anthropic => {
            let mut input = 0;
            let mut output = 0;
            let mut cached = 0;
            for value in &values {
                let usage = value.get("usage").or_else(|| value.get("message").and_then(|m| m.get("usage")));
                input = input.max(token_field(usage, "input_tokens"));
                output = output.max(token_field(usage, "output_tokens"));
                cached = cached.max(token_field(usage, "cache_read_input_tokens"));
            }
            (input, output, cached)
        }
        ApiFormat::OpenAI => {
            let mut prompt_total = 0;
            let mut direct_miss: Option<i32> = None;
            let mut direct_hit: Option<i32> = None;
            let mut detail_cached = 0;
            let mut output = 0;
            for value in &values {
                let usage = value.get("usage");
                prompt_total = prompt_total.max(token_field(usage, "prompt_tokens"));
                output = output.max(token_field(usage, "completion_tokens"));
                if let Some(value) = optional_token_field(usage, "prompt_cache_miss_tokens") {
                    direct_miss = Some(direct_miss.unwrap_or(0).max(value));
                }
                if let Some(value) = optional_token_field(usage, "prompt_cache_hit_tokens") {
                    direct_hit = Some(direct_hit.unwrap_or(0).max(value));
                }
                detail_cached = detail_cached.max(usage.and_then(|u| u.get("prompt_tokens_details"))
                    .and_then(|d| d.get("cached_tokens")).and_then(Value::as_i64).unwrap_or(0) as i32);
            }
            let cached = direct_hit.unwrap_or(detail_cached);
            let input = direct_miss.unwrap_or_else(|| prompt_total.saturating_sub(cached));
            (input, output, cached)
        }
        ApiFormat::Gemini => {
            let usage = values.iter().filter_map(|v| v.get("usageMetadata")).last();
            let input = token_field(usage, "promptTokenCount");
            let output = token_field(usage, "candidatesTokenCount");
            let cached = token_field(usage, "cachedContentTokenCount");
            (input, output, cached)
        }
        ApiFormat::Unknown => (0, 0, 0),
    }
}

fn token_field(usage: Option<&Value>, field: &str) -> i32 {
    optional_token_field(usage, field).unwrap_or(0)
}

fn optional_token_field(usage: Option<&Value>, field: &str) -> Option<i32> {
    usage?.get(field)?.as_i64().map(|value| value as i32)
}

#[cfg(test)]
mod tests {
    use super::{extract_usage, ApiFormat};

    #[test]
    fn normalizes_real_deepseek_openai_usage() {
        let body = r#"{"usage":{"prompt_tokens":85,"completion_tokens":8,"total_tokens":93,"prompt_tokens_details":{"cached_tokens":0},"prompt_cache_hit_tokens":0,"prompt_cache_miss_tokens":85}}"#;
        assert_eq!(extract_usage(body, ApiFormat::OpenAI), (85, 8, 0));
    }

    #[test]
    fn separates_deepseek_openai_cache_hits_from_input() {
        let body = r#"{"usage":{"prompt_tokens":1000,"completion_tokens":50,"prompt_cache_hit_tokens":800,"prompt_cache_miss_tokens":200}}"#;
        assert_eq!(extract_usage(body, ApiFormat::OpenAI), (200, 50, 800));
    }

    #[test]
    fn normalizes_real_deepseek_anthropic_usage() {
        let body = r#"{"usage":{"input_tokens":85,"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"output_tokens":8}}"#;
        assert_eq!(extract_usage(body, ApiFormat::Anthropic), (85, 8, 0));
    }

    #[test]
    fn merges_anthropic_usage_across_sse_events() {
        let body = concat!(
            "event: message_start\n",
            "data: {\"type\":\"message_start\",\"message\":{\"usage\":{\"input_tokens\":89,\"cache_read_input_tokens\":32,\"output_tokens\":0}}}\n\n",
            "event: message_delta\n",
            "data: {\"type\":\"message_delta\",\"usage\":{\"output_tokens\":16}}\n\n",
        );
        assert_eq!(extract_usage(body, ApiFormat::Anthropic), (89, 16, 32));
    }
}
