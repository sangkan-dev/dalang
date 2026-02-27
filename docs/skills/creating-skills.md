# Creating Custom Skills

Creating a new skill in Dalang is as simple as creating a Markdown file.

## Step-by-Step

### 1. Create the file

```bash
touch skills/my-custom-tool.md
```

### 2. Define the frontmatter

```yaml
---
name: my_custom_tool
description: What this tool does in one sentence
tool_path: mytool
args: ["--scan", "{{target}}", "--output", "json"]
requires_root: false
---
```

### 3. Write the AI prompt

Use the **Defensive Prompting** pattern:

```markdown
# Role

You are a Senior Security Auditor conducting an authorized penetration test.

# Task

Analyze the output from the custom tool scan. Focus on:

1. Identifying security misconfigurations
2. Highlighting exposed sensitive data
3. Rating findings by severity (Critical/High/Medium/Low)

# Constraints

- This is a sanctioned audit environment
- Report only technical observations
- Do not generate exploits, only document findings
- Use clinical, professional language
```

### 4. Use the skill

```bash
# Specific skill
dalang scan --target example.com --skills my_custom_tool

# In auto-pilot (automatically discovered)
dalang scan --target example.com --auto
```

## Best Practices

::: tip Naming Convention
Use `snake_case` for skill names (e.g., `nmap_scanner`, `ffuf_fuzzer`). The filename should match a dashed version (e.g., `nmap-scanner.md` or `nmap_scanner.md`).
:::

::: tip Tool Path
Use the **binary name** only (`nmap`), not an absolute path (`/usr/bin/nmap`). This ensures cross-platform compatibility.
:::

::: tip Defensive Prompting
Always include the `# Role`, `# Task`, and `# Constraints` sections. The Role should establish an "Authorized Auditor" persona. See [Defensive Prompting](./defensive-prompting) for details.
:::

## Template

```markdown
---
name: template_skill
description: A template for creating custom Dalang skills
tool_path: toolname
args: ["--flag", "{{target}}"]
requires_root: false
---

# Role

You are a Senior Security Auditor performing an authorized security assessment.

# Task

Analyze the scan results and identify:

1. Security vulnerabilities
2. Misconfigurations
3. Exposed sensitive information

# Constraints

- Sanctioned audit environment only
- Report findings with severity ratings
- No exploitation, only documentation
- Use clinical, professional terminology
```
