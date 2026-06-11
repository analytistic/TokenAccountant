// SPDX-License-Identifier: Apache-2.0
//
// Adapted from vLLM's deepseek_v4_encoding.py
// Original: https://github.com/vllm-project/vllm

use crate::auditor::tokenizer::Tokenizer;
use crate::auditor::message_converter::{Conversation, NormalizedMessage};
use crate::auditor::tokenizer::TemplateParams;
use super::gpt::GptTokenizer;
use serde_json::Value;

// ============================================================
// Special Tokens
// ============================================================

const BOS_TOKEN: &str = "<｜begin▁of▁sentence｜>";
const EOS_TOKEN: &str = "<｜end▁of▁sentence｜>";
const THINKING_START_TOKEN: &str = "<think>";
const THINKING_END_TOKEN: &str = "</think>";
const DSML_TOKEN: &str = "｜DSML｜";

const USER_SP_TOKEN: &str = "<｜User｜>";
const ASSISTANT_SP_TOKEN: &str = "<｜Assistant｜>";

// ============================================================
// Templates
// ============================================================

const SYSTEM_MSG_TEMPLATE: &str = "{content}";
const USER_MSG_TEMPLATE: &str = "{content}";
const ASSISTANT_MSG_TEMPLATE: &str = "{reasoning}{content}{tool_calls}";
const THINKING_TEMPLATE: &str = "{reasoning}";
const TOOL_OUTPUT_TEMPLATE: &str = "<tool_result>{content}</tool_result>";
const RESPONSE_FORMAT_TEMPLATE: &str =
    "## Response Format:\n\nYou MUST strictly adhere to the following schema to reply:\n{schema}";

const REASONING_EFFORT_MAX: &str = "\
Reasoning Effort: Absolute maximum with no shortcuts permitted.\n\
You MUST be very thorough in your thinking and comprehensively decompose the problem to resolve the root cause, rigorously stress-testing your logic against all potential paths, edge cases, and adversarial scenarios.\n\
Explicitly write out your entire deliberation process, documenting every intermediate step, considered alternative, and rejected hypothesis to ensure absolutely no assumption is left unchecked.\n\n";

const TOOLS_TEMPLATE_PREFIX: &str = "\
## Tools

You have access to a set of tools to help answer the user's question. You can invoke tools by writing a \"<｜DSML｜tool_calls>\" block like the following:

<｜DSML｜tool_calls>
<｜DSML｜invoke name=\"$TOOL_NAME\">
<｜DSML｜parameter name=\"$PARAMETER_NAME\" string=\"true|false\">$PARAMETER_VALUE</｜DSML｜parameter>
...
</｜DSML｜invoke>
<｜DSML｜invoke name=\"$TOOL_NAME2\">
...
</｜DSML｜invoke>
</｜DSML｜tool_calls>

String parameters should be specified as is and set `string=\"true\"`. For all other types (numbers, booleans, arrays, objects), pass the value in JSON format and set `string=\"false\"`.

If thinking_mode is enabled (triggered by <think>), you MUST output your complete reasoning inside <think>...</think> BEFORE any tool calls or final response.

Otherwise, output directly after </think> with tool calls or final response.

### Available Tool Schemas

";

// ============================================================
// Internal message model — mirrors official Python dict format
// ============================================================

#[derive(Clone)]
enum V4ContentBlock {
    Text(String),
    ToolResult { #[allow(dead_code)] tool_use_id: String, content: String },
}

#[derive(Clone)]
struct V4Message {
    role: String,
    content: Option<String>,
    content_blocks: Option<Vec<V4ContentBlock>>,
    tool_calls: Option<Vec<V4ToolCall>>,
    reasoning: Option<String>,
    tools: Option<Vec<String>>,          // JSON-serialized tool definitions
    response_format: Option<String>,     // JSON schema
    wo_eos: bool,
}

#[derive(Clone)]
struct V4ToolCall {
    name: String,
    arguments: String,  // JSON string
}

// ============================================================
// DeepSeek Tokenizer
// ============================================================

pub struct DeepSeekTokenizer {
    gpt: GptTokenizer,
}

impl DeepSeekTokenizer {
    pub fn new() -> Self {
        DeepSeekTokenizer {
            gpt: GptTokenizer::new(),
        }
    }
}

impl Tokenizer for DeepSeekTokenizer {
    fn encode(&self, text: &str) -> Vec<u32> {
        self.gpt.encode(text)
    }

    fn decode(&self, ids: &[u32]) -> String {
        self.gpt.decode(ids)
    }

    fn count_tokens(&self, text: &str) -> u32 {
        self.gpt.count_tokens(text)
    }

