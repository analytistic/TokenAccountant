// src/cache_detector.rs

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 缓存块（简化版，避免使用指针）
#[derive(Debug, Clone)]
struct CacheBlock {
    block_id: u64,                   // 块 ID
    block_hash: u64,                 // 块的哈希值
    ref_cnt: u32,                    // 引用计数
    prev_free: Option<usize>,      // 双向链表（前）
    next_free: Option<usize>,      // 双向链表（后）
    tokens: Vec<u32>,                // 块内的 Token IDs
    is_full: bool,                   // 是否满块
    timestamp: Instant,              // 最后访问时间
}

/// 缓存管理器（简化版）
#[derive(Debug)]
pub struct CacheManager {
    block_pool: Vec<CacheBlock>,          // 块池
    free_queue_head: Option<usize>,     // Free Queue 头
    free_queue_tail: Option<usize>,     // Free Queue 尾
    cache_blocks: HashMap<u64, u64>,   // block_hash → block_id
    request_blocks: HashMap<u64, Vec<u64>>, // request_id → [block_ids]
    block_size: usize,                     // 块大小
    max_blocks: usize,                    // 最大块数
    max_age: Duration,                   // 最大时效
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new(block_size: usize, max_blocks: usize, max_age_hours: u64) -> Self {
        let mut block_pool = Vec::with_capacity(max_blocks);
        
        // 预分配所有块
        for i in 0..max_blocks {
            block_pool.push(CacheBlock {
                block_id: i as u64,
                block_hash: 0,
                ref_cnt: 0,
                prev_free: None,
                next_free: None,
                tokens: Vec::new(),
                is_full: false,
                timestamp: Instant::now(),
            });
        }
        
        CacheManager {
            block_pool,
            free_queue_head: None,
            free_queue_tail: None,
            cache_blocks: HashMap::new(),
            request_blocks: HashMap::new(),
            block_size,
            max_blocks,
            max_age: Duration::from_secs(max_age_hours * 3600),
        }
    }
    
    /// 将 Token IDs 分块
    fn tokenize_to_blocks(&self, token_ids: &[u32]) -> Vec<Vec<u32>> {
        let mut blocks = Vec::new();
        for chunk in token_ids.chunks(self.block_size) {
            blocks.push(chunk.to_vec());
        }
        blocks
    }
    
