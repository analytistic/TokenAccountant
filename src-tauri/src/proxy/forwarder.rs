use axum::body::Body;
use http::Request;

pub struct RequestForwarder {
    client: reqwest::Client,
}

impl RequestForwarder {
    pub fn new() -> Self {
        RequestForwarder {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub async fn forward(
        &self,
        upstream_base: &str,
        req: Request<Body>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let path = req.uri().path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("");
        let url = format!("{}{}", upstream_base.trim_end_matches('/'), path);
        let method = req.method().clone();
        let headers = req.headers().clone();
        let body_bytes = axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024)
            .await
            .unwrap_or_default();

        let mut upstream_req = self.client.request(method, &url)
            .headers(headers);
        if !body_bytes.is_empty() {
            upstream_req = upstream_req.body(body_bytes);
        }
        upstream_req.send().await
    }
}
