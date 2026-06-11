use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub audit: AuditConfig,
    pub web: WebConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub timeout_seconds: u64,
    #[serde(default = "default_max_body_bytes")]
    pub max_body_bytes: u64,
    #[serde(default = "default_stream_buffer_size")]
    pub stream_buffer_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub suspicion_threshold: f64,
    pub cache_ttl_hours: u64,
    #[serde(default = "default_max_cached_blocks")]
    pub max_cached_blocks: usize,
}

fn default_max_body_bytes() -> u64 { 10 * 1024 * 1024 }
fn default_stream_buffer_size() -> u32 { 64 }
fn default_max_cached_blocks() -> usize { 100_000 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub listen_addr: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                bind_addr: "0.0.0.0:8080".into(),
                timeout_seconds: 120,
                max_body_bytes: 10 * 1024 * 1024,
                stream_buffer_size: 64,
            },
            audit: AuditConfig {
                suspicion_threshold: 0.05,
                cache_ttl_hours: 168,
                max_cached_blocks: 100_000,
            },
            web: WebConfig {
                listen_addr: "127.0.0.1:9090".into(),
            },
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".tokenaccountant"))
        .unwrap_or_else(|| PathBuf::from(".tokenaccountant"))
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn db_path() -> PathBuf {
    config_dir().join("data.db")
}

pub fn load_config() -> Result<AppConfig> {
    let path = config_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    } else {
        let config = AppConfig::default();
        std::fs::create_dir_all(config_dir())?;
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&path, content)?;
        Ok(config)
    }
}
