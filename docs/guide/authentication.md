# Authentication

Dalang supports multiple authentication methods with a well-defined priority chain.

## Auth Resolution Order

When executing a command, Dalang resolves credentials in this order:

```
Keyring → Environment Variable → CLI Extractor → None
```

| Priority | Source                   | Description                                           |
| -------- | ------------------------ | ----------------------------------------------------- |
| 1        | **Keyring**              | Tokens/keys stored via `dalang login`                 |
| 2        | **Environment Variable** | `LLM_API_KEY` environment variable                    |
| 3        | **CLI Extractor**        | Auto-detects `gcloud` or `gemini-cli` active sessions |
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

### Gemini CLI OAuth (Recommended — Gemini)

Uses the same OAuth flow as Gemini CLI with automatic Cloud Code Assist project discovery:

```bash
dalang login --provider gemini
# → Select "Gemini CLI OAuth (browser login + auto-discover project)"
```

This flow:
1. Opens your browser for Google OAuth consent
2. Exchanges the authorization code for tokens (PKCE)
3. Calls `loadCodeAssist` with multi-endpoint fallback to discover your GCP project
4. Onboards you if no project exists yet
5. Persists the access token, refresh token, and discovered project

::: warning Account Safety
When selecting Gemini CLI OAuth you will see a safety notice. The OAuth tokens grant
access to your Google Cloud resources. Only proceed on trusted machines.
:::

::: tip Environment Override
Set `GOOGLE_CLOUD_PROJECT` to skip the automatic discovery and use a specific project.
Set `DALANG_GEMINI_OAUTH_CLIENT_ID` / `DALANG_GEMINI_OAUTH_CLIENT_SECRET` to use custom OAuth credentials.
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
