use serde_json::Value;

/// A content part matching OpenAI's content part format.
/// vLLM's `_convert_block` produces text blocks and image blocks,
/// which become the message's content array.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ContentPart {
    Text(String),
    /// Data URI string, e.g. "data:image/jpeg;base64,/9j/4AAQ..."
    ImageUrl(String),
    /// Tool reference (sub-part of tool_result content in vLLM).
    /// Represented as `{"type": "tool_reference", "name": "..."}` in OpenAI format.
    ToolReference(String),
}

/// A tool call extracted from an assistant message.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: String,
}

/// Normalized message matching vLLM's `_convert_anthropic_to_openai_request` output.
///
/// vLLM produces OpenAI-format messages from Anthropic input:
/// - `role`: "user" | "assistant" | "system" | "tool"
/// - `content_parts`: text, image blocks (OpenAI content array)
/// - `reasoning`: extracted thinking blocks (assistant only)
/// - `tool_calls`: extracted tool_use blocks (assistant only)
/// - `tool_call_id`: set for "tool" role messages (from tool_result.tool_use_id)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NormalizedMessage {
    pub role: String,
    /// Content parts matching OpenAI format: text, image_url, tool_reference.
    /// Empty vec means no content (tool-only assistant messages, etc.).
    pub content_parts: Vec<ContentPart>,
    /// Extracted thinking/reasoning (assistant only).
    pub reasoning: Option<String>,
    /// Extracted tool_use blocks (assistant only).
    pub tool_calls: Option<Vec<ToolCall>>,
    /// tool_call_id from tool_result block (tool role only).
    pub tool_call_id: Option<String>,
}

impl NormalizedMessage {
    /// Render content parts to flat text, matching what vLLM's
    /// `apply_chat_template` would produce for the same parts.
    ///
    /// - Text parts: verbatim
    /// - Image URL parts: rendered as `<image>` (matches DeepSeek-V4's
    ///   vision template convention; vLLM's `render_image_token`)
    /// - Tool reference parts: rendered as `{"type":"tool_reference","name":"..."}`
    pub fn content_text(&self) -> String {
        let mut text = String::new();
        for part in &self.content_parts {
            match part {
                ContentPart::Text(t) => text.push_str(t),
                ContentPart::ImageUrl(_url) => {
                    // vLLM render_image_token: produces <image> placeholder
                    text.push_str("<image>");
                }
                ContentPart::ToolReference(name) => {
                    text.push_str(&format!(
                        "{{\"type\":\"tool_reference\",\"name\":\"{}\"}}", name
                    ));
                }
            }
        }
        text
    }
}

/// Tool definition extracted from the Anthropic request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: String,
}

