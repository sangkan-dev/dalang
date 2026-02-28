//! Shared application state for the web server.

use crate::llm::{self, AuthToken, Message};
use crate::web::events::EngineEvent;
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
#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: Uuid,
    pub target: String,
    pub mode: SessionMode,
    pub messages: Vec<Message>,
    pub created_at: String,
    pub active: bool,
}

/// Shared app state passed into axum handlers via `Extension<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub sessions: Arc<DashMap<Uuid, Session>>,
    /// Channel senders for active WebSocket connections, keyed by session ID.
    /// When an engine task emits events, they are sent through these channels.
    pub event_senders: Arc<DashMap<Uuid, mpsc::Sender<EngineEvent>>>,
    pub verbose: bool,
}

impl AppState {
    pub fn new(verbose: bool) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            event_senders: Arc::new(DashMap::new()),
            verbose,
        }
    }

    pub fn create_session(&self, target: String, mode: SessionMode) -> Session {
        let session = Session {
            id: Uuid::new_v4(),
            target,
            mode,
            messages: Vec::new(),
            created_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            active: true,
        };
        self.sessions.insert(session.id, session.clone());
        session
    }

    /// Resolve LLM provider using the same logic as CLI.
    pub fn create_llm_provider(
        &self,
    ) -> anyhow::Result<Box<dyn crate::llm::LlmProvider + Send + Sync>> {
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

        let base_url = std::env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| llm::get_default_base_url(&active_provider));

        let (codeassist_ep, gcp_project) = if endpoint_mode == "cloudcode" {
            (
                crate::auth::persistence::get_codeassist_endpoint().ok(),
                crate::auth::persistence::get_gcp_project().ok(),
            )
        } else {
            (None, None)
        };

        let model = std::env::var("LLM_MODEL")
            .or_else(|_| crate::auth::persistence::get_model_preference())
            .unwrap_or_else(|_| llm::get_default_model(&active_provider));

        llm::create_provider(&endpoint_mode, base_url, model, auth, codeassist_ep, gcp_project)
    }
}

fn resolve_auth_token(auth_method: &str) -> AuthToken {
    if let Ok(token) = crate::auth::persistence::get_access_token() {
        return match auth_method {
            "bearer" => AuthToken::Bearer(token),
            _ => AuthToken::ApiKey(token),
        };
    }
    if let Ok(key) = std::env::var("LLM_API_KEY") {
        return AuthToken::ApiKey(key);
    }
    if let Some(token) = crate::auth::cli_extractor::try_all_cli_extractors() {
        return AuthToken::Bearer(token);
    }
    AuthToken::None
}
