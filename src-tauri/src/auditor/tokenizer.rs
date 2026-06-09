use std::collections::HashMap;
use std::sync::Arc;

pub trait Tokenizer: Send + Sync {
    fn encode(&self, text: &str) -> Vec<u32>;
    fn decode(&self, ids: &[u32]) -> String;
    fn count_tokens(&self, text: &str) -> u32 {
        self.encode(text).len() as u32
    }
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
        factory
    }

    pub fn register(&mut self, name: String, tokenizer: Arc<dyn Tokenizer>) {
        self.tokenizers.insert(name, tokenizer);
    }

    pub fn for_model(&self, model: &str) -> Option<Arc<dyn Tokenizer>> {
        if model.starts_with("gpt-") || model.starts_with("text-")
            || model.starts_with("qwen-") || model.starts_with("deepseek-") {
            self.tokenizers.get("gpt").cloned()
        } else {
            None
        }
    }
}
