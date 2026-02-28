# Core Engine

The core engine (`src/core/engine.rs`) is the heart of Dalang — the "Dalang" (puppet master) itself.

## DalangEngine

```rust
pub struct DalangEngine {
    llm: Box<dyn LlmProvider + Send + Sync>,
    cmd_timeout: u64,
}
```

The engine holds a reference to the active LLM provider and a configurable command timeout (in seconds, 0 = unlimited). It exposes three main loops:

| Method                   | Description                          |
| ------------------------ | ------------------------------------ |
| `run_scan_loop()`        | Execute specific skills sequentially       |
| `run_autonomous_loop()`  | AI-driven meta-orchestration (configurable iteration limit) |
| `run_interactive_loop()` | Human-in-the-loop REPL                     |

## ReAct Loop

All three modes implement the **ReAct** (Reasoning + Acting) pattern:

1. **Reason**: LLM receives context and produces a response
2. **Act**: If the response contains a JSON tool call, execute it
3. **Observe**: Feed the tool's output back to the LLM
4. **Repeat**: Until the LLM produces a final text response or iteration limit

## Lazy Browser Initialization

The browser is **not** launched at startup. Instead, a `LazyBrowser` wrapper defers Chrome initialization until the first `browser-*` tool call:

```rust
struct LazyBrowser {
    inner: Arc<Mutex<Option<DalangBrowser>>>,
}
```

This ensures CLI-only skills (nmap, sqlmap) work even without Chrome installed.

## Shared Helper Methods

| Helper                           | Purpose                                                               |
| -------------------------------- | --------------------------------------------------------------------- |
| `handle_browser_tool()`          | Dispatches browser-navigate, browser-evaluate-js, browser-extract-dom |
| `execute_skill_native()`         | Runs a skill's tool_path with interpolated args + timeout             |
| `handle_os_command()`            | Executes raw os-command tool calls (30s safety timeout)               |
| `build_execute_skill_tool_def()` | Builds the JSON schema for native tool calling                        |
| `save_report()`                  | Writes vulnerability report to file                                   |
| `effective_timeout()`            | Resolves cmd_timeout (0 → unlimited)                                  |

## Context Memory

The `ContextManager` (`src/core/memory.rs`) maintains a sliding window of the last 20 observations:

```rust
pub struct ContextManager {
    memory: Vec<String>, // Last 20 observations (FIFO)
}
```

Memory is injected into the LLM prompt before each reasoning step.

## Safety Filter

The `safety.rs` module detects AI refusal patterns (16 patterns) and triggers re-prompting with the "Sanctioned Audit" override.
