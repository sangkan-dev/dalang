//! Browser CDP Adapter.
//!
//! Implements `BrowserPort` using `crate::cdp::browser::DalangBrowser` (chromiumoxide).
//!
//! The adapter is `Send + Sync` — it wraps `DalangBrowser` in an `Arc<Mutex<...>>`
//! so it can be shared across async tasks while the underlying browser state
//! (active page, tab list, etc.) remains consistently locked.

use crate::application::ports::browser_port::BrowserPort;
use anyhow::Result;
use async_trait::async_trait;

pub mod browser;
use browser::DalangBrowser;

use tokio::sync::Mutex;

/// Concrete browser adapter using chromiumoxide (Chrome DevTools Protocol).
pub struct ChromiumBrowserAdapter {
    // Mutex allows interior mutability for tab management methods
    inner: Mutex<DalangBrowser>,
}

impl ChromiumBrowserAdapter {
    /// Create and launch the browser. Returns an error if Chromium is not installed.
    pub async fn new(headless: bool) -> Result<Self> {
        let browser = DalangBrowser::new(headless).await?;
        Ok(Self {
            inner: Mutex::new(browser),
        })
    }
}

#[async_trait]
impl BrowserPort for ChromiumBrowserAdapter {
    // ── Navigation ─────────────────────────────────────────────────────────────

    async fn navigate(&self, url: &str) -> Result<String> {
        self.inner.lock().await.navigate(url).await
    }

    async fn get_url(&self) -> Result<String> {
        self.inner.lock().await.get_url().await
    }

    async fn get_title(&self) -> Result<String> {
        self.inner.lock().await.get_title().await
    }

    async fn get_html(&self) -> Result<String> {
        self.inner.lock().await.get_html().await
    }

    async fn go_back(&self) -> Result<String> {
        self.inner.lock().await.go_back().await
    }

    async fn go_forward(&self) -> Result<String> {
        self.inner.lock().await.go_forward().await
    }

    async fn reload(&self) -> Result<String> {
        self.inner.lock().await.reload().await
    }

    // ── DOM Extraction ─────────────────────────────────────────────────────────

    async fn extract_dom(&self) -> Result<String> {
        self.inner.lock().await.extract_dom().await
    }

    async fn evaluate_js(&self, script: &str) -> Result<String> {
        self.inner.lock().await.evaluate_js(script).await
    }

    // ── DOM Query ──────────────────────────────────────────────────────────────

    async fn query_selector(&self, selector: &str) -> Result<String> {
        self.inner.lock().await.query_selector(selector).await
    }

    async fn query_selector_all(&self, selector: &str, limit: usize) -> Result<String> {
        self.inner
            .lock()
            .await
            .query_selector_all(selector, limit)
            .await
    }

    async fn get_attribute(&self, selector: &str, attribute: &str) -> Result<String> {
        self.inner
            .lock()
            .await
            .get_attribute(selector, attribute)
            .await
    }

    async fn wait_for_selector(&self, selector: &str, timeout_ms: u64) -> Result<String> {
        self.inner
            .lock()
            .await
            .wait_for_selector(selector, timeout_ms)
            .await
    }

    // ── Element Interaction ────────────────────────────────────────────────────

    async fn click(&self, selector: &str) -> Result<String> {
        self.inner.lock().await.click(selector).await
    }

    async fn type_text(&self, selector: &str, text: &str, clear: bool) -> Result<String> {
        self.inner
            .lock()
            .await
            .type_text(selector, text, clear)
            .await
    }

    async fn hover(&self, selector: &str) -> Result<String> {
        self.inner.lock().await.hover(selector).await
    }

    async fn focus(&self, selector: &str) -> Result<String> {
        self.inner.lock().await.focus(selector).await
    }

    async fn select_option(&self, selector: &str, value: &str) -> Result<String> {
        self.inner.lock().await.select_option(selector, value).await
    }

    async fn press_key(&self, selector: &str, key: &str) -> Result<String> {
        self.inner.lock().await.press_key(selector, key).await
    }

    async fn fill_form(&self, fields: &serde_json::Value) -> Result<String> {
        self.inner.lock().await.fill_form(fields).await
    }

    async fn submit_form(&self, form_selector: &str) -> Result<String> {
        self.inner.lock().await.submit_form(form_selector).await
    }

    async fn scroll(&self, x: i64, y: i64, selector: Option<&str>) -> Result<String> {
        self.inner.lock().await.scroll(x, y, selector).await
    }

    // ── Screenshots ────────────────────────────────────────────────────────────

    async fn screenshot(&self, full_page: bool, selector: Option<&str>) -> Result<String> {
        self.inner
            .lock()
            .await
            .screenshot(full_page, selector)
            .await
    }

