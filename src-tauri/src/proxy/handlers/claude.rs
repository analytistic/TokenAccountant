use axum::{extract::State, http::Request, body::Body, response::Response};
use crate::proxy::server::ProxyState;

pub async fn handle_claude(
    State(state): State<ProxyState>,
    req: Request<Body>,
) -> Response<Body> {
    forward_with_audit(state, req, "claude").await
}

async fn forward_with_audit(
    state: ProxyState,
    req: Request<Body>,
    _api_type: &str,
) -> Response<Body> {
    let provider_url = {
        let pm = state.provider_manager.lock().await;
        pm.get_active().await
            .ok()
            .flatten()
            .map(|p| p.api_base_url)
            .unwrap_or_else(|| "https://api.anthropic.com".to_string())
    };

    let client = reqwest::Client::new();
    let path = req.uri().path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| "/v1/messages".to_string());
    let upstream_url = format!("{}{}", provider_url.trim_end_matches('/'), path);

    let method = req.method().clone();
    let mut upstream_req = client.request(method, &upstream_url);
    for (k, v) in req.headers().iter() {
        if k.as_str().eq_ignore_ascii_case("host") {
            continue;
        }
        upstream_req = upstream_req.header(k, v);
    }

    let body_bytes = axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await.unwrap_or_default();
    let upstream_req = upstream_req.body(body_bytes.clone()).send().await;

    match upstream_req {
        Ok(resp) => {
            let status = resp.status();
            tracing::info!(target: "proxy", "<-- {} ({} bytes)", status, resp.content_length().unwrap_or(0));
            let resp_headers = resp.headers().clone();
            let resp_body = resp.bytes().await.unwrap_or_default();

            let mut response = Response::builder()
                .status(status);
            for (k, v) in resp_headers.iter() {
                let name = k.as_str();
                if name.eq_ignore_ascii_case("transfer-encoding")
                    || name.eq_ignore_ascii_case("content-encoding")
                    || name.eq_ignore_ascii_case("connection")
                {
                    continue;
                }
                response = response.header(name, v.as_bytes());
            }
            response.body(axum::body::Body::from(resp_body)).unwrap()
        }
        Err(e) => {
            tracing::error!("Upstream request failed: {:?}", e);
            Response::builder()
                .status(502)
                .body(axum::body::Body::from(format!("Proxy error: {}", e)))
                .unwrap()
        }
    }
}
