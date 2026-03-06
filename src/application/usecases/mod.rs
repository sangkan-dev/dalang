//! Use case layer — business logic orchestration.
//!
//! Each use case calls ports (traits) to interact with the outside world
//! without depending on concrete adapter implementations.

pub mod orchestrator;
