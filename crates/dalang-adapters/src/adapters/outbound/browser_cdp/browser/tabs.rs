use super::DalangBrowser;
use anyhow::{Result, anyhow};
use serde_json;

impl DalangBrowser {
    // ──────────────────────────────────────────────
    //  TAB MANAGEMENT
    // ──────────────────────────────────────────────

    /// Open a new tab, optionally navigating to a URL.
    pub async fn new_tab(&mut self, url: Option<&str>) -> Result<String> {
        let target = url.unwrap_or("about:blank");
        let page = self.browser.new_page(target).await?;
        self.pages.push(page);
        self.active_idx = self.pages.len() - 1;
        Ok(format!("Opened new tab #{} -> {}", self.active_idx, target))
    }

    /// List all open tabs with their index, URL, and title.
    pub async fn list_tabs(&self) -> Result<String> {
        let mut tabs = Vec::new();
        for (i, page) in self.pages.iter().enumerate() {
            let url = page.url().await?.unwrap_or_default();
            let title = page.get_title().await?.unwrap_or_default();
            tabs.push(serde_json::json!({
                "index": i,
                "url": url,
                "title": title,
                "active": i == self.active_idx,
            }));
        }
        Ok(serde_json::to_string_pretty(&tabs)?)
    }

    /// Switch to a specific tab by index.
    pub fn switch_tab(&mut self, index: usize) -> Result<String> {
        if index >= self.pages.len() {
            return Err(anyhow!(
                "Tab index {} out of range (0..{})",
                index,
                self.pages.len()
            ));
        }
        self.active_idx = index;
        Ok(format!("Switched to tab #{}", index))
    }

    /// Close a tab by index (or the active tab if None).
    pub async fn close_tab(&mut self, index: Option<usize>) -> Result<String> {
        let idx = index.unwrap_or(self.active_idx);
        if idx >= self.pages.len() {
            return Err(anyhow!("Tab index {} out of range", idx));
        }
        if self.pages.len() == 1 {
            return Err(anyhow!("Cannot close the last remaining tab"));
        }
        let page = self.pages.remove(idx);
        drop(page);
        if self.active_idx >= self.pages.len() {
            self.active_idx = self.pages.len() - 1;
        }
        Ok(format!(
            "Closed tab #{}, active is now #{}",
            idx, self.active_idx
        ))
    }
}
