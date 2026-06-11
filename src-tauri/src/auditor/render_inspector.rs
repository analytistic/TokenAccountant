use crate::auditor::message_converter::{from_anthropic_body, Conversation, NormalizedMessage};
use crate::auditor::tokenizer::Tokenizer;

/// A single captured trace from one proxy request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DevTrace {
    pub turn: u32,
    pub model: String,
    /// Rendered text from the detect path (request body → from_anthropic_body → apply_chat_template)
    pub detect_text: String,
    /// Rendered text from the store path (conv + output_msg → apply_chat_template)
    pub store_text: String,
}

/// Ring buffer holding the most recent N traces.
pub struct DevTraceBuffer {
    traces: Vec<DevTrace>,
    max_entries: usize,
    counter: u32,
}

impl DevTraceBuffer {
    pub fn new(max_entries: usize) -> Self {
        DevTraceBuffer {
            traces: Vec::with_capacity(max_entries),
            max_entries,
            counter: 0,
        }
    }

    pub fn push(&mut self, model: String, detect_text: String, store_text: String) {
        self.counter += 1;
        let trace = DevTrace {
            turn: self.counter,
            model,
            detect_text,
            store_text,
        };
        if self.traces.len() >= self.max_entries {
            self.traces.remove(0);
        }
        self.traces.push(trace);
    }

    pub fn list(&self) -> Vec<DevTrace> {
        self.traces.clone()
    }

    pub fn clear(&mut self) {
        self.traces.clear();
        self.counter = 0;
    }
}

/// Run the detect path and return the rendered text.
/// Input: raw request body (Anthropic Messages API JSON).
/// Output: apply_chat_template output.
pub fn inspect_detect(tokenizer: &dyn Tokenizer, body_str: &str) -> String {
    let conv = from_anthropic_body(body_str);
    tokenizer.apply_chat_template(&conv)
}

/// Run the store path and return the rendered text.
/// Input: the conversation from the detect phase + the new assistant output message.
/// Output: apply_chat_template output of conv + output_msg.
pub fn inspect_store(
    tokenizer: &dyn Tokenizer,
    conv: &Conversation,
    output_msg: &NormalizedMessage,
) -> String {
    let mut full_conv = conv.clone();
    full_conv.messages.push(output_msg.clone());
    tokenizer.apply_chat_template(&full_conv)
}
