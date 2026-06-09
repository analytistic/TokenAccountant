use crate::auditor::tokenizer::Tokenizer;

/// Claude tokenizer — Phase 2
/// Claude's tokenizer is not public. Implementation will use character-count approximation.
pub struct ClaudeTokenizer;

impl Tokenizer for ClaudeTokenizer {
    fn encode(&self, _text: &str) -> Vec<u32> {
        unimplemented!("Claude tokenizer will be implemented in Phase 2")
    }

    fn decode(&self, _ids: &[u32]) -> String {
        unimplemented!("Claude tokenizer will be implemented in Phase 2")
    }

    fn count_tokens(&self, _text: &str) -> u32 {
        unimplemented!("Claude tokenizer will be implemented in Phase 2")
    }
}
