use axum::{http::StatusCode, response::Html};

pub async fn health_check() -> (StatusCode, Html<String>) {
    (StatusCode::OK, Html("OK".to_owned()))
}
