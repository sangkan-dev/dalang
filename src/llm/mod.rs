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
}
