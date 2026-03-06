//! Browser Controller Port.
//!
//! Defines the contract for interacting with a headless browser via CDP.
//! Concrete implementation: `adapters/outbound/browser_cdp/`.

use anyhow::Result;
use async_trait::async_trait;

/// Outbound port for controlling a headless browser.
///
/// The orchestrator calls these methods when the LLM invokes a browser tool.
/// The chromiumoxide implementation lives in `adapters/outbound/browser_cdp/`.
#[async_trait]
pub trait BrowserPort: Send + Sync {
    // ── Navigation ────────────────────────────────────────────────────────────
    async fn navigate(&mut self, url: &str) -> Result<String>;
    async fn get_url(&self) -> Result<String>;
    async fn get_title(&self) -> Result<String>;
    async fn get_html(&self) -> Result<String>;
    async fn go_back(&mut self) -> Result<String>;
    async fn go_forward(&mut self) -> Result<String>;
    async fn reload(&mut self) -> Result<String>;

    // ── DOM Query ─────────────────────────────────────────────────────────────
    async fn query_selector(&self, selector: &str) -> Result<String>;
    async fn query_selector_all(&self, selector: &str, limit: usize) -> Result<String>;
    async fn get_attribute(&self, selector: &str, attribute: &str) -> Result<String>;
    async fn wait_for_selector(&self, selector: &str, timeout_ms: u64) -> Result<String>;

    // ── Interaction ───────────────────────────────────────────────────────────
    async fn click(&mut self, selector: &str) -> Result<String>;
    async fn type_text(&mut self, selector: &str, text: &str, clear: bool) -> Result<String>;
    async fn hover(&mut self, selector: &str) -> Result<String>;
    async fn focus(&mut self, selector: &str) -> Result<String>;
    async fn select_option(&mut self, selector: &str, value: &str) -> Result<String>;
    async fn press_key(&mut self, key: &str) -> Result<String>;
    async fn fill_form(&mut self, fields_json: &str) -> Result<String>;
    async fn submit_form(&mut self, selector: &str) -> Result<String>;
    async fn scroll(&mut self, selector: Option<&str>, x: i64, y: i64) -> Result<String>;

    // ── Screenshots ───────────────────────────────────────────────────────────
    async fn screenshot(&self, full_page: bool, selector: Option<&str>) -> Result<String>;
    async fn screenshot_to_file(&self, path: &str) -> Result<String>;

    // ── Cookies ───────────────────────────────────────────────────────────────
    async fn get_cookies(&self) -> Result<String>;
    async fn set_cookie(&mut self, name: &str, value: &str, domain: Option<&str>)
    -> Result<String>;
    async fn delete_cookies(&mut self, name: Option<&str>) -> Result<String>;

    // ── Storage ───────────────────────────────────────────────────────────────
    async fn get_storage(&self, storage_type: &str) -> Result<String>;
    async fn set_storage(&mut self, storage_type: &str, key: &str, value: &str) -> Result<String>;
    async fn clear_storage(&mut self, storage_type: &str) -> Result<String>;

    // ── Network & Headers ─────────────────────────────────────────────────────
    async fn set_extra_headers(&mut self, headers_json: &str) -> Result<String>;
    async fn set_user_agent(&mut self, user_agent: &str) -> Result<String>;
    async fn enable_network_log(&mut self) -> Result<String>;
    async fn get_network_log(&mut self, clear: bool) -> Result<String>;
    async fn set_viewport(&mut self, width: u64, height: u64) -> Result<String>;

    // ── Tab Management ────────────────────────────────────────────────────────
    async fn new_tab(&mut self, url: Option<&str>) -> Result<String>;
    async fn list_tabs(&self) -> Result<String>;
    async fn switch_tab(&mut self, index: usize) -> Result<String>;
    async fn close_tab(&mut self, index: Option<usize>) -> Result<String>;

    // ── JavaScript ────────────────────────────────────────────────────────────
    async fn evaluate_js(&self, script: &str) -> Result<String>;
}
