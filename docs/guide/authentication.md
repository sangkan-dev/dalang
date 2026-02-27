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

### OAuth (Gemini/Google)

```bash
dalang login --provider gemini
```

This opens your browser for Google OAuth2 authentication. After authorization, tokens are stored securely in your OS keyring.

### API Key (OpenAI / Anthropic)

```bash
dalang login --provider openai
dalang login --provider anthropic
```

You'll be prompted to enter your API key securely (input is hidden). The key is stored in your OS keyring.

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
