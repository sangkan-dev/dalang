//! Domain models — core data structures shared across all layers.
//!
//! These types are pure Rust structs with no infrastructure dependencies.
//! They represent the fundamental concepts of the Dalang domain.

use serde::{Deserialize, Serialize};

// ── Skill Definition ──────────────────────────────────────────────────────────

/// Re-exported from `crate::skills_parser` — single source of truth.
/// Both `crate::skills_parser::SkillDefinition` and `crate::domain::models::SkillDefinition` refer to the same type.
pub use crate::skills_parser::SkillDefinition;

// ── LLM Conversation ─────────────────────────────────────────────────────────

/// An authentication token for an LLM provider.
#[derive(Debug, Clone)]
pub enum AuthToken {
    /// No authentication (e.g., local Ollama).
    None,
    /// API Key authentication (e.g., Gemini or OpenAI API key).
    ApiKey(String),
    /// Bearer token for OAuth/JWT (e.g., Gemini OAuth, GitHub Copilot).
    Bearer(String),
}

/// A single message in an LLM conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn system(content: &str) -> Self {
        Self {
            role: "system".to_string(),
            content: content.to_string(),
        }
    }
    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }
    pub fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
        }
    }
}

// ── Engine Events ─────────────────────────────────────────────────────────────

/// Events emitted from the engine during execution.
///
/// These are streamed over WebSocket to the frontend in real-time,
/// or printed to stdout in CLI mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EngineEvent {
    /// LLM is reasoning (start of iteration).
    Thinking {
        iteration: u32,
        max_iter: Option<u32>,
    },
    /// Text response from the LLM.
    AssistantMessage { content: String, done: bool },
    /// A skill/tool is about to be executed.
    ToolExecution { skill: String, command: String },
    /// Output from a tool execution.
    Observation {
        skill: String,
        content: String,
        bytes: usize,
    },
    /// LLM hit safety filter, auto-retrying.
    SafetyRefusal { retry: u32 },
    /// Browser tool was invoked.
    BrowserAction {
        action: String,
        success: bool,
        content: String,
    },
    /// Final vulnerability report generated.
    Report {
        markdown: String,
        filename: Option<String>,
    },
    /// Informational status message.
    Status { message: String },
    /// Error occurred during execution.
    Error { message: String },
    /// Engine has completed execution.
    Done { reason: String },
}

// ── WebSocket Client Messages ─────────────────────────────────────────────────

/// Messages the client sends over WebSocket.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Send a chat message (interactive mode).
    Chat { message: String },
    /// Start an auto-pilot scan.
    StartScan {
        target: String,
        #[serde(default = "default_max_iter")]
        max_iter: u32,
        #[serde(default = "default_cmd_timeout")]
        cmd_timeout: u64,
    },
    /// Start an interactive session.
    StartInteractive {
        target: String,
        #[serde(default = "default_cmd_timeout")]
        cmd_timeout: u64,
    },
}

fn default_max_iter() -> u32 {
    15
}
fn default_cmd_timeout() -> u64 {
    300
}

// ── Tool Call ─────────────────────────────────────────────────────────────────

/// A tool call parsed from the LLM's JSON output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}
