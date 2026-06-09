use axum::response::Response;
use axum::body::Body;

pub async fn handle_response(resp: reqwest::Response) -> Response<Body> {
    let status = resp.status();
    let headers = resp.headers().clone();
    let body = resp.bytes().await.unwrap_or_default();

    let mut response = Response::builder().status(status);
    for (k, v) in headers.iter() {
        let name = k.as_str();
        if name == "transfer-encoding" || name == "connection" {
            continue;
        }
        response = response.header(name, v.as_bytes());
    }
    response.body(Body::from(body)).unwrap()
}
