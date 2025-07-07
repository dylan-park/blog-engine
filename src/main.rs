use axum::{Router, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

mod blog;

#[tokio::main]
async fn main() {
    // Build axum router
    let app = Router::new()
        .route("/", get(blog::render_page))
        .route("/favicon.ico", get(blog::ignore_favicon))
        .nest_service("/static", ServeDir::new("static"))
        .route("/{*path}", get(blog::render_page));

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server running at http://{}", addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
