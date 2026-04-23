use super::DalangBrowser;
use anyhow::Result;

impl DalangBrowser {
    // ──────────────────────────────────────────────
    //  NAVIGATION
    // ──────────────────────────────────────────────

    /// Navigate to a specific URL.
    pub async fn navigate(&self, url: &str) -> Result<String> {
        let page = self.active_page()?;
        page.goto(url).await?;
        page.wait_for_navigation().await?;
        Ok(format!("Navigated to {}", url))
    }

    /// Get the current page URL.
    pub async fn get_url(&self) -> Result<String> {
        let page = self.active_page()?;
        let url = page.url().await?.unwrap_or_default();
        Ok(url)
    }

    /// Get the current page title.
    pub async fn get_title(&self) -> Result<String> {
        let page = self.active_page()?;
        let title = page.get_title().await?.unwrap_or_default();
        Ok(title)
    }

    /// Get the full HTML content of the page.
    pub async fn get_html(&self) -> Result<String> {
        let page = self.active_page()?;
        let html = page.content().await?;
        Ok(html)
    }

    /// Navigate back in history.
    pub async fn go_back(&self) -> Result<String> {
        let page = self.active_page()?;
        page.evaluate("history.back()").await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let url = page.url().await?.unwrap_or_default();
        Ok(format!("Navigated back to {}", url))
    }

    /// Navigate forward in history.
    pub async fn go_forward(&self) -> Result<String> {
        let page = self.active_page()?;
        page.evaluate("history.forward()").await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let url = page.url().await?.unwrap_or_default();
        Ok(format!("Navigated forward to {}", url))
    }

    /// Reload the current page.
    pub async fn reload(&self) -> Result<String> {
        let page = self.active_page()?;
        page.reload().await?;
        page.wait_for_navigation().await?;
        let url = page.url().await?.unwrap_or_default();
        Ok(format!("Reloaded {}", url))
    }
}
