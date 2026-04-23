//! Inbound adapter sub-modules.
//!
//! Inbound adapters receive external input (CLI commands, HTTP requests, WebSocket messages)
//! and translate them into calls to the application use cases.
//!
//! # Current Status
//! - `cli`: Delegates to `crate::cli` (Clap definitions remain in place — moving is cosmetic only)
//! - `web`: Delegates to `crate::web` (axum router + WebSocket handlers remain in place)
//!
//! These modules serve as the architectural boundary. As handlers grow, they can be
//! migrated here incrementally without breaking the rest of the codebase.

pub mod cli;
pub mod web;
