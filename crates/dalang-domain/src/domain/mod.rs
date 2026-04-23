//! Domain layer — pure data types and business rules.
//!
//! This module MUST NOT depend on any external infrastructure (HTTP, file system, OS commands).
//! It only uses the Rust standard library and serialization crates (serde).

pub mod errors;
pub mod models;
pub mod safety;
pub mod scope;
pub mod tool_call;