    async fn screenshot_to_file(&self, path: &str, full_page: bool) -> Result<String> {
        self.inner
            .lock()
            .await
            .screenshot_to_file(path, full_page)
            .await
    }

    // ── Cookies ────────────────────────────────────────────────────────────────

    async fn get_cookies(&self) -> Result<String> {
        self.inner.lock().await.get_cookies().await
    }

    async fn set_cookie(
        &self,
        name: &str,
        value: &str,
        domain: Option<&str>,
        path: Option<&str>,
        secure: Option<bool>,
        http_only: Option<bool>,
    ) -> Result<String> {
        self.inner
            .lock()
            .await
            .set_cookie(name, value, domain, path, secure, http_only)
            .await
    }

    async fn delete_cookies(&self, name: Option<&str>) -> Result<String> {
        self.inner.lock().await.delete_cookies(name).await
    }

    // ── Storage ────────────────────────────────────────────────────────────────

    async fn get_storage(&self, storage_type: &str) -> Result<String> {
        self.inner.lock().await.get_storage(storage_type).await
    }

    async fn set_storage(&self, storage_type: &str, key: &str, value: &str) -> Result<String> {
        self.inner
            .lock()
            .await
            .set_storage(storage_type, key, value)
            .await
    }

    async fn clear_storage(&self, storage_type: &str) -> Result<String> {
        self.inner.lock().await.clear_storage(storage_type).await
    }

    // ── Network / Headers ──────────────────────────────────────────────────────

    async fn set_extra_headers(&self, headers: &serde_json::Value) -> Result<String> {
        self.inner.lock().await.set_extra_headers(headers).await
    }

    async fn set_user_agent(&self, user_agent: &str) -> Result<String> {
        self.inner.lock().await.set_user_agent(user_agent).await
    }

    async fn enable_network_log(&self) -> Result<String> {
        self.inner.lock().await.enable_network_log().await
    }

    async fn get_network_log(&self, clear: bool) -> Result<String> {
        self.inner.lock().await.get_network_log(clear).await
    }

    async fn set_viewport(&self, width: u32, height: u32) -> Result<String> {
        self.inner.lock().await.set_viewport(width, height).await
    }

    // ── Tab Management ─────────────────────────────────────────────────────────

    async fn new_tab(&self, url: Option<&str>) -> Result<String> {
        self.inner.lock().await.new_tab(url).await
    }

    async fn list_tabs(&self) -> Result<String> {
        self.inner.lock().await.list_tabs().await
    }

    async fn switch_tab(&self, index: usize) -> Result<String> {
        self.inner.lock().await.switch_tab(index)
    }

    async fn close_tab(&self, index: Option<usize>) -> Result<String> {
        self.inner.lock().await.close_tab(index).await
    }
}

// ── Lazy Browser Adapter ──────────────────────────────────────────────────────

use tokio::sync::OnceCell;

/// A lazy-initializing browser adapter that creates the browser on first use.
///
/// This allows injecting a browser port at DI time without paying the startup
/// cost of launching Chromium until a browser tool is actually invoked.
pub struct LazyBrowserAdapter {
    headless: bool,
    inner: OnceCell<ChromiumBrowserAdapter>,
}

impl LazyBrowserAdapter {
    pub fn new(headless: bool) -> Self {
        Self {
            headless,
            inner: OnceCell::new(),
        }
    }

    async fn get_or_init(&self) -> Result<&ChromiumBrowserAdapter> {
        self.inner
            .get_or_try_init(|| ChromiumBrowserAdapter::new(self.headless))
            .await
    }
}

