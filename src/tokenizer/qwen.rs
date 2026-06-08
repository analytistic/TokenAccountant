// src/tokenizer/qwen.rs

use tiktoken_rs::{get_encoding, Encoding};
use crate::tokenizer::Tokenizer;

/// Qwen 分词器（Qwen2 基于 tiktoken，使用 cl100k_base）
pub struct QwenTokenizer {
    encoding: Encoding,
}

impl QwenTokenizer {
    /// 创建新的 Qwen 分词器
    pub fn new() -> Self {
        // Qwen2 使用 cl100k_base 编码（和 GPT-4 相同）
        let encoding = get_encoding("cl100k_base")
            .expect("Failed to load cl100k_base encoding for Qwen");
        
        QwenTokenizer { encoding }
    }
}

impl Tokenizer for QwenTokenizer {
    fn encode(&self, text: &str) -> Vec<u32> {
        self.encoding.encode_with_special_tokens(text, &[])
            .into_iter()
            .map(|x| x as u32)
            .collect()
    }
    
    fn decode(&self, ids: &[u32]) -> String {
        let ids_i64: Vec<i64> = ids.iter().map(|&x| x as i64).collect();
        self.encoding.decode(&ids_i64, true)
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
        let tokenizer = QwenTokenizer::new();
        let tokens = tokenizer.encode("你好，世界！");
        assert!(!tokens.is_empty());
        println!("Qwen Tokens: {:?}", tokens);
    }
    
    #[test]
    fn test_decode() {
        let tokenizer = QwenTokenizer::new();
        let tokens = tokenizer.encode("Hello, world!");
        let text = tokenizer.decode(&tokens);
        assert_eq!(text, "Hello, world!");
    }
    
    #[test]
    fn test_count_tokens() {
        let tokenizer = QwenTokenizer::new();
        let count = tokenizer.count_tokens("你好，世界！");
        assert!(count > 0);
        println!("Qwen Token count: {}", count);
    }
}
