//! Shared application state for the web server.

use crate::adapters::outbound::llm;
use crate::domain::models::{AuthToken, Message};
use crate::web::events::EngineEvent;
use crate::web::persistence;
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Session mode.
#[derive(Debug, Clone, Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionMode {
    Interactive,
    Scan,
}

/// A single session (one chat conversation with the engine).
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub target: String,
    pub mode: SessionMode,
    pub messages: Vec<Message>,
    /// All engine events emitted during this session (for UI replay).
    #[serde(default)]
    pub events: Vec<EngineEvent>,
    pub created_at: String,
    pub active: bool,
    /// Command execution timeout in seconds (0 = unlimited).
    #[serde(default = "default_cmd_timeout")]
    pub cmd_timeout: u64,
}

fn default_cmd_timeout() -> u64 {
    300
}

/// Shared app state passed into axum handlers via `Extension<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub sessions: Arc<DashMap<Uuid, Session>>,
    /// Channel senders for active WebSocket connections, keyed by session ID.
    /// Value is (connection_id, sender) so cleanup only removes its own connection.
    pub event_senders: Arc<DashMap<Uuid, (Uuid, mpsc::Sender<EngineEvent>)>>,
    /// Disabled skills (name -> true means disabled).
    pub disabled_skills: Arc<DashMap<String, bool>>,
    pub verbose: bool,
    pub headless: bool,
}

impl AppState {
    pub fn new(verbose: bool, headless: bool) -> Self {
        let sessions = Arc::new(DashMap::new());

        // Restore sessions from disk (~/.dalang/sessions/)
        let restored = persistence::load_all_sessions();
        let count = restored.len();
        for (session, _events) in restored {
            sessions.insert(session.id, session);
        }
        if count > 0 {
            println!("[*] Restored {} session(s) from disk.", count);
        }

        Self {
            sessions,
            event_senders: Arc::new(DashMap::new()),
            disabled_skills: Arc::new(DashMap::new()),
            verbose,
            headless,
        }
    }

    /// Create an empty AppState with no sessions loaded from disk.
    ///
    /// Use this in unit tests to avoid reading `~/.dalang/sessions/` from the
    /// developer's machine, which makes tests non-deterministic.
    #[cfg(test)]
    pub fn new_empty() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            event_senders: Arc::new(DashMap::new()),
            disabled_skills: Arc::new(DashMap::new()),
            verbose: false,
            headless: true,
        }
    }

    pub fn create_session(&self, target: String, mode: SessionMode, cmd_timeout: u64) -> Session {
        let session = Session {
            id: Uuid::new_v4(),
            target,
            mode,
            messages: Vec::new(),
            events: Vec::new(),
            created_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            active: true,
            cmd_timeout,
        };
        self.sessions.insert(session.id, session.clone());
        // Persist to disk immediately
        persistence::save_session_meta(&session);
        session
    }

    /// Resolve LLM provider using the same logic as CLI.
    pub fn create_llm_provider(
        &self,
    ) -> anyhow::Result<std::sync::Arc<dyn crate::application::ports::llm_port::LlmPort>> {
        let active_provider = crate::auth::persistence::get_active_provider()
            .unwrap_or_else(|_| "gemini".to_string());
        let auth_method =
            crate::auth::persistence::get_auth_method().unwrap_or_else(|_| "apikey".to_string());
        let endpoint_mode = crate::auth::persistence::get_endpoint_mode()
            .unwrap_or_else(|_| "openai_compat".to_string());

        // Resolve auth token
        let auth = resolve_auth_token(&auth_method);
        if matches!(auth, AuthToken::None) {
            return Err(anyhow::anyhow!(
                "No API key found. Please run 'dalang login' or set LLM_API_KEY."
            ));
        }

        let env_base_url = std::env::var("LLM_BASE_URL")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());

        let base_url = if let Some(url) = env_base_url {
            url
        } else if endpoint_mode == "openai_compat" {
            crate::auth::persistence::get_custom_base_url()
                .unwrap_or_else(|_| llm::get_default_base_url(&active_provider))
        } else {
            llm::get_default_base_url(&active_provider)
        };

        let (codeassist_ep, gcp_project) = if endpoint_mode == "cloudcode" {
            (
                crate::auth::persistence::get_codeassist_endpoint().ok(),
                crate::auth::persistence::get_gcp_project().ok(),
            )
        } else {
            (None, None)
        };

        let model = std::env::var("LLM_MODEL")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .or_else(|| {
                crate::auth::persistence::get_model_preference()
                    .ok()
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
            })
            .unwrap_or_else(|| llm::get_default_model(&active_provider));

        llm::create_provider(
            &endpoint_mode,
            base_url,
            model,
            auth,
            codeassist_ep,
            gcp_project,
        )
    }
}

fn resolve_auth_token(auth_method: &str) -> AuthToken {
    if let Ok(token) = crate::auth::persistence::get_access_token() {
        return match auth_method {
            "bearer" | "copilot_oauth" => AuthToken::Bearer(token),
            _ => AuthToken::ApiKey(token),
        };
    }
    if let Ok(key) = crate::auth::persistence::get_api_key() {
        return AuthToken::ApiKey(key);
    }
    if let Ok(key) = std::env::var("LLM_API_KEY") {
        let key = key.trim();
        if !key.is_empty() {
            return AuthToken::ApiKey(key.to_string());
        }
    }
    if let Some(token) = crate::auth::cli_extractor::try_all_cli_extractors() {
        return AuthToken::Bearer(token);
    }
    AuthToken::None
}
