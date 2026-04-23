//! Browser Controller Port.
//!
//! Defines the contract for interacting with a headless browser via CDP.
//! Concrete implementation: `adapters/outbound/browser_cdp/`.
//!
//! Design note: all methods take `&self` (not `&mut self`) to allow the adapter
//! to wrap DalangBrowser in a Mutex for `Arc<dyn BrowserPort>` usage.

use anyhow::Result;
use async_trait::async_trait;

/// Outbound port for controlling a headless browser.
///
/// The orchestrator calls these methods when the LLM invokes a browser tool.
/// The chromiumoxide implementation lives in `adapters/outbound/browser_cdp/`.
#[async_trait]
pub trait BrowserPort: Send + Sync {
    // ── Navigation ────────────────────────────────────────────────────────────
    async fn navigate(&self, url: &str) -> Result<String>;
    async fn get_url(&self) -> Result<String>;
    async fn get_title(&self) -> Result<String>;
    async fn get_html(&self) -> Result<String>;
    async fn go_back(&self) -> Result<String>;
    async fn go_forward(&self) -> Result<String>;
    async fn reload(&self) -> Result<String>;

    // ── DOM Extraction ────────────────────────────────────────────────────────
    async fn extract_dom(&self) -> Result<String>;
    async fn evaluate_js(&self, script: &str) -> Result<String>;

    // ── DOM Query ─────────────────────────────────────────────────────────────
    async fn query_selector(&self, selector: &str) -> Result<String>;
    async fn query_selector_all(&self, selector: &str, limit: usize) -> Result<String>;
    async fn get_attribute(&self, selector: &str, attribute: &str) -> Result<String>;
    async fn wait_for_selector(&self, selector: &str, timeout_ms: u64) -> Result<String>;

    // ── Interaction ───────────────────────────────────────────────────────────
    async fn click(&self, selector: &str) -> Result<String>;
    async fn type_text(&self, selector: &str, text: &str, clear: bool) -> Result<String>;
    async fn hover(&self, selector: &str) -> Result<String>;
    async fn focus(&self, selector: &str) -> Result<String>;
    async fn select_option(&self, selector: &str, value: &str) -> Result<String>;
    /// Press a keyboard key while a selector element is focused.
    async fn press_key(&self, selector: &str, key: &str) -> Result<String>;
    async fn fill_form(&self, fields: &serde_json::Value) -> Result<String>;
    async fn submit_form(&self, selector: &str) -> Result<String>;
    async fn scroll(&self, x: i64, y: i64, selector: Option<&str>) -> Result<String>;

    // ── Screenshots ───────────────────────────────────────────────────────────
    async fn screenshot(&self, full_page: bool, selector: Option<&str>) -> Result<String>;
    async fn screenshot_to_file(&self, path: &str, full_page: bool) -> Result<String>;

    // ── Cookies ───────────────────────────────────────────────────────────────
    async fn get_cookies(&self) -> Result<String>;
    async fn set_cookie(
        &self,
        name: &str,
        value: &str,
        domain: Option<&str>,
        path: Option<&str>,
        secure: Option<bool>,
        http_only: Option<bool>,
    ) -> Result<String>;
    async fn delete_cookies(&self, name: Option<&str>) -> Result<String>;

    // ── Storage ───────────────────────────────────────────────────────────────
    async fn get_storage(&self, storage_type: &str) -> Result<String>;
    async fn set_storage(&self, storage_type: &str, key: &str, value: &str) -> Result<String>;
    async fn clear_storage(&self, storage_type: &str) -> Result<String>;

    // ── Network & Headers ─────────────────────────────────────────────────────
    async fn set_extra_headers(&self, headers: &serde_json::Value) -> Result<String>;
    async fn set_user_agent(&self, user_agent: &str) -> Result<String>;
    async fn enable_network_log(&self) -> Result<String>;
    async fn get_network_log(&self, clear: bool) -> Result<String>;
    async fn set_viewport(&self, width: u32, height: u32) -> Result<String>;

    // ── Tab Management ────────────────────────────────────────────────────────
    async fn new_tab(&self, url: Option<&str>) -> Result<String>;
    async fn list_tabs(&self) -> Result<String>;
    async fn switch_tab(&self, index: usize) -> Result<String>;
    async fn close_tab(&self, index: Option<usize>) -> Result<String>;
}
