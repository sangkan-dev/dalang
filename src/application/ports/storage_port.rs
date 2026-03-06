//! Storage Ports.
//!
//! Defines contracts for persisting auth credentials and chat sessions.
//! Concrete implementations: `adapters/outbound/persistence/`.

use crate::domain::models::EngineEvent;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Auth Persistence ──────────────────────────────────────────────────────────

/// Outbound port for persisting authentication credentials securely.
///
/// Implementations should use the OS keyring (via `keyring` crate) for tokens
/// and a plain config file for non-sensitive settings like provider name.
pub trait AuthPersistence: Send + Sync {
    fn save_tokens(&self, access_token: &str, refresh_token: Option<&str>) -> Result<()>;
    fn get_access_token(&self) -> Result<String>;
    fn get_refresh_token(&self) -> Result<Option<String>>;
    fn save_active_provider(&self, provider: &str) -> Result<()>;
    fn get_active_provider(&self) -> Result<String>;
    fn save_auth_method(&self, method: &str) -> Result<()>;
    fn get_auth_method(&self) -> Result<String>;
    fn save_endpoint_mode(&self, mode: &str) -> Result<()>;
    fn get_endpoint_mode(&self) -> Result<String>;
    fn save_model_preference(&self, model: &str) -> Result<()>;
    fn get_model_preference(&self) -> Result<String>;
    fn save_api_key(&self, key: &str) -> Result<()>;
    fn get_api_key(&self) -> Result<Option<String>>;
    fn save_verbose(&self, verbose: bool) -> Result<()>;
    fn get_verbose(&self) -> Result<bool>;
    fn save_codeassist_endpoint(&self, endpoint: &str) -> Result<()>;
    fn get_codeassist_endpoint(&self) -> Result<String>;
    fn save_gcp_project(&self, project_id: &str) -> Result<()>;
    fn get_gcp_project(&self) -> Result<String>;
}

// ── Session Storage ───────────────────────────────────────────────────────────

/// Lightweight metadata for a single chat session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: Uuid,
    pub target: String,
    pub mode: String,
    pub created_at: String,
    pub active: bool,
    pub message_count: usize,
    pub event_count: usize,
}

/// Outbound port for persisting chat sessions to disk.
///
/// Implementations save session data under `~/.dalang/sessions/{uuid}/`.
pub trait SessionStorage: Send + Sync {
    fn save_session_meta(&self, meta: &SessionMeta) -> Result<()>;
    fn save_events(&self, session_id: Uuid, events: &[EngineEvent]) -> Result<()>;
    fn load_all_sessions(&self) -> Result<Vec<SessionMeta>>;
    fn load_events(&self, session_id: Uuid) -> Result<Vec<EngineEvent>>;
    fn load_memory(&self, session_id: Uuid) -> Result<Vec<String>>;
    fn save_memory(&self, session_id: Uuid, observations: &[String]) -> Result<()>;
    fn delete_session(&self, session_id: Uuid) -> Result<()>;
}
