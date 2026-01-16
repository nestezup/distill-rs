mod api;
mod browser;
mod scraper;

use axum::{
    routing::{post, get},
    Router,
};
use std::sync::Arc;
use browser::BrowserManager;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize Browser Manager
    let browser_manager = BrowserManager::new()?;
    let shared_state = Arc::new(browser_manager);

    // Build our application with a route
    let app = Router::new()
        .route("/scrape", post(api::scrape_handler))
        .route("/scrape/readable", post(api::scrape_readable_handler))
        .route("/health", get(|| async { "OK" }))
        .with_state(shared_state);

    // Run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
