# Scan Mode

Scan mode executes specific skills against a target in a controlled ReAct loop.

## Basic Usage

```bash
dalang scan --target <TARGET> --skills <SKILL_LIST>
```

### Parameters

| Parameter            | Short | Default | Description                                              |
| -------------------- | ----- | ------- | -------------------------------------------------------- |
| `--target`           | `-t`  | —       | Target URL or IP address                                 |
| `--skills`           | `-s`  | —       | Comma-separated list of skill names                      |
| `--auto`             | `-a`  | false   | Enable autonomous mode (see [Auto-Pilot](./auto-pilot))  |
| `--max-iter`         | `-n`  | 15      | Max iterations for auto-pilot (0 = unlimited)            |
| `--cmd-timeout`      | —     | 300     | Command execution timeout in seconds (0 = unlimited)     |

### Examples

```bash
# Single skill
dalang scan --target 192.168.1.1 --skills nmap_scanner

# Multiple skills (executed sequentially)
dalang scan --target https://example.com --skills nmap_scanner,web-audit,ffuf_fuzzer

# With custom timeout (10 minutes per command)
dalang scan --target https://example.com --skills nikto_scanner --cmd-timeout 600

# Unlimited command timeout
dalang scan --target https://example.com --skills nikto_scanner --cmd-timeout 0
```

## How It Works

For each skill in the list, Dalang:

1. **Loads** the skill definition from `skills/<name>.md`
2. **Builds** a system prompt with defensive prompting wrapper
3. **Sends** the prompt to the LLM with target context
4. **Processes** the LLM response:
   - If it's a **tool call** → executes the tool and feeds the result back
   - If it's a **text response** → displays as final analysis
5. **Repeats** up to 10 iterations per skill

```
┌────────────────────────────────────────┐
│         For Each Skill:                │
│  ┌──────────┐                          │
│  │  Reason  │ ← LLM thinks            │
│  └────┬─────┘                          │
│       │ JSON tool call?                │
│       ├─── YES → Execute → Observe ─┐ │
│       │                              │ │
│       └─── NO  → Final Response      │ │
│                                      │ │
│       ┌──────────────────────────────┘ │
│       └→ Feed observation back to LLM  │
└────────────────────────────────────────┘
```
