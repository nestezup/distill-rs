use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::browser::BrowserManager;
use crate::scraper::Scraper;

#[derive(Deserialize)]
pub struct ScrapeRequest {
    pub url: String,
}

#[derive(Serialize)]
pub struct ScrapeResponse {
    pub url: String,
    pub title: String,
    pub markdown: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn scrape_handler(
    State(browser_manager): State<Arc<BrowserManager>>,
    Json(payload): Json<ScrapeRequest>,
) -> impl IntoResponse {
    let browser = browser_manager.get_browser();
    
    match Scraper::scrape_url(browser, &payload.url) {
        Ok((markdown, title)) => {
            // 1. Show in Terminal
            println!("\n=== Scraped Content: {} ===\n{}\n============================\n", title, markdown);

            // 2. Save to File
            let _ = tokio::fs::create_dir_all("outputs").await;
            let safe_title = title.replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '-', "");
            let filename = format!("outputs/{}.md", safe_title.trim().replace(' ', "_"));
            
            if let Err(e) = tokio::fs::write(&filename, &markdown).await {
                eprintln!("Failed to write file: {}", e);
            } else {
                println!("Saved content to: {}", filename);
            }

            let response = ScrapeResponse {
                url: payload.url,
                title,
                markdown,
            };
            (StatusCode::OK, Json(response)).into_response()
        },
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

pub async fn scrape_readable_handler(
    State(browser_manager): State<Arc<BrowserManager>>,
    Json(payload): Json<ScrapeRequest>,
) -> impl IntoResponse {
    let browser = browser_manager.get_browser();

    match Scraper::scrape_readable_url(browser, &payload.url) {
        Ok((markdown, title, html)) => {
            // 1. Show in Terminal
            println!("\n=== READABLE Content: {} ===\n{}\n============================\n", title, &markdown.chars().take(500).collect::<String>());

            // 2. Save to Files (MD and HTML)
            let _ = tokio::fs::create_dir_all("outputs").await;
            let safe_title = title.replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '-', "");
            let base_name = safe_title.trim().replace(' ', "_");

            let md_filename = format!("outputs/{}_readable.md", base_name);
            let html_filename = format!("outputs/{}_readable.html", base_name);

            if let Err(e) = tokio::fs::write(&md_filename, &markdown).await {
                eprintln!("Failed to write MD file: {}", e);
            } else {
                println!("Saved MD to: {}", md_filename);
            }

            if let Err(e) = tokio::fs::write(&html_filename, &html).await {
                eprintln!("Failed to write HTML file: {}", e);
            } else {
                println!("Saved HTML to: {}", html_filename);
            }

            let response = ScrapeResponse {
                url: payload.url,
                title,
                markdown,
            };
            (StatusCode::OK, Json(response)).into_response()
        },
        Err(e) => {
            let error_response = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}