/// Full conversion result: messages (for chat template) + tools (separate).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Conversation {
    pub messages: Vec<NormalizedMessage>,
    pub tools: Vec<ToolDef>,
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Convert full Anthropic-format request body to Conversation.
///
/// Matches vLLM's `_convert_anthropic_to_openai_request` → `ChatCompletionRequest`.
/// All system messages are collected, tool_use → tool_calls, tool_result → "tool" role.
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
/// Produces OpenAI-style messages matching vLLM's `_convert_anthropic_to_openai_request`.
pub fn from_anthropic(
    system: Option<&Value>,
    messages: &[Value],
    tools: Option<&Value>,
) -> Conversation {
    let mut conv_messages: Vec<NormalizedMessage> = Vec::new();

    // ======================================================================
    // 1. System message
    //    vLLM: `_convert_system_message` — merges top-level `system` field
    //    AND all messages with role "system" into one system message.
    // ======================================================================
    let mut system_text = String::new();

    // Top-level system field (string or array of text blocks)
    if let Some(sys) = system {
        system_text.push_str(&extract_system_text(sys));
    }

    // System messages embedded in the messages array (ALL of them, not just first)
    for msg in messages {
        if msg.get("role").and_then(|r| r.as_str()) == Some("system") {
            if let Some(content) = msg.get("content") {
                system_text.push_str(&content_to_text(Some(content)));
            }
        }
    }

    // Strip per-request cache-busting headers (same as vLLM)
    let mut clean = String::with_capacity(system_text.len());
    for line in system_text.lines() {
        if line.starts_with("x-anthropic-billing-header") {
            continue;
        }
        clean.push_str(line);
        clean.push('\n');
    }
    system_text = clean.trim_end().to_string();

    if !system_text.is_empty() {
        conv_messages.push(NormalizedMessage {
            role: "system".into(),
            content_parts: vec![ContentPart::Text(system_text)],
            reasoning: None,
            tool_calls: None,
            tool_call_id: None,
        });
    }

    // ======================================================================
    // 2. Messages (skip system role)
    //    vLLM: `_convert_messages` — for each non-system message, converts
    //    content blocks into OpenAI format.
    // ======================================================================
    for msg in messages {
        let Some(role) = msg.get("role").and_then(|r| r.as_str()) else {
            continue;
        };
        if role == "system" {
            continue;
        }

        let content = msg.get("content");
        match role {
            "user" => {
                // vLLM: user messages — convert content blocks
                // text → text part, image → image_url part, tool_result → separate messages
                let (content_parts, _reasoning, _tool_calls) = convert_content_parts(content);
                conv_messages.push(NormalizedMessage {
                    role: "user".into(),
                    content_parts,
                    reasoning: None,
                    tool_calls: None,
                    tool_call_id: None,
                });

                // vLLM: tool_result blocks create additional messages AFTER the user message
                // (see _convert_user_tool_result)
                let tool_msgs = extract_tool_result_messages(content);
                conv_messages.extend(tool_msgs);
            }
            "assistant" => {
                // vLLM: assistant messages — text → content, thinking → reasoning,
                // tool_use → tool_calls, redacted_thinking → skip
                let (content_parts, reasoning, tool_calls) = convert_content_parts(content);
                conv_messages.push(NormalizedMessage {
                    role: "assistant".into(),
                    content_parts,
                    reasoning,
                    tool_calls,
                    tool_call_id: None,
                });
            }
            _ => {
                // Unknown role — just extract text
                let text = content_to_text(content);
                if !text.is_empty() {
                    conv_messages.push(NormalizedMessage {
                        role: role.to_string(),
                        content_parts: vec![ContentPart::Text(text)],
                        reasoning: None,
                        tool_calls: None,
                        tool_call_id: None,
                    });
                }
            }
        }
    }

    // ======================================================================
    // 3. Tools (separate field, not in messages)
    // ======================================================================
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

    tracing::info!(
        "converted: {} msgs, {} tools",
        conv_messages.len(),
        conv_tools.len()
    );

    Conversation {
        messages: conv_messages,
        tools: conv_tools,
    }
}

// ---------------------------------------------------------------------------
// Content block conversion — matches vLLM's `_convert_block`
// ---------------------------------------------------------------------------

/// Convert Anthropic content blocks to OpenAI-style parts.
///
/// vLLM's `_convert_message_content` iterates blocks and categorizes into:
/// - content_parts: text, image blocks
/// - reasoning_parts: thinking blocks
/// - tool_calls: tool_use blocks
/// - tool_result: handled separately (not here)
fn convert_content_parts(
    content: Option<&Value>,
) -> (Vec<ContentPart>, Option<String>, Option<Vec<ToolCall>>) {
    let mut content_parts: Vec<ContentPart> = Vec::new();
    let mut reasoning_parts: Vec<String> = Vec::new();
    let mut tool_calls: Vec<ToolCall> = Vec::new();

    match content {
        // String content — simple text
        Some(c) if c.is_string() => {
            if let Some(s) = c.as_str() {
                if !s.is_empty() {
                    content_parts.push(ContentPart::Text(s.to_string()));
                }
            }
        }
        // Array content — blocks
        Some(c) if c.is_array() => {
            for block in c.as_array().unwrap() {
                convert_block(block, &mut content_parts, &mut reasoning_parts, &mut tool_calls);
            }
        }
        _ => {}
    }

    let reasoning = if reasoning_parts.is_empty() {
        None
    } else {
        Some(reasoning_parts.join("\n"))
    };

    let tool_calls = if tool_calls.is_empty() {
        None
    } else {
        Some(tool_calls)
    };

    (content_parts, reasoning, tool_calls)
}

