//! Port definitions for all outbound interactions.
//!
//! Each port is a Rust `trait` that the Application layer depends on.
//! Concrete implementations live in `adapters/outbound/`.

pub mod browser_port;
pub mod llm_port;
pub mod os_port;
pub mod storage_port;
