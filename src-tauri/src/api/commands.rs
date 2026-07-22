use tauri::State;
use crate::provider::types::{Provider, CreateProviderRequest, UpdateProviderRequest};
use crate::proxy::server::ProxyServer;
use crate::proxy::types::ProxyStatus;
use crate::config::app_config::{self, AppConfig};
use crate::config::cli_config;
use crate::api::types::{
    DashboardData, AuditSummary, AppConfigPayload, ProviderWithStats,
};

pub struct TauriState {
    pub config: std::sync::Arc<tokio::sync::Mutex<AppConfig>>,
    pub provider_manager: std::sync::Arc<tokio::sync::Mutex<crate::provider::manager::ProviderManager>>,
    pub proxy_server: std::sync::Arc<tokio::sync::Mutex<Option<ProxyServer>>>,
    pub proxy_status: std::sync::Arc<tokio::sync::Mutex<ProxyStatus>>,
    pub proxy_handle: std::sync::Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    // Audit components
    pub tokenizer_factory: std::sync::Arc<crate::auditor::tokenizer::TokenizerFactory>,
    pub diff_comparator: std::sync::Arc<crate::auditor::diff_comparator::DiffComparator>,
    pub cache_detector: std::sync::Arc<tokio::sync::Mutex<crate::auditor::cache_detector::CacheDetector>>,
    pub db: std::sync::Arc<tokio::sync::Mutex<crate::storage::database::Database>>,
    pub dev_trace_buffer: std::sync::Arc<tokio::sync::Mutex<crate::auditor::render_inspector::DevTraceBuffer>>,
}

#[tauri::command]
pub async fn list_audit_logs(
    state: State<'_, TauriState>,
    limit: i64,
    offset: i64,
) -> Result<Vec<crate::auditor::diff_comparator::AuditRecord>, String> {
    state.provider_manager.lock().await
        .list_audit_logs(limit, offset, false)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_providers(state: State<'_, TauriState>) -> Result<Vec<ProviderWithStats>, String> {
    let db = state.db.lock().await;
    db.list_providers_with_stats().map_err(|e| e.to_string())
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
    // If proxy is running, stop it first
    let was_running = {
        let status = state.proxy_status.lock().await;
        status.running
    };
    if was_running {
        if let Some(handle) = state.proxy_handle.lock().await.take() {
            handle.abort();
        }
        {
            let mut status = state.proxy_status.lock().await;
            status.running = false;
        }
        *state.proxy_server.lock().await = None;
    }

    // Switch active provider
    let provider = state.provider_manager.lock().await
        .switch_active(&id).await
        .map_err(|e| e.to_string())?;

    // Write new provider's URL + key to Claude settings
    cli_config::write_claude_settings(&provider.api_base_url, &provider.api_key)
        .map_err(|e| e.to_string())?;

    Ok(provider)
}

#[tauri::command]
pub async fn start_proxy(state: State<'_, TauriState>, app_handle: tauri::AppHandle, bind_addr: String) -> Result<u16, String> {
    let mut proxy_guard = state.proxy_server.lock().await;
    if let Some(server) = proxy_guard.as_ref() {
        let status = server.state.status.lock().await;
        if status.running {
            return Err("Proxy already running".into());
        }
    }

    // Start proxy server
    let server = ProxyServer::new(
        state.config.lock().await.clone(),
        state.provider_manager.clone(),
        state.tokenizer_factory.clone(),
        state.diff_comparator.clone(),
        state.cache_detector.clone(),
        state.db.clone(),
        state.dev_trace_buffer.clone(),
        app_handle,
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

    // Write proxy localhost URL to ~/.claude/settings.json (API key stays same)
    let (api_key, upstream_url) = {
        let pm = state.provider_manager.lock().await;
        let active = pm.get_active().await.ok().flatten();
        (
            active.as_ref().map(|p| p.api_key.clone()).unwrap_or_default(),
            active.map(|p| p.api_base_url).unwrap_or_default(),
        )
    };
    tracing::info!("Proxy {} → {}", port, upstream_url);
    let proxy_url = format!("http://localhost:{}", port);
    cli_config::write_claude_settings(&proxy_url, &api_key).map_err(|e| e.to_string())?;

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

    // Write provider URL back to Claude settings
    let (api_key, url) = {
        let pm = state.provider_manager.lock().await;
        let active = pm.get_active().await.ok().flatten();
        (
            active.as_ref().map(|p| p.api_key.clone()).unwrap_or_default(),
            active.map(|p| p.api_base_url).unwrap_or_default(),
        )
    };
    cli_config::write_claude_settings(&url, &api_key).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_proxy_status(state: State<'_, TauriState>) -> Result<ProxyStatus, String> {
    Ok(state.proxy_status.lock().await.clone())
}

#[tauri::command]
pub async fn list_dev_traces(
    state: State<'_, TauriState>,
) -> Result<Vec<crate::auditor::render_inspector::DevTrace>, String> {
    let buffer = state.dev_trace_buffer.lock().await;
    Ok(buffer.list())
}

#[tauri::command]
pub async fn clear_dev_traces(
    state: State<'_, TauriState>,
) -> Result<(), String> {
    let mut buffer = state.dev_trace_buffer.lock().await;
    buffer.clear();
    Ok(())
}

#[tauri::command]
pub async fn get_dashboard_data(state: State<'_, TauriState>) -> Result<DashboardData, String> {
    let db = state.db.lock().await;
    db.get_dashboard_data().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_provider_detail(
    state: State<'_, TauriState>,
    provider_id: String,
    model: String,
) -> Result<AuditSummary, String> {
    let db = state.db.lock().await;
    db.get_provider_detail(&provider_id, &model).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_config(state: State<'_, TauriState>) -> Result<AppConfigPayload, String> {
    let cfg = state.config.lock().await;
    let port: u16 = cfg.server.bind_addr
        .split(':')
        .last()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    Ok(AppConfigPayload {
        proxy_port: port,
        language: cfg.ui.language.clone(),
        auto_start_proxy: cfg.ui.auto_start_proxy,
        dev_mode_enabled: cfg.ui.dev_mode_enabled,
        dev_trace_buffer_size: cfg.ui.dev_trace_buffer_size,
    })
}

#[tauri::command]
pub async fn save_app_config(
    state: State<'_, TauriState>,
    config: AppConfigPayload,
) -> Result<(), String> {
    let mut cfg = state.config.lock().await;
    cfg.server.bind_addr = format!("0.0.0.0:{}", config.proxy_port);
    cfg.ui.language = config.language;
    cfg.ui.auto_start_proxy = config.auto_start_proxy;
    cfg.ui.dev_mode_enabled = config.dev_mode_enabled;
    cfg.ui.dev_trace_buffer_size = config.dev_trace_buffer_size;

    app_config::save_config(&cfg).map_err(|e| e.to_string())?;
    state.dev_trace_buffer.lock().await
        .set_max_entries(config.dev_trace_buffer_size as usize);
    Ok(())
}
