use serde_json::Value;

/// A tool call extracted from an assistant message.
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arguments: String,
}

/// Normalized message with structured fields matching vLLM conventions.
///
/// - `content`: only text content (no thinking, no tool_calls)
/// - `reasoning`: extracted thinking/reasoning blocks (assistant only)
/// - `tool_calls`: extracted tool_use blocks (assistant only)
#[derive(Debug, Clone)]
pub struct NormalizedMessage {
    pub role: String,
    pub content: String,
    pub reasoning: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Tool definition extracted from the Anthropic request.
#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: String,
}

/// Full conversion result: messages (for chat template) + tools (separate).
#[derive(Debug, Clone)]
pub struct Conversation {
    pub messages: Vec<NormalizedMessage>,
    pub tools: Vec<ToolDef>,
}

/// Convert full Anthropic-format request body to Conversation.
pub fn from_anthropic_body(body_str: &str) -> Conversation {
    let Ok(body_val) = serde_json::from_str::<Value>(body_str) else {
        return Conversation { messages: vec![], tools: vec![] };
    };

    let system = body_val.get("system");
    let messages = body_val
        .get("messages")
        .and_then(|m| m.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    let tools = body_val.get("tools");

    from_anthropic(system, messages, tools)
}

/// Low-level conversion from parsed JSON values.
/// Produces flat OpenAI-style messages with separate tools.
pub fn from_anthropic(
    system: Option<&Value>,
    messages: &[Value],
    tools: Option<&Value>,
) -> Conversation {
    let mut conv_messages: Vec<NormalizedMessage> = Vec::new();

    // 1. Merge outer system + first system-role message in messages
    let mut system_text = String::new();
    if let Some(sys) = system {
        system_text.push_str(&extract_system_text(sys));
    }

    let mut first_system_idx = None;
    for (i, msg) in messages.iter().enumerate() {
        if msg.get("role").and_then(|r| r.as_str()) == Some("system") {
            first_system_idx = Some(i);
            break;
        }
    }

    if let Some(idx) = first_system_idx {
        if let Some(content) = messages[idx].get("content") {
            system_text.push_str(&content_to_text(Some(content)));
        }
    }

    // Strip per-request cache-busting headers from system text.
    // These contain unique hashes that defeat prefix caching.
    let mut clean = String::with_capacity(system_text.len());
    for line in system_text.lines() {
        if line.starts_with("x-anthropic-billing-header") {
            continue;
        }
        clean.push_str(line);
        clean.push('\n');
    }
    system_text = clean;

    if !system_text.is_empty() {
        conv_messages.push(NormalizedMessage {
            role: "system".into(),
            content: system_text,
            reasoning: None,
            tool_calls: None,
        });
    }

    // 2. Messages (skip first system, split tool_result from user)
    for (i, msg) in messages.iter().enumerate() {
        let Some(role) = msg.get("role").and_then(|r| r.as_str()) else {
            continue;
        };
        if Some(i) == first_system_idx {
            continue;
        }

        let content = msg.get("content");
        match role {
            "user" => {
                let user_text = content_to_text(content);
                let tool_texts = extract_tool_result_texts(content);
                if !user_text.is_empty() {
                    conv_messages.push(NormalizedMessage {
                        role: "user".into(),
                        content: user_text,
                        reasoning: None,
                        tool_calls: None,
                    });
                }
                for t in tool_texts {
                    conv_messages.push(NormalizedMessage {
                        role: "tool".into(),
                        content: t,
                        reasoning: None,
                        tool_calls: None,
                    });
                }
            }
            "assistant" => {
                let text = content_to_text(content);
                let reasoning = extract_thinking(content);
                let tool_calls = extract_tool_calls(content);
                conv_messages.push(NormalizedMessage {
                    role: "assistant".into(),
                    content: text,
                    reasoning,
                    tool_calls,
                });
            }
            _ => {
                let text = content_to_text(content);
                if !text.is_empty() {
                    conv_messages.push(NormalizedMessage {
                        role: role.to_string(),
                        content: text,
                        reasoning: None,
                        tool_calls: None,
                    });
                }
            }
        }
    }

    // 3. Tools (separate field, not in messages)
    let conv_tools = tools
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|tool| {
                    let name = tool.get("name").and_then(|n| n.as_str())?;
                    let desc = tool.get("description").and_then(|d| d.as_str()).unwrap_or("");
                    let schema = tool.get("input_schema").map(|s| s.to_string()).unwrap_or_default();
                    Some(ToolDef {
                        name: name.to_string(),
                        description: desc.to_string(),
                        input_schema: schema,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    tracing::info!("converted: {} msgs, {} tools", conv_messages.len(), conv_tools.len());

    Conversation {
        messages: conv_messages,
        tools: conv_tools,
    }
}

/// Extract plain text from system parameter, stripping per-request cache-busting headers.
fn extract_system_text(system: &Value) -> String {
    if let Some(s) = system.as_str() {
        if s.starts_with("x-anthropic-billing-header") {
            return String::new();
        }
        return s.to_string();
    }
    if let Some(arr) = system.as_array() {
        let mut text = String::new();
        for block in arr {
            if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                // Strip per-request hash that defeats prefix caching
                if t.starts_with("x-anthropic-billing-header") {
                    continue;
                }
                text.push_str(t);
            }
        }
        return text;
    }
    String::new()
}

/// Extract plain text blocks from content.
/// Only extracts type "text" — thinking and tool_use are handled separately.
fn content_to_text(content: Option<&Value>) -> String {
    match content {
        Some(c) if c.is_string() => c.as_str().unwrap_or("").to_string(),
        Some(c) if c.is_array() => {
            let mut text = String::new();
            for block in c.as_array().unwrap() {
                if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                    if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                        text.push_str(t);
                    }
                }
            }
            text
        }
        _ => String::new(),
    }
}

/// Extract thinking/reasoning content from assistant message blocks.
fn extract_thinking(content: Option<&Value>) -> Option<String> {
    let arr = content?.as_array()?;
    let mut parts = Vec::new();
    for block in arr {
        let typ = block.get("type").and_then(|t| t.as_str())?;
        if typ == "thinking" || typ == "reasoning" {
            if let Some(t) = block.get("thinking").or_else(|| block.get("reasoning")).and_then(|t| t.as_str()) {
                if !t.is_empty() {
                    parts.push(t.to_string());
                }
            }
        }
    }
    if parts.is_empty() { None } else { Some(parts.join("\n")) }
}

/// Extract tool_use blocks from assistant message content.
fn extract_tool_calls(content: Option<&Value>) -> Option<Vec<ToolCall>> {
    let arr = content?.as_array()?;
    let calls: Vec<ToolCall> = arr
        .iter()
        .filter(|block| block.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
        .filter_map(|block| {
            let name = block.get("name").and_then(|n| n.as_str())?;
            let args = block.get("input").map(|i| i.to_string()).unwrap_or_default();
            Some(ToolCall { name: name.to_string(), arguments: args })
        })
        .collect();
    if calls.is_empty() { None } else { Some(calls) }
}

/// Extract tool_result texts from content (for user messages).
fn extract_tool_result_texts(content: Option<&Value>) -> Vec<String> {
    let Some(c) = content else { return vec![] };
    let Some(arr) = c.as_array() else { return vec![] };

    let mut results = Vec::new();
    for block in arr {
        if block.get("type").and_then(|t| t.as_str()) != Some("tool_result") {
            continue;
        }
        let result_content = block.get("content");
        let text = content_to_text(result_content);
        if !text.is_empty() {
            results.push(text);
        }
    }
    results
}
