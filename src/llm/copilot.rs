use super::{LlmProvider, Message};
use crate::auth::copilot::{exchange_copilot_session_token, CopilotSessionToken};
use anyhow::{Result, anyhow};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Known Copilot models with billing info (from @github/copilot v0.0.420 source)
const COPILOT_MODELS: &[&str] = &[
    "claude-sonnet-4.6",
    "claude-sonnet-4.5",
    "claude-haiku-4.5",
    "claude-opus-4.6",
    "gpt-5.2",
    "gpt-4.1",
    "gemini-3-pro-preview",
];

/// The Copilot Internal API base URL
const COPILOT_API_BASE: &str = "https://api.githubcopilot.com";

/// The GitHub Models API base URL (fallback)
const GITHUB_MODELS_BASE: &str = "https://models.github.ai/inference";

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

// ─── Cached Session State ───────────────────────────────────

struct CopilotSession {
    /// The long-lived GitHub OAuth token
    github_token: String,
    /// Cached short-lived Copilot session token
    session_token: Option<CopilotSessionToken>,
}

// ─── CopilotProvider ────────────────────────────────────────

pub struct CopilotProvider {
    client: Client,
    model: String,
    session: Arc<Mutex<CopilotSession>>,
    /// Whether to use GitHub Models API as fallback
    use_models_api_fallback: bool,
}

impl CopilotProvider {
    pub fn new(model: String, github_token: String) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("GithubCopilot/1.155.0"),
        );
        headers.insert(
            "editor-version",
            header::HeaderValue::from_static("dalang/0.1.0"),
        );
        headers.insert(
            "Copilot-Integration-Id",
            header::HeaderValue::from_static("dalang"),
        );

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            model,
            session: Arc::new(Mutex::new(CopilotSession {
                github_token,
                session_token: None,
            })),
            use_models_api_fallback: true,
        })
    }

    /// Ensure we have a valid Copilot session token, refreshing if expired.
    async fn ensure_session_token(&self) -> Result<String> {
        let mut session = self.session.lock().await;

        // Check if we have a cached token that hasn't expired (with 5-min buffer)
        if let Some(ref st) = session.session_token {
            let now = chrono::Utc::now().timestamp();
            if now < st.expires_at - 300 {
                return Ok(st.token.clone());
            }
        }

        // Exchange the GitHub token for a new Copilot session token
        let new_token = exchange_copilot_session_token(&session.github_token).await?;
        let token_str = new_token.token.clone();
        session.session_token = Some(new_token);

        Ok(token_str)
    }

    /// Internal request implementation for Copilot API
    async fn perform_copilot_request(
        &self,
        messages: &[Message],
        tools: Option<Vec<serde_json::Value>>,
    ) -> Result<String> {
        // Try Copilot Internal API first
        match self.try_copilot_internal_api(messages, &tools).await {
            Ok(resp) => return Ok(resp),
            Err(e) => {
                if self.use_models_api_fallback {
                    eprintln!(
                        "[!] Copilot Internal API failed ({}), trying GitHub Models API fallback...",
                        e
                    );
                    return self.try_github_models_api(messages, &tools).await;
                }
                return Err(e);
            }
        }
    }

    /// Try the Copilot Internal API (api.githubcopilot.com)
    async fn try_copilot_internal_api(
        &self,
        messages: &[Message],
        tools: &Option<Vec<serde_json::Value>>,
    ) -> Result<String> {
        let session_token = self.ensure_session_token().await?;

        let endpoint = format!("{}/chat/completions", COPILOT_API_BASE);

        let req_body = OpenAiRequest {
            model: &self.model,
            messages,
            temperature: Some(0.0),
            tools: tools.clone(),
        };

        let response = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", session_token))
            .header("openai-intent", "conversation-panel")
            .json(&req_body)
            .send()
            .await?;

        self.parse_openai_response(response).await
    }

    /// Try the GitHub Models API (models.github.ai) as fallback
    async fn try_github_models_api(
        &self,
        messages: &[Message],
        tools: &Option<Vec<serde_json::Value>>,
    ) -> Result<String> {
        let github_token = {
            let session = self.session.lock().await;
            session.github_token.clone()
        };

        let endpoint = format!("{}/chat/completions", GITHUB_MODELS_BASE);

        let req_body = OpenAiRequest {
            model: &self.model,
            messages,
            temperature: Some(0.0),
            tools: tools.clone(),
        };

        let response = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", github_token))
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
impl LlmProvider for CopilotProvider {
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
