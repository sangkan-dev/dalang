//! OS Command Executor Adapter.
//!
//! Implements the `CommandExecutor` port using `tokio::process::Command`
//! with strict security rules from DEV_RULES.md (no shell, timeout, output cap).

use anyhow::Result;
use async_trait::async_trait;
use dalang_application::application::ports::os_port::CommandExecutor;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

/// Maximum stdout/stderr bytes to avoid memory exhaustion.
const STD_OUTPUT_LIMIT: usize = 1024 * 1024 * 5; // 5 MB

/// The concrete OS command executor.
///
/// Uses array-based argument passing via `Command::new(cmd).args(args)` —
/// NEVER uses `sh -c` or shell string interpolation.
pub struct OsCommandExecutor;

#[async_trait]
impl CommandExecutor for OsCommandExecutor {
    async fn execute(
        &self,
        cmd: &str,
        args: &[&str],
        timeout_secs: u64,
    ) -> Result<(String, String)> {
        // SECURITY: We never use `sh -c` here. Each argument is passed separately
        // to the OS via execve(), preventing shell injection from AI-generated args.
        let mut command = Command::new(cmd);
        command.args(args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        match timeout(Duration::from_secs(timeout_secs), command.output()).await {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if stdout.len() > STD_OUTPUT_LIMIT {
                    return Err(anyhow::anyhow!("Stdout exceeded 5MB limit"));
                }
                if stderr.len() > STD_OUTPUT_LIMIT {
                    return Err(anyhow::anyhow!("Stderr exceeded 5MB limit"));
                }

                Ok((stdout, stderr))
            }
            Ok(Err(e)) => Err(anyhow::anyhow!("Failed to execute process: {}", e)),
            Err(_) => Err(anyhow::anyhow!(
                "Command execution timed out after {} seconds",
                timeout_secs
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_valid_execution() {
        let ex = OsCommandExecutor;
        let (stdout, _) = ex.execute("echo", &["helloworld"], 2).await.unwrap();
        assert_eq!(stdout.trim(), "helloworld");
    }

    #[tokio::test]
    async fn test_timeout() {
        let ex = OsCommandExecutor;
        let result = ex.execute("sleep", &["3"], 1).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn test_invalid_command() {
        let ex = OsCommandExecutor;
        let result = ex.execute("nonexistent_command_12345", &[], 1).await;
        assert!(result.is_err());
    }
}
