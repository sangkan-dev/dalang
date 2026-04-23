use super::DalangBrowser;
use anyhow::Result;

impl DalangBrowser {
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
            st,
            key.replace('"', "\\\""),
            value.replace('"', "\\\"")
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
}
