/// Simplified cache detector using prefix matching
pub struct CacheDetector {
    known_prefixes: Vec<Vec<u32>>,
}

impl CacheDetector {
    pub fn new() -> Self {
        CacheDetector { known_prefixes: Vec::new() }
    }

    /// Store token IDs as a known prefix
    pub fn store(&mut self, token_ids: &[u32]) {
        self.known_prefixes.push(token_ids.to_vec());
        if self.known_prefixes.len() > 100 {
            self.known_prefixes.remove(0);
        }
    }

    /// Detect cache hit by longest prefix matching
    pub fn detect(&self, token_ids: &[u32]) -> (u32, u32) {
        let mut max_match = 0u32;
        for prefix in &self.known_prefixes {
            let mut matched = 0u32;
            for (a, b) in token_ids.iter().zip(prefix.iter()) {
                if a == b { matched += 1; } else { break; }
            }
            if matched > max_match {
                max_match = matched;
            }
        }
        (max_match, token_ids.len() as u32 - max_match)
    }
}
