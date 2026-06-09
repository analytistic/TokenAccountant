use tiktoken_rs::CoreBPE;
use crate::auditor::tokenizer::Tokenizer;

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
}
