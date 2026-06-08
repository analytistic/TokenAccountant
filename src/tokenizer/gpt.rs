// src/tokenizer/gpt.rs

use tiktoken_rs::{get_encoding, Encoding};
use crate::tokenizer::Tokenizer;

/// GPT 分词器（使用 tiktoken-rs）
pub struct GptTokenizer {
    encoding: Encoding,
}

impl GptTokenizer {
    /// 创建新的 GPT 分词器
    pub fn new() -> Self {
        // 使用 cl100k_base 编码（GPT-4, GPT-3.5-turbo）
        let encoding = get_encoding("cl100k_base")
            .expect("Failed to load cl100k_base encoding");
        
        GptTokenizer { encoding }
    }
    
    /// 根据模型名称获取编码
    #[allow(dead_code)]
    fn get_encoding_for_model(_model: &str) -> Encoding {
        // 简化版：都用 cl100k_base
        get_encoding("cl100k_base")
            .expect("Failed to load cl100k_base")
    }
}

impl Tokenizer for GptTokenizer {
    fn encode(&self, text: &str) -> Vec<u32> {
        self.encoding.encode_with_special_tokens(text, &[])
            .into_iter()
            .map(|x| x as u32)
            .collect()
    }
    
    fn decode(&self, ids: &[u32]) -> String {
        let ids_i64: Vec<i64> = ids.iter().map(|&x| x as i64).collect();
        self.encoding.decode(ids_i64, true)
            .expect("Failed to decode tokens")
    }
    
    fn count_tokens(&self, text: &str) -> u32 {
        self.encoding.encode_with_special_tokens(text, &[]).len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encode() {
        let tokenizer = GptTokenizer::new();
        let tokens = tokenizer.encode("Hello, world!");
        assert!(!tokens.is_empty());
        println!("Tokens: {:?}", tokens);
    }
    
    #[test]
    fn test_decode() {
        let tokenizer = GptTokenizer::new();
        let tokens = tokenizer.encode("Hello, world!");
        let text = tokenizer.decode(&tokens);
        assert_eq!(text, "Hello, world!");
    }
    
    #[test]
    fn test_count_tokens() {
        let tokenizer = GptTokenizer::new();
        let count = tokenizer.count_tokens("Hello, world!");
        assert!(count > 0);
        println!("Token count: {}", count);
    }
}
