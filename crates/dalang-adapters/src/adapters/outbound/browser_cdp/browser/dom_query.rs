use super::DalangBrowser;
use anyhow::{Result, anyhow};
use serde_json;

impl DalangBrowser {
    // ──────────────────────────────────────────────
    //  DOM QUERY
    // ──────────────────────────────────────────────

    /// Query a single element by CSS selector and return its info as JSON.
    pub async fn query_selector(&self, selector: &str) -> Result<String> {
        let page = self.active_page()?;
        let el = page
            .find_element(selector)
            .await
            .map_err(|e| anyhow!("Element not found '{}': {}", selector, e))?;
        let desc = el.description().await?;
        let text = el.inner_text().await?.unwrap_or_default();
        let outer = el.outer_html().await?.unwrap_or_default();
        let outer_trunc = if outer.len() > 500 {
            format!("{}...(truncated)", &outer[..500])
        } else {
            outer
        };
        let info = serde_json::json!({
            "tag": desc.node_name,
            "text": text,
            "html": outer_trunc,
        });
        Ok(serde_json::to_string_pretty(&info)?)
    }

    /// Query all elements matching a CSS selector (capped at `limit`).
    pub async fn query_selector_all(&self, selector: &str, limit: usize) -> Result<String> {
        let page = self.active_page()?;
        let elements = page
            .find_elements(selector)
            .await
            .map_err(|e| anyhow!("Elements not found '{}': {}", selector, e))?;
        let mut results = Vec::new();
        for (i, el) in elements.into_iter().enumerate() {
            if i >= limit {
                break;
            }
            let text = el.inner_text().await?.unwrap_or_default();
            let text_trunc = if text.len() > 200 {
                format!("{}...", &text[..200])
            } else {
                text
            };
            let href = el.attribute("href").await?.unwrap_or_default();
            let id = el.attribute("id").await?.unwrap_or_default();
            let class = el.attribute("class").await?.unwrap_or_default();
            let name = el.attribute("name").await?.unwrap_or_default();
            let el_type = el.attribute("type").await?.unwrap_or_default();
            let desc = el.description().await?;
            results.push(serde_json::json!({
                "index": i,
                "tag": desc.node_name,
                "text": text_trunc,
                "href": href,
                "id": id,
                "class": class,
                "name": name,
                "type": el_type,
            }));
        }
        let out = serde_json::json!({
            "count": results.len(),
            "elements": results,
        });
        Ok(serde_json::to_string_pretty(&out)?)
    }

    /// Get a specific attribute of an element.
    pub async fn get_attribute(&self, selector: &str, attribute: &str) -> Result<String> {
        let page = self.active_page()?;
        let el = page
            .find_element(selector)
            .await
            .map_err(|e| anyhow!("Element not found '{}': {}", selector, e))?;
        let val = el.attribute(attribute).await?;
        Ok(val.unwrap_or_default())
    }

    /// Wait for an element matching a CSS selector to appear (polls with timeout).
    pub async fn wait_for_selector(&self, selector: &str, timeout_ms: u64) -> Result<String> {
        let page = self.active_page()?;
        let start = tokio::time::Instant::now();
        let timeout = tokio::time::Duration::from_millis(timeout_ms);
        loop {
            if page.find_element(selector.to_string()).await.is_ok() {
                return Ok(format!(
                    "Element '{}' found after {}ms",
                    selector,
                    start.elapsed().as_millis()
                ));
            }
            if start.elapsed() > timeout {
                return Err(anyhow!(
                    "Timeout waiting for '{}' after {}ms",
                    selector,
                    timeout_ms
                ));
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    }
}
