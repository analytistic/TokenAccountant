// src/tokenizer/claude.rs

use crate::tokenizer::Tokenizer;

/// Claude 分词器（近似：字符数 / 1.5）
/// 
/// 注意：Claude 的分词器未公开，这是近似计算。
/// - 英文：约 4 字符/token
/// - 中文：约 1.5 字符/token
/// 
/// 如果需要精确计算，请提供 Anthropic API Key，
/// 然后调用 `client.messages.count_tokens()` API。
pub struct ClaudeTokenizer;

impl ClaudeTokenizer {
    /// 创建新的 Claude 分词器
    pub fn new() -> Self {
        ClaudeTokenizer
    }
    
    /// 近似计算 token 数
    /// 
    /// 策略：
    /// - 如果有中文 → 按 1.5 字符/token 计算
    /// - 如果纯英文 → 按 4 字符/token 计算
    /// - 混合 → 按 2 字符/token 计算（保守估计）
    fn approximate_token_count(text: &str) -> u32 {
        let char_count = text.chars().count() as f64;
        
        // 检测是否包含中文
        let has_chinese = text.chars().any(|c| {
            let cp = c as u32;
            (0x4E00..=0x9FFF).contains(&cp) ||  // CJK 统一表意文字
            (0x3000..=0x303F).contains(&cp) ||  // CJK 符号和标点
            (0xFF00..=0xFFEF).contains(&cp)    // 全角 ASCII
        });
        
        // 检测是否包含日文/韩文
        let has_japanese = text.chars().any(|c| {
            let cp = c as u32;
            (0x3040..=0x309F).contains(&cp) ||  // 平假名
            (0x30A0..=0x30FF).contains(&cp)    // 片假名
        });
        
        let has_korean = text.chars().any(|c| {
            let cp = c as u32;
            (0xAC00..=0xD7AF).contains(&cp)    // 韩文音节
        });
        
        let chars_per_token = if has_chinese || has_japanese || has_korean {
            1.5  // 东亚文字
        } else {
            4.0  // 西方文字
        };
        
        (char_count / chars_per_token).ceil() as u32
    }
}

impl Tokenizer for ClaudeTokenizer {
    fn encode(&self, text: &str) -> Vec<u32> {
        let token_count = Self::approximate_token_count(text);
        // 返回伪 Token IDs（因为我们不知道真实的 Token IDs）
        // 这只能用于近似计算 token 数，不能用于缓存检测
        (0..token_count).collect()
    }
    
    fn decode(&self, ids: &[u32]) -> String {
        // 无法从近似的 Token IDs 还原文本
        format!("[{} tokens (approximate)]", ids.len())
    }
    
    fn count_tokens(&self, text: &str) -> u32 {
        Self::approximate_token_count(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_english_text() {
        let tokenizer = ClaudeTokenizer::new();
        let count = tokenizer.count_tokens("Hello, world!");
        println!("English token count: {}", count);
        // "Hello, world!" 约 4 tokens
        assert!(count >= 3 && count <= 5);
    }
    
    #[test]
    fn test_chinese_text() {
        let tokenizer = ClaudeTokenizer::new();
        let count = tokenizer.count_tokens("你好，世界！");
        println!("Chinese token count: {}", count);
        // "你好，世界！" 约 6 chars / 1.5 = 4 tokens
        assert!(count >= 3 && count <= 5);
    }
    
    #[test]
    fn test_mixed_text() {
        let tokenizer = ClaudeTokenizer::new();
        let count = tokenizer.count_tokens("Hello 你好！");
        println!("Mixed token count: {}", count);
        assert!(count > 0);
    }
    
    #[test]
    fn test_code_text() {
        let tokenizer = ClaudeTokenizer::new();
        let code = "def add(a, b):\n    return a + b";
        let count = tokenizer.count_tokens(code);
        println!("Code token count: {}", count);
        assert!(count > 0);
    }
    
    #[test]
    fn test_long_text() {
        let tokenizer = ClaudeTokenizer::new();
        let text = "a".repeat(1000);
        let count = tokenizer.count_tokens(&text);
        println!("Long text token count: {}", count);
        // 1000 个 'a' 约 250 tokens
        assert!(count >= 200 && count <= 300);
    }
}
