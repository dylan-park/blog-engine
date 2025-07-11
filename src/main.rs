use crate::utils::health_check::health_check;
use axum::{Router, routing::get};
use env_logger::Builder;
use log::info;
use serde::Deserialize;
use std::{fs, net::SocketAddr, path::Path};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

mod blog;
mod utils;

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    page: Option<usize>,
}

#[tokio::main]
async fn main() {
    Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    fs::create_dir_all(Path::new("posts")).unwrap();

    // Build axum router
    let app = Router::new()
        .route("/", get(blog::render_page))
        .route("/health", get(health_check))
        .nest_service("/static", ServeDir::new("static"))
        .route("/{*path}", get(blog::render_page));

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("Server running on http://{addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
