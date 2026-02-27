use anyhow::{Result, anyhow};
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

const STD_OUTPUT_LIMIT: usize = 1024 * 1024 * 5; // 5 MB output limit

/// Execute OS command safely by parsing arguments individually to prevent shell injection.
/// Provides a hard timeout to avoid hanging processes.
pub async fn execute_safe_command(
    cmd: &str,
    args: &[&str],
    timeout_secs: u64,
) -> Result<(String, String)> {
    // SECURITY RULE: strictly avoid `sh -c` or `cmd /c` here.
    // Argument list is passed directly to the OS APIs ensuring safe execution.
    let mut command = Command::new(cmd);
    command.args(args);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let child_future = command.output();

    match timeout(Duration::from_secs(timeout_secs), child_future).await {
        Ok(Ok(output)) => {
            let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

            if stdout_str.len() > STD_OUTPUT_LIMIT {
                return Err(anyhow!("Stdout exceeded limit of 5MB"));
            }

            if stderr_str.len() > STD_OUTPUT_LIMIT {
                return Err(anyhow!("Stderr exceeded limit of 5MB"));
            }

            Ok((stdout_str, stderr_str))
        }
        Ok(Err(e)) => Err(anyhow!("Failed to execute process: {}", e)),
        Err(_) => Err(anyhow!(
            "Command execution timed out after {} seconds",
            timeout_secs
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_valid_execution() {
        let (stdout, _) = execute_safe_command("echo", &["helloworld"], 2)
            .await
            .unwrap();
        assert_eq!(stdout.trim(), "helloworld");
    }

    #[tokio::test]
    async fn test_timeout_execution() {
        // 'sleep 3' should exceed a 1-second timeout
        let result = execute_safe_command("sleep", &["3"], 1).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("timed out"));
    }

    #[tokio::test]
    async fn test_invalid_command() {
        let result = execute_safe_command("nonexistentcommand_random", &[], 1).await;
        assert!(result.is_err());
    }
}