/// Convert a single content block — matches vLLM's `_convert_block`.
fn convert_block(
    block: &Value,
    content_parts: &mut Vec<ContentPart>,
    reasoning_parts: &mut Vec<String>,
    tool_calls: &mut Vec<ToolCall>,
) {
    let Some(block_type) = block.get("type").and_then(|t| t.as_str()) else {
        return;
    };

    match block_type {
        "text" => {
            // vLLM: `if block.type == "text" and block.text:`
            //        content_parts.append({"type": "text", "text": block.text})
            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                if !text.is_empty() {
                    content_parts.push(ContentPart::Text(text.to_string()));
                }
            }
        }
        "image" => {
            // vLLM: `convert_image_source_to_url` → data URI
            //        content_parts.append({"type": "image_url", "image_url": {"url": data_uri}})
            if let Some(source) = block.get("source") {
                let url = convert_image_source_to_url(source);
                if !url.is_empty() {
                    content_parts.push(ContentPart::ImageUrl(url));
                }
            }
        }
        "thinking" => {
            // vLLM: reasoning_parts.append(block.thinking)
            if let Some(thinking) = block.get("thinking").and_then(|t| t.as_str()) {
                if !thinking.is_empty() {
                    reasoning_parts.push(thinking.to_string());
                }
            }
        }
        "redacted_thinking" => {
            // vLLM: pass — prevents validation error, content is opaque
        }
        "tool_use" => {
            // vLLM: `_convert_tool_use_block`
            //        tool_call = {"id": ..., "type": "function", "function": {"name": ..., "arguments": ...}}
            let name = block.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = block.get("input").map(|i| i.to_string()).unwrap_or_else(|| "{}".to_string());
            tool_calls.push(ToolCall {
                name: name.to_string(),
                arguments: args,
            });
        }
        "tool_result" => {
            // vLLM: handled by `_convert_user_tool_result` (called from `_convert_tool_result_block`)
            // which creates separate "tool" role messages. Not processed here inline — caller
            // uses extract_tool_result_messages for this.
        }
        "tool_reference" => {
            // vLLM: pass — expanded during tool_result processing
            // (tool_reference blocks appear inside tool_result content)
            if let Some(name) = block.get("tool_name")
                .or_else(|| block.get("name"))
                .and_then(|n| n.as_str())
            {
                content_parts.push(ContentPart::ToolReference(name.to_string()));
            }
        }
        _ => {
            // Unknown block type — vLLM ignores
        }
    }
}

// ---------------------------------------------------------------------------
// Image source conversion — matches vLLM's `_convert_image_source_to_url`
// ---------------------------------------------------------------------------

/// Convert an Anthropic image source to a data URI.
///
/// vLLM's `_convert_image_source_to_url`:
/// - `type: "url"` → returns the URL directly
/// - `type: "base64"` (or missing) → `data:{media_type};base64,{data}`
fn convert_image_source_to_url(source: &Value) -> String {
    let source_type = source.get("type").and_then(|t| t.as_str()).unwrap_or("base64");
    if source_type == "url" {
        return source.get("url").and_then(|u| u.as_str()).unwrap_or("").to_string();
    }
    let media_type = source.get("media_type").and_then(|m| m.as_str()).unwrap_or("image/jpeg");
    let data = source.get("data").and_then(|d| d.as_str()).unwrap_or("");
    format!("data:{};base64,{}", media_type, data)
}