    fn render_output(&self, msg: &NormalizedMessage) -> String {
        // Standalone assistant output matching official `parse_message_from_completion_text` output
        let mut out = String::new();

        if let Some(ref r) = msg.reasoning {
            out.push_str(THINKING_START_TOKEN);
            out.push_str(r);
            out.push_str(THINKING_END_TOKEN);
        }

        out.push_str(&msg.content_text());

        if let Some(ref tcs) = msg.tool_calls {
            let tc_dsml: Vec<String> = tcs.iter().map(|tc| {
                let v4_tc = V4ToolCall {
                    name: tc.name.clone(),
                    arguments: tc.arguments.clone(),
                };
                format!(
                    "<{dsml}invoke name=\"{name}\">\n{args}\n</{dsml}invoke>",
                    dsml = DSML_TOKEN,
                    name = v4_tc.name,
                    args = encode_arguments_to_dsml(&v4_tc),
                )
            }).collect();
            out.push_str(&format!(
                "\n\n<{dsml}{block}>\n{tcs}\n</{dsml}{block}>",
                dsml = DSML_TOKEN,
                block = "tool_calls",
                tcs = tc_dsml.join("\n"),
            ));
        }

        out.push_str(EOS_TOKEN);
        out
    }

    fn apply_chat_template(&self, conv: &Conversation) -> String {
        self.apply_chat_template_with(conv, &TemplateParams::default())
    }

