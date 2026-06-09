# TokenAccountant 架构设计方案（最终版）

## 项目目标

构建一个开源工具，帮助用户检测 API 中转站是否虚报 token 用量。支持多模型自动识别、本地 token 计算、Web 可视化监控。

---

## 核心需求（已确认）

1. **多模型支持**：自动识别 qwen、glm、minimax、gemini、gpt、mimo、deepseek 等模型及 API 格式
2. **产品形态**：独立软件，发布到 GitHub，用户安装后启动 Web 前端进行可视化监控和配置
3. **检测粒度**：区分 prefill 阶段的缓存命中/未命中，以及 decode 阶段的 token 统计
4. **缓存检测**：通过 Trie 树 + Token IDs 最长前缀匹配，检测中转站是否虚报缓存命中
5. **持久化**：支持 Resume 场景（跨会话检测缓存命中）

---

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                    TokenAccountant                          │
│  ┌─────────────────┐  ┌──────────────────┐  ┌───────────┐ │
│  │  Web 前端       │  │  后端 API 服务   │  │  数据库    │ │
│  │  (Tauri/React) │◄─┤  (Axum + Rust) │◄─┤  (SQLite) │ │
│  └────────┬────────┘  └────────┬─────────┘  └───────────┘ │
│           │                     │                           │
│           │          ┌──────────┴──────────┐               │
│           │          │  代理拦截层         │               │
│           │          │  (Axum HTTP Proxy) │               │
│           │          └──────────┬──────────┘               │
│           │                     │                           │
│  ┌────────┴────────────────────┴──────────────────┐      │
│  │            Token 计算引擎                        │      │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐ │      │
│  │  │ Tiktoken │  │ TF/Python│  │ Cache    │ │      │
│  │  │ -rs      │  │ Script   │  │ Detector │ │      │
│  │  │(OpenAI/  │  │(Other    │  │(Trie     │ │      │
│  │  │ Qwen/    │  │ Models)  │  │ Tree +  │ │      │
│  │  │ DeepSeek)│  │          │  │ LRU)     │ │      │
│  │  └──────────┘  └──────────┘  └──────────┘ │      │
│  └─────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
                    ┌───────────────┐
                    │  中转站 API   │
                    └───────────────┘
```

### 核心模块

#### 1. 启动时检查并修改 settings.json

**功能**：自动修改 `~/.claude/settings.json`，将 `ANTHROPIC_BASE_URL` 指向本地代理。

**实现**（Rust）：
```rust
fn check_and_modify_settings() {
    let settings_path = dirs::home_dir().unwrap().join(".claude/settings.json");
    let mut settings = read_json(&settings_path);
    
    if settings["env"]["ANTHROPIC_BASE_URL"] != "http://localhost:8080" {
        settings["env"]["ANTHROPIC_BASE_URL"] = json!("http://localhost:8080");
        write_json(&settings_path, &settings);
        println!("已自动修改 settings.json，请重启 Claude Code");
    }
}
```

**注意**：Claude Code 启动时只读一次配置，修改后需要重启。

---

#### 2. HTTP 代理拦截层（Axum）

**功能**：拦截所有通过 TokenAccountant 的 API 请求和响应，转发到真实中转站。

**实现**（Axum + tokio）：
```rust
use axum::{
    body::Body,
    extract::Request,
    handler::Handler,
    http::{Response, StatusCode},
    Router,
};

