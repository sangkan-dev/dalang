use super::DalangBrowser;
use anyhow::Result;
use serde_json;

impl DalangBrowser {
    // ──────────────────────────────────────────────
    //  DOM EXTRACTION
    // ──────────────────────────────────────────────

    /// Extract inner text representation to avoid huge HTML.
    pub async fn extract_dom(&self) -> Result<String> {
        let page = self.active_page()?;
        let content = page.evaluate("document.body.innerText").await?;
        Ok(content.into_value::<String>()?)
    }

    /// Evaluate raw JS on the active page.
    pub async fn evaluate_js(&self, script: &str) -> Result<String> {
        let page = self.active_page()?;
        let result = page.evaluate(script).await?;
        let json_res = serde_json::to_string(&result.value())?;
        Ok(json_res)
    }
}
