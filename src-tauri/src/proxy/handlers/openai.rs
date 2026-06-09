use axum::{extract::State, http::Request, body::Body, response::Response};
use crate::proxy::server::ProxyState;

pub async fn handle_openai(
    State(state): State<ProxyState>,
    req: Request<Body>,
) -> Response<Body> {
    let provider_url = {
        let pm = state.provider_manager.lock().await;
        pm.get_active().await
            .ok()
            .flatten()
            .map(|p| p.api_base_url)
            .unwrap_or_else(|| "https://api.openai.com".to_string())
    };

    let client = reqwest::Client::new();
    let path = req.uri().path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/v1/chat/completions");
    let upstream_url = format!("{}{}", provider_url.trim_end_matches('/'), path);

    let method = req.method().clone();
    let headers = req.headers().clone();
    let body_bytes = axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await.unwrap_or_default();

    let upstream_req = client.request(method, &upstream_url)
        .headers(headers)
        .body(body_bytes)
        .send()
        .await;

    match upstream_req {
        Ok(resp) => {
            let status = resp.status();
            let resp_headers = resp.headers().clone();
            let resp_body = resp.bytes().await.unwrap_or_default();
            let mut response = Response::builder().status(status);
            for (k, v) in resp_headers.iter() {
                response = response.header(k.as_str(), v.as_bytes());
            }
            response.body(axum::body::Body::from(resp_body)).unwrap()
        }
        Err(e) => {
            Response::builder()
                .status(502)
                .body(axum::body::Body::from(format!("Proxy error: {}", e)))
                .unwrap()
        }
    }
}
