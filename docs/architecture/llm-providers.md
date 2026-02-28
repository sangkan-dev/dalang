# LLM Providers

Dalang uses an OpenAI-compatible API abstraction that works with multiple LLM providers.

## Provider Trait

```rust
#[async_trait]
pub trait LlmProvider {
    async fn send_messages(&self, messages: &[Message]) -> Result<String>;
    async fn send_messages_with_tools(
        &self, messages: &[Message], tools: Vec<serde_json::Value>
    ) -> Result<String>;
    async fn get_available_models(&self) -> Result<Vec<String>>;
}
```

## Supported Providers

| Provider              | Base URL                                       | Auth Method       | Default Model       |
| --------------------- | ---------------------------------------------- | ----------------- | ------------------- |
| **Gemini**            | `generativelanguage.googleapis.com/v1beta`     | API Key / Bearer  | `gemini-2.5-flash`  |
| **Gemini CloudCode**  | `cloudcode-pa.googleapis.com/v1internal`       | Bearer (OAuth)    | `gemini-2.5-flash`  |
| **OpenAI**            | `api.openai.com/v1`                            | API Key           | `gpt-4o`            |
| **Anthropic**         | `api.anthropic.com/v1`                         | API Key           | `claude-3-5-sonnet` |
| **Ollama/Local**      | `localhost:11434/api`                          | None              | `llama3.1:latest`   |

## Endpoint Modes

Dalang supports two endpoint modes for Gemini:

| Mode             | Description                                                              |
| ---------------- | ------------------------------------------------------------------------ |
| `openai_compat`  | Standard OpenAI-compatible endpoint (API key, CLI extract, or gcloud)    |
| `cloudcode`      | Gemini CLI OAuth flow with Cloud Code Assist project discovery           |

In **cloudcode** mode, the `GeminiCodeAssistProvider` uses Google's native `generateContent` API
format (not OpenAI-compatible) with Bearer token obtained via the Gemini CLI OAuth flow. Inference
goes to `cloudcode-pa.googleapis.com/v1internal:generateContent` with the discovered GCP project.
The provider automatically handles 429 errors with model fallback (RATE_LIMIT_EXCEEDED → wait &
retry, MODEL_CAPACITY_EXHAUSTED → fall back to next model in chain).

### Cloud Code Assist Discovery

When logging in via Gemini CLI OAuth, dalang calls `loadCodeAssist` with a multi-endpoint
fallback strategy (prod → daily → autopush) to discover the user's GCP project. If discovery
fails, it falls back to the `GOOGLE_CLOUD_PROJECT` environment variable.

## OpenAI-Compatible Provider

The `OpenAiCompatibleProvider` handles all providers through the OpenAI chat completions API format:

```
POST {base_url}/chat/completions
```

### Native Tool Calling

When the LLM returns a native tool call (via `tool_calls` in the API response), Dalang converts it back to its internal JSON format:

```json
{
  "tool": "execute_skill",
  "args": { "skill_name": "nmap_scanner", "reasoning": "..." }
}
```

This allows seamless integration between providers that support native function calling and the Dalang engine's tool dispatch logic.

## Auth Token Types

```rust
pub enum AuthToken {
    ApiKey(String),   // x-api-key header
    Bearer(String),   // Authorization: Bearer
    None,             // No authentication
}
```

Token type is determined by the active provider:

- **OpenAI / Anthropic** → `ApiKey`
- **Gemini / Google** → `Bearer`
