use std::collections::HashMap;
use std::sync::Arc;
use super::message_converter::{Conversation, NormalizedMessage};

pub trait Tokenizer: Send + Sync {
    fn encode(&self, text: &str) -> Vec<u32>;
    fn decode(&self, ids: &[u32]) -> String;
    fn count_tokens(&self, text: &str) -> u32 {
        self.encode(text).len() as u32
    }
    /// Apply model-specific chat template to a Conversation.
    /// Output is the formatted prompt string ready for tokenization.
    fn apply_chat_template(&self, conv: &Conversation) -> String;
    /// Render a single assistant output message to model-specific format.
    /// Used for accurate output token counting (not raw SSE text).
    fn render_output(&self, msg: &NormalizedMessage) -> String;
}

pub struct TokenizerFactory {
    tokenizers: HashMap<String, Arc<dyn Tokenizer>>,
}

impl TokenizerFactory {
    pub fn new() -> Self {
        let mut factory = TokenizerFactory {
            tokenizers: HashMap::new(),
        };
        factory.register("gpt".into(), Arc::new(super::tokenizers::gpt::GptTokenizer::new()));
        factory.register("deepseek".into(), Arc::new(super::tokenizers::deepseek::DeepSeekTokenizer::new()));
        factory
    }

    pub fn register(&mut self, name: String, tokenizer: Arc<dyn Tokenizer>) {
        self.tokenizers.insert(name, tokenizer);
    }

    pub fn for_model(&self, model: &str) -> Option<Arc<dyn Tokenizer>> {
        if model.starts_with("deepseek-") {
            self.tokenizers.get("deepseek").cloned()
        } else if model.starts_with("gpt-") || model.starts_with("text-")
            || model.starts_with("qwen-") {
            self.tokenizers.get("gpt").cloned()
        } else {
            None
        }
    }
}
