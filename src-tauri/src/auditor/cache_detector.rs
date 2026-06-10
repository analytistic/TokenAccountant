use std::collections::HashSet;
use std::hash::{Hash, Hasher};

const BLOCK_SIZE: usize = 16;
/// Initial hash for the first block (no parent), matching vLLM's init_none_hash concept.
const INIT_HASH: u64 = 0;

pub struct CacheDetector {
    /// Set of block hashes currently in the simulated KV cache.
    cached_blocks: HashSet<u64>,
    /// Prefill tail tokens that didn't fill a full block.
    /// Stored here during `store()`, used by `store_combined()` to chain with output.
    pending_tail: Vec<u32>,
    /// Last full block's chain hash — parent hash for the first tail+output block.
    pending_parent_hash: u64,
}

impl CacheDetector {
    pub fn new() -> Self {
        CacheDetector {
            cached_blocks: HashSet::new(),
            pending_tail: Vec::new(),
            pending_parent_hash: INIT_HASH,
        }
    }

    /// Store combined prompt tail + output tokens as block hashes.
    /// Mimics vLLM's hash(parent_hash + block_tokens) chain for KV blocks.
    /// Takes only output_ids; the prompt tail was saved by `store()` earlier.
    /// Only full blocks (16 tokens) are stored; partial tail blocks are skipped.
    pub fn store_combined(&mut self, output_ids: &[u32]) {
        if self.pending_tail.is_empty() && output_ids.is_empty() {
            return;
        }

        let combined: Vec<u32> = self
            .pending_tail
            .iter()
            .chain(output_ids.iter())
            .copied()
            .collect();

        let mut ph = self.pending_parent_hash;
        for chunk in combined.chunks(BLOCK_SIZE) {
            if chunk.len() < BLOCK_SIZE {
                break; // vLLM only caches full blocks
            }
            let hash = compute_chain_hash(ph, chunk);
            self.cached_blocks.insert(hash);
            ph = hash;
        }

        // Reset pending state
        self.pending_tail.clear();
        self.pending_parent_hash = INIT_HASH;
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

    /// Store prompt-only token IDs as block hashes. Used by the pre-stream audit path.
    /// Also saves the partial tail (tokens that don't fill a full block) for later
    /// combination with output tokens in store_combined().
    pub fn store(&mut self, token_ids: &[u32]) {
        let full_blocks = token_ids.len() / BLOCK_SIZE;
        let mut ph = INIT_HASH;
        for i in 0..full_blocks {
            let chunk = &token_ids[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE];
            let hash = compute_chain_hash(ph, chunk);
            self.cached_blocks.insert(hash);
            ph = hash;
        }

        // Save tail (partial block) for later combination with output
        let tail_start = full_blocks * BLOCK_SIZE;
        self.pending_tail = token_ids[tail_start..].to_vec();
        self.pending_parent_hash = ph;
    }
}

fn compute_chain_hash(parent_hash: u64, tokens: &[u32]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    parent_hash.hash(&mut hasher);
    tokens.hash(&mut hasher);
    hasher.finish()
}
