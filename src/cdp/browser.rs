use anyhow::{Result, anyhow};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DalangBrowser {
    browser: Browser,
    active_page: Arc<Mutex<Option<Page>>>,
}

impl DalangBrowser {
    /// Initialize headless browser and spawn its event handler loop
    pub async fn new() -> Result<Self> {
        let config = BrowserConfig::builder()
            .with_head() // For testing visibility if needed, or omit for true headless
            // .disable_default_args() // Sometimes needed to bypass some detections
            .build()
            .map_err(|e| anyhow!("Failed to build browser config: {:?}", e))?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| anyhow!("Failed to launch browser: {}", e))?;

        // Spawn a background task to drive the websocket events
        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    eprintln!("Browser handler error: {:?}", h);
                    break;
                }
            }
        });

        // Initialize with a blank page
        let page = browser.new_page("about:blank").await?;

        Ok(Self {
            browser,
            active_page: Arc::new(Mutex::new(Some(page))),
        })
    }

    /// Navigate to a specific URL
    pub async fn navigate(&self, url: &str) -> Result<String> {
        let mut page_guard = self.active_page.lock().await;

        // Ensure page exists or create a new one
        let page = match page_guard.as_ref() {
            Some(p) => p.clone(),
            None => {
                let p = self.browser.new_page("about:blank").await?;
                *page_guard = Some(p.clone());
                p
            }
        };

        page.goto(url).await?;
        page.wait_for_navigation().await?;

        Ok(format!("Navigated to {}", url))
    }

    /// Extract inner text representation to avoid huge HTML
    pub async fn extract_dom(&self) -> Result<String> {
        let page_guard = self.active_page.lock().await;
        let page = page_guard
            .as_ref()
            .ok_or_else(|| anyhow!("No active page"))?;

        // Extract plain text to reduce token usage
        let content = page.evaluate("document.body.innerText").await?;

        Ok(content.into_value::<String>()?)
    }

    /// Evaluate raw JS on the active page
    pub async fn evaluate_js(&self, script: &str) -> Result<String> {
        let page_guard = self.active_page.lock().await;
        let page = page_guard
            .as_ref()
            .ok_or_else(|| anyhow!("No active page"))?;

        let result = page.evaluate(script).await?;

        // Serialize whatever generic value comes back
        let json_res = serde_json::to_string(&result.value())?;
        Ok(json_res)
    }
}
