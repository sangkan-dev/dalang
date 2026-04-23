//! LLM Provider Port.
//!
//! Defines the contract the application expects from any LLM backend.
//! Concrete implementations: `adapters/outbound/llm/openai.rs`, `gemini.rs`, `copilot.rs`.

use anyhow::Result;
use async_trait::async_trait;
use dalang_domain::domain::models::Message;

/// The primary outbound port for communicating with Large Language Models.
///
/// Any LLM adapter (OpenAI, Gemini, Copilot, Ollama...) must implement this trait
/// to be usable by the orchestrator use case.
#[async_trait]
pub trait LlmPort: Send + Sync {
    /// Sends a conversation history and returns the text response.
    async fn send_messages(&self, messages: &[Message]) -> Result<String>;

    /// Sends messages with explicit tool/function definitions.
    /// Defaults to `send_messages` for providers without native tool calling.
    async fn send_messages_with_tools(
        &self,
        messages: &[Message],
        tools: Vec<serde_json::Value>,
    ) -> Result<String> {
        let _ = tools; // ignore tools for providers that don't support it
        self.send_messages(messages).await
    }

    /// Fetches the list of available models from this provider.
    async fn get_available_models(&self) -> Result<Vec<String>> {
        Err(anyhow::anyhow!(
            "get_available_models not implemented for this provider"
        ))
    }
}

// ── Helper types used by the LLM factory ─────────────────────────────────────

/// Returns the default API base URL for a given provider name.
pub fn get_default_base_url(provider: &str) -> String {
    match provider {
        "openai" => "https://api.openai.com/v1".to_string(),
        "anthropic" => "https://api.anthropic.com/v1".to_string(),
        "gemini" | "google" => {
            "https://generativelanguage.googleapis.com/v1beta/openai".to_string()
        }
        "copilot" | "github" | "github-copilot" => "https://api.githubcopilot.com".to_string(),
        "ollama" | "local" => "http://localhost:11434/api".to_string(),
        _ => "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
    }
}

/// Returns the default model name for a given provider.
pub fn get_default_model(provider: &str) -> String {
    match provider {
        "openai" => "gpt-4o".to_string(),
        "anthropic" => "claude-sonnet-4-20250514".to_string(),
        "gemini" | "google" => "gemini-2.5-flash".to_string(),
        "copilot" | "github" | "github-copilot" => "claude-sonnet-4.5".to_string(),
        "ollama" | "local" => "llama3.1:latest".to_string(),
        _ => "gemini-2.5-flash".to_string(),
    }
}

/// Build the Vertex AI OpenAI-compatible base URL for OAuth Bearer tokens.
pub fn get_vertex_base_url(project: &str, region: &str) -> String {
    format!(
        "https://{}-aiplatform.googleapis.com/v1beta1/projects/{}/locations/{}/endpoints/openapi",
        region, project, region
    )
}
