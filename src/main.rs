mod api;
mod browser;
mod scraper;

use axum::{
    routing::{post, get},
    Router,
};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/scrape", post(api::scrape_handler))
        .route("/scrape/readable", post(api::scrape_readable_handler))
        .route("/health", get(|| async { "OK" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:19419").await?;
    println!("Server running on http://0.0.0.0:19419");
    axum::serve(listener, app).await?;

    Ok(())
}
