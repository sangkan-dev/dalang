//! CLI inbound adapter.
//!
//! This module is the architectural boundary for CLI-triggered use cases.
//! Clap argument definitions live in `crate::cli`. Command handlers live in `crate::main`.
//!
//! As CLI command handlers grow larger, they can be extracted here.
//! For now, this module re-exports the Clap types and exposes a `CliInput` type
//! that command handlers can use as a clean contract.

pub use crate::cli::{Commands, DalangArgs};
