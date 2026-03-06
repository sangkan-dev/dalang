//! Engine event types for WebSocket streaming.
//!
//! **MIGRATION NOTE (Sprint 2):** The canonical definitions now live in
//! `crate::domain::models`. This module re-exports them so all existing
//! `use crate::web::events::EngineEvent` imports continue to compile unchanged.

// Re-export canonical types from the domain layer.
pub use crate::domain::models::{ClientMessage, EngineEvent};
