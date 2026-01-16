use anyhow::{Context, Result};
use headless_chrome::{Browser, LaunchOptionsBuilder};

pub struct BrowserManager;

impl BrowserManager {
    pub fn create_browser() -> Result<Browser> {
        let launch_options = LaunchOptionsBuilder::default()
            .headless(true)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build launch options: {}", e))?;
        
        Browser::new(launch_options).context("Failed to launch browser")
    }
}
