//! Outbound LLM adapter — creates the correct native `LlmPort` implementation
//! based on `endpoint_mode` configuration.
//!
//! This module is a *factory*. It reads configuration and constructs
//! the appropriate adapter (Copilot, Gemini, or OpenAI API compatible) that
//! directly implements `LlmPort`.

use crate::application::ports::llm_port::LlmPort;
use anyhow::Result;
use std::sync::Arc;

pub use crate::application::ports::llm_port::LlmPort as LlmProvider;
pub use crate::domain::models::{AuthToken, Message};

pub mod copilot;
pub mod gemini_codeassist;
pub mod openai;

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
        "copilot" | "github" | "github-copilot" => "claude-sonnet-4.5".to_string(),
        "ollama" | "local" => "llama3.1:latest".to_string(),
        _ => "gemini-2.5-flash".to_string(),
    }
}

/// Create the appropriate LLM provider adapter based on endpoint mode.
///
/// - `endpoint_mode == "copilot"` → `CopilotProvider`
/// - `endpoint_mode == "cloudcode"` → `GeminiCodeAssistProvider`
/// - anything else → `OpenAiCompatibleProvider`
pub fn create_provider(
    endpoint_mode: &str,
    base_url: String,
    model: String,
    auth: AuthToken,
    codeassist_endpoint: Option<String>,
    gcp_project: Option<String>,
) -> Result<Arc<dyn LlmPort>> {
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
        return Ok(Arc::new(provider));
    }

    if endpoint_mode == "cloudcode" {
        let token = match &auth {
            AuthToken::Bearer(t) => t.clone(),
            AuthToken::ApiKey(t) => t.clone(),
            AuthToken::None => {
                return Err(anyhow::anyhow!("CloudCode mode requires a bearer token"));
            }
        };
        let endpoint = codeassist_endpoint
            .unwrap_or_else(|| "https://cloudcode-pa.googleapis.com".to_string());
        let project = gcp_project.ok_or_else(|| {
            anyhow::anyhow!("CloudCode mode requires a GCP project (run 'dalang login' first)")
        })?;
        let provider =
            gemini_codeassist::GeminiCodeAssistProvider::new(model, token, project, endpoint)?;
        Ok(Arc::new(provider))
    } else {
        let provider = openai::OpenAiCompatibleProvider::new(base_url, model, auth)?;
        Ok(Arc::new(provider))
    }
}