async fn proxy_handler(req: Request<Body>) -> Response<Body> {
    // 1. 提取请求体
    let (parts, body) = req.into_parts();
    let bytes = body_to_bytes(body).await.unwrap();
    let request_body = String::from_utf8(bytes.to_vec()).unwrap();
    
    // 2. 并行执行：分词计算 + 转发请求
    let tokenizer_handle = tokio::spawn(async move {
        tokenize(&request_body, &model).await
    });
    
    let upstream_handle = tokio::spawn(async move {
        forward_to_upstream(request_body, &upstream_url).await
    });
    
    let (input_tokens, upstream_response) = tokio::join!(tokenizer_handle, upstream_handle);
    
    // 3. 提取响应中的 usage
    let claimed_input = upstream_response.usage.input_tokens;
    let claimed_output = upstream_response.usage.output_tokens;
    let claimed_cached = upstream_response.usage.cache_read_input_tokens;
    
    // 4. 本地计算 output tokens
    let output_text = extract_output_text(&upstream_response);
    let output_tokens = tokenize(&output_text, &model).await;
    
    // 5. 缓存检测（Trie 树最长前缀匹配）
    let (cached_tokens, uncached_tokens) = cache_detector.detect(&input_token_ids).await;
    
    // 6. 对比差异
    let input_diff = claimed_input - input_tokens;
    let output_diff = claimed_output - output_tokens;
    let cache_diff = claimed_cached - cached_tokens;
    
    // 7. 记录到数据库
    save_to_db(RequestRecord {
        model: model.clone(),
        claimed_input,
        claimed_output,
        real_input: input_tokens,
        real_output: output_tokens,
        cached_tokens,
        input_diff,
        output_diff,
        cache_diff,
        is_suspicious: input_diff > input_tokens * 0.05, // > 5%
    }).await;
    
    // 8. 返回响应给用户
    upstream_response
}