// ---------------------------------------------------------------------------
// Tool result handling — matches vLLM's `_convert_user_tool_result`
// ---------------------------------------------------------------------------

/// Extract tool_result blocks as separate "tool" role messages.
///
/// vLLM's `_convert_user_tool_result` creates:
/// - "tool" role message with text content + tool_call_id
/// - "user" role message with image URLs (if any images in the result)
/// - "tool" role message with tool_references (if any)
fn extract_tool_result_messages(content: Option<&Value>) -> Vec<NormalizedMessage> {
    let Some(c) = content else { return vec![] };
    let Some(arr) = c.as_array() else { return vec![] };

    let mut results: Vec<NormalizedMessage> = Vec::new();

    for block in arr {
        if block.get("type").and_then(|t| t.as_str()) != Some("tool_result") {
            continue;
        }

        let tool_use_id = block.get("tool_use_id")
            .and_then(|id| id.as_str())
            .unwrap_or("")
            .to_string();

        let result_content = block.get("content");
        let (text_content, image_urls, tool_refs) = extract_tool_result_content_parts(result_content);

        // Tool role message with text content (vLLM: {"role": "tool", "tool_call_id": ..., "content": text})
        if !text_content.is_empty() || tool_refs.is_empty() {
            let mut parts = Vec::new();
            if !text_content.is_empty() {
                parts.push(ContentPart::Text(text_content));
            }
            results.push(NormalizedMessage {
                role: "tool".into(),
                content_parts: parts,
                reasoning: None,
                tool_calls: None,
                tool_call_id: Some(tool_use_id.clone()),
            });
        }

        // Image URLs as separate "user" role messages (vLLM pattern)
        for url in image_urls {
            results.push(NormalizedMessage {
                role: "user".into(),
                content_parts: vec![ContentPart::ImageUrl(url)],
                reasoning: None,
                tool_calls: None,
                tool_call_id: None,
            });
        }

        // Tool references as separate "tool" role messages
        if !tool_refs.is_empty() {
            results.push(NormalizedMessage {
                role: "tool".into(),
                content_parts: tool_refs
                    .into_iter()
                    .map(|name| ContentPart::ToolReference(name))
                    .collect(),
                reasoning: None,
                tool_calls: None,
                tool_call_id: Some(tool_use_id),
            });
        }
    }

    results
}

/// Extract content parts from a tool_result's content field.
///
/// vLLM's `_convert_user_tool_result` handles:
/// - String content → direct text
/// - Array content → iterates items looking for text, image, tool_reference
fn extract_tool_result_content_parts(
    content: Option<&Value>,
) -> (String, Vec<String>, Vec<String>) {
    let mut text_parts: Vec<String> = Vec::new();
    let mut image_urls: Vec<String> = Vec::new();
    let mut tool_refs: Vec<String> = Vec::new();

    match content {
        Some(c) if c.is_string() => {
            text_parts.push(c.as_str().unwrap_or("").to_string());
        }
        Some(c) if c.is_array() => {
            for item in c.as_array().unwrap() {
                let item_type = item.get("type").and_then(|t| t.as_str());
                match item_type {
                    Some("text") => {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            text_parts.push(text.to_string());
                        }
                    }
                    Some("image") => {
                        if let Some(source) = item.get("source") {
                            let url = convert_image_source_to_url(source);
                            if !url.is_empty() {
                                image_urls.push(url);
                            }
                        }
                    }
                    Some("tool_reference") => {
                        if let Some(name) = item.get("tool_name")
                            .or_else(|| item.get("name"))
                            .and_then(|n| n.as_str())
                        {
                            tool_refs.push(name.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    (text_parts.join("\n"), image_urls, tool_refs)
}

// ---------------------------------------------------------------------------
// Legacy/helper functions
// ---------------------------------------------------------------------------

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

/// Extract plain text from content (text blocks only, no images/thinking/tool_calls).
/// Used for backward compatibility and simple text extraction.
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
