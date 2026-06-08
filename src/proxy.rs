// src/proxy.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::{Result, anyhow};

/// API 格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiFormat {
    OpenAI,
    Anthropic,
    Gemini,
    Unknown,
}

/// 请求记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestRecord {
    pub id: Option<i64>,  // SQLite 自增 ID
    pub timestamp: String,        // ISO 8601 格式
    pub model: String,          // 模型名称
    pub api_format: String,     // API 格式
    pub endpoint: String,       // API 端点
    pub request_text: String,   // 请求内容
    pub response_text: String,  // 响应内容
    pub claimed_input: i32,     // 中转站声称的 input tokens
    pub claimed_output: i32,    // 中转站声称的 output tokens
    pub claimed_cached: i32,    // 中转站声称的缓存命中 tokens
    pub claimed_creation: i32,  // 中转站声称的缓存创建 tokens
    pub real_input: i32,        // 本地计算的 input tokens
    pub real_output: i32,       // 本地计算的 output tokens
    pub cached_tokens: i32,     // 本地检测的缓存命中 tokens
    pub input_diff: i32,        // 差异 = claimed - real
    pub output_diff: i32,
    pub cache_diff: i32,
    pub is_suspicious: bool,    // 是否可疑（差异 > 5%）
    pub suspicion_reason: String, // 可疑原因
    pub raw_response: String,    // 原始响应（JSON）
}

/// 模型识别器
pub fn detect_model_and_format(request_body: &str) -> (String, ApiFormat) {
    // 解析请求体
    let body: Value = match serde_json::from_str(request_body) {
        Ok(v) => v,
        Err(_) => return ("unknown".to_string(), ApiFormat::Unknown),
    };
    
    // 获取 model 字段
    let model = body.get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown");
    
    // 根据 model 判断
    if model.starts_with("gpt-") || model.starts_with("text-") {
        ("gpt".to_string(), ApiFormat::OpenAI)
    } else if model.starts_with("claude-") {
        ("claude".to_string(), ApiFormat::Anthropic)
    } else if model.starts_with("gemini-") {
        ("gemini".to_string(), ApiFormat::Gemini)
    } else if model.starts_with("qwen-") {
        ("qwen".to_string(), ApiFormat::OpenAI)
    } else if model.starts_with("deepseek-") {
        ("deepseek".to_string(), ApiFormat::OpenAI)
    } else if model.starts_with("glm-") {
        ("glm".to_string(), ApiFormat::OpenAI)
    } else if model.contains("abab") {  // MiniMax
        ("minimax".to_string(), ApiFormat::OpenAI)
    } else if model.starts_with("mimo") {
        ("mimo".to_string(), ApiFormat::OpenAI)
    } else {
        ("unknown".to_string(), ApiFormat::Unknown)
    }
}

/// 从请求体中提取文本
pub fn extract_request_text(request_body: &str) -> Result<String> {
    let body: Value = serde_json::from_str(request_body)
        .map_err(|e| anyhow!("Failed to parse request body: {}", e))?;
    
    let mut text = String::new();
    
    // OpenAI 格式：messages
    if let Some(messages) = body.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                text.push_str(content);
                text.push('\n');
            }
        }
    }
    // Anthropic 格式：messages
    else if let Some(messages) = body.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                text.push_str(content);
                text.push('\n');
            }
        }
    }
    // Gemini 格式：contents
    else if let Some(contents) = body.get("contents").and_then(|c| c.as_array()) {
        for content in contents {
            if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                for part in parts {
                    if let Some(text_part) = part.get("text").and_then(|t| t.as_str()) {
                        text.push_str(text_part);
                        text.push('\n');
                    }
                }
            }
        }
    }
    
    Ok(text)
}