// 启动代理服务器
fn start_proxy_server() {
    let app = Router::new()
        .route("/v1/messages", post(proxy_handler))  // Anthropic 格式
        .route("/v1/chat/completions", post(proxy_handler));  // OpenAI 格式
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

**关键特性**：
- ✅ **异步并行**：分词计算和 API 调用并行，不增加延迟
- ✅ **支持流式响应**（SSE/Streaming）
- ✅ **自动识别模型**（根据 URL 路径和请求体中的 `model` 字段）

---

#### 3. 模型识别器

**功能**：根据 HTTP 请求特征自动判断模型类型和 API 格式。

**识别规则**：

| 特征 | 判断结果 |
|------|---------|
| URL 包含 `/v1/messages` | Claude 原生格式 |
| URL 包含 `generativelanguage.googleapis.com` | Gemini 格式 |
| `model` 字段包含 `gpt-` | OpenAI GPT |
| `model` 字段包含 `claude-` | Claude |
| `model` 字段包含 `gemini-` | Gemini |
| `model` 字段包含 `qwen-` | Qwen |
| `model` 字段包含 `glm-` | GLM |
| `model` 字段包含 `abab` (MiniMax) | MiniMax |
| `model` 字段包含 `mimo` | MiMo |
| `model` 字段包含 `deepseek-` | DeepSeek |

**实现**：
```rust
fn detect_model_and_format(req: &Request) -> (String, ApiFormat) {
    let url = req.uri().path();
    let body: Value = serde_json::from_slice(&req.body()).unwrap();
    let model = body["model"].as_str().unwrap_or("");
    
    if url.contains("/v1/messages") {
        ("claude".to_string(), ApiFormat::Anthropic)
    } else if url.contains("generativelanguage.googleapis.com") {
        ("gemini".to_string(), ApiFormat::Gemini)
    } else if model.starts_with("gpt-") {
        ("gpt".to_string(), ApiFormat::OpenAI)
    } else if model.starts_with("claude-") {
        ("claude".to_string(), ApiFormat::Anthropic)
    } else if model.starts_with("qwen-") {
        ("qwen".to_string(), ApiFormat::OpenAI)
    } else if model.starts_with("deepseek-") {
        ("deepseek".to_string(), ApiFormat::OpenAI)
    } else {
        ("unknown".to_string(), ApiFormat::Unknown)
    }
}
```

---

#### 4. Token 计算引擎（模块化设计）

**功能**：根据模型类型，使用对应的分词器本地计算 token 数。

**模块化设计**：
```rust
// src/tokenizer/mod.rs

// 分词器 trait（接口）
pub trait Tokenizer: Send + Sync {
    fn encode(&self, text: &str) -> Vec<u32>;
    fn decode(&self, ids: &[u32]) -> String;
    fn count_tokens(&self, text: &str) -> u32 {
        self.encode(text).len() as u32
    }
}

// GPT 分词器（tiktoken-rs）
pub struct GptTokenizer {
    encoding: tiktoken_rs::Encoding,
}

impl Tokenizer for GptTokenizer {
    fn encode(&self, text: &str) -> Vec<u32> {
        self.encoding.encode(text).into_iter().map(|x| x as u32).collect()
    }
    fn decode(&self, ids: &[u32]) -> String {
        let ids_i64: Vec<i64> = ids.iter().map(|&x| x as i64).collect();
        self.encoding.decode(&ids_i64, true).unwrap()
    }
}

// Qwen 分词器（tiktoken-rs，Qwen2 基于 tiktoken）
pub struct QwenTokenizer {
    encoding: tiktoken_rs::Encoding,
}

impl Tokenizer for QwenTokenizer {
    fn encode(&self, text: &str) -> Vec<u32> {
        self.encoding.encode(text).into_iter().map(|x| x as u32).collect()
    }
    // ...
}

// DeepSeek 分词器（tiktoken-rs，DeepSeek-V3 基于 tiktoken）
pub struct DeepSeekTokenizer {
    encoding: tiktoken_rs::Encoding,
}

// Claude 分词器（近似：字符数 / 1.5）
pub struct ClaudeTokenizer;

impl Tokenizer for ClaudeTokenizer {
    fn encode(&self, text: &str) -> Vec<u32> {
        let char_count = text.chars().count();
        let token_count = (char_count as f64 / 1.5).ceil() as u32;
        (0..token_count).collect()  // 伪实现
    }
    // ...
}

// 其他模型分词器（调用 Python 脚本）
pub struct PythonTokenizer {
    model_name: String,
}

impl Tokenizer for PythonTokenizer {
    fn encode(&self, text: &str) -> Vec<u32> {
        // 调用 Python 脚本
        let output = std::process::Command::new("python3")
            .arg("tokenize.py")
            .arg("--model").arg(&self.model_name)
            .arg("--text").arg(text)
            .output()
            .expect("Failed to execute Python script");
        
        // 解析输出: "123 456 789\n"
        let token_ids: Vec<u32> = output.stdout
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        
        token_ids
    }
    // ...
}

// 分词器工厂
pub struct TokenizerFactory {
    tokenizers: HashMap<String, Box<dyn Tokenizer>>,
}

impl TokenizerFactory {
    pub fn new() -> Self {
        let mut factory = TokenizerFactory {
            tokenizers: HashMap::new(),
        };
        
        // 注册 GPT 分词器
        factory.register("gpt".to_string(), Box::new(GptTokenizer::new()));
        
        // 注册 Qwen 分词器
        factory.register("qwen".to_string(), Box::new(QwenTokenizer::new()));
        
        // 注册 DeepSeek 分词器
        factory.register("deepseek".to_string(), Box::new(DeepSeekTokenizer::new()));
        
        // 注册 Claude 分词器（近似）
        factory.register("claude".to_string(), Box::new(ClaudeTokenizer));
        
        factory
    }
    
    pub fn register(&mut self, name: String, tokenizer: Box<dyn Tokenizer>) {
        self.tokenizers.insert(name, tokenizer);
    }
    
    pub fn get(&self, model: &str) -> &dyn Tokenizer {
        // 根据模型名称自动选择分词器
        if model.starts_with("gpt-") {
            self.tokenizers.get("gpt").unwrap()
        } else if model.starts_with("qwen-") {
            self.tokenizers.get("qwen").unwrap()
        } else if model.starts_with("deepseek-") {
            self.tokenizers.get("deepseek").unwrap()
        } else if model.starts_with("claude-") {
            self.tokenizers.get("claude").unwrap()
        } else {
            // 默认用 GPT 分词器（近似）
            self.tokenizers.get("gpt").unwrap()
        }
    }
}
```

**分词器选择策略**：

| 模型 | 精确方案 | 近似方案 | 推荐 |
|------|---------|---------|------|
| **OpenAI GPT** | `tiktoken-rs`（本地） | 字符数 / 4 | ✅ 本地 |
| **Qwen** | `tiktoken-rs`（本地，Qwen2 基于 tiktoken） | 字符数 / 1.5 | ✅ 本地 |
| **DeepSeek** | `tiktoken-rs`（本地，DeepSeek-V3 基于 tiktoken） | 字符数 / 1.5 | ✅ 本地 |
| **Claude** | `anthropic-sdk` `count_tokens()` API（需要 API Key） | 字符数 / 1.5 | ⚠️ 近似（除非用户提供 API Key） |
| **Gemini** | `google-ai` `count_tokens()` API（需要 API Key） | 字符数 / 4 | ⚠️ 近似（除非用户提供 API Key） |
| **GLM/MiniMax/MiMo** | `tokenizers`（HuggingFace，需要下载模型） | 字符数 / 1.5 | ⚠️ 近似 |

---

#### 5. 缓存检测器（Trie 树 + LRU 驱逐）

**功能**：通过 Token IDs 最长前缀匹配，检测中转站是否虚报缓存命中。

**核心算法**（模仿 vLLM 的 Prefix Caching）：

##### 5.1 数据结构调整（模仿 vLLM）

```rust
// src/cache_detector.rs

// 块结构（模仿 vLLM 的 KVCacheBlock）
struct CacheBlock {
    block_id: u64,               // 块 ID（不可变）
    block_hash: u64,             // 块的哈希值（满块时分配，被驱逐时重置）
    ref_cnt: u32,                // 当前使用该块的请求数量
    prev_free: Option<*mut CacheBlock>,  // 双向链表指针
    next_free: Option<*mut CacheBlock>,
    tokens: Vec<u32>,            // 块内的 Token IDs（最多 Block Size 个）
    is_full: bool,               // 是否满块
    timestamp: Instant,          // 最后访问时间
}

// 缓存管理器（模仿 vLLM 的 KV Cache Manager）
struct CacheManager {
    block_pool: Vec<CacheBlock>,          // 块池（预分配）
    free_queue_head: Option<*mut CacheBlock>,  // Free Queue 头
    free_queue_tail: Option<*mut CacheBlock>,  // Free Queue 尾
    cache_blocks: HashMap<u64, u64>,   // block_hash → block_id
    request_blocks: HashMap<u64, Vec<u64>>, // request_id → [block_ids]
    block_size: usize,                     // 块大小（如 16 tokens/块）
    max_blocks: usize,                    // 最大块数
}

impl CacheManager {
    fn new(block_size: usize, max_blocks: usize) -> Self {
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
        }
    }
    
    // 将 Token IDs 分块
    fn tokenize_to_blocks(&self, token_ids: &[u32]) -> Vec<Vec<u32>> {
        let mut blocks = Vec::new();
        for chunk in token_ids.chunks(self.block_size) {
            blocks.push(chunk.to_vec());
        }
        blocks
    }
    
    // 计算块的哈希（模仿 vLLM）
    fn compute_block_hash(&self, 
        parent_hash: u64, 
        tokens: &[u32],
        extra_hash: u64
    ) -> u64 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&parent_hash.to_le_bytes());
        for &token in tokens {
            hasher.update(&token.to_le_bytes());
        }
        hasher.update(&extra_hash.to_le_bytes());
        let result = hasher.finalize();
        u64::from_le_bytes(result[0..8].try_into().unwrap())
    }
    
    // 分配新块（模仿 vLLM 的 allocate_slots）
    fn allocate_block(&mut self, request_id: u64, tokens: &[u32]) -> u64 {
        // 1. 检查是否命中缓存
        let parent_hash = 0;  // 第一个块的父哈希是 0
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
        let new_block_id = self.pop_from_free_queue();  // LRU 驱逐
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
        
        // 如果是满块，加入 Cache blocks
        if block.is_full {
            self.cache_blocks.insert(block_hash, new_block_id);
        }
        
        // 记录到 request_blocks
        self.request_blocks.entry(request_id).or_insert(Vec::new()).push(new_block_id);
        
        new_block_id
    }
    
    // 从 Free Queue 弹出块（LRU 驱逐）
    fn pop_from_free_queue(&mut self) -> u64 {
        let head = self.free_queue_head.unwrap();
        let block_id = unsafe { (*head).block_id };
        
        // 从链表中移除头部
        self.free_queue_head = unsafe { (*head).next_free };
        if let Some(ref mut next) = self.free_queue_head {
            unsafe { (***next).prev_free = None; }
        }
        
        block_id
    }
    
    // 将块加入 Free Queue 尾部（逆序）
    fn push_to_free_queue(&mut self, block_id: u64) {
        let block = &mut self.block_pool[block_id as usize];
        
        if let Some(ref mut tail) = self.free_queue_tail {
            unsafe {
                (***tail).next_free = Some(Box::new(block as *mut CacheBlock));
                block.prev_free = Some(Box::new(***tail));
            }
            self.free_queue_tail = Some(Box::new(block as *mut CacheBlock));
        } else {
            // 第一个块
            self.free_queue_head = Some(Box::new(block as *mut CacheBlock));
            self.free_queue_tail = Some(Box::new(block as *mut CacheBlock));
        }
    }
    
    // 从 Free Queue 移除（当 ref_cnt 从 0 → 1）
    fn remove_from_free_queue(&mut self, block_id: u64) {
        let block = &mut self.block_pool[block_id as usize];
        
        if let Some(ref mut prev) = block.prev_free {
            unsafe { (***prev).next_free = block.next_free.take(); }
        }
        if let Some(ref mut next) = block.next_free {
            unsafe { (***next).prev_free = block.prev_free.take(); }
        }
        
        block.prev_free = None;
        block.next_free = None;
    }
    
    // 释放请求的所有块（请求完成时调用）
    fn free_request_blocks(&mut self, request_id: u64) {
        if let Some(block_ids) = self.request_blocks.remove(&request_id) {
            for block_id in block_ids.iter().rev() {  // 逆序！
                let block = &mut self.block_pool[*block_id as usize];
                block.ref_cnt -= 1;
                
                if block.ref_cnt == 0 {
                    // 加入 Free Queue 尾部（逆序）
                    self.push_to_free_queue(*block_id);
                }
            }
        }
    }
    
    // 存储到硬盘（服务关闭时）
    fn save_to_disk(&self, path: &str) {
        let mut file = File::create(path).unwrap();
        
        // 序列化 Cache blocks（只存满块）
        for (&block_hash, &block_id) in &self.cache_blocks {
            let block = &self.block_pool[block_id as usize];
            let data = (block_hash, block.tokens.clone());
            bincode::serialize_into(&mut file, &data).unwrap();
        }
    }
    
    // 从硬盘加载（服务启动时）
    fn load_from_disk(&mut self, path: &str) {
        let mut file = File::open(path).unwrap();
        
        loop {
            let data: Result<(u64, Vec<u32>), _> = bincode::deserialize_from(&mut file);
            match data {
                Ok((block_hash, tokens)) => {
                    // 分配新块，恢复数据
                    let request_id = 0;  // 未知，用 0 表示
                    let block_id = self.allocate_block(request_id, &tokens);
                    self.block_pool[block_id as usize].block_hash = block_hash;
                    self.cache_blocks.insert(block_hash, block_id);
                }
                Err(_) => break,
            }
        }
    }
}
```

##### 5.2 缓存检测流程

```rust
impl CacheManager {
    // 检测缓存命中，返回 (命中 token 数, 未命中 token 数)
    fn detect(&mut self, token_ids: &[u32], model: &str, request_id: u64) -> (u32, u32) {
        let blocks = self.tokenize_to_blocks(token_ids);
        let mut total_cached = 0;
        let mut total_uncached = 0;
        
        let mut parent_hash = 0;
        
        for block_tokens in blocks {
            let block_hash = self.compute_block_hash(parent_hash, &block_tokens, 0);
            
            if let Some(&block_id) = self.cache_blocks.get(&block_hash) {
                // 命中缓存
                total_cached += block_tokens.len() as u32;
                parent_hash = block_hash;
            } else {
                // 未命中缓存
                total_uncached += block_tokens.len() as u32;
                let block_id = self.allocate_block(request_id, &block_tokens);
                parent_hash = self.block_pool[block_id as usize].block_hash;
            }
        }
        
        (total_cached, total_uncached)
    }
}
```

**使用示例**：
```rust
let mut cache_manager = CacheManager::new(16, 1000);  // Block Size = 16, Max Blocks = 1000

// 请求 1: "帮我写一个 Python 函数"
let token_ids_1 = vec![123, 456, 789, 234, 567];
let (cached, uncached) = cache_manager.detect(&token_ids_1, "gpt-4", 1);
println!("Cached: {}, Uncached: {}", cached, uncached);
// 输出: Cached: 0, Uncached: 5 （未命中）

// 请求 2: "帮我写一个 Python 函数，要求支持加法"（前缀相同）
let token_ids_2 = vec![123, 456, 789, 234, 567, 890, 123, 456];
let (cached, uncached) = cache_manager.detect(&token_ids_2, "gpt-4", 2);
println!("Cached: {}, Uncached: {}", cached, uncached);
// 输出: Cached: 5, Uncached: 3 （前 5 个 token 命中缓存）
```

---

#### 6. 差异对比器

**功能**：对比中转站返回的 usage 和本地计算的 token 数，标记可疑虚报。

**对比维度**：

```rust
struct TokenDiff {
    claimed_input: u32,   // 中转站声称的 input tokens
    claimed_output: u32,  // 中转站声称的 output tokens
    claimed_cached: u32, // 中转站声称的缓存命中 tokens
    real_input: u32,      // 本地计算的 input tokens
    real_output: u32,     // 本地计算的 output tokens
    cached_tokens: u32,   // 本地检测的缓存命中 tokens
    input_diff: i32,      // 差异 = claimed - real
    output_diff: i32,
    cache_diff: i32,
    is_suspicious: bool,   // 是否可疑（差异 > 5%）
}

fn compare_tokens(claimed: &Usage, real: &TokenDiff) -> TokenDiff {
    let input_diff = claimed.input_tokens as i32 - real.real_input as i32;
    let output_diff = claimed.output_tokens as i32 - real.real_output as i32;
    let cache_diff = claimed.cache_read_input_tokens as i32 - real.cached_tokens as i32;
    
    let is_suspicious = input_diff > real.real_input as i32 * 5 / 100  // > 5%
        || output_diff > real.real_output as i32 * 5 / 100
        || cache_diff > real.cached_tokens as i32 * 5 / 100;
    
    TokenDiff {
        claimed_input: claimed.input_tokens,
        claimed_output: claimed.output_tokens,
        claimed_cached: claimed.cache_read_input_tokens,
        real_input: real.real_input,
        real_output: real.real_output,
        cached_tokens: real.cached_tokens,
        input_diff,
        output_diff,
        cache_diff,
        is_suspicious,
    }
}
```

**判定规则**：
- 如果 `input_diff > 0` → 中转站多报了 input tokens
- 如果 `output_diff > 0` → 中转站多报了 output tokens
- 如果 `cache_diff > 0` → 中转站多报了缓存命中 tokens
- 差异比例 > 5% → 标记为可疑

---

#### 7. 数据存储

**数据库**：SQLite（轻量级，无需额外安装）

**表结构**：

```sql
-- 请求记录表
CREATE TABLE requests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    model TEXT,  -- 模型名称
    api_format TEXT,  -- API 格式 (openai/claude/gemini)
    endpoint TEXT,  -- API 端点
    request_text TEXT,  -- 请求内容（messages/prompt）
    response_text TEXT,  -- 响应内容
    -- 中转站声称的 token 数
    claimed_input_tokens INTEGER,
    claimed_output_tokens INTEGER,
    claimed_cached_tokens INTEGER,
    claimed_cache_creation_tokens INTEGER,
    -- 本地计算的 token 数
    real_input_tokens INTEGER,
    real_output_tokens INTEGER,
    cached_tokens INTEGER,
    -- 差异
    input_diff INTEGER,
    output_diff INTEGER,
    cache_diff INTEGER,
    -- 判定结果
    is_suspicious BOOLEAN,
    suspicion_reason TEXT,
    -- 原始响应（JSON）
    raw_response TEXT
);

-- 统计数据表（按模型/按日期聚合）
CREATE TABLE daily_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date DATE,
    model TEXT,
    api_format TEXT,
    total_requests INTEGER,
    total_claimed_input INTEGER,
    total_real_input INTEGER,
    total_claimed_output INTEGER,
    total_real_output INTEGER,
    total_input_overcharge INTEGER,  -- 总多收 input token
    total_output_overcharge INTEGER,  -- 总多收 output token
    suspicion_count INTEGER  -- 可疑请求数
);
```

**Rust 实现**（使用 `rusqlite`）：
```rust
use rusqlite::{Connection, params};

struct Db {
    conn: Connection,
}

impl Db {
    fn new(path: &str) -> Self {
        let conn = Connection::open(path).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS requests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                model TEXT,
                api_format TEXT,
                endpoint TEXT,
                request_text TEXT,
                response_text TEXT,
                claimed_input_tokens INTEGER,
                claimed_output_tokens INTEGER,
                claimed_cached_tokens INTEGER,
                claimed_cache_creation_tokens INTEGER,
                real_input_tokens INTEGER,
                real_output_tokens INTEGER,
                cached_tokens INTEGER,
                input_diff INTEGER,
                output_diff INTEGER,
                cache_diff INTEGER,
                is_suspicious BOOLEAN,
                suspicion_reason TEXT,
                raw_response TEXT
            )",
            params![],
        ).unwrap();
        
        Db { conn }
    }
    
    fn save_request(&self, record: &RequestRecord) {
        self.conn.execute(
            "INSERT INTO requests (
                model, api_format, endpoint, request_text, response_text,
                claimed_input_tokens, claimed_output_tokens, claimed_cached_tokens, claimed_cache_creation_tokens,
                real_input_tokens, real_output_tokens, cached_tokens,
                input_diff, output_diff, cache_diff,
                is_suspicious, suspicion_reason, raw_response
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8, ?9,
                ?10, ?11, ?12,
                ?13, ?14, ?15,
                ?16, ?17, ?18
            )",
            params![
                record.model,
                record.api_format,
                record.endpoint,
                record.request_text,
                record.response_text,
                record.claimed_input_tokens,
                record.claimed_output_tokens,
                record.claimed_cached_tokens,
                record.claimed_cache_creation_tokens,
                record.real_input_tokens,
                record.real_output_tokens,
                record.cached_tokens,
                record.input_diff,
                record.output_diff,
                record.cache_diff,
                record.is_suspicious,
                record.suspicion_reason,
                record.raw_response,
            ],
        ).unwrap();
    }
}
```

---

#### 8. Web 前端（Tauri + React）

**功能**：提供可视化监控和配置界面。

**技术栈**：
- **前端**：React + TypeScript + Tailwind CSS
- **后端**：Tauri（Rust）+ Axum（Web API）
- **图表**：Chart.js 或 ECharts

**核心页面**：

1. **仪表盘**：
   - 总请求数、总差异、可疑请求数
   - 按模型统计（柱状图）
   - 按日期统计（折线图）

2. **请求列表**：
   - 所有请求记录，可筛选/搜索
   - 标记可疑请求（红色高亮）
   - 点击查看详情（claimed vs real 对比）

3. **差异分析**：
   - 图表展示 claimed vs real token 数对比
   - 缓存命中率分析

4. **配置页面**：
   - 中转站 URL 配置
   - 模型配置（API Key、分词器选择）
   - 告警阈值配置（默认 5%）

**实现**（Tauri + Axum）：
```rust
// src/main.rs

#[tokio::main]
async fn main() {
    // 启动 HTTP 代理服务器
    let proxy_handle = tokio::spawn(async {
        start_proxy_server().await;
    });
    
    // 启动 Web API 服务器（供前端调用）
    let api_handle = tokio::spawn(async {
        start_web_api_server().await;
    });
    
    // 启动 Tauri 前端
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Web API 服务器（供前端调用）
async fn start_web_api_server() {
    let app = Router::new()
        .route("/api/requests", get(get_requests))
        .route("/api/stats", get(get_stats))
        .route("/api/config", get(get_config).post(update_config));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

---

## 技术栈选型（最终确认）

| 层级 | 技术选型 | 理由 |
|------|---------|------|
| **后端框架** | Axum（Rust） | 高性能，异步支持好，适合代理场景 |
| **HTTP 代理** | Axum + tokio | 异步，支持流式响应 |
| **前端框架** | Tauri + React + TypeScript | 跨平台桌面应用，和 CC Switch 一致 |
| **数据库** | SQLite (`rusqlite`) | 轻量级，无需额外安装 |
| **Token 计算** | `tiktoken-rs`（GPT/Qwen/DeepSeek）、Python 脚本（其他模型） | 精确 + 近似 |
| **缓存检测** | 自定义 Trie 树 + LRU 驱逐（模仿 vLLM） | 精确检测缓存命中 |
| **WebSocket** | `tokio-websockets` | 实时推送监控数据 |

---

## 实现优先级

### Phase 1: MVP（最小可行产品）

- [ ] 启动时检查并修改 `settings.json`
- [ ] HTTP 代理拦截层（基础版，支持 OpenAI 格式）
- [ ] 模型识别器（支持 OpenAI GPT、Qwen、DeepSeek）
- [ ] Token 计算引擎（支持 `tiktoken-rs`）
- [ ] 差异对比器（基础对比）
- [ ] SQLite 数据存储
- [ ] 简单的 Web 前端（请求列表 + 差异显示）

### Phase 2: 完整功能

- [ ] 支持 Claude、Gemini 格式
- [ ] 支持 GLM、MiniMax、MiMo
- [ ] 缓存检测（Trie 树 + LRU 驱逐）
- [ ] 可疑请求标记 + 告警
- [ ] 数据统计聚合
- [ ] 持久化（服务关闭时写入硬盘，启动时加载）

### Phase 3: 高级功能

- [ ] 实时 WebSocket 推送
- [ ] 导出报告（CSV/PDF）
- [ ] 多用户支持
- [ ] Docker 部署

---

## 关键挑战

### 1. 分词器精度

**问题**：GLM、MiniMax、MiMo 没有公开的本地分词器，只能用近似方法。

**解决方案**：
- 使用 `transformers`（HuggingFace）加载对应模型的分词器（如果存在）
- 如果不存在，用字符数 / 1.5 作为近似（中文约 1.5 字符/token，英文约 4 字符/token）
- 在报告中标明哪些是近似值，哪些是精确值

### 2. HTTPS 拦截

**问题**：中转站 API 通常走 HTTPS，需要解密才能看到请求/响应内容。

**解决方案**：
- **不需要解密**！用户直接在代码里设置 `base_url="http://localhost:8080/v1"` 即可
- TokenAccountant 作为**反向代理**，接收 HTTP 请求，转发到 HTTPS 的中转站

### 3. 流式响应处理

**问题**：API 响应可能是流式（SSE/Streaming），如何拦截并解析？

**解决方案**：
- Axum 支持流式响应（`StreamBody`）
- 在流式传输过程中，实时计算 token 数（需要缓存所有 chunk，最后统一计算）

### 4. 近似计算的准确性

**问题**：Claude/Gemini 的近似计算可能不准确。

**解决方案**：
- 如果用户提供了 API Key，可以调用官方 `count_tokens()` API 精确计算
- 在配置中增加选项：`use_exact_token_count = true/false`

---

## 项目结构

```
tokenaccountant/
├── Cargo.toml
├── src/
│   ├── main.rs              # 入口
│   ├── proxy.rs             # HTTP 代理（Axum）
│   ├── tokenizer/           # 分词器模块（模块化）
│   │   ├── mod.rs          # 分词器 trait + 工厂
│   │   ├── gpt.rs         # GPT 分词器（tiktoken-rs）
│   │   ├── qwen.rs        # Qwen 分词器（tiktoken-rs）
│   │   ├── deepseek.rs    # DeepSeek 分词器（tiktoken-rs）
│   │   ├── claude.rs      # Claude 分词器（近似）
│   │   └── python.rs      # 其他模型分词器（调用 Python 脚本）
│   ├── cache_detector.rs   # 缓存检测器（Trie 树 + LRU 驱逐）
│   ├── config.rs           # 配置管理（读写 config.toml）
│   ├── db.rs               # 数据存储（SQLite）
│   └── api.rs              # Web API（供前端调用）
├── frontend/               # Tauri 前端（React + TypeScript）
│   ├── src/
│   │   ├── components/
│   │   │   ├── Dashboard.tsx
│   │   │   ├── RequestList.tsx
│   │   │   └── StatsChart.tsx
│   │   └── App.tsx
│   └── package.json
├── python/
│   ├── tokenize.py         # Python 分词器脚本
│   └── requirements.txt
├── config.toml            # 配置文件
└── tests/
```

---

## 下一步行动

1. **确认技术栈**：你是否同意上述技术选型？
2. **确认优先级**：是否先实现 Phase 1（MVP），还是有其他优先级？
3. **确认部署方式**：用户是通过 `cargo install tokenaccountant` 安装，还是下载二进制文件运行？

---

## 附录：各模型 API 格式详细对比

（已保存在之前的调研报告中，此处不再重复）
