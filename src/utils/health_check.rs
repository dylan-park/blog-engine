use crate::utils::error::AppError;
use axum::{http::StatusCode, response::Html};

pub async fn health_check() -> Result<(StatusCode, Html<String>), AppError> {
    Ok((StatusCode::OK, Html("OK".to_owned())))
}
