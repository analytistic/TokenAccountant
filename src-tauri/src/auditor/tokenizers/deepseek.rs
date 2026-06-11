use crate::auditor::tokenizer::Tokenizer;
use crate::auditor::message_converter::{ContentPart, Conversation, NormalizedMessage};
use super::gpt::GptTokenizer;

/// DeepSeek-V4 tokenizer with DSML chat template.
/// Uses GptTokenizer for byte-pair encoding; the key difference is
/// apply_chat_template which produces DeepSeek's DSML format.
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
        render_assistant_output(msg)
    }

    fn apply_chat_template(&self, conv: &Conversation) -> String {
        let mut messages = conv.messages.clone();

        // 1. merge_tool_messages: fold role:"tool" back into user messages
        messages = merge_tool_messages(messages);

        // 2. Render tool definitions
        let tools_text = if !conv.tools.is_empty() {
            Some(render_tools(&conv.tools))
        } else {
            None
        };

        // 3. Encode messages into DSML format
        encode_messages(&messages, tools_text)
    }
}

/// Merge role:"tool" messages into the preceding user message as <tool_result> blocks.
/// DeepSeek-V4 has no standalone "tool" role — tool results are embedded in user messages.
/// Tool results are placed BEFORE user text, matching Anthropic's tool_result ordering.
///
/// vLLM's conversion creates "tool" role messages from tool_result blocks.
/// This function reassembles them into DSML's inline format.
fn merge_tool_messages(messages: Vec<NormalizedMessage>) -> Vec<NormalizedMessage> {
    let mut tool_results: Vec<Option<Vec<String>>> = vec![None; messages.len()];

    let mut i = 0;
    while i < messages.len() {
        if messages[i].role == "tool" {
            let tool_start = i;
            let mut texts = Vec::new();
            while i < messages.len() && messages[i].role == "tool" {
                texts.push(messages[i].content_text());
                i += 1;
            }
            // Assign to the PRECEDING user message (tool result belongs to the user that follows assistant's tool_use)
            for j in (0..tool_start).rev() {
                if messages[j].role == "user" {
                    tool_results[j] = Some(texts);
                    break;
                }
            }
        } else {
            i += 1;
        }
    }

    let mut result = Vec::new();
    for (i, msg) in messages.iter().enumerate() {
        if msg.role == "tool" {
            continue;
        }
        let mut merged = msg.clone();
        if let Some(ref texts) = tool_results[i] {
            // Tool results come BEFORE user text (matches Anthropic and DeepSeek format)
            let mut new_parts = Vec::new();
            for t in texts {
                new_parts.push(ContentPart::Text(
                    format!("<tool_result>{}</tool_result>\n", t),
                ));
            }
            new_parts.extend(merged.content_parts.clone());
            merged.content_parts = new_parts;
        }
        result.push(merged);
    }
    result
}

/// Render tool definitions into DeepSeek DSML system prompt section.
/// Matches vLLM's TOOLS_TEMPLATE.
fn render_tools(tools: &[crate::auditor::message_converter::ToolDef]) -> String {
    let dsml = "｜DSML｜";
    let mut text = String::new();
    text.push_str(&format!("\n\n## Tools\n\nYou have access to a set of tools to help answer the user's question. You can invoke tools by writing a \"<{dsml}tool_calls>\" block like the following:\n\n"));
    text.push_str(&format!("<{dsml}tool_calls>\n"));
    text.push_str(&format!("<{dsml}invoke name=\"$TOOL_NAME\">\n"));
    text.push_str(&format!("<{dsml}parameter name=\"$PARAMETER_NAME\" string=\"true|false\">$PARAMETER_VALUE</{dsml}parameter>\n"));
    text.push_str("...\n");
    text.push_str(&format!("</{dsml}invoke>\n"));
    text.push_str(&format!("</{dsml}tool_calls>\n\n"));

    text.push_str("### Available Tool Schemas\n\n");
    for tool in tools {
        text.push_str(&format!("{}\n", tool.input_schema));
        text.push('\n');
    }

    text.push_str(&format!("You MUST strictly follow the above defined tool name and parameter schemas to invoke tool calls.\n"));
    text
}

