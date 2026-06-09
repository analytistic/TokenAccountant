use axum::body::Body;
use axum::response::Response;
use bytes::Bytes;
use futures_util::StreamExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;

pub struct StreamForwarder {
    pub accumulated_text: Arc<Mutex<String>>,
    pub stream_ended: Arc<AtomicBool>,
    pub on_chunk: Option<Arc<dyn Fn(&[u8]) + Send + Sync>>,
}

impl StreamForwarder {
    pub fn new() -> Self {
        StreamForwarder {
            accumulated_text: Arc::new(Mutex::new(String::new())),
            stream_ended: Arc::new(AtomicBool::new(false)),
            on_chunk: None,
        }
    }

    /// Set per-chunk callback (Pattern B entry point for real-time token counting).
    pub fn with_on_chunk(mut self, cb: Arc<dyn Fn(&[u8]) + Send + Sync>) -> Self {
        self.on_chunk = Some(cb);
        self
    }

    /// Forward SSE response to client while accumulating text for post-stream audit.
    pub async fn forward_stream(&self, upstream_resp: reqwest::Response) -> Response<Body> {
        let status = upstream_resp.status();
        let mut rb = Response::builder().status(status);
        for (k, v) in upstream_resp.headers().iter() {
            let name = k.as_str();
            if name.eq_ignore_ascii_case("transfer-encoding")
                || name.eq_ignore_ascii_case("content-encoding")
                || name.eq_ignore_ascii_case("connection")
            {
                continue;
            }
            rb = rb.header(name, v.as_bytes());
        }

        let (tx, rx) = mpsc::channel::<Bytes>(64);
        let acc = self.accumulated_text.clone();
        let ended = self.stream_ended.clone();
        let on_chunk = self.on_chunk.clone();

        tokio::spawn(async move {
            let mut stream = upstream_resp.bytes_stream();
            while let Some(item) = stream.next().await {
                match item {
                    Ok(chunk) => {
                        if let Ok(text) = std::str::from_utf8(&chunk) {
                            acc.lock().await.push_str(text);
                        } else {
                            // Chunk boundary may split a multi-byte UTF-8 sequence;
                            // use lossy conversion rather than silent data loss.
                            tracing::trace!("Stream chunk split UTF-8 boundary, using lossy conversion");
                            let text = String::from_utf8_lossy(&chunk);
                            acc.lock().await.push_str(&text);
                        }
                        if let Some(ref cb) = on_chunk {
                            cb(&chunk);
                        }
                        if tx.send(chunk).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Stream chunk error: {}", e);
                        break;
                    }
                }
            }
            ended.store(true, Ordering::SeqCst);
        });

        let body = Body::from_stream(ReceiverStream::new(rx).map(|b| Ok::<_, axum::Error>(b)));
        rb.body(body).expect("valid response builder after setting status and headers")
    }

    pub async fn get_text(&self) -> String {
        self.accumulated_text.lock().await.clone()
    }
}