    fn apply_chat_template_with(&self, conv: &Conversation, params: &TemplateParams) -> String {
        // Convert NormalizedMessage → V4Message
        let mut v4_msgs: Vec<V4Message> = conv.messages.iter().map(normalized_to_v4).collect();

        // Merge tool messages
        v4_msgs = merge_tool_messages(v4_msgs);

        // Sort tool results by preceding tool_call order
        v4_msgs = sort_tool_results_by_call_order(v4_msgs);

        // Convert tool definitions to JSON strings
        let tool_schemas: Option<Vec<String>> = if !conv.tools.is_empty() {
            Some(conv.tools.iter().map(|t| {
                let mut map = serde_json::Map::new();
                map.insert("name".into(), Value::String(t.name.clone()));
                map.insert("description".into(), Value::String(t.description.clone()));
                // Parse input_schema as JSON if valid, otherwise use as raw string
                let schema_val = serde_json::from_str::<Value>(&t.input_schema)
                    .unwrap_or(Value::String(t.input_schema.clone()));
                map.insert("parameters".into(), schema_val);
                Value::Object(map).to_string()
            }).collect())
        } else {
            None
        };

        // Attach tools to first system/developer message
        if let Some(ref schemas) = tool_schemas {
            if let Some(first) = v4_msgs.first_mut() {
                if first.role == "system" || first.role == "developer" {
                    first.tools = Some(schemas.clone());
                }
            }
        }

        // Resolve params with DeepSeek defaults
        let thinking_mode = params.thinking_mode.as_deref().unwrap_or("thinking");
        // Default true: matches vLLM's encode_messages default.
        // vLLM drops reasoning from assistant messages before the last user,
        // so we must do the same for store↔detect consistency.
        let drop_thinking = params.drop_thinking.unwrap_or(true);
        let add_default_bos_token = params.add_default_bos_token.unwrap_or(true);
        let reasoning_effort = params.reasoning_effort.as_deref();

        // Encode
        encode_messages(
            &v4_msgs,
            thinking_mode,
            &[],       // no context
            drop_thinking,
            add_default_bos_token,
            reasoning_effort,
        )
    }
}

// ============================================================
// Conversion: NormalizedMessage → V4Message
// ============================================================

fn normalized_to_v4(msg: &NormalizedMessage) -> V4Message {
    V4Message {
        role: msg.role.clone(),
        content: {
            let text = msg.content_text();
            if text.is_empty() { None } else { Some(text) }
        },
        content_blocks: None,  // populated by merge_tool_messages
        tool_calls: msg.tool_calls.as_ref().map(|tcs| {
            tcs.iter().map(|tc| V4ToolCall {
                name: tc.name.clone(),
                arguments: tc.arguments.clone(),
            }).collect()
        }),
        reasoning: msg.reasoning.clone(),
        tools: None,
        response_format: None,
        wo_eos: false,
    }
}

// ============================================================
// Tool call DSML encoding — matches official encode_arguments_to_dsml
// ============================================================

fn encode_arguments_to_dsml(tool_call: &V4ToolCall) -> String {
    let arguments: serde_json::Value = serde_json::from_str(&tool_call.arguments)
        .unwrap_or(Value::Object(serde_json::Map::new()));

    let obj = match arguments {
        Value::Object(ref m) => m,
        _ => return String::new(),
    };

    let mut parts = Vec::new();
    for (key, val) in obj {
        let is_str = val.is_string();
        let val_str = if is_str {
            val.as_str().unwrap_or("").to_string()
        } else {
            serde_json::to_string(val).unwrap_or_default()
        };
        parts.push(format!(
            "<{dsml}parameter name=\"{key}\" string=\"{is_str}\">{val}</{dsml}parameter>",
            dsml = DSML_TOKEN,
            key = key,
            is_str = if is_str { "true" } else { "false" },
            val = val_str,
        ));
    }
    parts.join("\n")
}

fn encode_arguments_to_dsml_from_msg(tool_call: &V4ToolCall) -> String {
    encode_arguments_to_dsml(tool_call)
}

// ============================================================
// Tool rendering — matches official render_tools
// ============================================================

fn render_tools(tools: &[String]) -> String {
    let mut text = TOOLS_TEMPLATE_PREFIX.to_string();
    for t in tools {
        text.push_str(t);
        text.push('\n');
    }
    text.push_str(
        "You MUST strictly follow the above defined tool name and parameter schemas to invoke tool calls.\n"
    );
    text
}

// ============================================================
// merge_tool_messages — matches official merge_tool_messages
// ============================================================

fn merge_tool_messages(messages: Vec<V4Message>) -> Vec<V4Message> {
    let mut merged: Vec<V4Message> = Vec::new();

    for msg in messages {
        let role = msg.role.clone();

        if role == "tool" {
            let tool_content = msg.content.clone().unwrap_or_default();
            let tool_block = V4ContentBlock::ToolResult {
                tool_use_id: String::new(), // tool_call_id not preserved in our V4Message; OK for template
                content: tool_content,
            };
            if let Some(last) = merged.last_mut() {
                if last.role == "user" && last.content_blocks.is_some() {
                    last.content_blocks.as_mut().unwrap().push(tool_block);
                } else {
                    merged.push(V4Message {
                        role: "user".into(),
                        content: None,
                        content_blocks: Some(vec![tool_block]),
                        tool_calls: None,
                        reasoning: None,
                        tools: None,
                        response_format: None,
                        wo_eos: false,
                    });
                }
            } else {
                merged.push(V4Message {
                    role: "user".into(),
                    content: None,
                    content_blocks: Some(vec![tool_block]),
                    tool_calls: None,
                    reasoning: None,
                    tools: None,
                    response_format: None,
                    wo_eos: false,
                });
            }
        } else if role == "user" {
            let text_block = V4ContentBlock::Text(msg.content.clone().unwrap_or_default());
            if let Some(last) = merged.last_mut() {
                if last.role == "user" && last.content_blocks.is_some() && last.tools.is_none() {
                    last.content_blocks.as_mut().unwrap().push(text_block);
                } else {
                    let mut new_msg = V4Message {
                        role: "user".into(),
                        content: msg.content.clone(),
                        content_blocks: Some(vec![text_block]),
                        tool_calls: None,
                        reasoning: None,
                        tools: None,
                        response_format: None,
                        wo_eos: false,
                    };
                    // Preserve extra fields
                    if msg.wo_eos {
                        new_msg.wo_eos = true;
                    }
                    merged.push(new_msg);
                }
            } else {
                merged.push(V4Message {
                    role: "user".into(),
                    content: msg.content.clone(),
                    content_blocks: Some(vec![text_block]),
                    tool_calls: None,
                    reasoning: None,
                    tools: None,
                    response_format: None,
                    wo_eos: false,
                });
            }
        } else {
            merged.push(msg);
        }
    }

    merged
}

// ============================================================
// sort_tool_results_by_call_order — matches official
// ============================================================

fn sort_tool_results_by_call_order(messages: Vec<V4Message>) -> Vec<V4Message> {
    // Collect tool_call order from preceding assistant messages
    // Key: tool_call name → order index
    let mut last_tool_call_order: Vec<String> = Vec::new();

    messages.into_iter().map(|mut msg| {
        if msg.role == "assistant" {
            if let Some(ref tcs) = msg.tool_calls {
                last_tool_call_order = tcs.iter().map(|tc| tc.name.clone()).collect();
            }
        } else if msg.role == "user" {
            if let Some(ref blocks) = msg.content_blocks {
                let tool_blocks: Vec<usize> = blocks.iter().enumerate()
                    .filter(|(_, b)| matches!(b, V4ContentBlock::ToolResult { .. }))
                    .map(|(i, _)| i)
                    .collect();

                if tool_blocks.len() > 1 && !last_tool_call_order.is_empty() {
                    let mut indexed: Vec<(usize, &V4ContentBlock)> = tool_blocks.iter()
                        .map(|&i| (i, &blocks[i]))
                        .collect();

                    // Sort by position in last_tool_call_order (stable for unknown names)
                    indexed.sort_by(|a, b| {
                        let name_a = match a.1 {
                            V4ContentBlock::ToolResult { ref content, .. } => content.clone(),
                            _ => String::new(),
                        };
                        let name_b = match b.1 {
                            V4ContentBlock::ToolResult { ref content, .. } => content.clone(),
                            _ => String::new(),
                        };
                        let pos_a = last_tool_call_order.iter().position(|n| name_a.contains(n))
                            .unwrap_or(usize::MAX);
                        let pos_b = last_tool_call_order.iter().position(|n| name_b.contains(n))
                            .unwrap_or(usize::MAX);
                        pos_a.cmp(&pos_b)
                    });

                    let mut sorted_blocks = blocks.clone();
                    for (new_pos, &(old_idx, _)) in indexed.iter().enumerate() {
                        sorted_blocks[tool_blocks[new_pos]] = blocks[old_idx].clone();
                    }
                    msg.content_blocks = Some(sorted_blocks);
                }
            }
        }
        msg
    }).collect()
}

// ============================================================
// render_message — matches official render_message
// ============================================================

fn find_last_user_index(messages: &[V4Message]) -> isize {
    for idx in (0..messages.len()).rev() {
        if messages[idx].role == "user" || messages[idx].role == "developer" {
            return idx as isize;
        }
    }
    -1
}

fn render_message(
    index: usize,
    messages: &[V4Message],
    thinking_mode: &str,
    drop_thinking: bool,
    reasoning_effort: Option<&str>,
) -> String {
    assert!(index < messages.len());
    assert!(thinking_mode == "chat" || thinking_mode == "thinking");

    let mut prompt = String::new();
    let msg = &messages[index];
    let last_user_idx = find_last_user_index(messages);

    // Reasoning effort prefix (only at index 0 in thinking mode with max effort)
    if index == 0 && thinking_mode == "thinking" && reasoning_effort == Some("max") {
        prompt.push_str(REASONING_EFFORT_MAX);
    }

    match msg.role.as_str() {
        "system" => {
            let content = msg.content.as_deref().unwrap_or("");
            prompt.push_str(&SYSTEM_MSG_TEMPLATE.replace("{content}", content));
            if let Some(ref tools) = msg.tools {
                prompt.push_str("\n\n");
                prompt.push_str(&render_tools(tools));
            }
            if let Some(ref rf) = msg.response_format {
                prompt.push_str("\n\n");
                prompt.push_str(&RESPONSE_FORMAT_TEMPLATE.replace("{schema}", rf));
            }
        }
        "developer" => {
            let content = msg.content.as_deref().unwrap_or("");
            let mut content_dev = USER_SP_TOKEN.to_string();
            content_dev.push_str(content);
            if let Some(ref tools) = msg.tools {
                content_dev.push_str("\n\n");
                content_dev.push_str(&render_tools(tools));
            }
            if let Some(ref rf) = msg.response_format {
                content_dev.push_str("\n\n");
                content_dev.push_str(&RESPONSE_FORMAT_TEMPLATE.replace("{schema}", rf));
            }
            prompt.push_str(&USER_MSG_TEMPLATE.replace("{content}", &content_dev));
        }
        "user" => {
            prompt.push_str(USER_SP_TOKEN);

            if let Some(ref blocks) = msg.content_blocks {
                let mut parts: Vec<String> = Vec::new();
                for block in blocks {
                    match block {
                        V4ContentBlock::Text(text) => {
                            parts.push(text.clone());
                        }
                        V4ContentBlock::ToolResult { content, .. } => {
                            parts.push(TOOL_OUTPUT_TEMPLATE.replace("{content}", content));
                        }
                    }
                }
                prompt.push_str(&parts.join("\n\n"));
            } else if let Some(ref content) = msg.content {
                prompt.push_str(content);
            }
        }
        "assistant" => {
            let reasoning = msg.reasoning.as_deref().unwrap_or("");
            let content = msg.content.as_deref().unwrap_or("");
            let mut tc_content = String::new();

            if let Some(ref tcs) = msg.tool_calls {
                let tc_list: Vec<String> = tcs.iter().map(|tc| {
                    format!(
                        "<{dsml}invoke name=\"{name}\">\n{args}\n</{dsml}invoke>",
                        dsml = DSML_TOKEN,
                        name = tc.name,
                        args = encode_arguments_to_dsml_from_msg(tc),
                    )
                }).collect();
                tc_content.push_str(&format!(
                    "\n\n<{dsml}tool_calls>\n{tcs}\n</{dsml}tool_calls>",
                    dsml = DSML_TOKEN,
                    tcs = tc_list.join("\n"),
                ));
            }

            let thinking_part = if thinking_mode == "thinking" && !is_prev_task(messages, index) {
                if !drop_thinking || index as isize > last_user_idx {
                    THINKING_TEMPLATE.replace("{reasoning}", reasoning) + THINKING_END_TOKEN
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let tmpl = if msg.wo_eos {
                ASSISTANT_MSG_TEMPLATE.replace("{reasoning}", &thinking_part)
                    .replace("{content}", content)
                    .replace("{tool_calls}", &tc_content)
            } else {
                ASSISTANT_MSG_TEMPLATE.replace("{reasoning}", &thinking_part)
                    .replace("{content}", content)
                    .replace("{tool_calls}", &tc_content)
                    + EOS_TOKEN
            };
            prompt.push_str(&tmpl);
        }
        _ => {
            // Unknown role — output content as-is
            if let Some(ref content) = msg.content {
                prompt.push_str(content);
            }
        }
    }

    // Transition tokens — matches official logic
    if index + 1 < messages.len()
        && messages[index + 1].role != "assistant"
        && messages[index + 1].role != "latest_reminder"
    {
        return prompt;
    }

    if messages[index].role == "user" || messages[index].role == "developer" {
        prompt.push_str(ASSISTANT_SP_TOKEN);
        if !drop_thinking && thinking_mode == "thinking" {
            prompt.push_str(THINKING_START_TOKEN);
        } else if drop_thinking && thinking_mode == "thinking" && (index as isize) >= last_user_idx {
            prompt.push_str(THINKING_START_TOKEN);
        } else {
            prompt.push_str(THINKING_END_TOKEN);
        }
    }

    prompt
}

fn is_prev_task(_messages: &[V4Message], index: usize) -> bool {
    if index == 0 { return false; }
    // In the official code, `task` is a message-level field.
    // Our messages don't have this, so always return false.
    false
}

// ============================================================
// encode_messages — matches official encode_messages
// ============================================================

fn encode_messages(
    messages: &[V4Message],
    thinking_mode: &str,
    context: &[V4Message],    // preceding context (empty for our use)
    drop_thinking: bool,
    add_default_bos_token: bool,
    reasoning_effort: Option<&str>,
) -> String {
    let full_messages: Vec<V4Message> = {
        let mut combined = context.to_vec();
        combined.extend(messages.iter().cloned());
        combined
    };

    let context_len = context.len();

    // Resolve drop_thinking: if any message has tools, don't drop thinking
    let effective_drop_thinking = if full_messages.iter().any(|m| m.tools.is_some()) {
        false
    } else {
        drop_thinking
    };

    let prompt = if add_default_bos_token && context_len == 0 {
        BOS_TOKEN.to_string()
    } else {
        String::new()
    };

    // Apply drop_thinking if needed
    let (rendered_msgs, context_render_len) = if thinking_mode == "thinking" && effective_drop_thinking {
        let dropped = drop_thinking_messages(&full_messages);
        let dropped_context_len = drop_thinking_messages(context).len();
        (dropped, dropped_context_len)
    } else {
        (full_messages, context_len)
    };

    let num_to_render = rendered_msgs.len() - context_render_len;

    let mut result = prompt;
    for idx in 0..num_to_render {
        result.push_str(&render_message(
            idx + context_render_len,
            &rendered_msgs,
            thinking_mode,
            effective_drop_thinking,
            reasoning_effort,
        ));
    }

    result
}

// ============================================================
// _drop_thinking_messages — matches official
// ============================================================

fn drop_thinking_messages(messages: &[V4Message]) -> Vec<V4Message> {
    let last_user_idx = find_last_user_index(messages);
    let keep_roles = ["user", "system", "tool", "latest_reminder", "direct_search_results"];

    let mut result: Vec<V4Message> = Vec::new();
    for (idx, msg) in messages.iter().enumerate() {
        let role = msg.role.as_str();
        if keep_roles.contains(&role) || (idx as isize) >= last_user_idx {
            result.push(msg.clone());
        } else if role == "assistant" {
            let mut m = msg.clone();
            m.reasoning = None;
            result.push(m);
        }
        // developer and other roles before last_user_idx are dropped
    }
    result
}
