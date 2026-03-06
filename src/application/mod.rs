//! Application layer — use cases and ports (interfaces).
//!
//! This layer defines:
//! - **Ports**: `trait` definitions the application needs from the outside world.
//! - **Use Cases**: orchestration logic that uses ports to implement business workflows.
//!
//! Dependencies allowed: `domain` only. No adapters, no axum, no reqwest.

pub mod ports;
pub mod usecases;
