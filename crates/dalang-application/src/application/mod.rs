//! Application layer — use cases and ports (interfaces).
//!
//! This layer defines:
//! - **Ports**: `trait` definitions the application needs from the outside world.
//! - **Use Cases**: orchestration logic that uses ports to implement business workflows.
//!
//! Dependencies allowed: `domain` only. No adapters, no axum, no reqwest.
//!
//! ## Naming
//!
//! - **`ports/`** — outbound trait contracts (`BrowserPort`, `LlmPort`, …). The codebase uses
//!   the term **port** (not “interface”) for these traits.
//! - **`usecases/`** — application services (orchestrator, auth helpers, …).
//! - Concrete **adapters** live in the `dalang-adapters` crate: **inbound** (CLI, HTTP/WebSocket)
//!   drives use cases; **outbound** implements ports (LLM, OS, browser, persistence).

pub(crate) mod browser_tool_dispatch;
pub mod ports;
pub mod usecases;
