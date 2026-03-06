//! LLM provider using Gemini Cloud Code Assist endpoints.
//!
//! Inference goes through `cloudcode-pa.googleapis.com/v1internal:generateContent`
//! using the Google-native generateContent format (NOT OpenAI-compatible).
//! This matches how the official Gemini CLI routes OAuth-authenticated requests.

use crate::application::ports::llm_port::LlmPort;
use crate::domain::models::Message;
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Request types (Google generateContent via CloudCode)
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
struct CloudCodeRequest {
    model: String,
    project: String,
    request: InnerGenerateContentRequest,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct InnerGenerateContentRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolSpec>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<FunctionCallPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_response: Option<FunctionResponsePart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FunctionCallPart {
    name: String,
    args: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FunctionResponsePart {
    name: String,
    response: serde_json::Value,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ToolSpec {
    function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Serialize, Clone)]
struct FunctionDeclaration {
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
struct CloudCodeResponse {
    response: Option<InnerResponse>,
    #[serde(rename = "traceId")]
    #[allow(dead_code)]
    trace_id: Option<String>,
}

#[derive(Deserialize, Debug)]
struct InnerResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: Option<CandidateContent>,
}

#[derive(Deserialize, Debug)]
struct CandidateContent {
    parts: Option<Vec<ResponsePart>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ResponsePart {
    text: Option<String>,
    function_call: Option<FunctionCallPart>,
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

/// Fallback model order matching Gemini CLI behavior:
/// Pro models fall back to flash, flash falls back to other flash variants.
const FALLBACK_MODELS: &[&str] = &[
    "gemini-3.1-pro-preview",
    "gemini-3-pro-preview",
    "gemini-3-flash-preview",
    "gemini-2.5-pro",
    "gemini-2.5-flash",
    "gemini-2.5-flash-lite",
];

pub struct GeminiCodeAssistProvider {
    client: Client,
    model: String,
    access_token: Arc<Mutex<String>>,
    project: String,
    codeassist_endpoint: String,
}

impl GeminiCodeAssistProvider {
    pub fn new(
        model: String,
        access_token: String,
        project: String,
        codeassist_endpoint: String,
    ) -> Result<Self> {
        let client = Client::new();
        Ok(Self {
            client,
            model,
            access_token: Arc::new(Mutex::new(access_token)),
            project,
            codeassist_endpoint,
        })
    }

    /// Convert our Message list to Google Content list + optional system instruction.
    fn convert_messages(messages: &[Message]) -> (Vec<Content>, Option<Content>) {
        let mut system_instruction: Option<Content> = None;
        let mut contents: Vec<Content> = Vec::new();

        for msg in messages {
            if msg.role == "system" {
                // Accumulate system messages into one system instruction
                if let Some(ref mut si) = system_instruction {
                    si.parts.push(Part {
                        text: Some(msg.content.clone()),
                        function_call: None,
                        function_response: None,
                    });
                } else {
                    system_instruction = Some(Content {
                        role: "user".to_string(),
                        parts: vec![Part {
                            text: Some(msg.content.clone()),
                            function_call: None,
                            function_response: None,
                        }],
                    });
                }
            } else {
                let role = match msg.role.as_str() {
                    "assistant" => "model",
                    "user" | _ => "user",
                };
                contents.push(Content {
                    role: role.to_string(),
                    parts: vec![Part {
                        text: Some(msg.content.clone()),
                        function_call: None,
                        function_response: None,
                    }],
                });
            }
        }

        (contents, system_instruction)
    }

    /// Convert OpenAI-format tool definitions to Google functionDeclarations.
    fn convert_tools(tools: &[serde_json::Value]) -> Vec<ToolSpec> {
        let mut declarations = Vec::new();
        for tool in tools {
            // OpenAI format: {"type": "function", "function": {"name": "...", "description": "...", "parameters": {...}}}
            if let Some(func) = tool.get("function") {
                let name = func
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let description = func
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let parameters = func.get("parameters").cloned();
                declarations.push(FunctionDeclaration {
                    name,
                    description,
                    parameters,
                });
            }
        }
        vec![ToolSpec {
            function_declarations: declarations,
        }]
    }

    /// Parse "Your quota will reset after Xs." from error body.
    fn parse_retry_after(body: &str) -> Option<u64> {
        // Look for pattern like "reset after 21s" or "after 5s"
        let idx = body.find("after ")?;
        let rest = &body[idx + 6..];
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        num_str.parse().ok()
    }

    /// Extract text content or tool call from the CloudCode response.
    fn parse_response(resp: CloudCodeResponse) -> Result<String> {
        let inner = resp
            .response
            .ok_or_else(|| anyhow!("CloudCode response missing 'response' field"))?;
        let candidates = inner
            .candidates
            .ok_or_else(|| anyhow!("CloudCode response missing 'candidates'"))?;
        let candidate = candidates
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No candidates in CloudCode response"))?;
        let parts = candidate.content.and_then(|c| c.parts).unwrap_or_default();

        // Check for function calls first
        for part in &parts {
            if let Some(ref fc) = part.function_call {
                let dalang_json = serde_json::json!({
                    "tool": fc.name,
                    "args": fc.args
                });
                return Ok(serde_json::to_string(&dalang_json)?);
            }
        }

        // Collect text parts
        let mut text = String::new();
        for part in &parts {
            if let Some(ref t) = part.text {
                text.push_str(t);
            }
        }

        if text.is_empty() {
            Err(anyhow!(
                "CloudCode response has no text content and no function calls"
            ))
        } else {
            Ok(text)
        }
    }

    async fn perform_request(
        &self,
        messages: &[Message],
        tools: Option<Vec<serde_json::Value>>,
    ) -> Result<String> {
        let url = format!(
            "{}/v1internal:generateContent",
            self.codeassist_endpoint.trim_end_matches('/')
        );

        let (contents, system_instruction) = Self::convert_messages(messages);

        let google_tools = tools
            .as_ref()
            .filter(|t| !t.is_empty())
            .map(|t| Self::convert_tools(t));

        // Build list of models to try: primary first, then fallbacks
        let mut models_to_try: Vec<String> = vec![self.model.clone()];
        for fb in FALLBACK_MODELS {
            if *fb != self.model {
                models_to_try.push(fb.to_string());
            }
        }

        let mut last_error = String::new();
        let mut token_refreshed = false;

        for (attempt, model) in models_to_try.iter().enumerate() {
            // Per-model retry loop: handles RATE_LIMIT_EXCEEDED by waiting
            const MAX_RATE_LIMIT_RETRIES: u32 = 3;
            let mut rate_limit_retries = 0;

            loop {
                let req_body = CloudCodeRequest {
                    model: model.clone(),
                    project: self.project.clone(),
                    request: InnerGenerateContentRequest {
                        contents: contents.clone(),
                        system_instruction: system_instruction.clone(),
                        generation_config: Some(GenerationConfig {
                            temperature: Some(0.0),
                            max_output_tokens: None,
                        }),
                        tools: google_tools.clone(),
                    },
                };

                let current_token = self.access_token.lock().await.clone();

                let response = self
                    .client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", current_token))
                    .header("Content-Type", "application/json")
                    .header("User-Agent", "google-api-rust-client/dalang")
                    .header("X-Goog-Api-Client", "gl-rust/dalang")
                    .json(&req_body)
                    .timeout(std::time::Duration::from_secs(120))
                    .send()
                    .await?;

                let status = response.status();

                if status.is_success() {
                    if attempt > 0 {
                        eprintln!(
                            "[!] Model {} unavailable, using fallback: {}",
                            self.model, model
                        );
                    }
                    let parsed: CloudCodeResponse = response.json().await?;
                    return Self::parse_response(parsed);
                }

                let body = response.text().await.unwrap_or_default();

                // Handle 401 Unauthorized — attempt token refresh once
                if status.as_u16() == 401 && !token_refreshed {
                    eprintln!("[!] Access token expired. Attempting refresh...");
                    match crate::auth::gemini_codeassist::refresh_access_token().await {
                        Ok(new_token) => {
                            eprintln!("[+] Token refreshed successfully.");
                            *self.access_token.lock().await = new_token;
                            token_refreshed = true;
                            continue; // Retry with new token
                        }
                        Err(e) => {
                            return Err(anyhow!(
                                "Access token expired and refresh failed: {}. Please re-login with: dalang login --provider gemini",
                                e
                            ));
                        }
                    }
                }

                if status.as_u16() == 429 {
                    // Distinguish RATE_LIMIT_EXCEEDED (wait+retry) from MODEL_CAPACITY_EXHAUSTED (fallback)
                    let is_rate_limit = body.contains("RATE_LIMIT_EXCEEDED");

                    if is_rate_limit && rate_limit_retries < MAX_RATE_LIMIT_RETRIES {
                        // Parse wait time from "Your quota will reset after Xs."
                        let wait_secs = Self::parse_retry_after(&body).unwrap_or(10);
                        eprintln!(
                            "[!] Rate limited on {}. Waiting {}s before retry ({}/{})...",
                            model,
                            wait_secs,
                            rate_limit_retries + 1,
                            MAX_RATE_LIMIT_RETRIES
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(wait_secs + 2)).await;
                        rate_limit_retries += 1;
                        continue; // Retry same model
                    }

                    // MODEL_CAPACITY_EXHAUSTED or rate limit retries exhausted → try next model
                    eprintln!(
                        "[!] Model {} returned 429 ({}), trying next fallback...",
                        model,
                        if is_rate_limit {
                            "rate limit exhausted"
                        } else {
                            "capacity exhausted"
                        }
                    );
                    last_error = format!("LLM request failed with {}: {}", status, body);
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    break; // Move to next model
                }

                // Non-429 error — fail immediately
                return Err(anyhow!("LLM request failed with {}: {}", status, body));
            }
        }

        Err(anyhow!(
            "All models exhausted (429). Last error: {}. Try again later or switch model with: dalang model",
            last_error
        ))
    }
}

#[async_trait::async_trait]
impl LlmPort for GeminiCodeAssistProvider {
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
        // CloudCode doesn't expose a models listing endpoint.
        Err(anyhow!(
            "CloudCode endpoint does not support listing models"
        ))
    }
}
