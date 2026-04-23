//! Engine event types for WebSocket streaming.
//!
//! Domain definitions live in `dalang_domain::domain::models`. The wire JSON shape is
//! [`crate::WsEngineEvent`](crate::WsEngineEvent) (see mapping in the web handler and session persistence).

pub use dalang_domain::domain::models::{ClientMessage, EngineEvent};

pub use crate::WsEngineEvent;
