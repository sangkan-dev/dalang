use super::DalangBrowser;
use anyhow::{Result, anyhow};
use serde_json;

impl DalangBrowser {
    // ──────────────────────────────────────────────
    //  ELEMENT INTERACTION
    // ──────────────────────────────────────────────

    /// Click an element by CSS selector.
    pub async fn click(&self, selector: &str) -> Result<String> {
        let page = self.active_page()?;
        let el = page
            .find_element(selector)
            .await
            .map_err(|e| anyhow!("Element not found '{}': {}", selector, e))?;
        el.scroll_into_view().await?;
        el.click().await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        Ok(format!("Clicked '{}'", selector))
    }

    /// Type text into an element. If `clear` is true, clear the field first.
    pub async fn type_text(&self, selector: &str, text: &str, clear: bool) -> Result<String> {
        let page = self.active_page()?;
        let el = page
            .find_element(selector)
            .await
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
        let el = page
            .find_element(selector)
            .await
            .map_err(|e| anyhow!("Element not found '{}': {}", selector, e))?;
        el.scroll_into_view().await?;
        el.hover().await?;
        Ok(format!("Hovered over '{}'", selector))
    }

    /// Focus on an element by CSS selector.
    pub async fn focus(&self, selector: &str) -> Result<String> {
        let page = self.active_page()?;
        let el = page
            .find_element(selector)
            .await
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
        let el = page
            .find_element(selector)
            .await
            .map_err(|e| anyhow!("Element not found '{}': {}", selector, e))?;
        el.press_key(key).await?;
        Ok(format!("Pressed '{}' on '{}'", key, selector))
    }

    /// Fill multiple form fields at once. `fields` is a JSON object: {"selector": "value", ...}.
    pub async fn fill_form(&self, fields: &serde_json::Value) -> Result<String> {
        let obj = fields
            .as_object()
            .ok_or_else(|| anyhow!("fill_form fields must be a JSON object"))?;
        let page = self.active_page()?;
        let mut filled = 0;
        for (selector, value) in obj {
            let val = value.as_str().unwrap_or("");
            let el = page
                .find_element(selector.as_str())
                .await
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
            let el = page
                .find_element(sel)
                .await
                .map_err(|e| anyhow!("Element not found '{}': {}", sel, e))?;
            el.scroll_into_view().await?;
            Ok(format!("Scrolled '{}' into view", sel))
        } else {
            page.evaluate(format!("window.scrollTo({}, {})", x, y))
                .await?;
            Ok(format!("Scrolled to ({}, {})", x, y))
        }
    }
}
