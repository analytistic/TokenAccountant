mod cli;
mod provider;
mod config;
mod proxy;
mod auditor;
mod storage;
mod api;

use std::sync::Arc;
use tokio::sync::Mutex;
use config::app_config;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cfg = app_config::load_config().expect("Failed to load config");
    tracing::info!("Config loaded, proxy bind: {}", cfg.server.bind_addr);

    let db_path = app_config::db_path();
    std::fs::create_dir_all(app_config::config_dir()).expect("Failed to create config dir");
    let db = storage::database::Database::open(&db_path).expect("Failed to open database");
    let db = Arc::new(Mutex::new(db));

    let provider_manager = provider::manager::ProviderManager::new(db.clone());
    let provider_manager = Arc::new(Mutex::new(provider_manager));

    let proxy_status = Arc::new(Mutex::new(proxy::types::ProxyStatus {
        running: false,
        port: 0,
        uptime_secs: 0,
        requests_served: 0,
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .manage(api::commands::TauriState {
            provider_manager,
            proxy_server: Arc::new(Mutex::new(None)),
            proxy_status,
            proxy_handle: Arc::new(Mutex::new(None)),
            original_env: Arc::new(Mutex::new(None)),
        })
        .setup(|_app| {
            tracing::info!("TokenAccountant started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::commands::list_providers,
            api::commands::get_active_provider,
            api::commands::create_provider,
            api::commands::update_provider,
            api::commands::delete_provider,
            api::commands::switch_provider,
            api::commands::start_proxy,
            api::commands::stop_proxy,
            api::commands::get_proxy_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running TokenAccountant");
}