    /// 计算块的哈希
    fn compute_block_hash(&self, parent_hash: u64, tokens: &[u32], extra_hash: u64) -> u64 {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&parent_hash.to_le_bytes());
        for &token in tokens {
            hasher.update(&token.to_le_bytes());
        }
        hasher.update(&extra_hash.to_le_bytes());
        let result = hasher.finalize();
        u64::from_le_bytes(result[0..8].try_into().unwrap())
    }
    
    /// 分配新块
    fn allocate_block(&mut self, request_id: u64, tokens: &[u32]) -> u64 {
        // 1. 检查是否命中缓存
        let parent_hash = 0;
        let block_hash = self.compute_block_hash(parent_hash, tokens, 0);
        
        if let Some(&block_id) = self.cache_blocks.get(&block_hash) {
            // 命中缓存！
            let block = &mut self.block_pool[block_id as usize];
            block.ref_cnt += 1;
            
            // 如果 ref_cnt 从 0 → 1，从 Free Queue 移除
            if block.ref_cnt == 1 {
                self.remove_from_free_queue(block_id);
            }
            
            return block_id;
        }
        
        // 2. 未命中，需要分配新块
        let new_block_id = self.pop_from_free_queue();
        let block = &mut self.block_pool[new_block_id as usize];
        
        // 如果该块是被缓存的，驱逐它
        if block.block_hash != 0 {
            self.cache_blocks.remove(&block.block_hash);
            block.block_hash = 0;
        }
        
        // 设置新块的数据
        block.tokens = tokens.to_vec();
        block.block_hash = block_hash;
        block.ref_cnt = 1;
        block.is_full = tokens.len() == self.block_size;
        block.timestamp = Instant::now();
        
        // 如果是满块，加入 cache_blocks
        if block.is_full {
            self.cache_blocks.insert(block_hash, new_block_id);
        }
        
        // 记录到 request_blocks
        self.request_blocks.entry(request_id).or_insert(Vec::new()).push(new_block_id);
        
        new_block_id
    }
    
    /// 从 Free Queue 弹出块（LRU 驱逐）
    fn pop_from_free_queue(&mut self) -> u64 {
        if let Some(head_idx) = self.free_queue_head {
            let block_id = self.block_pool[head_idx].block_id;
            
            // 从链表中移除头部
            self.free_queue_head = self.block_pool[head_idx].next_free;
            if let Some(ref mut next_idx) = self.free_queue_head {
                self.block_pool[*next_idx].prev_free = None;
            } else {
                // 队列为空
                self.free_queue_tail = None;
            }
            
            block_id
        } else {
            panic!("Free queue is empty!");
        }
    }
    
    /// 将块加入 Free Queue 尾部
    fn push_to_free_queue(&mut self, block_id: u64) {
        let idx = block_id as usize;
        
        if let Some(tail_idx) = self.free_queue_tail {
            self.block_pool[tail_idx].next_free = Some(idx);
            self.block_pool[idx].prev_free = Some(tail_idx);
            self.free_queue_tail = Some(idx);
        } else {
            // 第一个块
            self.free_queue_head = Some(idx);
            self.free_queue_tail = Some(idx);
        }
    }
    
    /// 从 Free Queue 移除
    fn remove_from_free_queue(&mut self, block_id: u64) {
        let idx = block_id as usize;
        let block = &self.block_pool[idx];
        
        let prev = block.prev_free;
        let next = block.next_free;
        
        // 更新前驱
        if let Some(prev_idx) = prev {
            self.block_pool[prev_idx].next_free = next;
        } else {
            self.free_queue_head = next;
        }
        
        // 更新后继
        if let Some(next_idx) = next {
            self.block_pool[next_idx].prev_free = prev;
        } else {
            self.free_queue_tail = prev;
        }
        
        // 清空当前块的指针
        let block = &mut self.block_pool[idx];
        block.prev_free = None;
        block.next_free = None;
    }
    
    /// 释放请求的所有块
    pub fn free_request_blocks(&mut self, request_id: u64) {
        if let Some(block_ids) = self.request_blocks.remove(&request_id) {
            for &block_id in block_ids.iter().rev() {
                let block = &mut self.block_pool[block_id as usize];
                block.ref_cnt -= 1;
                
                if block.ref_cnt == 0 {
                    self.push_to_free_queue(block_id);
                }
            }
        }
    }
    
    /// 检测缓存命中
    pub fn detect(
        &mut self, 
        token_ids: &[u32], 
        _model: &str, 
        request_id: u64,
        _system_prompt_len: usize
    ) -> (u32, u32) {
        let blocks = self.tokenize_to_blocks(token_ids);
        let mut total_cached = 0u32;
        let mut total_uncached = 0u32;
        
        let mut parent_hash = 0u64;
        
        for block_tokens in &blocks {
            let block_hash = self.compute_block_hash(parent_hash, block_tokens, 0);
            
            if let Some(&block_id) = self.cache_blocks.get(&block_hash) {
                // 命中缓存
                total_cached += block_tokens.len() as u32;
                parent_hash = block_hash;
            } else {
                // 未命中缓存
                total_uncached += block_tokens.len() as u32;
                let block_id = self.allocate_block(request_id, block_tokens);
                parent_hash = self.block_pool[block_id as usize].block_hash;
            }
        }
        
        (total_cached, total_uncached)
    }
    
    /// 驱逐过期的缓存
    pub fn evict_expired_cache(&mut self) {
        let now = Instant::now();
        
        // 找出所有过期的块
        let expired_hashes: Vec<u64> = self.cache_blocks
            .iter()
            .filter(|(&hash, &block_id)| {
                let block = &self.block_pool[block_id as usize];
                now.duration_since(block.timestamp) > self.max_age
            })
            .map(|(&hash, &&block_id)| hash)
            .collect();
        
        // 驱逐
        for hash in expired_hashes {
            if let Some(block_id) = self.cache_blocks.remove(&hash) {
                self.block_pool[block_id as usize].block_hash = 0;
                self.push_to_free_queue(block_id);
            }
        }
    }
    
    /// 驱逐只命中 system prompt 的缓存
    fn evict_stale_cache(&mut self, _system_prompt_len: usize) {
        // 简化版：暂时不实现
    }
    
    /// 存储到硬盘
    pub fn save_to_disk(&self, _path: &str) -> anyhow::Result<()> {
        // TODO: 实现序列化
        Ok(())
    }
    
    /// 从硬盘加载
    pub fn load_from_disk(&mut self, _path: &str) -> anyhow::Result<()> {
        // TODO: 实现反序列化
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new() {
        let manager = CacheManager::new(16, 1000, 24 * 7);
        assert_eq!(manager.block_size, 16);
        assert_eq!(manager.max_blocks, 1000);
    }
    
    #[test]
    fn test_detect_no_cache() {
        let mut manager = CacheManager::new(16, 1000, 24 * 7);
        let token_ids = vec![123, 456, 789, 234, 567];
        let (cached, uncached) = manager.detect(&token_ids, "gpt-4", 1, 0);
        assert_eq!(cached, 0);
        assert_eq!(uncached, 5);
    }
    
    #[test]
    fn test_detect_with_cache() {
        let mut manager = CacheManager::new(16, 1000, 24 * 7);
        
        // 请求 1：未命中
        let token_ids_1 = vec![123, 456, 789, 234, 567];
        let (cached, uncached) = manager.detect(&token_ids_1, "gpt-4", 1, 0);
        assert_eq!(cached, 0);
        assert_eq!(uncached, 5);
        
        // 请求 2：前缀相同，应该命中 5 个 token
        let token_ids_2 = vec![123, 456, 789, 234, 567, 890, 123, 456];
        let (cached, uncached) = manager.detect(&token_ids_2, "gpt-4", 2, 0);
        assert_eq!(cached, 5);
        assert_eq!(uncached, 3);
    }
}
