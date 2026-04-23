//! Maps LLM `browser-*` tool calls to [`crate::application::ports::browser_port::BrowserPort`].
//!
//! Kept separate from the orchestrator so dispatch logic can be unit-tested with stub ports
//! without spinning the full ReAct loop (see `#[cfg(test)]` module).

use crate::application::ports::browser_port::BrowserPort;
use anyhow::anyhow;
use dalang_domain::domain::models::ToolCall;

/// Dispatch a `browser-*` tool call to `browser`.
///
/// The tool name follows `browser-<action>` (e.g. `browser-navigate`).
/// Arguments come from the tool call JSON `arguments` object.
pub async fn dispatch_browser_tool(browser: &dyn BrowserPort, tool_call: &ToolCall) -> String {
    let args = &tool_call.arguments;
    let action = tool_call
        .name
        .strip_prefix("browser-")
        .unwrap_or(&tool_call.name);

    let result = match action {
        // Navigation
        "navigate" => {
            let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
            browser.navigate(url).await
        }
        "get-url" => browser.get_url().await,
        "get-title" => browser.get_title().await,
        "get-html" => browser.get_html().await,
        "go-back" => browser.go_back().await,
        "go-forward" => browser.go_forward().await,
        "reload" => browser.reload().await,

        // DOM Extraction
        "extract-dom" => browser.extract_dom().await,
        "evaluate-js" => {
            let script = args.get("script").and_then(|v| v.as_str()).unwrap_or("");
            browser.evaluate_js(script).await
        }

        // DOM Query
        "query-selector" => {
            let selector = args.get("selector").and_then(|v| v.as_str()).unwrap_or("");
            browser.query_selector(selector).await
        }
        "query-selector-all" => {
            let selector = args.get("selector").and_then(|v| v.as_str()).unwrap_or("");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            browser.query_selector_all(selector, limit).await
        }
        "get-attribute" => {
            let selector = args.get("selector").and_then(|v| v.as_str()).unwrap_or("");
            let attribute = args.get("attribute").and_then(|v| v.as_str()).unwrap_or("");
            browser.get_attribute(selector, attribute).await
        }
        "wait-for-selector" => {
            let selector = args.get("selector").and_then(|v| v.as_str()).unwrap_or("");
            let timeout = args
                .get("timeout_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(5000);
            browser.wait_for_selector(selector, timeout).await
        }

        // Interaction
        "click" => {
            let selector = args.get("selector").and_then(|v| v.as_str()).unwrap_or("");
            browser.click(selector).await
        }
        "type-text" => {
            let selector = args.get("selector").and_then(|v| v.as_str()).unwrap_or("");
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let clear = args.get("clear").and_then(|v| v.as_bool()).unwrap_or(false);
            browser.type_text(selector, text, clear).await
        }
        "hover" => {
            let selector = args.get("selector").and_then(|v| v.as_str()).unwrap_or("");
            browser.hover(selector).await
        }
        "focus" => {
            let selector = args.get("selector").and_then(|v| v.as_str()).unwrap_or("");
            browser.focus(selector).await
        }
        "select-option" => {
            let selector = args.get("selector").and_then(|v| v.as_str()).unwrap_or("");
            let value = args.get("value").and_then(|v| v.as_str()).unwrap_or("");
            browser.select_option(selector, value).await
        }
        "press-key" => {
            let selector = args.get("selector").and_then(|v| v.as_str()).unwrap_or("");
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("");
            browser.press_key(selector, key).await
        }
        "fill-form" => {
            let fields = args.get("fields").unwrap_or(args);
            browser.fill_form(fields).await
        }
        "submit-form" => {
            let selector = args
                .get("selector")
                .and_then(|v| v.as_str())
                .unwrap_or("form");
            browser.submit_form(selector).await
        }
        "scroll" => {
            let x = args.get("x").and_then(|v| v.as_i64()).unwrap_or(0);
            let y = args.get("y").and_then(|v| v.as_i64()).unwrap_or(0);
            let selector = args.get("selector").and_then(|v| v.as_str());
            browser.scroll(x, y, selector).await
        }

        // Screenshots
        "screenshot" => {
            let full_page = args
                .get("full_page")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let selector = args.get("selector").and_then(|v| v.as_str());
            browser.screenshot(full_page, selector).await
        }
        "screenshot-to-file" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("screenshot.png");
            let full_page = args
                .get("full_page")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            browser.screenshot_to_file(path, full_page).await
        }

        // Cookies
        "get-cookies" => browser.get_cookies().await,
        "set-cookie" => {
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let value = args.get("value").and_then(|v| v.as_str()).unwrap_or("");
            let domain = args.get("domain").and_then(|v| v.as_str());
            let path = args.get("path").and_then(|v| v.as_str());
            let secure = args.get("secure").and_then(|v| v.as_bool());
            let http_only = args.get("http_only").and_then(|v| v.as_bool());
            browser
                .set_cookie(name, value, domain, path, secure, http_only)
                .await
        }
        "delete-cookies" => {
            let name = args.get("name").and_then(|v| v.as_str());
            browser.delete_cookies(name).await
        }

        // Storage
        "get-storage" => {
            let stype = args
                .get("storage_type")
                .and_then(|v| v.as_str())
                .unwrap_or("local");
            browser.get_storage(stype).await
        }
        "set-storage" => {
            let stype = args
                .get("storage_type")
                .and_then(|v| v.as_str())
                .unwrap_or("local");
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("");
            let value = args.get("value").and_then(|v| v.as_str()).unwrap_or("");
            browser.set_storage(stype, key, value).await
        }
        "clear-storage" => {
            let stype = args
                .get("storage_type")
                .and_then(|v| v.as_str())
                .unwrap_or("local");
            browser.clear_storage(stype).await
        }

        // Network & Headers
        "set-extra-headers" => {
            let headers = args.get("headers").unwrap_or(args);
            browser.set_extra_headers(headers).await
        }
        "set-user-agent" => {
            let ua = args
                .get("user_agent")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            browser.set_user_agent(ua).await
        }
        "enable-network-log" => browser.enable_network_log().await,
        "get-network-log" => {
            let clear = args.get("clear").and_then(|v| v.as_bool()).unwrap_or(false);
            browser.get_network_log(clear).await
        }
        "set-viewport" => {
            let width = args.get("width").and_then(|v| v.as_u64()).unwrap_or(1280) as u32;
            let height = args.get("height").and_then(|v| v.as_u64()).unwrap_or(720) as u32;
            browser.set_viewport(width, height).await
        }

        // Tab Management
        "new-tab" => {
            let url = args.get("url").and_then(|v| v.as_str());
            browser.new_tab(url).await
        }
        "list-tabs" => browser.list_tabs().await,
        "switch-tab" => {
            let index = args.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            browser.switch_tab(index).await
        }
        "close-tab" => {
            let index = args
                .get("index")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);
            browser.close_tab(index).await
        }

        other => Err(anyhow!("Unknown browser action: {}", other)),
    };

    match result {
        Ok(output) => output,
        Err(e) => format!("Browser tool error ({}): {}", action, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::sync::Mutex;

    /// Records the last `navigate` URL; other methods return empty Ok.
    struct RecordingBrowser {
        last_navigate: Mutex<Option<String>>,
        last_type_text: Mutex<Option<(String, String, bool)>>,
    }

    impl RecordingBrowser {
        fn new() -> Self {
            Self {
                last_navigate: Mutex::new(None),
                last_type_text: Mutex::new(None),
            }
        }
    }

    fn ok_empty() -> Result<String> {
        Ok(String::new())
    }

    #[async_trait::async_trait]
    impl BrowserPort for RecordingBrowser {
        async fn navigate(&self, url: &str) -> Result<String> {
            *self.last_navigate.lock().unwrap() = Some(url.to_string());
            Ok(format!("ok:{url}"))
        }
        async fn get_url(&self) -> Result<String> {
            ok_empty()
        }
        async fn get_title(&self) -> Result<String> {
            ok_empty()
        }
        async fn get_html(&self) -> Result<String> {
            ok_empty()
        }
        async fn go_back(&self) -> Result<String> {
            ok_empty()
        }
        async fn go_forward(&self) -> Result<String> {
            ok_empty()
        }
        async fn reload(&self) -> Result<String> {
            ok_empty()
        }
        async fn extract_dom(&self) -> Result<String> {
            ok_empty()
        }
        async fn evaluate_js(&self, _script: &str) -> Result<String> {
            ok_empty()
        }
        async fn query_selector(&self, _selector: &str) -> Result<String> {
            ok_empty()
        }
        async fn query_selector_all(&self, _selector: &str, _limit: usize) -> Result<String> {
            ok_empty()
        }
        async fn get_attribute(&self, _selector: &str, _attribute: &str) -> Result<String> {
            ok_empty()
        }
        async fn wait_for_selector(&self, _selector: &str, _timeout_ms: u64) -> Result<String> {
            ok_empty()
        }
        async fn click(&self, _selector: &str) -> Result<String> {
            ok_empty()
        }
        async fn type_text(&self, selector: &str, text: &str, clear: bool) -> Result<String> {
            *self.last_type_text.lock().unwrap() =
                Some((selector.to_string(), text.to_string(), clear));
            Ok("typed".into())
        }
        async fn hover(&self, _selector: &str) -> Result<String> {
            ok_empty()
        }
        async fn focus(&self, _selector: &str) -> Result<String> {
            ok_empty()
        }
        async fn select_option(&self, _selector: &str, _value: &str) -> Result<String> {
            ok_empty()
        }
        async fn press_key(&self, _selector: &str, _key: &str) -> Result<String> {
            ok_empty()
        }
        async fn fill_form(&self, _fields: &serde_json::Value) -> Result<String> {
            ok_empty()
        }
        async fn submit_form(&self, _selector: &str) -> Result<String> {
            ok_empty()
        }
        async fn scroll(&self, _x: i64, _y: i64, _selector: Option<&str>) -> Result<String> {
            ok_empty()
        }
        async fn screenshot(&self, _full_page: bool, _selector: Option<&str>) -> Result<String> {
            ok_empty()
        }
        async fn screenshot_to_file(&self, _path: &str, _full_page: bool) -> Result<String> {
            ok_empty()
        }
        async fn get_cookies(&self) -> Result<String> {
            ok_empty()
        }
        async fn set_cookie(
            &self,
            _name: &str,
            _value: &str,
            _domain: Option<&str>,
            _path: Option<&str>,
            _secure: Option<bool>,
            _http_only: Option<bool>,
        ) -> Result<String> {
            ok_empty()
        }
        async fn delete_cookies(&self, _name: Option<&str>) -> Result<String> {
            ok_empty()
        }
        async fn get_storage(&self, _storage_type: &str) -> Result<String> {
            ok_empty()
        }
        async fn set_storage(
            &self,
            _storage_type: &str,
            _key: &str,
            _value: &str,
        ) -> Result<String> {
            ok_empty()
        }
        async fn clear_storage(&self, _storage_type: &str) -> Result<String> {
            ok_empty()
        }
        async fn set_extra_headers(&self, _headers: &serde_json::Value) -> Result<String> {
            ok_empty()
        }
        async fn set_user_agent(&self, _user_agent: &str) -> Result<String> {
            ok_empty()
        }
        async fn enable_network_log(&self) -> Result<String> {
            ok_empty()
        }
        async fn get_network_log(&self, _clear: bool) -> Result<String> {
            ok_empty()
        }
        async fn set_viewport(&self, _width: u32, _height: u32) -> Result<String> {
            ok_empty()
        }
        async fn new_tab(&self, _url: Option<&str>) -> Result<String> {
            ok_empty()
        }
        async fn list_tabs(&self) -> Result<String> {
            ok_empty()
        }
        async fn switch_tab(&self, _index: usize) -> Result<String> {
            ok_empty()
        }
        async fn close_tab(&self, _index: Option<usize>) -> Result<String> {
            ok_empty()
        }
    }

    #[tokio::test]
    async fn dispatch_navigate_forwards_url() {
        let b = RecordingBrowser::new();
        let tc = ToolCall {
            name: "browser-navigate".into(),
            arguments: serde_json::json!({ "url": "https://example.test/path" }),
        };
        let out = dispatch_browser_tool(&b, &tc).await;
        assert_eq!(out, "ok:https://example.test/path");
        assert_eq!(
            b.last_navigate.lock().unwrap().as_deref(),
            Some("https://example.test/path")
        );
    }

    #[tokio::test]
    async fn dispatch_strips_browser_prefix_from_name() {
        let b = RecordingBrowser::new();
        let tc = ToolCall {
            name: "navigate".into(),
            arguments: serde_json::json!({ "url": "https://a" }),
        };
        let _ = dispatch_browser_tool(&b, &tc).await;
        assert_eq!(
            b.last_navigate.lock().unwrap().as_deref(),
            Some("https://a")
        );
    }

    #[tokio::test]
    async fn dispatch_type_text_passes_clear_flag() {
        let b = RecordingBrowser::new();
        let tc = ToolCall {
            name: "browser-type-text".into(),
            arguments: serde_json::json!({
                "selector": "#q",
                "text": "hello",
                "clear": true
            }),
        };
        let out = dispatch_browser_tool(&b, &tc).await;
        assert_eq!(out, "typed");
        let got = b.last_type_text.lock().unwrap().clone().unwrap();
        assert_eq!(got.0, "#q");
        assert_eq!(got.1, "hello");
        assert!(got.2);
    }

    #[tokio::test]
    async fn unknown_action_returns_error_message() {
        let b = RecordingBrowser::new();
        let tc = ToolCall {
            name: "browser-not-a-real-tool".into(),
            arguments: serde_json::json!({}),
        };
        let out = dispatch_browser_tool(&b, &tc).await;
        assert!(
            out.contains("Unknown browser action") && out.contains("not-a-real-tool"),
            "{}",
            out
        );
    }
}
