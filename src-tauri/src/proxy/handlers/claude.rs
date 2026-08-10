use axum::{extract::State, http::{HeaderMap, Request}, body::Body, response::Response};
use tauri::Emitter;
use crate::proxy::server::ProxyState;
use crate::proxy::stream_forwarder::StreamForwarder;
use crate::auditor::model_detector::{detect, extract_usage};

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
    let body_bytes = axum::body::to_bytes(req.into_body(), state.config.server.max_body_bytes as usize).await.unwrap_or_default();
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    // --- 2. Detect model and API format ---
    let detected = detect(&path, &body_str);
    let tokenizer = state.tokenizer_factory.for_model(&detected.model);
    let audit_tokenizer = tokenizer.clone();

    // Extract template params from request body
    let template_params = crate::auditor::tokenizer::extract_template_params(&body_str, &detected.model);

    // --- 3. Spawn parallel audit task (input tokens + cache) ---
    let audit_state = state.clone();
    let audit_body = body_str.clone();
    let audit_detected_model = detected.model.clone();
    let _audit_fmt = detected.api_format;
    let audit_params = template_params.clone();
    let audit_handle = tokio::spawn(async move {
        let (real_input, real_cached, conv, detect_text) = if let Some(ref t) = audit_tokenizer {
            let conv = crate::auditor::message_converter::from_anthropic_body(&audit_body);
            let request_text = t.apply_chat_template_with(&conv, &audit_params);
            let ids = t.encode(&request_text);
            let (cached_hit, _remaining) = audit_state.cache_detector.lock().await.detect(&ids);
            let needs_prefill = ids.len() as i32 - cached_hit as i32;
            (needs_prefill, cached_hit as i32, conv, request_text)
        } else {
            (0, 0, crate::auditor::message_converter::Conversation {
                messages: vec![], tools: vec![], response_format: None,
            }, String::new())
        };
        (real_input, real_cached, audit_detected_model, conv, detect_text)
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
            let captured_response_headers = crate::auditor::response_fingerprint::capture_headers(resp.headers());
            let is_streaming = resp.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.contains("text/event-stream"))
                .unwrap_or(false);

            if is_streaming {
                let forwarder = StreamForwarder::new();
                let response = forwarder.forward_stream(resp).await;

                // --- 6. Post-stream audit in background ---
                // The stream isn't consumed yet — forward_stream returns immediately.
                // Spawn a background task that waits for stream end, then audits.
                let audit_state = state.clone();
                let audit_provider_id = provider_id.clone();
                let audit_fmt = detected.api_format;
                let _audit_model_name = detected.model.clone();
                let audit_body_str = body_str.clone();
                let audit_tokenizer = tokenizer.clone();
                let audit_params = template_params.clone();
                let audit_response_headers = captured_response_headers.clone();
                tokio::spawn(async move {
                    // Wait for the streamed response to be fully consumed
                    while !forwarder.stream_ended.load(std::sync::atomic::Ordering::SeqCst) {
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    }

                    let full_text = forwarder.get_text().await;
                    let (claimed_input, claimed_output, claimed_cached) =
                        extract_usage(&full_text, audit_fmt);

                    let audit_result = audit_handle.await.unwrap_or((0, 0, String::new(), crate::auditor::message_converter::Conversation {
                        messages: vec![], tools: vec![], response_format: None,
                    }, String::new()));
                    let (real_input, real_cached, model_name, conv, detect_text) = audit_result;

                    // Build structured output and count tokens properly
                    let output_msg = forwarder.build_output().await;
                    let real_output = if let Some(ref t) = audit_tokenizer {
                        let output_text = t.render_output(&output_msg);
                        t.count_tokens(&output_text) as i32
                    } else { 0 };

                    // Store: re-render full conversation WITH the new assistant message,
                    // so cached block hashes match the next request's prompt prefix exactly.
                    let mut emit_detect_text = String::new();
                    let mut emit_store_text = String::new();
                    if let Some(ref t) = audit_tokenizer {
                        let mut full_conv = conv.clone();
                        full_conv.messages.push(output_msg.clone());
                        let store_text = t.apply_chat_template_with(&full_conv, &audit_params);
                        let ids = t.encode(&store_text);
                        audit_state.cache_detector.lock().await.store_combined(&ids);

                        // Capture for both DevTraceBuffer and emit
                        emit_detect_text = detect_text.clone();
                        emit_store_text = store_text.clone();
                        audit_state.dev_trace_buffer.lock().await.push(
                            model_name.clone(),
                            detect_text,
                            store_text,
                        );
                    }

                    let record = audit_state.diff_comparator.compare(
                        &audit_provider_id, &model_name, audit_fmt.as_str(),
                        &audit_body_str, &full_text, &audit_response_headers,
                        claimed_input, claimed_output, claimed_cached,
                        real_input, real_output, real_cached,
                    );
                    audit_state.db.lock().await.insert_audit_log(&record).ok();

                    // Emit events for frontend refresh
                    audit_state.app_handle.emit("audit-tick", ()).ok();
                    audit_state.app_handle.emit("dev-trace", serde_json::json!({
                        "model": model_name,
                        "detect_text": emit_detect_text,
                        "store_text": emit_store_text,
                    })).ok();

                    {
                        let mut s = audit_state.status.lock().await;
                        s.requests_served += 1;
                    }
                });

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
