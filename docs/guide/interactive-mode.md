# Interactive Mode

Interactive mode provides a conversational REPL where you collaborate with the AI on security assessments.

## Usage

```bash
dalang interact --target <TARGET>
```

## How It Works

Interactive mode starts a `dalang>` prompt where you type natural-language requests. The AI:

1. Interprets your request in the security context
2. Decides whether to reason, call a tool, or respond directly
3. Executes tools and presents results
4. Maintains context memory across the session

## Session Commands

| Command  | Description                               |
| -------- | ----------------------------------------- |
| Any text | Send a natural-language request to the AI |
| `exit`   | End the interactive session               |
| `quit`   | End the interactive session               |

## Example Session

```
[*] Starting Interactive Human-in-the-Loop Session...
[*] Target: https://example.com
[*] Loaded 16 skills into catalog.
[*] Type 'exit' or 'quit' to end session.

dalang> find all open ports on the target
[...] Strategic Reasoning...
[>] Assistant wants to execute skill: nmap_scanner
    $ nmap -sV -T4 example.com
[<] Observation received (1832 bytes)

[✓] Assistant:
Based on the nmap scan results, I found the following open ports:
- Port 22 (SSH) - OpenSSH 8.9
- Port 80 (HTTP) - nginx 1.24.0
- Port 443 (HTTPS) - nginx 1.24.0

dalang> check for web vulnerabilities on port 80

dalang> exit
[*] Ending session. Goodbye!
```

::: tip Human-in-the-Loop
Interactive mode is ideal when you want to guide the AI's investigation while retaining full control over what gets executed.
:::
