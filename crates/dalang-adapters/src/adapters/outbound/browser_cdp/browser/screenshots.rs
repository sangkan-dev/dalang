use super::DalangBrowser;
use anyhow::{Result, anyhow};
use base64::Engine as _;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::ScreenshotParams;

impl DalangBrowser {
    // ──────────────────────────────────────────────
    //  SCREENSHOTS
    // ──────────────────────────────────────────────

    /// Take a screenshot of the current page. Returns info string with base64 data.
    pub async fn screenshot(&self, full_page: bool, selector: Option<&str>) -> Result<String> {
        let page = self.active_page()?;
        let bytes = if let Some(sel) = selector {
            let el = page
                .find_element(sel)
                .await
                .map_err(|e| anyhow!("Element not found '{}': {}", sel, e))?;
            el.screenshot(CaptureScreenshotFormat::Png).await?
        } else if full_page {
            let params = ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .capture_beyond_viewport(true)
                .build();
            page.screenshot(params).await?
        } else {
            let params = ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .build();
            page.screenshot(params).await?
        };
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let size_kb = bytes.len() / 1024;
        let mode = if selector.is_some() {
            "element"
        } else if full_page {
            "full-page"
        } else {
            "viewport"
        };
        Ok(format!(
            "Screenshot captured: {}KB ({}). Base64 data: {}",
            size_kb, mode, b64
        ))
    }

    /// Take a screenshot and save to a file path.
    pub async fn screenshot_to_file(&self, path: &str, full_page: bool) -> Result<String> {
        let page = self.active_page()?;
        let bytes = if full_page {
            let params = ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .capture_beyond_viewport(true)
                .build();
            page.screenshot(params).await?
        } else {
            let params = ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .build();
            page.screenshot(params).await?
        };
        std::fs::write(path, &bytes)?;
        Ok(format!(
            "Screenshot saved to {} ({}KB)",
            path,
            bytes.len() / 1024
        ))
    }
}
