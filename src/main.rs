use crate::utils::{health_check, memory_manager};
use anyhow::{Context, Result};
use axum::{Router, routing::get, serve};
use serde::Deserialize;
use std::{fs, net::SocketAddr, path::Path};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::info;

mod blog;
mod utils;

#[derive(Debug, Deserialize, Clone)]
pub struct PageQuery {
    page: Option<usize>,
    category: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup Logging
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    // Create 'posts' directory if not already created
    fs::create_dir_all(Path::new("posts")).context("Failed to create 'posts' directory")?;

    memory_manager::build_frontmatter_index()
        .await
        .context("Failed to build frontmatter index")?;
    memory_manager::setup_file_watcher()
        .await
        .context("Failed to setup file watcher")?;

    // Build axum router
    let app = Router::new()
        .route("/", get(blog::render_page))
        .route("/category", get(blog::render_category_page))
        .route("/health", get(health_check::health_check))
        .nest_service("/static", ServeDir::new("static"))
        .route("/{*path}", get(blog::render_page));

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr)
        .await
        .context("Failed to bind TCP listener")?;
    info!("Server running on http://{addr}");
    serve(listener, app.into_make_service())
        .await
        .context("Server error")?;

    Ok(())
}