/// 从响应体中提取文本
pub fn extract_response_text(response_body: &str) -> Result<String> {
    let body: Value = serde_json::from_str(response_body)
        .map_err(|e| anyhow!("Failed to parse response body: {}", e))?;
    
    let mut text = String::new();
    
    // OpenAI 格式：choices[0].message.content
    if let Some(choices) = body.get("choices").and_then(|c| c.as_array()) {
        if let Some(first_choice) = choices.get(0) {
            if let Some(content) = first_choice.get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str()) {
                text.push_str(content);
            }
        }
    }
    // Anthropic 格式：content[0].text
    else if let Some(content_array) = body.get("content").and_then(|c| c.as_array()) {
        for content in content_array {
            if let Some(text_part) = content.get("text").and_then(|t| t.as_str()) {
                text.push_str(text_part);
            }
        }
    }
    // Gemini 格式：candidates[0].content.parts[0].text
    else if let Some(candidates) = body.get("candidates").and_then(|c| c.as_array()) {
        if let Some(first_candidate) = candidates.get(0) {
            if let Some(parts) = first_candidate.get("content")
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.as_array()) {
                for part in parts {
                    if let Some(text_part) = part.get("text").and_then(|t| t.as_str()) {
                        text.push_str(text_part);
                    }
                }
            }
        }
    }
    
    Ok(text)
}

/// 从响应体中提取 usage
pub fn extract_usage(response_body: &str, api_format: ApiFormat) -> Result<Usage> {
    let body: Value = serde_json::from_str(response_body)
        .map_err(|e| anyhow!("Failed to parse response body: {}", e))?;
    
    let usage = match api_format {
        ApiFormat::OpenAI => {
            let usage_obj = body.get("usage").ok_or_else(|| anyhow!("No usage field"))?;
            
            Usage {
                input_tokens: usage_obj.get("prompt_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                output_tokens: usage_obj.get("completion_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                cached_tokens: usage_obj.get("prompt_tokens_details")
                    .and_then(|d| d.get("cached_tokens"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                cache_creation_tokens: 0,  // OpenAI 没有这个字段
            }
        }
        ApiFormat::Anthropic => {
            let usage_obj = body.get("usage").ok_or_else(|| anyhow!("No usage field"))?;
            
            Usage {
                input_tokens: usage_obj.get("input_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                output_tokens: usage_obj.get("output_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                cached_tokens: usage_obj.get("cache_read_input_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                cache_creation_tokens: usage_obj.get("cache_creation_input_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
            }
        }
        ApiFormat::Gemini => {
            let usage_obj = body.get("usageMetadata").ok_or_else(|| anyhow!("No usageMetadata field"))?;
            
            Usage {
                input_tokens: usage_obj.get("promptTokenCount")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                output_tokens: usage_obj.get("candidatesTokenCount")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                cached_tokens: usage_obj.get("cachedContentTokenCount")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                cache_creation_tokens: 0,  // Gemini 没有这个字段
            }
        }
        ApiFormat::Unknown => {
            return Err(anyhow!("Unknown API format"));
        }
    };
    
    Ok(usage)
}

/// Usage 信息
#[derive(Debug, Clone, Default)]
pub struct Usage {
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub cached_tokens: i32,
    pub cache_creation_tokens: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_model_openai() {
        let request_body = r#"{"model": "gpt-4", "messages": []}"#;
        let (model, format) = detect_model_and_format(request_body);
        assert_eq!(model, "gpt");
        assert_eq!(format, ApiFormat::OpenAI);
    }
    
    #[test]
    fn test_detect_model_anthropic() {
        let request_body = r#"{"model": "claude-3-5-sonnet", "messages": []}"#;
        let (model, format) = detect_model_and_format(request_body);
        assert_eq!(model, "claude");
        assert_eq!(format, ApiFormat::Anthropic);
    }
    
    #[test]
    fn test_extract_usage_openai() {
        let response_body = r#"{
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "prompt_tokens_details": {
                    "cached_tokens": 80
                }
            }
        }"#;
        
        let usage = extract_usage(response_body, ApiFormat::OpenAI).unwrap();
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.cached_tokens, 80);
    }
    
    #[test]
    fn test_extract_usage_anthropic() {
        let response_body = r#"{
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50,
                "cache_read_input_tokens": 80,
                "cache_creation_input_tokens": 20
            }
        }"#;
        
        let usage = extract_usage(response_body, ApiFormat::Anthropic).unwrap();
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.cached_tokens, 80);
        assert_eq!(usage.cache_creation_tokens, 20);
    }
}
