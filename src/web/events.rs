//! Event types emitted by the DalangEngine for web UI consumption.
//!
//! Each variant is serialized to JSON and sent over WebSocket to the frontend.

use serde::Serialize;

/// Events emitted from the engine during execution.
/// These are streamed over WebSocket to the frontend in real-time.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EngineEvent {
    /// LLM is reasoning (start of iteration)
    Thinking {
        iteration: u32,
        max_iter: Option<u32>,
    },

    /// Text response from the LLM (may arrive in chunks for streaming)
    AssistantMessage {
        content: String,
        done: bool,
    },

    /// A skill/tool is about to be executed
    ToolExecution {
        skill: String,
        command: String,
    },

    /// Output from a tool execution
    Observation {
        skill: String,
        content: String,
        bytes: usize,
    },

    /// LLM hit safety filter, auto-retrying
    SafetyRefusal {
        retry: u32,
    },

    /// Browser tool was invoked
    BrowserAction {
        action: String,
        success: bool,
        content: String,
    },

    /// Final vulnerability report generated
    Report {
        markdown: String,
        filename: Option<String>,
    },

    /// Informational status message
    Status {
        message: String,
    },

    /// Error occurred during execution
    Error {
        message: String,
    },

    /// Engine has completed execution
    Done {
        reason: String,
    },
}

/// Messages the client sends over WebSocket.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Send a chat message (interactive mode)
    Chat {
        message: String,
    },

    /// Start an auto-pilot scan
    StartScan {
        target: String,
        #[serde(default = "default_max_iter")]
        max_iter: u32,
        #[serde(default = "default_cmd_timeout")]
        cmd_timeout: u64,
    },

    /// Start an interactive session
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