/// Encode messages into DeepSeek DSML prompt format.
///
/// Structured to match vLLM's deepseek_v4_encoding.encode_messages + render_message.
///
/// vLLM architecture:
///   - system:  `{content}` (+ tools if first message)
///   - user:    `<｜User｜>{content}` + transition `<｜Assistant｜><think>` or `</think>`
///   - assistant: `{reasoning}</think>{content}{tool_calls}<｜end▁of▁sentence｜>`
///
/// The `<think>` before reasoning comes from the user→assistant transition,
/// not from the assistant template itself.
fn encode_messages(messages: &[NormalizedMessage], tools_text: Option<String>) -> String {
    let mut prompt = String::new();
    prompt.push_str("<｜begin▁of▁sentence｜>");

    for (i, msg) in messages.iter().enumerate() {
        let content = msg.content_text();
        match msg.role.as_str() {
            "system" => {
                // system_msg_template
                prompt.push_str(&content);
                if i == 0 {
                    if let Some(ref t) = tools_text {
                        prompt.push_str(t);
                    }
                }
            }
            "user" => {
                // user_msg_template: <｜User｜>{content}
                prompt.push_str("<｜User｜>");
                prompt.push_str(&content);

                // Transition: look ahead to determine if the next assistant uses thinking.
                // vLLM: ASSISTANT_SP_TOKEN + <think> if reasoning follows, else </think>
                let next_is_thinking = messages.get(i + 1)
                    .map(|next| next.role == "assistant" && next.reasoning.is_some())
                    .unwrap_or(false);
                prompt.push_str("<｜Assistant｜>");
                prompt.push_str(if next_is_thinking { "<think>" } else { "</think>" });
            }
            "assistant" => {
                // In-conversation: <think> comes from user→assistant transition
                prompt.push_str(&render_assistant_output_internal(msg));
            }
            _ => {
                prompt.push_str(&content);
            }
        }
    }

    prompt
}

/// Render a single assistant message to DSML format (standalone).
///
/// Output format (thinking mode):
///   <think>{reasoning}</think>{content}{tool_calls}<｜end▁of▁sentence｜>
///
/// Output format (chat mode, no reasoning):
///   {content}{tool_calls}<｜end▁of▁sentence｜>
///
/// Use for standalone output token counting (post-stream audit).
/// For in-conversation rendering, the <think> prefix comes from the
/// user→assistant transition instead.
pub fn render_assistant_output(msg: &NormalizedMessage) -> String {
    render_assistant_output_internal_(msg, true)
}

/// Render assistant output without the leading <think> tag.
/// Used internally by encode_messages — <think> is provided by the
/// user→assistant transition in the conversation.
pub fn render_assistant_output_internal(msg: &NormalizedMessage) -> String {
    render_assistant_output_internal_(msg, false)
}

fn render_assistant_output_internal_(
    msg: &NormalizedMessage,
    add_think: bool,
) -> String {
    let mut out = String::new();
    let content = msg.content_text();

    if let Some(ref r) = msg.reasoning {
        if add_think {
            out.push_str("<think>");
        }
        out.push_str(r);
        out.push_str("</think>");
    }

    // content
    out.push_str(&content);

    // tool calls (DSML)
    if let Some(ref tcs) = msg.tool_calls {
        for tc in tcs {
            out.push_str(&format!(
                "\n\n<｜DSML｜tool_calls>\n<｜DSML｜invoke name=\"{}\">\n<｜DSML｜parameter name=\"arguments\" string=\"true\">{}</｜DSML｜parameter>\n</｜DSML｜invoke>\n</｜DSML｜tool_calls>",
                tc.name, tc.arguments
            ));
        }
    }

    out.push_str("<｜end▁of▁sentence｜>");
    out
}
