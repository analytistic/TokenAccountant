use std::collections::{HashSet, VecDeque};

const BLOCK_SIZE: usize = 16;
/// Initial hash for the first block (no parent), matching vLLM's init_none_hash concept.
const INIT_HASH: u64 = 0;
pub struct CacheDetector {
    /// Set of block hashes currently in the simulated KV cache.
    cached_blocks: HashSet<u64>,
    /// FIFO eviction queue — oldest inserted blocks at the front.
    eviction_queue: VecDeque<u64>,
    /// Max blocks before eviction (from config).
    max_blocks: usize,
}

impl CacheDetector {
    pub fn new(max_blocks: usize) -> Self {
        CacheDetector {
            cached_blocks: HashSet::new(),
            eviction_queue: VecDeque::new(),
            max_blocks,
        }
    }

    /// Insert a block hash, evicting oldest if over capacity.
    fn insert_block(&mut self, hash: u64) {
        if self.cached_blocks.insert(hash) {
            self.eviction_queue.push_back(hash);
        }
        while self.eviction_queue.len() > self.max_blocks {
            if let Some(oldest) = self.eviction_queue.pop_front() {
                self.cached_blocks.remove(&oldest);
            }
        }
    }

    /// Store block hashes for the full rendered prompt (including the new assistant output).
    /// The input must be the complete rendered prompt that the next request will see.
    pub fn store_combined(&mut self, token_ids: &[u32]) {
        let mut ph = INIT_HASH;
        for chunk in token_ids.chunks(BLOCK_SIZE) {
            if chunk.len() < BLOCK_SIZE {
                break;
            }
            let hash = compute_chain_hash(ph, chunk);
            self.insert_block(hash);
            ph = hash;
        }
    }

    /// Detect cache hit by walking the token sequence in block-sized steps.
    /// For each full 16-token block, compute its chain hash and check if cached.
    /// Returns (cached_token_count, remaining_token_count).
    pub fn detect(&self, token_ids: &[u32]) -> (u32, u32) {
        let mut matched: u32 = 0;
        let mut parent_hash = INIT_HASH;

        for chunk in token_ids.chunks(BLOCK_SIZE) {
            if chunk.len() < BLOCK_SIZE {
                break; // partial block is never cached
            }
            let hash = compute_chain_hash(parent_hash, chunk);
            if self.cached_blocks.contains(&hash) {
                matched += BLOCK_SIZE as u32;
                parent_hash = hash;
            } else {
                break;
            }
        }

        let total = token_ids.len() as u32;
        (matched, total - matched)
    }

}

/// Deterministic hash: linear congruential generator.
/// DefaultHasher uses random seeds per invocation — NOT deterministic.
fn compute_chain_hash(parent_hash: u64, tokens: &[u32]) -> u64 {
    let mut h = parent_hash;
    for &t in tokens {
        h = h.wrapping_mul(6364136223846793005).wrapping_add(t as u64);
    }
    h
}
