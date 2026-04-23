use anyhow::{Result, anyhow};
use dalang_application::application::ports::llm_port::LlmPort;
use dalang_domain::domain::models::Message;
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

fn preview_body(body: &str, max: usize) -> String {
    let trimmed = body.trim();
    if trimmed.len() <= max {
        trimmed.to_string()
    } else {
        format!("{}...", &trimmed[..max])
    }
}

fn extract_last_sse_json_chunk(body: &str) -> Option<String> {
    let mut last: Option<String> = None;

    for line in body.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("data:") {
            continue;
        }

        let payload = trimmed.trim_start_matches("data:").trim();
        if payload.is_empty() || payload == "[DONE]" {
            continue;
        }

        if payload.starts_with('{') {
            last = Some(payload.to_string());
        }
    }

    last
}

fn parse_openai_body(body: &str) -> Result<OpenAiResponse> {
    if let Ok(parsed) = serde_json::from_str::<OpenAiResponse>(body) {
        return Ok(parsed);
    }

    if let Some(json_chunk) = extract_last_sse_json_chunk(body)
        && let Ok(parsed) = serde_json::from_str::<OpenAiResponse>(&json_chunk)
    {
        return Ok(parsed);
    }

    Err(anyhow!(
        "Failed to decode Copilot JSON response. Body preview: {}",
        preview_body(body, 400)
    ))
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
    /// If true, use CAPI messages endpoint (/chat/completions/{model}/messages)
    use_capi_messages: bool,
}

impl CopilotProvider {
    pub fn new(model: String, github_token: String, use_capi_messages: bool) -> Result<Self> {
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
            use_capi_messages,
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

        // Copilot CLI uses `capi:<model>` for some newer models (eg gpt-5.3-codex).
        // When `use_capi_messages` is enabled, we always send the `capi:`-prefixed model id.
        let model_id = self.model.trim();
        let capi_model = if self.use_capi_messages && !model_id.starts_with("capi:") {
            format!("capi:{}", model_id)
        } else {
            model_id.to_string()
        };

        let req_body = OpenAiRequest {
            model: &capi_model,
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

        let body = response.text().await?;
        let mut parsed = parse_openai_body(&body)?;

        let choice = parsed
            .choices
            .pop()
            .ok_or_else(|| anyhow!("No choices returned by LLM"))?;

        // Priority 1: Native tool calls
        if let Some(tool_calls) = choice.message.tool_calls
            && !tool_calls.is_empty()
        {
            let call = &tool_calls[0];
            let dalang_json = serde_json::json!({
                "tool": call.function.name,
                "args": serde_json::from_str::<serde_json::Value>(&call.function.arguments)
                    .unwrap_or_default()
            });
            return Ok(serde_json::to_string(&dalang_json)?);
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