#[async_trait]
impl BrowserPort for LazyBrowserAdapter {
    async fn navigate(&self, url: &str) -> Result<String> {
        self.get_or_init().await?.navigate(url).await
    }
    async fn get_url(&self) -> Result<String> {
        self.get_or_init().await?.get_url().await
    }
    async fn get_title(&self) -> Result<String> {
        self.get_or_init().await?.get_title().await
    }
    async fn get_html(&self) -> Result<String> {
        self.get_or_init().await?.get_html().await
    }
    async fn go_back(&self) -> Result<String> {
        self.get_or_init().await?.go_back().await
    }
    async fn go_forward(&self) -> Result<String> {
        self.get_or_init().await?.go_forward().await
    }
    async fn reload(&self) -> Result<String> {
        self.get_or_init().await?.reload().await
    }
    async fn extract_dom(&self) -> Result<String> {
        self.get_or_init().await?.extract_dom().await
    }
    async fn evaluate_js(&self, script: &str) -> Result<String> {
        self.get_or_init().await?.evaluate_js(script).await
    }
    async fn query_selector(&self, selector: &str) -> Result<String> {
        self.get_or_init().await?.query_selector(selector).await
    }
    async fn query_selector_all(&self, selector: &str, limit: usize) -> Result<String> {
        self.get_or_init()
            .await?
            .query_selector_all(selector, limit)
            .await
    }
    async fn get_attribute(&self, selector: &str, attribute: &str) -> Result<String> {
        self.get_or_init()
            .await?
            .get_attribute(selector, attribute)
            .await
    }
    async fn wait_for_selector(&self, selector: &str, timeout_ms: u64) -> Result<String> {
        self.get_or_init()
            .await?
            .wait_for_selector(selector, timeout_ms)
            .await
    }
    async fn click(&self, selector: &str) -> Result<String> {
        self.get_or_init().await?.click(selector).await
    }
    async fn type_text(&self, selector: &str, text: &str, clear: bool) -> Result<String> {
        self.get_or_init()
            .await?
            .type_text(selector, text, clear)
            .await
    }
    async fn hover(&self, selector: &str) -> Result<String> {
        self.get_or_init().await?.hover(selector).await
    }
    async fn focus(&self, selector: &str) -> Result<String> {
        self.get_or_init().await?.focus(selector).await
    }
    async fn select_option(&self, selector: &str, value: &str) -> Result<String> {
        self.get_or_init()
            .await?
            .select_option(selector, value)
            .await
    }
    async fn press_key(&self, selector: &str, key: &str) -> Result<String> {
        self.get_or_init().await?.press_key(selector, key).await
    }
    async fn fill_form(&self, fields: &serde_json::Value) -> Result<String> {
        self.get_or_init().await?.fill_form(fields).await
    }
    async fn submit_form(&self, selector: &str) -> Result<String> {
        self.get_or_init().await?.submit_form(selector).await
    }
    async fn scroll(&self, x: i64, y: i64, selector: Option<&str>) -> Result<String> {
        self.get_or_init().await?.scroll(x, y, selector).await
    }
    async fn screenshot(&self, full_page: bool, selector: Option<&str>) -> Result<String> {
        self.get_or_init()
            .await?
            .screenshot(full_page, selector)
            .await
    }
    async fn screenshot_to_file(&self, path: &str, full_page: bool) -> Result<String> {
        self.get_or_init()
            .await?
            .screenshot_to_file(path, full_page)
            .await
    }
    async fn get_cookies(&self) -> Result<String> {
        self.get_or_init().await?.get_cookies().await
    }
    async fn set_cookie(
        &self,
        name: &str,
        value: &str,
        domain: Option<&str>,
        path: Option<&str>,
        secure: Option<bool>,
        http_only: Option<bool>,
    ) -> Result<String> {
        self.get_or_init()
            .await?
            .set_cookie(name, value, domain, path, secure, http_only)
            .await
    }
    async fn delete_cookies(&self, name: Option<&str>) -> Result<String> {
        self.get_or_init().await?.delete_cookies(name).await
    }
    async fn get_storage(&self, storage_type: &str) -> Result<String> {
        self.get_or_init().await?.get_storage(storage_type).await
    }
    async fn set_storage(&self, storage_type: &str, key: &str, value: &str) -> Result<String> {
        self.get_or_init()
            .await?
            .set_storage(storage_type, key, value)
            .await
    }
    async fn clear_storage(&self, storage_type: &str) -> Result<String> {
        self.get_or_init().await?.clear_storage(storage_type).await
    }
    async fn set_extra_headers(&self, headers: &serde_json::Value) -> Result<String> {
        self.get_or_init().await?.set_extra_headers(headers).await
    }
    async fn set_user_agent(&self, user_agent: &str) -> Result<String> {
        self.get_or_init().await?.set_user_agent(user_agent).await
    }
    async fn enable_network_log(&self) -> Result<String> {
        self.get_or_init().await?.enable_network_log().await
    }
    async fn get_network_log(&self, clear: bool) -> Result<String> {
        self.get_or_init().await?.get_network_log(clear).await
    }
    async fn set_viewport(&self, width: u32, height: u32) -> Result<String> {
        self.get_or_init().await?.set_viewport(width, height).await
    }
    async fn new_tab(&self, url: Option<&str>) -> Result<String> {
        self.get_or_init().await?.new_tab(url).await
    }
    async fn list_tabs(&self) -> Result<String> {
        self.get_or_init().await?.list_tabs().await
    }
    async fn switch_tab(&self, index: usize) -> Result<String> {
        self.get_or_init().await?.switch_tab(index).await
    }
    async fn close_tab(&self, index: Option<usize>) -> Result<String> {
        self.get_or_init().await?.close_tab(index).await
    }
}
