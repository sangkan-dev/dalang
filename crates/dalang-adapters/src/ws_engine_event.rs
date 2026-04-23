//! WebSocket / JSON representation of domain [`EngineEvent`](dalang_domain::domain::models::EngineEvent).

use dalang_domain::domain::models::EngineEvent;
use serde::{Deserialize, Serialize};

/// Wire / JSON representation of [`EngineEvent`], stable for the dashboard and disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEngineEvent {
    Thinking {
        iteration: u32,
        max_iter: Option<u32>,
    },
    AssistantMessage {
        content: String,
        done: bool,
    },
    ToolExecution {
        skill: String,
        command: String,
    },
    Observation {
        skill: String,
        content: String,
        bytes: usize,
    },
    SafetyRefusal {
        retry: u32,
    },
    BrowserAction {
        action: String,
        success: bool,
        content: String,
    },
    Report {
        markdown: String,
        filename: Option<String>,
    },
    Status {
        message: String,
    },
    Error {
        message: String,
    },
    Done {
        reason: String,
    },
}

impl From<&EngineEvent> for WsEngineEvent {
    fn from(e: &EngineEvent) -> Self {
        match e {
            EngineEvent::Thinking {
                iteration,
                max_iter,
            } => WsEngineEvent::Thinking {
                iteration: *iteration,
                max_iter: *max_iter,
            },
            EngineEvent::AssistantMessage { content, done } => WsEngineEvent::AssistantMessage {
                content: content.clone(),
                done: *done,
            },
            EngineEvent::ToolExecution { skill, command } => WsEngineEvent::ToolExecution {
                skill: skill.clone(),
                command: command.clone(),
            },
            EngineEvent::Observation {
                skill,
                content,
                bytes,
            } => WsEngineEvent::Observation {
                skill: skill.clone(),
                content: content.clone(),
                bytes: *bytes,
            },
            EngineEvent::SafetyRefusal { retry } => WsEngineEvent::SafetyRefusal { retry: *retry },
            EngineEvent::BrowserAction {
                action,
                success,
                content,
            } => WsEngineEvent::BrowserAction {
                action: action.clone(),
                success: *success,
                content: content.clone(),
            },
            EngineEvent::Report { markdown, filename } => WsEngineEvent::Report {
                markdown: markdown.clone(),
                filename: filename.clone(),
            },
            EngineEvent::Status { message } => WsEngineEvent::Status {
                message: message.clone(),
            },
            EngineEvent::Error { message } => WsEngineEvent::Error {
                message: message.clone(),
            },
            EngineEvent::Done { reason } => WsEngineEvent::Done {
                reason: reason.clone(),
            },
        }
    }
}

impl From<EngineEvent> for WsEngineEvent {
    fn from(e: EngineEvent) -> Self {
        match e {
            EngineEvent::Thinking {
                iteration,
                max_iter,
            } => WsEngineEvent::Thinking {
                iteration,
                max_iter,
            },
            EngineEvent::AssistantMessage { content, done } => {
                WsEngineEvent::AssistantMessage { content, done }
            }
            EngineEvent::ToolExecution { skill, command } => {
                WsEngineEvent::ToolExecution { skill, command }
            }
            EngineEvent::Observation {
                skill,
                content,
                bytes,
            } => WsEngineEvent::Observation {
                skill,
                content,
                bytes,
            },
            EngineEvent::SafetyRefusal { retry } => WsEngineEvent::SafetyRefusal { retry },
            EngineEvent::BrowserAction {
                action,
                success,
                content,
            } => WsEngineEvent::BrowserAction {
                action,
                success,
                content,
            },
            EngineEvent::Report { markdown, filename } => {
                WsEngineEvent::Report { markdown, filename }
            }
            EngineEvent::Status { message } => WsEngineEvent::Status { message },
            EngineEvent::Error { message } => WsEngineEvent::Error { message },
            EngineEvent::Done { reason } => WsEngineEvent::Done { reason },
        }
    }
}

impl From<WsEngineEvent> for EngineEvent {
    fn from(e: WsEngineEvent) -> Self {
        match e {
            WsEngineEvent::Thinking {
                iteration,
                max_iter,
            } => EngineEvent::Thinking {
                iteration,
                max_iter,
            },
            WsEngineEvent::AssistantMessage { content, done } => {
                EngineEvent::AssistantMessage { content, done }
            }
            WsEngineEvent::ToolExecution { skill, command } => {
                EngineEvent::ToolExecution { skill, command }
            }
            WsEngineEvent::Observation {
                skill,
                content,
                bytes,
            } => EngineEvent::Observation {
                skill,
                content,
                bytes,
            },
            WsEngineEvent::SafetyRefusal { retry } => EngineEvent::SafetyRefusal { retry },
            WsEngineEvent::BrowserAction {
                action,
                success,
                content,
            } => EngineEvent::BrowserAction {
                action,
                success,
                content,
            },
            WsEngineEvent::Report { markdown, filename } => {
                EngineEvent::Report { markdown, filename }
            }
            WsEngineEvent::Status { message } => EngineEvent::Status { message },
            WsEngineEvent::Error { message } => EngineEvent::Error { message },
            WsEngineEvent::Done { reason } => EngineEvent::Done { reason },
        }
    }
}
