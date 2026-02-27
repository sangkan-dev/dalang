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
    // TODO: Add support for explicit tools definition here later in Sprint 2
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
        let endpoint = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let req_body = OpenAiRequest {
            model: &self.model,
            messages,
            temperature: Some(0.0), // Low temperature for tool calling predictability
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

        if let Some(content) = choice.message.content {
            Ok(content)
        } else {
            // Note: If the response is a tool call specifically, the 'content' might be null in standard OpenAI.
            // We will handle JSON Tool Calls explicitly soon. For now we assume the tool call is serialized in content.
            Err(anyhow!(
                "Received null content. Direct tool calling structure not yet parsed."
            ))
        }
    }
}
