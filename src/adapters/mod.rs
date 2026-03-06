//! Adapters — concrete implementations of application ports.
//!
//! - `inbound/`:  CLI args (Clap), HTTP/WebSocket (axum)
//! - `outbound/`: LLM providers, OS commands, persistence

pub mod inbound;
pub mod outbound;
