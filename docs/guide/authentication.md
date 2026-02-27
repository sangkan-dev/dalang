# Authentication

Dalang supports multiple authentication methods with a well-defined priority chain.

## Auth Resolution Order

When executing a command, Dalang resolves credentials in this order:

```
CLI Extractor → Keyring → Environment Variable → None
```

| Priority | Source                   | Description                                           |
| -------- | ------------------------ | ----------------------------------------------------- |
| 1        | **CLI Extractor**        | Auto-detects `gcloud` or `gemini-cli` active sessions |
| 2        | **Keyring**              | Tokens/keys stored via `dalang login`                 |
| 3        | **Environment Variable** | `LLM_API_KEY` environment variable                    |
| 4        | **None**                 | Proceeds without auth (will fail on API calls)        |

## Login Methods

### API Key (Default — All Providers)

The simplest and recommended method for all providers:

```bash
# Gemini (Google AI Studio)
dalang login --provider gemini
# → Prompts: Enter your Gemini API Key (from https://aistudio.google.com/apikey)

# OpenAI
dalang login --provider openai
# → Prompts: Enter your OpenAI API Key (from https://platform.openai.com/api-keys)

# Anthropic
dalang login --provider anthropic
# → Prompts: Enter your Anthropic API Key (from https://console.anthropic.com/settings/keys)
```

API keys are stored securely in your OS keyring. After entering the key, you'll be prompted to select your preferred AI model.

### OAuth (Optional — Gemini Only)

If you have a Google Cloud project with OAuth configured:

```bash
dalang login --provider gemini --oauth
```

::: warning
OAuth requires a valid `client_secret` configured in the application. If you get a `client_secret is missing` error, use the API Key method above instead.
:::

### Environment Variable

```bash
export LLM_API_KEY="sk-your-api-key"
```

This method takes precedence over keyring-stored keys when set.

## Dynamic Provider Configuration

After login, Dalang persists your active provider preference. The following values resolve dynamically:

| Setting      | Resolution Order                                        |
| ------------ | ------------------------------------------------------- |
| **Base URL** | `LLM_BASE_URL` env → Provider default                   |
| **Model**    | `LLM_MODEL` env → Keyring preference → Provider default |
| **Provider** | Stored during `dalang login`                            |

### Override with Environment Variables

```bash
# Use a specific model
export LLM_MODEL="gpt-4o"

# Use a custom endpoint (e.g., Azure OpenAI)
export LLM_BASE_URL="https://my-deployment.openai.azure.com/v1"
```

## Keyring Storage

Dalang uses the operating system's native keyring:

| OS      | Backend                                  |
| ------- | ---------------------------------------- |
| Linux   | Secret Service (GNOME Keyring / KWallet) |
| macOS   | Keychain                                 |
| Windows | Credential Manager                       |

All stored items are under the service name `dalang-cli`.
