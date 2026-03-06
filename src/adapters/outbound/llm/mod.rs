//! Outbound LLM adapter — creates the correct LlmPort implementation
//! based on `endpoint_mode` configuration.
//!
//! This module is a *factory*. It reads configuration and constructs
//! the appropriate adapter that implements `LlmPort`.

use crate::application::ports::llm_port::LlmPort;
use crate::domain::models::AuthToken;
use anyhow::Result;
use std::sync::Arc;

// Re-export the provider implementations from the existing llm module
// (which lives at crate::llm). These are the concrete structs that
// implement LlmPort. We avoid duplicating their code.

/// Create the appropriate LLM provider adapter based on endpoint mode.
///
/// - `endpoint_mode == "copilot"` → `CopilotProvider`
/// - `endpoint_mode == "cloudcode"` → `GeminiCodeAssistProvider`
/// - anything else → `OpenAiCompatibleProvider`
pub fn create_llm_adapter(
    endpoint_mode: &str,
    base_url: String,
    model: String,
    auth: AuthToken,
    codeassist_endpoint: Option<String>,
    gcp_project: Option<String>,
) -> Result<Arc<dyn LlmPort>> {
    // Convert domain AuthToken to the legacy llm::AuthToken (they have the same shape)
    let legacy_auth = match auth {
        AuthToken::None => crate::llm::AuthToken::None,
        AuthToken::ApiKey(k) => crate::llm::AuthToken::ApiKey(k),
        AuthToken::Bearer(t) => crate::llm::AuthToken::Bearer(t),
    };

    let boxed: Box<dyn crate::llm::LlmProvider + Send + Sync> = crate::llm::create_provider(
        endpoint_mode,
        base_url,
        model,
        legacy_auth,
        codeassist_endpoint,
        gcp_project,
    )?;

    // Wrap the legacy LlmProvider in a shim that implements LlmPort
    Ok(Arc::new(LlmProviderShim(boxed)))
}

/// Shim that adapts the legacy `crate::llm::LlmProvider` trait to the new `LlmPort` trait.
///
/// This lets existing OpenAI/Gemini/Copilot code continue to work without rewriting,
/// while being usable through the new hexagonal port abstraction.
struct LlmProviderShim(Box<dyn crate::llm::LlmProvider + Send + Sync>);

#[async_trait::async_trait]
impl LlmPort for LlmProviderShim {
    async fn send_messages(&self, messages: &[crate::domain::models::Message]) -> Result<String> {
        // Convert domain messages to legacy llm::Message
        let legacy_msgs: Vec<crate::llm::Message> = messages
            .iter()
            .map(|m| crate::llm::Message {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();
        self.0.send_messages(&legacy_msgs).await
    }

    async fn send_messages_with_tools(
        &self,
        messages: &[crate::domain::models::Message],
        tools: Vec<serde_json::Value>,
    ) -> Result<String> {
        let legacy_msgs: Vec<crate::llm::Message> = messages
            .iter()
            .map(|m| crate::llm::Message {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();
        self.0.send_messages_with_tools(&legacy_msgs, tools).await
    }

    async fn get_available_models(&self) -> Result<Vec<String>> {
        self.0.get_available_models().await
    }
}
