//! Adapters — concrete implementations of application ports.
//!
//! - **`inbound/`** — drives the app from the outside: CLI (Clap), HTTP + WebSocket (axum). These
//!   are *delivery* mechanisms; they call into `dalang_application::usecases`.
//! - **`outbound/`** — infrastructure: LLM providers, OS command runner, browser/CDP, auth &
//!   session persistence. Each module here implements one or more `dalang_application::ports::*` traits.

pub mod inbound;
pub mod outbound;
