# Scan Mode

Scan mode executes specific skills against a target in a controlled ReAct loop.

## Basic Usage

```bash
dalang scan --target <TARGET> --skills <SKILL_LIST>
```

### Parameters

| Parameter         | Description                                             |
| ----------------- | ------------------------------------------------------- |
| `--target` / `-t` | Target URL or IP address                                |
| `--skills` / `-s` | Comma-separated list of skill names                     |
| `--auto` / `-a`   | Enable autonomous mode (see [Auto-Pilot](./auto-pilot)) |

### Examples

```bash
# Single skill
dalang scan --target 192.168.1.1 --skills nmap_scanner

# Multiple skills (executed sequentially)
dalang scan --target https://example.com --skills nmap_scanner,web-audit,ffuf_fuzzer
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
