use anyhow::{Result, anyhow};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use chromiumoxide::cdp::browser_protocol::network::{
    CookieParam, DeleteCookiesParams, EnableParams as NetworkEnableParams,
    EventRequestWillBeSent, EventResponseReceived, Headers,
    SetExtraHttpHeadersParams, SetUserAgentOverrideParams,
};
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use futures::StreamExt;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    browser: Browser,
    /// All open pages (tabs). Index 0 is always the first tab.
    pages: Vec<Page>,
    /// Index of the currently active tab in `pages`.
    active_idx: usize,
    /// Captured network entries (populated after `enable_network_log`).
    network_log: Arc<Mutex<Vec<NetworkEntry>>>,
    /// Whether network logging is active.
    network_logging: bool,
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
    fn active_page(&self) -> Result<&Page> {
        self.pages
            .get(self.active_idx)
            .ok_or_else(|| anyhow!("No active page (idx {} of {})", self.active_idx, self.pages.len()))
    }

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

    // ──────────────────────────────────────────────
    //  DOM QUERY
    // ──────────────────────────────────────────────

    /// Query a single element by CSS selector and return its info as JSON.
    pub async fn query_selector(&self, selector: &str) -> Result<String> {
        let page = self.active_page()?;
        let el = page.find_element(selector).await
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
        let elements = page.find_elements(selector).await
            .map_err(|e| anyhow!("Elements not found '{}': {}", selector, e))?;
        let mut results = Vec::new();
        for (i, el) in elements.into_iter().enumerate() {
            if i >= limit { break; }
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
        let el = page.find_element(selector).await
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
                return Ok(format!("Element '{}' found after {}ms", selector, start.elapsed().as_millis()));
            }
            if start.elapsed() > timeout {
                return Err(anyhow!("Timeout waiting for '{}' after {}ms", selector, timeout_ms));
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    }

    // ──────────────────────────────────────────────
    //  ELEMENT INTERACTION
    // ──────────────────────────────────────────────

    /// Click an element by CSS selector.
    pub async fn click(&self, selector: &str) -> Result<String> {
        let page = self.active_page()?;
        let el = page.find_element(selector).await
            .map_err(|e| anyhow!("Element not found '{}': {}", selector, e))?;
        el.scroll_into_view().await?;
        el.click().await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        Ok(format!("Clicked '{}'", selector))
    }

    /// Type text into an element. If `clear` is true, clear the field first.
    pub async fn type_text(&self, selector: &str, text: &str, clear: bool) -> Result<String> {
        let page = self.active_page()?;
        let el = page.find_element(selector).await
            .map_err(|e| anyhow!("Element not found '{}': {}", selector, e))?;
        el.scroll_into_view().await?;
        el.focus().await?;
        if clear {
            el.click().await?;
            let js = format!(
                "document.querySelector(\"{}\").value = \"\"",
                selector.replace('"', "\\\"")
            );
            page.evaluate(js).await?;
        }
        el.type_str(text).await?;
        Ok(format!("Typed {} chars into '{}'", text.len(), selector))
    }

    /// Hover over an element by CSS selector.
    pub async fn hover(&self, selector: &str) -> Result<String> {
        let page = self.active_page()?;
        let el = page.find_element(selector).await
            .map_err(|e| anyhow!("Element not found '{}': {}", selector, e))?;
        el.scroll_into_view().await?;
        el.hover().await?;
        Ok(format!("Hovered over '{}'", selector))
    }

    /// Focus on an element by CSS selector.
    pub async fn focus(&self, selector: &str) -> Result<String> {
        let page = self.active_page()?;
        let el = page.find_element(selector).await
            .map_err(|e| anyhow!("Element not found '{}': {}", selector, e))?;
        el.focus().await?;
        Ok(format!("Focused on '{}'", selector))
    }

    /// Select an option in a <select> element by value.
    pub async fn select_option(&self, selector: &str, value: &str) -> Result<String> {
        let page = self.active_page()?;
        let js = format!(
            "(() => {{ const el = document.querySelector(\"{sel}\"); if (!el) return \"Element not found\"; el.value = \"{val}\"; el.dispatchEvent(new Event(\"change\", {{ bubbles: true }})); return \"Selected: \" + el.value; }})()",
            sel = selector.replace('"', "\\\""),
            val = value.replace('"', "\\\"")
        );
        let result = page.evaluate(js).await?;
        Ok(result.into_value::<String>()?)
    }

    /// Press a keyboard key on a focused element (e.g., "Enter", "Tab", "Escape").
    pub async fn press_key(&self, selector: &str, key: &str) -> Result<String> {
        let page = self.active_page()?;
        let el = page.find_element(selector).await
            .map_err(|e| anyhow!("Element not found '{}': {}", selector, e))?;
        el.press_key(key).await?;
        Ok(format!("Pressed '{}' on '{}'", key, selector))
    }

    /// Fill multiple form fields at once. `fields` is a JSON object: {"selector": "value", ...}.
    pub async fn fill_form(&self, fields: &serde_json::Value) -> Result<String> {
        let obj = fields.as_object()
            .ok_or_else(|| anyhow!("fill_form fields must be a JSON object"))?;
        let page = self.active_page()?;
        let mut filled = 0;
        for (selector, value) in obj {
            let val = value.as_str().unwrap_or("");
            let el = page.find_element(selector.as_str()).await
                .map_err(|e| anyhow!("Field '{}' not found: {}", selector, e))?;
            el.focus().await?;
            let js = format!(
                "document.querySelector(\"{}\").value = \"\"",
                selector.replace('"', "\\\"")
            );
            page.evaluate(js).await?;
            el.type_str(val).await?;
            filled += 1;
        }
        Ok(format!("Filled {} form field(s)", filled))
    }

    /// Submit a form by clicking its submit button or calling .submit().
    pub async fn submit_form(&self, form_selector: &str) -> Result<String> {
        let page = self.active_page()?;
        let submit_sel = format!(
            "{} [type=\"submit\"], {} button[type=\"submit\"]",
            form_selector, form_selector
        );
        if let Ok(btn) = page.find_element(&submit_sel).await {
            btn.click().await?;
        } else {
            let js = format!(
                "document.querySelector(\"{}\").submit()",
                form_selector.replace('"', "\\\"")
            );
            page.evaluate(js).await?;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        Ok(format!("Submitted form '{}'", form_selector))
    }

    /// Scroll the page or a specific element into view.
    pub async fn scroll(&self, x: i64, y: i64, selector: Option<&str>) -> Result<String> {
        let page = self.active_page()?;
        if let Some(sel) = selector {
            let el = page.find_element(sel).await
                .map_err(|e| anyhow!("Element not found '{}': {}", sel, e))?;
            el.scroll_into_view().await?;
            Ok(format!("Scrolled '{}' into view", sel))
        } else {
            page.evaluate(format!("window.scrollTo({}, {})", x, y)).await?;
            Ok(format!("Scrolled to ({}, {})", x, y))
        }
    }

    // ──────────────────────────────────────────────
    //  SCREENSHOTS
    // ──────────────────────────────────────────────

    /// Take a screenshot of the current page. Returns info string with base64 data.
    pub async fn screenshot(&self, full_page: bool, selector: Option<&str>) -> Result<String> {
        let page = self.active_page()?;
        let bytes = if let Some(sel) = selector {
            let el = page.find_element(sel).await
                .map_err(|e| anyhow!("Element not found '{}': {}", sel, e))?;
            el.screenshot(CaptureScreenshotFormat::Png).await?
        } else if full_page {
            use chromiumoxide::page::ScreenshotParams;
            let params = ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .capture_beyond_viewport(true)
                .build();
            page.screenshot(params).await?
        } else {
            use chromiumoxide::page::ScreenshotParams;
            let params = ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .build();
            page.screenshot(params).await?
        };
        use base64::Engine as _;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let size_kb = bytes.len() / 1024;
        let mode = if selector.is_some() { "element" } else if full_page { "full-page" } else { "viewport" };
        Ok(format!("Screenshot captured: {}KB ({}). Base64 data: {}", size_kb, mode, b64))
    }

    /// Take a screenshot and save to a file path.
    pub async fn screenshot_to_file(&self, path: &str, full_page: bool) -> Result<String> {
        let page = self.active_page()?;
        let bytes = if full_page {
            use chromiumoxide::page::ScreenshotParams;
            let params = ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .capture_beyond_viewport(true)
                .build();
            page.screenshot(params).await?
        } else {
            use chromiumoxide::page::ScreenshotParams;
            let params = ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .build();
            page.screenshot(params).await?
        };
        std::fs::write(path, &bytes)?;
        Ok(format!("Screenshot saved to {} ({}KB)", path, bytes.len() / 1024))
    }

    // ──────────────────────────────────────────────
    //  COOKIES
    // ──────────────────────────────────────────────

    /// Get all cookies for the current page.
    pub async fn get_cookies(&self) -> Result<String> {
        let page = self.active_page()?;
        let cookies = page.get_cookies().await?;
        let cookie_list: Vec<serde_json::Value> = cookies.iter().map(|c| {
            serde_json::json!({
                "name": c.name,
                "value": c.value,
                "domain": c.domain,
                "path": c.path,
                "secure": c.secure,
                "httpOnly": c.http_only,
                "sameSite": format!("{:?}", c.same_site),
                "expires": c.expires,
            })
        }).collect();
        Ok(serde_json::to_string_pretty(&cookie_list)?)
    }

    /// Set a cookie on the current page.
    pub async fn set_cookie(
        &self,
        name: &str,
        value: &str,
        domain: Option<&str>,
        path: Option<&str>,
        secure: Option<bool>,
        http_only: Option<bool>,
    ) -> Result<String> {
        let page = self.active_page()?;
        let mut cookie = CookieParam::new(name, value);
        if let Some(d) = domain {
            cookie.domain = Some(d.to_string());
        }
        if let Some(p) = path {
            cookie.path = Some(p.to_string());
        }
        if let Some(s) = secure {
            cookie.secure = Some(s);
        }
        if let Some(h) = http_only {
            cookie.http_only = Some(h);
        }
        page.set_cookie(cookie).await?;
        Ok(format!("Cookie '{}' set", name))
    }

    /// Delete cookies. If `name` is Some, delete that specific cookie; otherwise delete all.
    pub async fn delete_cookies(&self, name: Option<&str>) -> Result<String> {
        let page = self.active_page()?;
        if let Some(n) = name {
            let params = DeleteCookiesParams::new(n);
            page.delete_cookie(params).await?;
            Ok(format!("Deleted cookie '{}'", n))
        } else {
            let cookies = page.get_cookies().await?;
            let count = cookies.len();
            for c in &cookies {
                let params = DeleteCookiesParams::new(&c.name);
                let _ = page.delete_cookie(params).await;
            }
            Ok(format!("Deleted {} cookie(s)", count))
        }
    }

    // ──────────────────────────────────────────────
    //  STORAGE (localStorage / sessionStorage)
    // ──────────────────────────────────────────────

    /// Get all items from localStorage or sessionStorage.
    pub async fn get_storage(&self, storage_type: &str) -> Result<String> {
        let st = match storage_type {
            "session" | "sessionStorage" => "sessionStorage",
            _ => "localStorage",
        };
        let page = self.active_page()?;
        let js = format!("JSON.stringify(Object.fromEntries(Object.entries({})))", st);
        let result = page.evaluate(js).await?;
        Ok(result.into_value::<String>()?)
    }

    /// Set an item in localStorage or sessionStorage.
    pub async fn set_storage(&self, storage_type: &str, key: &str, value: &str) -> Result<String> {
        let st = match storage_type {
            "session" | "sessionStorage" => "sessionStorage",
            _ => "localStorage",
        };
        let page = self.active_page()?;
        let js = format!(
            "{}.setItem(\"{}\", \"{}\")",
            st, key.replace('"', "\\\""), value.replace('"', "\\\"")
        );
        page.evaluate(js).await?;
        Ok(format!("Set {}.{} = '{}'", st, key, value))
    }

    /// Clear all items from localStorage or sessionStorage.
    pub async fn clear_storage(&self, storage_type: &str) -> Result<String> {
        let st = match storage_type {
            "session" | "sessionStorage" => "sessionStorage",
            _ => "localStorage",
        };
        let page = self.active_page()?;
        page.evaluate(format!("{}.clear()", st)).await?;
        Ok(format!("Cleared {}", st))
    }

    // ──────────────────────────────────────────────
    //  NETWORK & HEADERS
    // ──────────────────────────────────────────────

    /// Set extra HTTP headers for all subsequent requests.
    pub async fn set_extra_headers(&self, headers: &serde_json::Value) -> Result<String> {
        let page = self.active_page()?;
        let obj = headers.as_object()
            .ok_or_else(|| anyhow!("headers must be a JSON object"))?;
        let header_json = serde_json::Value::Object(obj.clone());
        let params = SetExtraHttpHeadersParams::new(Headers::new(header_json));
        page.execute(params).await?;
        Ok(format!("Set {} extra header(s)", obj.len()))
    }

    /// Set the User-Agent string.
    pub async fn set_user_agent(&self, user_agent: &str) -> Result<String> {
        let page = self.active_page()?;
        let params = SetUserAgentOverrideParams::new(user_agent);
        page.set_user_agent(params).await?;
        Ok(format!("User-Agent set to: {}", user_agent))
    }

    /// Enable network logging - starts capturing HTTP requests/responses.
    pub async fn enable_network_log(&mut self) -> Result<String> {
        if self.network_logging {
            return Ok("Network logging already enabled".to_string());
        }
        let page = self.active_page()?;

        page.execute(NetworkEnableParams::default()).await?;

        let network_log = self.network_log.clone();
        let mut request_listener = page.event_listener::<EventRequestWillBeSent>().await?;
        tokio::spawn(async move {
            while let Some(event) = request_listener.next().await {
                let mut headers_map = HashMap::new();
                if let serde_json::Value::Object(hdrs) = event.request.headers.inner() {
                    for (k, v) in hdrs {
                        headers_map.insert(k.clone(), v.as_str().unwrap_or("").to_string());
                    }
                }
                let entry = NetworkEntry {
                    url: event.request.url.clone(),
                    method: event.request.method.clone(),
                    status: None,
                    mime_type: None,
                    request_headers: headers_map,
                    response_headers: HashMap::new(),
                    timestamp: *event.timestamp.inner(),
                };
                network_log.lock().await.push(entry);
            }
        });

        let network_log2 = self.network_log.clone();
        let mut response_listener = page.event_listener::<EventResponseReceived>().await?;
        tokio::spawn(async move {
            while let Some(event) = response_listener.next().await {
                let mut log = network_log2.lock().await;
                if let Some(entry) = log.iter_mut().rev().find(|e| e.url == event.response.url) {
                    entry.status = Some(event.response.status);
                    entry.mime_type = event.response.mime_type.clone().into();
                    if let serde_json::Value::Object(hdrs) = event.response.headers.inner() {
                        for (k, v) in hdrs {
                            entry.response_headers.insert(k.clone(), v.as_str().unwrap_or("").to_string());
                        }
                    }
                }
            }
        });

        self.network_logging = true;
        Ok("Network logging enabled".to_string())
    }

    /// Retrieve captured network log entries as JSON, optionally clearing.
    pub async fn get_network_log(&self, clear: bool) -> Result<String> {
        let mut log = self.network_log.lock().await;
        let json = serde_json::to_string_pretty(&*log)?;
        let count = log.len();
        if clear {
            log.clear();
        }
        Ok(format!("[{} entries{}]\n{}", count, if clear { ", cleared" } else { "" }, json))
    }

    /// Set the viewport size (device emulation).
    pub async fn set_viewport(&self, width: u32, height: u32) -> Result<String> {
        let page = self.active_page()?;
        let params = SetDeviceMetricsOverrideParams::builder()
            .width(width)
            .height(height)
            .device_scale_factor(1.0)
            .mobile(false)
            .build()
            .map_err(|e| anyhow!("Failed to build viewport params: {}", e))?;
        page.execute(params).await?;
        Ok(format!("Viewport set to {}x{}", width, height))
    }

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
            return Err(anyhow!("Tab index {} out of range (0..{})", index, self.pages.len()));
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
        Ok(format!("Closed tab #{}, active is now #{}", idx, self.active_idx))
    }
}
