use super::{DalangBrowser, NetworkEntry};
use anyhow::{Result, anyhow};
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::cdp::browser_protocol::network::{
    EnableParams as NetworkEnableParams, EventRequestWillBeSent, EventResponseReceived, Headers,
    SetExtraHttpHeadersParams, SetUserAgentOverrideParams,
};
use futures::StreamExt;
use std::collections::HashMap;

impl DalangBrowser {
    // ──────────────────────────────────────────────
    //  NETWORK & HEADERS
    // ──────────────────────────────────────────────

    /// Set extra HTTP headers for all subsequent requests.
    pub async fn set_extra_headers(&self, headers: &serde_json::Value) -> Result<String> {
        let page = self.active_page()?;
        let obj = headers
            .as_object()
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
                            entry
                                .response_headers
                                .insert(k.clone(), v.as_str().unwrap_or("").to_string());
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
        Ok(format!(
            "[{} entries{}]\n{}",
            count,
            if clear { ", cleared" } else { "" },
            json
        ))
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
}
