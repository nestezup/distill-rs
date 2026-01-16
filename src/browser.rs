use anyhow::{Context, Result};
use headless_chrome::{Browser, LaunchOptionsBuilder};
use std::sync::Arc;

#[derive(Clone)]
pub struct BrowserManager {
    browser: Arc<Browser>,
}

impl BrowserManager {
    pub fn new() -> Result<Self> {
        let launch_options = LaunchOptionsBuilder::default()
            .headless(true)
            .idle_browser_timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build launch options: {}", e))?;
        
        // Browser::new(options) launches a new chrome instance
        let browser = Browser::new(launch_options).context("Failed to launch browser")?;
        
        Ok(Self {
            browser: Arc::new(browser),
        })
    }

    pub fn get_browser(&self) -> Arc<Browser> {
        self.browser.clone()
    }
}
