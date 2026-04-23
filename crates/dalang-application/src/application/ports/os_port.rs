//! OS Command Executor Port.
//!
//! Defines the contract for safely executing external OS commands.
//! Concrete implementation: `adapters/outbound/os_command/`.

use anyhow::Result;
use async_trait::async_trait;

/// Outbound port for executing OS commands in a safe, sandboxed manner.
///
/// All implementations MUST:
/// - Use array-based argument passing (never `sh -c` string construction)
/// - Enforce a configurable execution timeout
/// - Cap stdout/stderr output size to prevent memory exhaustion
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    /// Execute a command with the given arguments.
    ///
    /// Returns a tuple `(stdout, stderr)` on success.
    async fn execute(
        &self,
        cmd: &str,
        args: &[&str],
        timeout_secs: u64,
    ) -> Result<(String, String)>;
}
