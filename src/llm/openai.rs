use super::{AuthToken, LlmProvider, Message};
use anyhow::{Result, anyhow};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};

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
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        match &auth {
            AuthToken::ApiKey(key) => {
                let auth_val = format!("Bearer {}", key);
                headers.insert(
                    header::AUTHORIZATION,
                    header::HeaderValue::from_str(&auth_val)?,
                );
            }
            AuthToken::Bearer(token) => {
                let auth_val = format!("Bearer {}", token);
                headers.insert(
                    header::AUTHORIZATION,
                    header::HeaderValue::from_str(&auth_val)?,
                );
            }
            AuthToken::None => {}
        }

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            base_url,
            model,
        })
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
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
        let endpoint = format!("{}/models", self.base_url.trim_end_matches('/'));
        let response = self.client.get(&endpoint).send().await?;

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
        let endpoint = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let req_body = OpenAiRequest {
            model: &self.model,
            messages,
            temperature: Some(0.0),
            tools,
        };

        let response = self.client.post(&endpoint).json(&req_body).send().await?;

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

        // Priority 1: Check for native tool calls
        if let Some(tool_calls) = choice.message.tool_calls {
            if !tool_calls.is_empty() {
                let call = &tool_calls[0];
                // Convert native tool call back to JSON string for the Dalang Engine to parse
                // so we don't need to refactor the entire orchestrator loop yet.
                let dalang_json = serde_json::json!({
                    "tool": call.function.name,
                    "args": serde_json::from_str::<serde_json::Value>(&call.function.arguments).unwrap_or_default()
                });
                return Ok(serde_json::to_string(&dalang_json)?);
            }
        }

        // Priority 2: Check for text content (regular response or JSON-in-string)
        if let Some(content) = choice.message.content {
            Ok(content)
        } else {
            Err(anyhow!(
                "Received null content and no tool calls. LLM response is empty."
            ))
        }
    }
}
