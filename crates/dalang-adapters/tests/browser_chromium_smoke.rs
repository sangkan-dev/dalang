//! Optional Chromium integration tests for `BrowserPort`.
//!
//! Run locally when Chromium is installed:
//! `cargo test -p dalang-adapters --test browser_chromium_smoke -- --ignored`

use dalang_adapters::adapters::outbound::browser_cdp::ChromiumBrowserAdapter;
use dalang_application::application::ports::browser_port::BrowserPort;

#[tokio::test]
#[ignore = "requires Chromium; run with: cargo test -p dalang-adapters --test browser_chromium_smoke -- --ignored"]
async fn headless_navigate_about_blank() {
    let browser = ChromiumBrowserAdapter::new(true)
        .await
        .expect("launch Chromium (install chromium/google-chrome or set CHROME path)");
    browser
        .navigate("about:blank")
        .await
        .expect("navigate about:blank");
    let url = browser.get_url().await.expect("get_url");
    assert!(
        url.contains("blank") || url.starts_with("chrome://") || url.starts_with("about:"),
        "unexpected url: {url}"
    );
}

#[tokio::test]
#[ignore = "requires Chromium; run with: cargo test -p dalang-adapters --test browser_chromium_smoke -- --ignored"]
async fn headless_evaluate_js_returns_value() {
    let browser = ChromiumBrowserAdapter::new(true)
        .await
        .expect("launch Chromium");
    browser.navigate("about:blank").await.expect("navigate");
    let out = browser.evaluate_js("1 + 2").await.expect("evaluate_js");
    assert!(
        out.contains('3'),
        "expected result to mention 3, got: {out:?}"
    );
}
