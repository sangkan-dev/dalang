# Architecture Overview

Dalang follows a modular, layered architecture designed for extensibility and security.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI (clap)                           │
│              init │ login │ scan │ interact                 │
├─────────────────────────────────────────────────────────────┤
│                     Core Engine                             │
│  ┌──────────────┐  ┌────────────┐  ┌──────────────────┐    │
│  │ ReAct Loop   │  │ Context    │  │ Safety Filter    │    │
│  │ (Orchestrate)│  │ Memory     │  │ (Refusal Detect) │    │
│  └──────┬───────┘  └────────────┘  └──────────────────┘    │
│         │                                                   │
│    ┌────┴────────────────────────────────────────────┐      │
│    │              Tool Dispatcher                     │     │
│    │  ┌─────────┐  ┌──────────┐  ┌───────────────┐  │     │
│    │  │ os-cmd  │  │ browser-*│  │ execute_skill │  │     │
│    │  └────┬────┘  └────┬─────┘  └───────┬───────┘  │     │
│    └───────┼────────────┼────────────────┼──────────┘      │
├────────────┼────────────┼────────────────┼──────────────────┤
│  ┌─────────┴───┐ ┌──────┴──────┐ ┌──────┴──────────┐      │
│  │  Executor   │ │ CDP Browser │ │ Skills Parser   │      │
│  │  (Safe Cmd) │ │ (Chromium)  │ │ (YAML+Markdown) │      │
│  └─────────────┘ └─────────────┘ └─────────────────┘      │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌────────────────────────────┐      │
│  │  LLM Provider    │  │  Auth                      │      │
│  │  (OpenAI compat) │  │  OAuth │ API Key │ Keyring │      │
│  └──────────────────┘  └────────────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Module Breakdown

| Module               | Path                       | Purpose                                     |
| -------------------- | -------------------------- | ------------------------------------------- |
| **CLI**              | `src/cli.rs`               | Argument parsing with `clap`                |
| **Core Engine**      | `src/core/engine.rs`       | ReAct orchestration, tool dispatch          |
| **Context Memory**   | `src/core/memory.rs`       | Persistent observation tracking             |
| **Safety**           | `src/core/safety.rs`       | AI refusal detection, argument sanitization |
| **Tool Call Parser** | `src/core/tool_call.rs`    | JSON tool call extraction from LLM output   |
| **LLM Provider**     | `src/llm/`                 | OpenAI-compatible API abstraction           |
| **CDP Browser**      | `src/cdp/browser.rs`       | Chromium DevTools Protocol integration      |
| **Executor**         | `src/executor/mod.rs`      | Secure OS command runner                    |
| **Skills Parser**    | `src/skills_parser/mod.rs` | Markdown frontmatter parser                 |
| **Auth**             | `src/auth/`                | OAuth, CLI extraction, keyring persistence  |

## Technology Stack

| Layer              | Technology          |
| ------------------ | ------------------- |
| Language           | Rust (2024 Edition) |
| Async Runtime      | Tokio               |
| HTTP Client        | Reqwest             |
| CLI Framework      | Clap                |
| Browser Automation | Chromiumoxide (CDP) |
| Credential Storage | Keyring             |
| Serialization      | Serde (JSON + YAML) |
