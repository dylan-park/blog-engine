use axum::{Router, routing::get};
use env_logger::Builder;
use log::info;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

mod blog;

#[tokio::main]
async fn main() {
    Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    // Build axum router
    let app = Router::new()
        .route("/", get(blog::render_page))
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
