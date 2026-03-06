//! Domain models — core data structures shared across all layers.
//!
//! These types are pure Rust structs with no infrastructure dependencies.
//! They represent the fundamental concepts of the Dalang domain.

use serde::{Deserialize, Serialize};

// ── Skill Definition ──────────────────────────────────────────────────────────

/// A fully parsed skill from a `.md` file in the `skills/` directory.
///
/// Skills define both the OS command to execute (YAML frontmatter) and the
/// AI persona/instructions (Markdown body) used by the LLM.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    /// Path to the executable binary (e.g., `/usr/bin/nmap` or just `nmap`)
    pub tool_path: Option<String>,
    /// Argument template list. May contain `{{target}}` placeholder.
    pub args: Option<Vec<String>>,
    /// Whether this skill requires root privileges to run.
    pub requires_root: Option<bool>,
    /// The full Markdown body, used as the LLM system prompt.
    #[serde(skip)]
    pub system_prompt: String,
    /// Extracted `# Role` section from the prompt body.
    #[serde(skip)]
    pub role: Option<String>,
    /// Extracted `# Task` section from the prompt body.
    #[serde(skip)]
    pub task: Option<String>,
    /// Extracted `# Constraints` section from the prompt body.
    #[serde(skip)]
    pub constraints: Option<String>,
    /// Whether the tool binary is installed and reachable on the system PATH.
    /// Skills with `tool_path: null` (browser-based) are always considered available.
    #[serde(skip)]
    pub tool_available: bool,
}

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
