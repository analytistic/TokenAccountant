use axum::{Router, routing::any};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use super::handlers;
use super::types::ProxyStatus;
use crate::auditor::tokenizer::TokenizerFactory;
use crate::auditor::diff_comparator::DiffComparator;
use crate::auditor::cache_detector::CacheDetector;
use crate::storage::database::Database;

#[derive(Clone)]
pub struct ProxyState {
    pub provider_manager: Arc<Mutex<crate::provider::manager::ProviderManager>>,
    pub status: Arc<Mutex<ProxyStatus>>,
    pub tokenizer_factory: Arc<TokenizerFactory>,
    pub diff_comparator: Arc<DiffComparator>,
    pub cache_detector: Arc<Mutex<CacheDetector>>,
    pub db: Arc<Mutex<Database>>,
}

pub struct ProxyServer {
    pub state: ProxyState,
}

impl ProxyServer {
    pub fn new(
        provider_manager: Arc<Mutex<crate::provider::manager::ProviderManager>>,
        tokenizer_factory: Arc<TokenizerFactory>,
        diff_comparator: Arc<DiffComparator>,
        cache_detector: Arc<Mutex<CacheDetector>>,
        db: Arc<Mutex<Database>>,
    ) -> Self {
        let state = ProxyState {
            provider_manager,
            status: Arc::new(Mutex::new(ProxyStatus {
                running: false,
                port: 8080,
                uptime_secs: 0,
                requests_served: 0,
            })),
            tokenizer_factory,
            diff_comparator,
            cache_detector,
            db,
        };
        ProxyServer { state }
    }

    pub async fn start(&self, bind_addr: &str) -> Result<(u16, JoinHandle<()>), anyhow::Error> {
        let app = Router::new()
            .route("/v1/messages", any(handlers::claude::handle_claude))
            .route("/v1/chat/completions", any(handlers::openai::handle_openai))
            .route("/v1/*path", any(fallback_handler))
            .with_state(self.state.clone());

        let listener = TcpListener::bind(bind_addr).await?;
        let port = listener.local_addr()?.port();

        {
            let mut status = self.state.status.lock().await;
            status.running = true;
            status.port = port;
        }

        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        Ok((port, handle))
    }
}

async fn fallback_handler(
    axum::extract::State(state): axum::extract::State<ProxyState>,
    req: axum::http::Request<axum::body::Body>,
) -> axum::response::Response<axum::body::Body> {
    let path = req.uri().path().to_string();
    if path.contains("/v1beta") || path.contains("/gemini") {
        handlers::openai::handle_openai(axum::extract::State(state.clone()), req).await
    } else {
        handlers::openai::handle_openai(axum::extract::State(state), req).await
    }
}
