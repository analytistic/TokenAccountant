// src/main.rs - 简化版

mod tokenizer;
mod cache_detector;
mod proxy;

use std::sync::Arc;
use tokio;
use anyhow::{Result, anyhow};
use rusqlite::Connection;

use tokenizer::TokenizerFactory;
use cache_detector::CacheManager;

/// 应用程序状态
struct AppState {
    tokenizer_factory: Arc<TokenizerFactory>,
    cache_manager: Arc<tokio::sync::Mutex<CacheManager>>,
    db_conn: Arc<tokio::sync::Mutex<Connection>>,
}

/// 配置
#[derive(Clone)]
struct Config {
    upstream_url: String,
    bind_addr: String,
    cache_ttl_hours: u64,
    suspicion_threshold: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            upstream_url: "https://api.openai.com".to_string(),
            bind_addr: "0.0.0.0:8080".to_string(),
            cache_ttl_hours: 24 * 7,  // 1 周
            suspicion_threshold: 0.05,  // 5%
        }
    }
}

/// 初始化数据库
fn init_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            model TEXT NOT NULL,
            api_format TEXT NOT NULL,
            endpoint TEXT,
            request_text TEXT,
            response_text TEXT,
            claimed_input INTEGER,
            claimed_output INTEGER,
            claimed_cached INTEGER,
            claimed_creation INTEGER,
            real_input INTEGER,
            real_output INTEGER,
            cached_tokens INTEGER,
            input_diff INTEGER,
            output_diff INTEGER,
            cache_diff INTEGER,
            is_suspicious BOOLEAN,
            suspicion_reason TEXT,
            raw_response TEXT
        )",
        [],
    )?;
    
    Ok(conn)
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    tracing::info!("Starting TokenAccountant...");
    
    // 加载配置
    let config = Config::default();
    let config = Arc::new(config);
    
    // 初始化分词器工厂
    let tokenizer_factory = TokenizerFactory::new();
    let tokenizer_factory = Arc::new(tokenizer_factory);
    
    // 初始化缓存管理器
    let cache_manager = CacheManager::new(
        16,          // Block Size = 16
        10000,      // Max Blocks = 10000
        24 * 7,     // Max Age = 1 周
    );
    let cache_manager = Arc::new(tokio::sync::Mutex::new(cache_manager));
    
    // 初始化数据库
    let db_conn = init_db("tokenaccountant.db")?;
    let db_conn = Arc::new(tokio::sync::Mutex::new(db_conn));
    
    // 启动时驱逐过期缓存
    {
        let mut cache_manager = cache_manager.lock().await;
        cache_manager.evict_expired_cache();
    }
    
    // 创建应用状态
    let state = Arc::new(AppState {
        tokenizer_factory,
        cache_manager,
        db_conn,
    });
    
    // 启动代理服务器
    tracing::info!("TokenAccountant started successfully!");
    tracing::info!("Proxy listening on: {}", config.bind_addr);
    tracing::info!("Upstream URL: {}", config.upstream_url);
    
    // TODO: 启动 Axum 服务器
    // start_proxy_server(state).await?;
    
    Ok(())
}
