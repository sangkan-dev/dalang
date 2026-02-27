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

| Provider         | Base URL                                   | Auth Method  | Default Model       |
| ---------------- | ------------------------------------------ | ------------ | ------------------- |
| **Gemini**       | `generativelanguage.googleapis.com/v1beta` | OAuth Bearer | `gemini-1.5-pro`    |
| **OpenAI**       | `api.openai.com/v1`                        | API Key      | `gpt-4o`            |
| **Anthropic**    | `api.anthropic.com/v1`                     | API Key      | `claude-3-5-sonnet` |
| **Ollama/Local** | `localhost:11434/api`                      | None         | `llama3.1:latest`   |

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
