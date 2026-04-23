//! Neutral on-disk session storage under `~/.dalang/sessions/{uuid}/`.
//!
//! Used by web handlers and by [`crate::adapters::outbound::persistence::FileSessionStorage`]
//! so outbound adapters do not depend on `inbound::web`.

use crate::WsEngineEvent;
use dalang_application::application::usecases::memory::ContextManager;
use dalang_domain::domain::models::{EngineEvent, Message};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Session mode (persisted in `session.json`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionMode {
    Interactive,
    Scan,
}

/// A single chat session (runtime + disk shape).
#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub target: String,
    pub mode: SessionMode,
    pub messages: Vec<Message>,
    pub events: Vec<EngineEvent>,
    pub created_at: String,
    pub active: bool,
    pub cmd_timeout: u64,
}

/// Returns `~/.dalang/sessions/`, creating it if necessary.
pub fn sessions_dir() -> PathBuf {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".dalang")
        .join("sessions");
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Returns `~/.dalang/sessions/{uuid}/`.
pub fn session_dir(id: &Uuid) -> PathBuf {
    sessions_dir().join(id.to_string())
}

fn session_file_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionMeta {
    #[serde(default = "session_file_schema_version")]
    schema_version: u32,
    id: Uuid,
    target: String,
    mode: SessionMode,
    created_at: String,
    active: bool,
    #[serde(default)]
    cmd_timeout: Option<u64>,
}

impl From<&Session> for SessionMeta {
    fn from(s: &Session) -> Self {
        Self {
            schema_version: session_file_schema_version(),
            id: s.id,
            target: s.target.clone(),
            mode: s.mode.clone(),
            created_at: s.created_at.clone(),
            active: s.active,
            cmd_timeout: Some(s.cmd_timeout),
        }
    }
}

/// Persist session metadata to `session.json`.
pub fn save_session_meta(session: &Session) {
    let dir = session_dir(&session.id);
    let _ = fs::create_dir_all(&dir);
    let meta = SessionMeta::from(session);
    if let Ok(json) = serde_json::to_string_pretty(&meta) {
        let _ = fs::write(dir.join("session.json"), json);
    }
}

/// Persist LLM messages to `messages.json`.
pub fn save_messages(id: &Uuid, messages: &[Message]) {
    let dir = session_dir(id);
    let _ = fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(messages) {
        let _ = fs::write(dir.join("messages.json"), json);
    }
}

/// Persist engine events to `events.json`.
pub fn save_events(id: &Uuid, events: &[EngineEvent]) {
    let dir = session_dir(id);
    let _ = fs::create_dir_all(&dir);
    let dto: Vec<WsEngineEvent> = events.iter().map(WsEngineEvent::from).collect();
    if let Ok(json) = serde_json::to_string_pretty(&dto) {
        let _ = fs::write(dir.join("events.json"), json);
    }
}

/// Load engine events from `events.json`.
pub fn load_events(id: &Uuid) -> Option<Vec<EngineEvent>> {
    let path = session_dir(id).join("events.json");
    let content = fs::read_to_string(path).ok()?;
    let dto: Vec<WsEngineEvent> = serde_json::from_str(&content).ok()?;
    Some(dto.into_iter().map(EngineEvent::from).collect())
}

/// Persist the conversation memory to `MEMORY.md`.
pub fn save_memory(id: &Uuid, target: &str, memory: &ContextManager) {
    let dir = session_dir(id);
    let _ = fs::create_dir_all(&dir);

    let observations = memory.observations();
    let updated_at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    let mut md = String::new();
    md.push_str("---\n");
    md.push_str(&format!("session_id: \"{}\"\n", id));
    md.push_str(&format!("target: \"{}\"\n", target));
    md.push_str(&format!("updated_at: \"{}\"\n", updated_at));
    md.push_str(&format!("observation_count: {}\n", observations.len()));
    md.push_str("---\n\n");
    md.push_str("# Session Memory\n\n");

    if observations.is_empty() {
        md.push_str("No observations recorded yet.\n");
    } else {
        md.push_str("### PERSISTENT CONTEXT MEMORY (Last observations):\n");
        md.push_str(
            "Reference these observations to avoid repeating work. Note specific URLs, parameters,\n\
             and findings from previous tool executions when planning next steps.\n\n",
        );
        for (idx, obs) in observations.iter().enumerate() {
            md.push_str(&format!("{}. {}\n", idx + 1, obs));
        }
    }

    let _ = fs::write(dir.join("MEMORY.md"), md);
}

/// Load the conversation memory from `MEMORY.md`.
pub fn load_memory(id: &Uuid) -> Option<ContextManager> {
    let path = session_dir(id).join("MEMORY.md");
    let content = fs::read_to_string(path).ok()?;

    let mut observations = Vec::new();
    let mut in_body = false;
    let mut frontmatter_count = 0;

    for line in content.lines() {
        if line.trim() == "---" {
            frontmatter_count += 1;
            if frontmatter_count == 2 {
                in_body = true;
            }
            continue;
        }
        if !in_body {
            continue;
        }
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_ascii_digit()) {
            let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
            if let Some(text) = rest.strip_prefix(". ") {
                observations.push(text.to_string());
            }
        }
    }

    if observations.is_empty() {
        return None;
    }

    Some(ContextManager::from_observations(observations))
}

/// Scan `~/.dalang/sessions/` and restore all persisted sessions.
pub fn load_all_sessions() -> Vec<(Session, Vec<EngineEvent>)> {
    let dir = sessions_dir();
    let mut results = Vec::new();

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return results,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let meta_path = path.join("session.json");
        let meta: SessionMeta = match fs::read_to_string(&meta_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
        {
            Some(m) => m,
            None => continue,
        };

        let messages: Vec<Message> = fs::read_to_string(path.join("messages.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let events: Vec<EngineEvent> = fs::read_to_string(path.join("events.json"))
            .ok()
            .and_then(|s| serde_json::from_str::<Vec<WsEngineEvent>>(&s).ok())
            .map(|v| v.into_iter().map(EngineEvent::from).collect())
            .unwrap_or_default();

        let session = Session {
            id: meta.id,
            target: meta.target,
            mode: meta.mode,
            messages,
            events,
            created_at: meta.created_at,
            active: meta.active,
            cmd_timeout: meta.cmd_timeout.unwrap_or(300),
        };

        results.push((session, vec![]));
    }

    results
}

/// Delete all session files for one session id.
pub fn delete_session_dir(id: &Uuid) {
    let dir = session_dir(id);
    let _ = fs::remove_dir_all(dir);
}
