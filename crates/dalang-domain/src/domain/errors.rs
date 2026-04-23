//! Domain-level error types.
//!
//! Using `thiserror` for structured, specific error variants that describe
//! *what went wrong* at the business logic level, without leaking infrastructure details.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DalangError {
    // ── Skill & Parsing ──────────────────────────────────────────────────────
    #[error("Skill '{0}' not found in the skills directory")]
    SkillNotFound(String),

    #[error("Failed to parse skill file '{0}': {1}")]
    SkillParseError(String, String),

    #[error("Skill '{0}' requires root privileges but the process is not running as root")]
    RequiresRoot(String),

    #[error("Tool binary for skill '{skill}' is not installed: '{tool_path}'")]
    ToolNotInstalled { skill: String, tool_path: String },

    // ── LLM / AI ─────────────────────────────────────────────────────────────
    #[error("LLM connection error: {0}")]
    LlmConnectionError(String),

    #[error("LLM returned an empty or invalid response")]
    LlmEmptyResponse,

    #[error("LLM refused the request (safety filter triggered)")]
    SafetyRefusal,

    #[error("LLM rate limit exceeded, retry after {retry_after_secs}s")]
    RateLimitExceeded { retry_after_secs: u64 },

    #[error("All LLM model fallbacks exhausted after rate limiting")]
    AllModelsFailed,

    // ── OS Command Execution ─────────────────────────────────────────────────
    #[error("Command execution timed out after {timeout_secs}s")]
    CommandTimeout { timeout_secs: u64 },

    #[error("Command produced output exceeding the size limit")]
    OutputTooLarge,

    #[error("Argument contains unsafe shell metacharacter: '{0}'")]
    UnsafeArgument(String),

    // ── Authentication ────────────────────────────────────────────────────────
    #[error("No authentication credentials found. Run `dalang login` first.")]
    NoCredentials,

    #[error("Authentication token is invalid or expired")]
    InvalidToken,

    #[error("Failed to persist credentials: {0}")]
    PersistenceError(String),

    // ── Browser / CDP ─────────────────────────────────────────────────────────
    #[error("Browser action failed: {0}")]
    BrowserError(String),

    #[error("Browser is not initialized")]
    BrowserNotInitialized,

    // ── General ───────────────────────────────────────────────────────────────
    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for DalangError {
    fn from(e: anyhow::Error) -> Self {
        DalangError::Other(e.to_string())
    }
}
