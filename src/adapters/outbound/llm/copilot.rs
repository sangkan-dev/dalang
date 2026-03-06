use crate::application::ports::llm_port::LlmPort;
use crate::domain::models::Message;
use anyhow::{Result, anyhow};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Known Copilot models (from @github/copilot v0.0.420 source, TY array).
/// `claude-opus-4.6-1m` excluded by CLI as premium-only (q7t set).
const COPILOT_MODELS: &[&str] = &[
    "claude-sonnet-4.6",
    "claude-sonnet-4.5",
    "claude-haiku-4.5",
    "claude-opus-4.6",
    "claude-opus-4.6-fast",
    "claude-opus-4.5",
    "claude-sonnet-4",
    "gemini-3-pro-preview",
    "gpt-5.3-codex",
    "gpt-5.2-codex",
    "gpt-5.2",
    "gpt-5.1-codex-max",
    "gpt-5.1-codex",
    "gpt-5.1",
    "gpt-5.1-codex-mini",
    "gpt-5-mini",
    "gpt-4.1",
];

/// The Copilot Internal API base URL
const COPILOT_API_BASE: &str = "https://api.githubcopilot.com";

// ─── Request / Response types ───────────────────────────────

#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: &'a [Message],
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(Deserialize, Debug)]
struct OpenAiResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: OutputMessage,
}

#[derive(Deserialize, Debug)]
struct OutputMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize, Debug)]
struct ToolCall {
    function: FunctionCall,
}

#[derive(Deserialize, Debug)]
struct FunctionCall {
    name: String,
    arguments: String,
}

// ─── CopilotProvider ────────────────────────────────────────

/// GitHub Copilot LLM provider.
///
/// Uses the OAuth token directly with `Authorization: Bearer {token}` to call
/// `api.githubcopilot.com`, exactly like the official @github/copilot CLI does.
pub struct CopilotProvider {
    client: Client,
    model: String,
    /// The long-lived GitHub OAuth token (from Device Flow or keychain)
    github_token: String,
}

impl CopilotProvider {
    pub fn new(model: String, github_token: String) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        // Match exact baseHeaders from CLI source (Dj.baseHeaders)
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "Openai-Intent",
            header::HeaderValue::from_static("conversation-agent"),
        );
        headers.insert("X-Initiator", header::HeaderValue::from_static("user"));
        // Required header per Copilot CLI source (AHs/fHs constants)
        headers.insert(
            "X-GitHub-Api-Version",
            header::HeaderValue::from_static("2025-05-01"),
        );
        // Match CLI defaultHeaders: Copilot-Integration-Id = "copilot-developer-cli"
        headers.insert(
            "Copilot-Integration-Id",
            header::HeaderValue::from_static("copilot-developer-cli"),
        );
        // User-Agent matching CLI format: copilot/{version} ({platform})
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("copilot/0.0.420 (linux)"),
        );

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            model,
            github_token,
        })
    }

    /// Internal request implementation for Copilot API
    async fn perform_copilot_request(
        &self,
        messages: &[Message],
        tools: Option<Vec<serde_json::Value>>,
    ) -> Result<String> {
        self.try_copilot_internal_api(messages, &tools).await
    }

    /// Try the Copilot Internal API (api.githubcopilot.com)
    /// Uses OAuth token directly as Bearer auth, same as @github/copilot CLI.
    async fn try_copilot_internal_api(
        &self,
        messages: &[Message],
        tools: &Option<Vec<serde_json::Value>>,
    ) -> Result<String> {
        let endpoint = format!("{}/chat/completions", COPILOT_API_BASE);
        // Per-request X-Interaction-Id (UUID), same as CLI defaultHeaders
        let interaction_id = Uuid::new_v4().to_string();

        let req_body = OpenAiRequest {
            model: &self.model,
            messages,
            temperature: Some(0.0),
            tools: tools.clone(),
        };

        let response = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", self.github_token))
            .header("X-Interaction-Id", &interaction_id)
            .json(&req_body)
            .send()
            .await?;

        self.parse_openai_response(response).await
    }

    /// Parse a standard OpenAI-compatible response
    async fn parse_openai_response(&self, response: reqwest::Response) -> Result<String> {
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("LLM request failed with {}: {}", status, text));
        }

        let mut parsed: OpenAiResponse = response.json().await?;

        let choice = parsed
            .choices
            .pop()
            .ok_or_else(|| anyhow!("No choices returned by LLM"))?;

        // Priority 1: Native tool calls
        if let Some(tool_calls) = choice.message.tool_calls {
            if !tool_calls.is_empty() {
                let call = &tool_calls[0];
                let dalang_json = serde_json::json!({
                    "tool": call.function.name,
                    "args": serde_json::from_str::<serde_json::Value>(&call.function.arguments)
                        .unwrap_or_default()
                });
                return Ok(serde_json::to_string(&dalang_json)?);
            }
        }

        // Priority 2: Text content
        if let Some(content) = choice.message.content {
            Ok(content)
        } else {
            Err(anyhow!(
                "Received null content and no tool calls. LLM response is empty."
            ))
        }
    }
}

#[async_trait::async_trait]
impl LlmPort for CopilotProvider {
    async fn send_messages(&self, messages: &[Message]) -> Result<String> {
        self.perform_copilot_request(messages, None).await
    }

    async fn send_messages_with_tools(
        &self,
        messages: &[Message],
        tools: Vec<serde_json::Value>,
    ) -> Result<String> {
        self.perform_copilot_request(messages, Some(tools)).await
    }

    async fn get_available_models(&self) -> Result<Vec<String>> {
        Ok(COPILOT_MODELS.iter().map(|s| s.to_string()).collect())
    }
}
