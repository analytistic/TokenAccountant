// src/tokenizer/deepseek.rs//

use tiktoken_rs::{get_encoding, Encoding};
use crate::tokenizer::Tokenizer;

/// DeepSeek 分词器（DeepSeek-V3 基于 tiktoken，使用 cl100k_base）
pub struct DeepSeekTokenizer {
    encoding: Encoding,
}

impl DeepSeekTokenizer {
    /// 创建新的 DeepSeek 分词器
    pub fn new() -> Self {
        // DeepSeek-V3 使用 cl100k_base 编码（和 GPT-4 相同）
        let encoding = get_encoding("cl100k_base")
            .expect("Failed to load cl100k_base encoding for DeepSeek");
        
        DeepSeekTokenizer { encoding }
    }
}

impl Tokenizer for DeepSeekTokenizer {
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
        let tokenizer = DeepSeekTokenizer::new();
        let tokens = tokenizer.encode("Hello, world!");
        assert!(!tokens.is_empty());
        println!("DeepSeek Tokens: {:?}", tokens);
    }
    
    #[test]
    fn test_decode() {
        let tokenizer = DeepSeekTokenizer::new();
        let tokens = tokenizer.encode("Hello, world!");
        let text = tokenizer.decode(&tokens);
        assert_eq!(text, "Hello, world!");
    }
    
    #[test]
    fn test_count_tokens() {
        let tokenizer = DeepSeekTokenizer::new();
        let count = tokenizer.count_tokens("你好，世界！");
        assert!(count > 0);
        println!("DeepSeek Token count: {}", count);
    }
    
    #[test]
    fn test_chinese_text() {
        let tokenizer = DeepSeekTokenizer::new();
        let tokens = tokenizer.encode("def add(a, b):\n    return a + b");
        println!("Code tokens: {:?}", tokens);
        assert!(tokens.len() > 0);
    }
}
