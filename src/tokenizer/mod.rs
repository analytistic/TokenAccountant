// src/tokenizer/mod.rs

pub mod gpt;
pub mod qwen;
pub mod deepseek;
pub mod claude;
pub mod python;

use std::collections::HashMap;
use std::sync::Arc;

/// 分词器 trait（接口）
pub trait Tokenizer: Send + Sync {
    /// 将文本编码为 Token IDs
    fn encode(&self, text: &str) -> Vec<u32>;
    
    /// 将 Token IDs 解码为文本
    fn decode(&self, ids: &[u32]) -> String;
    
    /// 计算文本的 token 数（默认实现）
    fn count_tokens(&self, text: &str) -> u32 {
        self.encode(text).len() as u32
    }
}

/// 分词器工厂
pub struct TokenizerFactory {
    tokenizers: HashMap<String, Arc<dyn Tokenizer>>,
}

impl TokenizerFactory {
    /// 创建新的分词器工厂（注册默认分词器）
    pub fn new() -> Self {
        let mut factory = TokenizerFactory {
            tokenizers: HashMap::new(),
        };
        
        // 注册 GPT 分词器
        factory.register("gpt".to_string(), Arc::new(gpt::GptTokenizer::new()));
        
        // 注册 Qwen 分词器
        factory.register("qwen".to_string(), Arc::new(qwen::QwenTokenizer::new()));
        
        // 注册 DeepSeek 分词器
        factory.register("deepseek".to_string(), Arc::new(deepseek::DeepSeekTokenizer::new()));
        
        // 注册 Claude 分词器（近似）
        factory.register("claude".to_string(), Arc::new(claude::ClaudeTokenizer));
        
        factory
    }
    
    /// 注册新的分词器
    pub fn register(&mut self, name: String, tokenizer: Arc<dyn Tokenizer>) {
        self.tokenizers.insert(name, tokenizer);
    }
    
    /// 根据模型名称获取分词器
    pub fn get(&self, model: &str) -> Arc<dyn Tokenizer> {
        // 根据模型名称自动选择分词器
        if model.starts_with("gpt-") || model.starts_with("text-") {
            self.tokenizers.get("gpt").unwrap().clone()
        } else if model.starts_with("qwen-") {
            self.tokenizers.get("qwen").unwrap().clone()
        } else if model.starts_with("deepseek-") {
            self.tokenizers.get("deepseek").unwrap().clone()
        } else if model.starts_with("claude-") {
            self.tokenizers.get("claude").unwrap().clone()
        } else if model.starts_with("gemini-") {
            // Gemini 用近似（字符数 / 4）
            self.tokenizers.get("claude").unwrap().clone()
        } else {
            // 默认用 GPT 分词器（近似）
            eprintln!("Warning: Unknown model '{}', using GPT tokenizer as fallback", model);
            self.tokenizers.get("gpt").unwrap().clone()
        }
    }
}
