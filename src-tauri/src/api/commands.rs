use tauri::State;
use crate::provider::types::{Provider, CreateProviderRequest, UpdateProviderRequest};
use crate::proxy::server::ProxyServer;
use crate::proxy::types::ProxyStatus;

pub struct TauriState {
    pub provider_manager: std::sync::Arc<tokio::sync::Mutex<crate::provider::manager::ProviderManager>>,
    pub proxy_server: std::sync::Arc<tokio::sync::Mutex<Option<ProxyServer>>>,
    pub proxy_status: std::sync::Arc<tokio::sync::Mutex<ProxyStatus>>,
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

    let pm = state.provider_manager.clone();
    let server = ProxyServer::new(pm);
    let port = server.start(&bind_addr).await.map_err(|e| e.to_string())?;

    {
        let mut status = server.state.status.lock().await;
        status.running = true;
        status.port = port;
        *state.proxy_status.lock().await = status.clone();
    }

    *proxy_guard = Some(server);
    Ok(port)
}

#[tauri::command]
pub async fn get_proxy_status(state: State<'_, TauriState>) -> Result<ProxyStatus, String> {
    Ok(state.proxy_status.lock().await.clone())
}
