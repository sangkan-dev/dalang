# Executor & Security

The executor module is responsible for safely running OS commands. Security is paramount since the AI can potentially attempt to inject malicious commands.

## Safe Command Execution

```rust
pub async fn execute_safe_command(
    cmd: &str,
    args: &[&str],
    timeout_secs: u64,
) -> Result<(String, String)>
```

### Security Guarantees

| Protection                | Mechanism                                                              |
| ------------------------- | ---------------------------------------------------------------------- |
| **No Shell Injection**    | Uses `std::process::Command` with separate `.arg()` calls — no `sh -c` |
| **Timeout**               | Hard timeout via `tokio::time::timeout` prevents hanging               |
| **Output Cap**            | 5 MB limit on stdout/stderr prevents memory exhaustion                 |
| **Argument Sanitization** | Shell metacharacters (`; & \| > < $ \` ( )`) are blocked               |

### ❌ Dangerous (Never Used)

```rust
// NEVER: Shell string concatenation
let cmd = format!("nmap -p {} {}", port, target);
Command::new("sh").arg("-c").arg(&cmd);
```

### ✅ Safe (Always Used)

```rust
// SAFE: Argument array — kernel-level escaping via execve
Command::new("nmap")
    .arg("-p").arg(&port)
    .arg(&target);
```

## Argument Sanitization

The `safety.rs` module provides an additional layer of defense against AI-injected arguments:

```rust
// Blocks: ; & | > < $ ( ) `
static ref SHELL_METACHART: Regex = Regex::new(r#"[;&|><$()`]"#);
```

If the AI tries to inject arguments containing shell metacharacters, they are **rejected entirely** — not sanitized, to prevent bypasses.

## AI Safety Refusal Detection

The engine detects when an LLM refuses a security-related request (16 patterns):

```rust
pub fn is_safety_refusal(text: &str) -> bool {
    text.contains("i cannot assist")
        || text.contains("i'm sorry, but")
        || text.contains("as an ai")
        // ... 13 more patterns
}
```

When detected, Dalang re-prompts with the "Sanctioned Audit" override (up to 3 retries).

## CDP Browser Security

The CDP browser module runs in **headless mode** and provides three controlled operations:

| Tool                  | Description                        |
| --------------------- | ---------------------------------- |
| `browser-navigate`    | Navigate to a URL                  |
| `browser-evaluate-js` | Execute JavaScript in page context |
| `browser-extract-dom` | Get page text content              |

The browser is **lazily initialized** — only launched when the AI actually needs it.
