# Skill System Overview

Dalang's skill system is the core extensibility mechanism. Skills are Markdown files with YAML frontmatter that define security tools and their AI context.

## How Skills Work

```
skills/nmap_scanner.md
        │
        ├── YAML Frontmatter → Parsed by Rust (tool registration)
        │   ├── name, description
        │   ├── tool_path (binary to execute)
        │   └── args (command arguments with {{target}} interpolation)
        │
        └── Markdown Body → Sent to LLM as system prompt
            ├── # Role (AI persona)
            ├── # Task (what to do)
            └── # Constraints (safety boundaries)
```

## Skill File Structure

```markdown
---
name: skill_name
description: What this skill does
tool_path: binary_name
args: ["-flag", "{{target}}"]
requires_root: false
---

# Role

You are a [persona description]...

# Task

[What the AI should do]...

# Constraints

[Safety boundaries and limitations]...
```

## Frontmatter Fields

| Field           | Type   | Required | Description                               |
| --------------- | ------ | -------- | ----------------------------------------- |
| `name`          | String | ✅       | Unique skill identifier                   |
| `description`   | String | ✅       | Human-readable description                |
| `tool_path`     | String | ❌       | Binary name to execute (e.g., `nmap`)     |
| `args`          | List   | ❌       | Arguments with `{{target}}` interpolation |
| `requires_root` | Bool   | ❌       | Whether root/sudo is needed               |

## Target Interpolation

Use `{{target}}` in args to inject the scan target:

```yaml
args: ["-sV", "-T4", "{{target}}"]
# Becomes: nmap -sV -T4 192.168.1.1
```

## Skills Without tool_path

Skills without a `tool_path` (like `web-audit`) use browser tools instead. The AI calls `browser-navigate`, `browser-extract-dom`, and `browser-evaluate-js` based on the skill's task description.
