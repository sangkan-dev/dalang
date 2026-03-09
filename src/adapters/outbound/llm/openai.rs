use crate::application::ports::llm_port::LlmPort;
use crate::domain::models::{AuthToken, Message};
use anyhow::{Context, Result, anyhow};
use reqwest::{Client, StatusCode, header};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

const MAX_RATE_LIMIT_RETRIES: usize = 2;

fn parse_retry_after_seconds(retry_after_header: Option<&str>, body: &str) -> Option<f64> {
    if let Some(raw) = retry_after_header {
        let trimmed = raw.trim();
        if let Ok(sec) = trimmed.parse::<f64>()
            && sec.is_finite()
            && sec > 0.0
        {
            return Some(sec);
        }
    }

    // Groq typically returns: "Please try again in 13.982s"
    let marker = "try again in ";
    let lower = body.to_lowercase();
    if let Some(start) = lower.find(marker) {
        let rest = &lower[start + marker.len()..];
        if let Some(end) = rest.find('s')
            && let Ok(sec) = rest[..end].trim().parse::<f64>()
            && sec.is_finite()
            && sec > 0.0
        {
            return Some(sec);
        }
    }

    None
}

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

#[derive(Deserialize, Debug)]
struct ModelsResponse {
    models: Option<Vec<ModelItem>>,
    data: Option<Vec<ModelItem>>,
}

#[derive(Deserialize, Debug)]
struct ModelItem {
    name: Option<String>,
    id: Option<String>,
}

pub struct OpenAiCompatibleProvider {
    client: Client,
    base_url: String,
    model: String,
}

impl OpenAiCompatibleProvider {
    pub fn new(base_url: String, model: String, auth: AuthToken) -> Result<Self> {
        let normalized_base_url = base_url.trim().trim_end_matches('/').to_string();
        if normalized_base_url.is_empty() {
            return Err(anyhow!("Base URL is empty for OpenAI-compatible provider"));
        }

        let normalized_model = model.trim().to_string();
        if normalized_model.is_empty() {
            return Err(anyhow!(
                "Model name is empty for OpenAI-compatible provider"
            ));
        }

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        match &auth {
            AuthToken::ApiKey(key) => {
                let key = key.trim();
                let auth_val = format!("Bearer {}", key);
                headers.insert(
                    header::AUTHORIZATION,
                    header::HeaderValue::from_str(&auth_val)
                        .context("Invalid API key format for Authorization header")?,
                );
            }
            AuthToken::Bearer(token) => {
                let token = token.trim();
                let auth_val = format!("Bearer {}", token);
                headers.insert(
                    header::AUTHORIZATION,
                    header::HeaderValue::from_str(&auth_val)
                        .context("Invalid bearer token format for Authorization header")?,
                );
            }
            AuthToken::None => {}
        }

        let client = match Client::builder().default_headers(headers.clone()).build() {
            Ok(client) => client,
            Err(primary_err) => {
                // If system proxy env is malformed, reqwest can fail at build-time.
                Client::builder()
                    .default_headers(headers)
                    .no_proxy()
                    .build()
                    .map_err(|fallback_err| {
                        anyhow!(
                            "Failed to build HTTP client. with-proxy: {}; no-proxy: {}",
                            primary_err,
                            fallback_err
                        )
                    })?
            }
        };

        Ok(Self {
            client,
            base_url: normalized_base_url,
            model: normalized_model,
        })
    }
}

#[async_trait::async_trait]
impl LlmPort for OpenAiCompatibleProvider {
    async fn send_messages(&self, messages: &[Message]) -> Result<String> {
        self.perform_request(messages, None).await
    }

    async fn send_messages_with_tools(
        &self,
        messages: &[Message],
        tools: Vec<serde_json::Value>,
    ) -> Result<String> {
        self.perform_request(messages, Some(tools)).await
    }

    async fn get_available_models(&self) -> Result<Vec<String>> {
        let endpoint = format!("{}/models", self.base_url);
        let response = self
            .client
            .get(&endpoint)
            .send()
            .await
            .with_context(|| format!("Failed requesting model list at {}", endpoint))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to fetch models: {} - {}", status, text));
        }

        let parsed: ModelsResponse = response.json().await?;

        let mut model_names = Vec::new();
        // Handle standard OpenAI `data` array or Gemini's potential `models` array
        let items = parsed.data.or(parsed.models).unwrap_or_default();

        for item in items {
            if let Some(name) = item.id.or(item.name) {
                // Return all models as requested by the user
                // Strip APIs internal prefixes like 'models/' if they exist (common in Gemini API)
                let clean_name = name.strip_prefix("models/").unwrap_or(&name).to_string();
                if !model_names.contains(&clean_name) {
                    model_names.push(clean_name);
                }
            }
        }

        if model_names.is_empty() {
            Err(anyhow!("No models found in the provider response"))
        } else {
            Ok(model_names)
        }
    }
}

impl OpenAiCompatibleProvider {
    async fn perform_request(
        &self,
        messages: &[Message],
        tools: Option<Vec<serde_json::Value>>,
    ) -> Result<String> {
        let endpoint = format!("{}/chat/completions", self.base_url);
        reqwest::Url::parse(&endpoint)
            .with_context(|| format!("Invalid OpenAI-compatible endpoint URL: {}", endpoint))?;

        let req_body = OpenAiRequest {
            model: &self.model,
            messages,
            temperature: Some(0.0),
            tools,
        };

        for attempt in 0..=MAX_RATE_LIMIT_RETRIES {
            let response = self
                .client
                .post(&endpoint)
                .json(&req_body)
                .send()
                .await
                .with_context(|| {
                    format!(
                        "Failed sending LLM request to {} using model {}",
                        endpoint, self.model
                    )
                })?;

            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());
                let text = response.text().await.unwrap_or_default();

                if attempt < MAX_RATE_LIMIT_RETRIES {
                    let wait_secs = parse_retry_after_seconds(retry_after.as_deref(), &text)
                        .unwrap_or(15.0)
                        .clamp(1.0, 60.0);
                    sleep(Duration::from_secs_f64(wait_secs)).await;
                    continue;
                }

                return Err(anyhow!(
                    "LLM request hit rate limit after {} attempt(s) using model {}: {}",
                    MAX_RATE_LIMIT_RETRIES + 1,
                    self.model,
                    text
                ));
            }

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(anyhow!(
                    "LLM request failed with {} using model {}: {}",
                    status,
                    self.model,
                    text
                ));
            }

            let mut parsed: OpenAiResponse = response.json().await?;

            let choice = parsed
                .choices
                .pop()
                .ok_or_else(|| anyhow!("No choices returned by LLM"))?;

            // Priority 1: Check for native tool calls
            if let Some(tool_calls) = choice.message.tool_calls
                && !tool_calls.is_empty()
            {
                let call = &tool_calls[0];
                // Convert native tool call back to JSON string for the Dalang Engine to parse
                // so we don't need to refactor the entire orchestrator loop yet.
                let dalang_json = serde_json::json!({
                    "tool": call.function.name,
                    "args": serde_json::from_str::<serde_json::Value>(&call.function.arguments).unwrap_or_default()
                });
                return Ok(serde_json::to_string(&dalang_json)?);
            }

            // Priority 2: Check for text content (regular response or JSON-in-string)
            if let Some(content) = choice.message.content {
                return Ok(content);
            }

            return Err(anyhow!(
                "Received null content and no tool calls. LLM response is empty."
            ));
        }

        Err(anyhow!(
            "LLM request exhausted retries unexpectedly for model {}",
            self.model
        ))
    }
}
