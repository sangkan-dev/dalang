use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod copilot;
pub mod gemini_codeassist;
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
        "gemini" | "google" => {
            "https://generativelanguage.googleapis.com/v1beta/openai".to_string()
        }
        "copilot" | "github" | "github-copilot" => {
            "https://api.githubcopilot.com".to_string()
        }
        "ollama" | "local" => "http://localhost:11434/api".to_string(),
        _ => "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
    }
}

/// Build the Vertex AI OpenAI-compatible base URL for OAuth Bearer tokens.
pub fn get_vertex_base_url(project: &str, region: &str) -> String {
    format!(
        "https://{}-aiplatform.googleapis.com/v1beta1/projects/{}/locations/{}/endpoints/openapi",
        region, project, region
    )
}

pub fn get_default_model(provider: &str) -> String {
    match provider {
        "openai" => "gpt-4o".to_string(),
        "anthropic" => "claude-sonnet-4-20250514".to_string(),
        "gemini" | "google" => "gemini-2.5-flash".to_string(),
        "copilot" | "github" | "github-copilot" => "claude-sonnet-4.6".to_string(),
        "ollama" | "local" => "llama3.1:latest".to_string(),
        _ => "gemini-2.5-flash".to_string(),
    }
}

/// Create the appropriate LLM provider based on endpoint mode.
///
/// - `endpoint_mode == "cloudcode"` → `GeminiCodeAssistProvider`
///   (inference goes through cloudcode-pa.googleapis.com native API)
/// - everything else → `OpenAiCompatibleProvider`
pub fn create_provider(
    endpoint_mode: &str,
    base_url: String,
    model: String,
    auth: AuthToken,
    codeassist_endpoint: Option<String>,
    gcp_project: Option<String>,
) -> Result<Box<dyn LlmProvider + Send + Sync>> {
    if endpoint_mode == "copilot" {
        // GitHub Copilot provider with auto-refreshing session tokens
        let github_token = match &auth {
            AuthToken::Bearer(t) | AuthToken::ApiKey(t) => t.clone(),
            AuthToken::None => {
                return Err(anyhow::anyhow!(
                    "Copilot mode requires a GitHub token (run 'dalang login --provider copilot')"
                ));
            }
        };
        let provider = copilot::CopilotProvider::new(model, github_token)?;
        return Ok(Box::new(provider));
    }

    if endpoint_mode == "cloudcode" {
        let token = match &auth {
            AuthToken::Bearer(t) => t.clone(),
            AuthToken::ApiKey(t) => t.clone(),
            AuthToken::None => {
                return Err(anyhow::anyhow!(
                    "CloudCode mode requires a bearer token"
                ));
            }
        };
        let endpoint = codeassist_endpoint.unwrap_or_else(|| {
            "https://cloudcode-pa.googleapis.com".to_string()
        });
        let project = gcp_project.ok_or_else(|| {
            anyhow::anyhow!("CloudCode mode requires a GCP project (run 'dalang login' first)")
        })?;
        let provider = gemini_codeassist::GeminiCodeAssistProvider::new(
            model, token, project, endpoint,
        )?;
        Ok(Box::new(provider))
    } else {
        let provider = openai::OpenAiCompatibleProvider::new(base_url, model, auth)?;
        Ok(Box::new(provider))
    }
}
