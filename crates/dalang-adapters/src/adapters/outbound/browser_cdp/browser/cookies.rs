use super::DalangBrowser;
use anyhow::Result;
use chromiumoxide::cdp::browser_protocol::network::{CookieParam, DeleteCookiesParams};
use serde_json;

impl DalangBrowser {
    // ──────────────────────────────────────────────
    //  COOKIES
    // ──────────────────────────────────────────────

    /// Get all cookies for the current page.
    pub async fn get_cookies(&self) -> Result<String> {
        let page = self.active_page()?;
        let cookies = page.get_cookies().await?;
        let cookie_list: Vec<serde_json::Value> = cookies
            .iter()
            .map(|c| {
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
            })
            .collect();
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
}
