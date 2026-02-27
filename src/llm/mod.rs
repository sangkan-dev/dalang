use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod openai;

#[derive(Debug, Clone)]
pub enum AuthToken {
    /// No authentication (e.g., local Ollama)
    None,
    /// API Key authentication (e.g., Anthropic, Gemini Key)
    ApiKey(String),
    /// Bearer token for OAuth/JWT (e.g., Gemini OAuth, expected standard Bearer)
    Bearer(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn system(content: &str) -> Self {
        Self {
            role: "system".to_string(),
            content: content.to_string(),
        }
    }

    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
        }
    }
}

/// Abstract provider trait to support multiple LLM backends (OpenAI, Anthropic, Gemini, Ollama, etc)
#[async_trait::async_trait]
pub trait LlmProvider {
    /// Sends a conversation history and returns the text response (or JSON tool call string).
    async fn send_messages(&self, messages: &[Message]) -> Result<String>;

    /// Sends a conversation history with explicit tool definitions.
    /// Default implementation falls back to regular send_messages.
    async fn send_messages_with_tools(
        &self,
        messages: &[Message],
        _tools: Vec<serde_json::Value>,
    ) -> Result<String> {
        self.send_messages(messages).await
    }

    /// Fetches the list of available models from the provider.
    async fn get_available_models(&self) -> Result<Vec<String>> {
        Err(anyhow::anyhow!(
            "get_available_models not implemented for this provider"
        ))
    }
}

pub fn get_default_base_url(provider: &str) -> String {
    match provider {
        "openai" => "https://api.openai.com/v1".to_string(),
        "anthropic" => "https://api.anthropic.com/v1".to_string(),
        "gemini" | "google" => "https://generativelanguage.googleapis.com/v1beta".to_string(),
        "ollama" | "local" => "http://localhost:11434/api".to_string(),
        _ => "https://generativelanguage.googleapis.com/v1beta".to_string(),
    }
}

pub fn get_default_model(provider: &str) -> String {
    match provider {
        "openai" => "gpt-4o".to_string(),
        "anthropic" => "claude-3-5-sonnet-20241022".to_string(),
        "gemini" | "google" => "gemini-1.5-pro".to_string(),
        "ollama" | "local" => "llama3.1:latest".to_string(),
        _ => "gemini-1.5-pro".to_string(),
    }
}
