use axum::{extract::State, http::{HeaderMap, Request}, body::Body, response::Response};
use crate::proxy::server::ProxyState;
use crate::proxy::stream_forwarder::StreamForwarder;
use crate::auditor::model_detector::{detect, extract_request_text, extract_usage};

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
    // --- 0. Extract request metadata before consuming body ---
    let method = req.method().clone();
    let path = req.uri()
        .path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| "/v1/messages".to_string());
    let mut upstream_headers = HeaderMap::new();
    for (k, v) in req.headers().iter() {
        if k.as_str().eq_ignore_ascii_case("host") { continue; }
        upstream_headers.insert(k.clone(), v.clone());
    }

    // --- 1. Read request body ---
    let body_bytes = axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await.unwrap_or_default();
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    // --- 2. Detect model and API format ---
    let detected = detect(&path, &body_str);
    let tokenizer = state.tokenizer_factory.for_model(&detected.model);
    let audit_tokenizer = tokenizer.clone();

    // --- 3. Spawn parallel audit task (input tokens + cache) ---
    let audit_state = state.clone();
    let audit_body = body_str.clone();
    let audit_detected_model = detected.model.clone();
    let audit_fmt = detected.api_format;
    let audit_handle = tokio::spawn(async move {
        let request_text = extract_request_text(&audit_body, audit_fmt);
        let (real_input, real_cached) = if let Some(ref t) = audit_tokenizer {
            let ids = t.encode(&request_text);
            let (cached_hit, _) = audit_state.cache_detector.lock().await.detect(&ids);
            audit_state.cache_detector.lock().await.store(&ids);
            (t.count_tokens(&request_text) as i32, cached_hit as i32)
        } else {
            (0, 0)
        };
        (real_input, real_cached, audit_detected_model, request_text)
    });

    // --- 4. Build upstream URL and headers ---
    let provider_id = {
        let pm = state.provider_manager.lock().await;
        pm.get_active().await.ok().flatten()
            .map(|p| p.id.clone())
            .unwrap_or_default()
    };
    let provider_url = {
        let pm = state.provider_manager.lock().await;
        pm.get_active().await.ok().flatten()
            .map(|p| p.api_base_url)
            .unwrap_or_else(|| "https://api.anthropic.com".to_string())
    };
    let upstream_url = format!("{}{}", provider_url.trim_end_matches('/'), path);
    let client = reqwest::Client::new();
    let mut upstream_req = client.request(method, &upstream_url);
    for (k, v) in upstream_headers.iter() {
        upstream_req = upstream_req.header(k, v);
    }
    let upstream_req = upstream_req.body(body_bytes).send().await;

    // --- 5. Handle response ---
    match upstream_req {
        Ok(resp) => {
            let is_streaming = resp.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.contains("text/event-stream"))
                .unwrap_or(false);

            if is_streaming {
                let forwarder = StreamForwarder::new();
                let response = forwarder.forward_stream(resp).await;

                // --- 6. Post-stream audit ---
                let full_text = forwarder.get_text().await;
                let (claimed_input, claimed_output, claimed_cached) =
                    extract_usage(&full_text, detected.api_format);

                let audit_result = audit_handle.await.unwrap_or((0, 0, String::new(), String::new()));
                let (real_input, real_cached, model_name, _req_text) = audit_result;

                let real_output = if let Some(ref t) = tokenizer {
                    t.count_tokens(&full_text) as i32
                } else { 0 };

                let record = state.diff_comparator.compare(
                    &provider_id, &model_name, detected.api_format.as_str(),
                    &body_str, &full_text,
                    claimed_input, claimed_output, claimed_cached,
                    real_input, real_output, real_cached,
                );
                state.db.lock().await.insert_audit_log(&record).ok();
                {
                    let mut s = state.status.lock().await;
                    s.requests_served += 1;
                }

                response
            } else {
                // Non-streaming: existing buffer path
                let status = resp.status();
                let resp_headers = resp.headers().clone();
                let resp_body = resp.bytes().await.unwrap_or_default();
                let mut response = Response::builder().status(status);
                for (k, v) in resp_headers.iter() {
                    let name = k.as_str();
                    if name.eq_ignore_ascii_case("transfer-encoding")
                        || name.eq_ignore_ascii_case("content-encoding")
                        || name.eq_ignore_ascii_case("connection")
                    { continue; }
                    response = response.header(name, v.as_bytes());
                }
                response.body(axum::body::Body::from(resp_body)).unwrap()
            }
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
