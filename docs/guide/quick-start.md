# Quick Start

Get up and running with Dalang in under 5 minutes.

## 1. Login to an LLM Provider

Dalang supports multiple AI providers. Choose one:

::: code-group

```bash [Gemini (OAuth)]
dalang login --provider gemini
# Opens browser for Google OAuth authentication
```

```bash [OpenAI (API Key)]
dalang login --provider openai
# Prompts for your OpenAI API key securely
```

```bash [Anthropic (API Key)]
dalang login --provider anthropic
# Prompts for your Anthropic API key securely
```

```bash [Environment Variable]
export LLM_API_KEY="sk-your-api-key-here"
# No login required
```

:::

After login, you'll be prompted to select your preferred AI model.

## 2. Run a Simple Scan

```bash
# Scan a target with a specific skill
dalang scan --target 192.168.1.1 --skills nmap_scanner

# Run multiple skills
dalang scan --target https://example.com --skills nmap_scanner,web-audit
```

## 3. Enable Auto-Pilot Mode

Let the AI decide which tools to use:

```bash
dalang scan --target https://example.com --auto
```

The AI will:

1. Analyze the target
2. Select appropriate skills from the library
3. Execute tools and observe results
4. Chain new actions based on observations
5. Generate a vulnerability report

## 4. Interactive Mode

Start a conversational security session:

```bash
dalang interact --target https://example.com
```

Type commands in natural language:

```
dalang> scan for open ports on the target
dalang> check if port 80 has any web vulnerabilities
dalang> look for SQL injection on the login form
dalang> exit
```
