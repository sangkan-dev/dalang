//! Low-level Chromiumoxide / CDP session (`DalangBrowser`).
//!
//! Implementation is split by concern into submodules; each extends [`DalangBrowser`] via `impl` blocks.

use anyhow::{Result, anyhow};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use futures::StreamExt;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

mod cookies;
mod dom_extraction;
mod dom_query;
mod interaction;
mod navigation;
mod network;
mod screenshots;
mod storage;
mod tabs;

/// A single captured network entry (request + optional response).
#[derive(Debug, Clone, Serialize)]
pub struct NetworkEntry {
    pub url: String,
    pub method: String,
    pub status: Option<i64>,
    pub mime_type: Option<String>,
    pub request_headers: HashMap<String, String>,
    pub response_headers: HashMap<String, String>,
    pub timestamp: f64,
}

/// Full-featured headless browser wrapper for the Dalang AI agent.
///
/// The agent can navigate, interact with DOM elements, take screenshots,
/// manage cookies/storage, intercept network traffic, and manage multiple tabs.
pub struct DalangBrowser {
    pub(super) browser: Browser,
    /// All open pages (tabs). Index 0 is always the first tab.
    pub(super) pages: Vec<Page>,
    /// Index of the currently active tab in `pages`.
    pub(super) active_idx: usize,
    /// Captured network entries (populated after `enable_network_log`).
    pub(super) network_log: Arc<Mutex<Vec<NetworkEntry>>>,
    /// Whether network logging is active.
    pub(super) network_logging: bool,
}

impl DalangBrowser {
    // ──────────────────────────────────────────────
    //  LIFECYCLE
    // ──────────────────────────────────────────────

    /// Initialize browser and spawn its event handler loop.
    /// If `headless` is false, the browser window will be visible.
    pub async fn new(headless: bool) -> Result<Self> {
        let mut builder = BrowserConfig::builder();
        builder = builder
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--no-sandbox");
        if !headless {
            builder = builder.with_head();
        }
        let config = builder
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
            pages: vec![page],
            active_idx: 0,
            network_log: Arc::new(Mutex::new(Vec::new())),
            network_logging: false,
        })
    }

    /// Get reference to the active page, or error.
    pub(super) fn active_page(&self) -> Result<&Page> {
        self.pages.get(self.active_idx).ok_or_else(|| {
            anyhow!(
                "No active page (idx {} of {})",
                self.active_idx,
                self.pages.len()
            )
        })
    }
}
