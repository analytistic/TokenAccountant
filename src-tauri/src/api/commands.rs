use tauri::State;
use crate::provider::types::{Provider, CreateProviderRequest, UpdateProviderRequest};
use crate::proxy::server::ProxyServer;
use crate::proxy::types::ProxyStatus;
use crate::config::cli_config;

pub struct TauriState {
    pub provider_manager: std::sync::Arc<tokio::sync::Mutex<crate::provider::manager::ProviderManager>>,
    pub proxy_server: std::sync::Arc<tokio::sync::Mutex<Option<ProxyServer>>>,
    pub proxy_status: std::sync::Arc<tokio::sync::Mutex<ProxyStatus>>,
    pub proxy_handle: std::sync::Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    pub original_env: std::sync::Arc<tokio::sync::Mutex<Option<serde_json::Value>>>,
    // Audit components
    pub tokenizer_factory: std::sync::Arc<crate::auditor::tokenizer::TokenizerFactory>,
    pub diff_comparator: std::sync::Arc<crate::auditor::diff_comparator::DiffComparator>,
    pub cache_detector: std::sync::Arc<tokio::sync::Mutex<crate::auditor::cache_detector::CacheDetector>>,
    pub db: std::sync::Arc<tokio::sync::Mutex<crate::storage::database::Database>>,
}

#[tauri::command]
pub async fn list_providers(state: State<'_, TauriState>) -> Result<Vec<Provider>, String> {
    state.provider_manager.lock().await.list().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_active_provider(state: State<'_, TauriState>) -> Result<Option<Provider>, String> {
    state.provider_manager.lock().await.get_active().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_provider(state: State<'_, TauriState>, req: CreateProviderRequest) -> Result<Provider, String> {
    state.provider_manager.lock().await.create(req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_provider(state: State<'_, TauriState>, id: String, req: UpdateProviderRequest) -> Result<Provider, String> {
    state.provider_manager.lock().await.update(&id, req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_provider(state: State<'_, TauriState>, id: String) -> Result<(), String> {
    state.provider_manager.lock().await.delete(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn switch_provider(state: State<'_, TauriState>, id: String) -> Result<Provider, String> {
    state.provider_manager.lock().await.switch_active(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_proxy(state: State<'_, TauriState>, bind_addr: String) -> Result<u16, String> {
    let mut proxy_guard = state.proxy_server.lock().await;
    if let Some(server) = proxy_guard.as_ref() {
        let status = server.state.status.lock().await;
        if status.running {
            return Err("Proxy already running".into());
        }
    }

    // Save original env before overwriting
    let settings_path = cli_config::claude_settings_path();
    if settings_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&settings_path) {
            if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                let orig = settings.get("env").cloned();
                *state.original_env.lock().await = orig;
            }
        }
    }

    // Start proxy server
    let server = ProxyServer::new(
        state.provider_manager.clone(),
        state.tokenizer_factory.clone(),
        state.diff_comparator.clone(),
        state.cache_detector.clone(),
        state.db.clone(),
    );
    let (port, handle) = server.start(&bind_addr).await.map_err(|e| e.to_string())?;

    // Store the handle so we can stop it later
    *state.proxy_handle.lock().await = Some(handle);

    // Update status
    {
        let mut status = server.state.status.lock().await;
        status.running = true;
        status.port = port;
        *state.proxy_status.lock().await = status.clone();
    }

    *proxy_guard = Some(server);

    // Write proxy settings to ~/.claude/settings.json
    let (api_key, upstream_url) = {
        let pm = state.provider_manager.lock().await;
        let active = pm.get_active().await.ok().flatten();
        (
            active.as_ref().map(|p| p.api_key.clone()).unwrap_or_default(),
            active.map(|p| p.api_base_url).unwrap_or_default(),
        )
    };
    tracing::info!("Proxy {} → {}", port, upstream_url);
    cli_config::write_claude_settings(port, &api_key).map_err(|e| e.to_string())?;

    Ok(port)
}

#[tauri::command]
pub async fn stop_proxy(state: State<'_, TauriState>) -> Result<(), String> {
    tracing::info!("Stopping proxy");
    // Stop the Axum server
    if let Some(handle) = state.proxy_handle.lock().await.take() {
        handle.abort();
    }

    // Clear proxy state
    {
        let mut status = state.proxy_status.lock().await;
        status.running = false;
    }
    *state.proxy_server.lock().await = None;

    // Restore Claude settings
    let orig = state.original_env.lock().await.take();
    cli_config::restore_claude_settings(orig.as_ref()).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_proxy_status(state: State<'_, TauriState>) -> Result<ProxyStatus, String> {
    Ok(state.proxy_status.lock().await.clone())
}
