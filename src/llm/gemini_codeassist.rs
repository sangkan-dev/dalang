//! LLM provider using Gemini Cloud Code Assist endpoints.
//!
//! This provider hits the `/v1internal:generateContent` style endpoints
//! on the cloudcode-pa endpoint family. However, after project discovery,
//! actual inference goes through the standard Gemini generativelanguage
//! OpenAI-compatible endpoint using the OAuth bearer token.
//!
//! The CloudCode endpoints are primarily for auth/onboarding/project discovery.
//! Once we have a valid bearer token + project, inference uses the same
//! OpenAI-compatible endpoint as API key mode, just with Bearer auth.

use super::{AuthToken, LlmProvider, Message};
use super::openai::OpenAiCompatibleProvider;
use anyhow::Result;

/// Provider that wraps `OpenAiCompatibleProvider` but uses bearer token
/// obtained from Gemini CLI OAuth flow. The actual inference endpoint
/// is the standard Gemini generativelanguage OpenAI-compatible API.
///
/// The distinction from raw `OpenAiCompatibleProvider` is:
/// - Auth is always Bearer (OAuth token)
/// - Can attempt token refresh on 401
/// - Tracks the CloudCode endpoint for future discovery/onboard calls
pub struct GeminiCodeAssistProvider {
    inner: OpenAiCompatibleProvider,
    #[allow(dead_code)]
    codeassist_endpoint: String,
}

impl GeminiCodeAssistProvider {
    pub fn new(
        model: String,
        access_token: String,
        codeassist_endpoint: String,
    ) -> Result<Self> {
        // Inference goes through standard Gemini OpenAI-compatible endpoint
        let base_url =
            "https://generativelanguage.googleapis.com/v1beta/openai".to_string();
        let auth = AuthToken::Bearer(access_token);
        let inner = OpenAiCompatibleProvider::new(base_url, model, auth)?;
        Ok(Self {
            inner,
            codeassist_endpoint,
        })
    }
}

#[async_trait::async_trait]
impl LlmProvider for GeminiCodeAssistProvider {
    async fn send_messages(&self, messages: &[Message]) -> Result<String> {
        self.inner.send_messages(messages).await
    }

    async fn send_messages_with_tools(
        &self,
        messages: &[Message],
        tools: Vec<serde_json::Value>,
    ) -> Result<String> {
        self.inner.send_messages_with_tools(messages, tools).await
    }

    async fn get_available_models(&self) -> Result<Vec<String>> {
        self.inner.get_available_models().await
    }
}
