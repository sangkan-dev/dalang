//! Web inbound adapter.
//!
//! This module is the architectural boundary for HTTP/WebSocket-triggered use cases.
//! The axum router and WebSocket handlers live in `crate::web`.
//!
//! This adapter re-exports the public web server API so consumers can depend on
//! `adapters::inbound::web` instead of `crate::web` directly.

pub use crate::web::{build_router, start_server};
