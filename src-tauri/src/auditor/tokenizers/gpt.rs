use tiktoken_rs::CoreBPE;
use crate::auditor::tokenizer::Tokenizer;
use crate::auditor::message_converter::{Conversation, NormalizedMessage};

pub struct GptTokenizer {
    encoding: CoreBPE,
}

impl GptTokenizer {
    pub fn new() -> Self {
        let encoding = tiktoken_rs::cl100k_base()
            .expect("Failed to load cl100k_base encoding");
        GptTokenizer { encoding }
    }
}

impl Tokenizer for GptTokenizer {
    fn encode(&self, text: &str) -> Vec<u32> {
        self.encoding.encode_with_special_tokens(text)
            .iter().map(|&x| x as u32).collect()
    }

    fn decode(&self, ids: &[u32]) -> String {
        let ids_vec: Vec<usize> = ids.iter().map(|&x| x as usize).collect();
        self.encoding.decode(ids_vec).unwrap_or_default()
    }

    fn count_tokens(&self, text: &str) -> u32 {
        self.encoding.encode_with_special_tokens(text).len() as u32
    }

    fn apply_chat_template(&self, conv: &Conversation) -> String {
        let mut text = String::new();
        for msg in &conv.messages {
            if let Some(ref r) = msg.reasoning {
                text.push_str(r);
                text.push('\n');
            }
            text.push_str(&msg.content);
            text.push('\n');
            if let Some(ref tcs) = msg.tool_calls {
                for tc in tcs {
                    text.push_str(&format!("[tool_use: {}({})]\n", tc.name, tc.arguments));
                }
            }
        }
        for tool in &conv.tools {
            text.push_str(&format!("tool: {}\ndescription: {}\nschema: {}\n\n", tool.name, tool.description, tool.input_schema));
        }
        text
    }

    fn render_output(&self, msg: &NormalizedMessage) -> String {
        let mut text = String::new();
        if let Some(ref r) = msg.reasoning {
            text.push_str("[thinking]");
            text.push_str(r);
            text.push_str("[/thinking]\n");
        }
        text.push_str(&msg.content);
        text.push('\n');
        if let Some(ref tcs) = msg.tool_calls {
            for tc in tcs {
                text.push_str(&format!("[tool_use: {}({})]\n", tc.name, tc.arguments));
            }
        }
        text
    }
}
