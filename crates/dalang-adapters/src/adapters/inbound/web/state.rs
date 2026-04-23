//! Shared application state for the web server.

use crate::adapters::inbound::web::events::EngineEvent;
use crate::adapters::inbound::web::persistence;
use crate::adapters::outbound::llm;
use dalang_application::application::ports::storage_port::{AuthPersistence, ReportStorage};
use dalang_domain::domain::models::AuthToken;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

pub use crate::session_files::{Session, SessionMode};

/// Shared app state passed into axum handlers via `Extension<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub sessions: Arc<DashMap<Uuid, Session>>,
    /// Channel senders for active WebSocket connections, keyed by session ID.
    /// Value is (connection_id, sender) so cleanup only removes its own connection.
    pub event_senders: Arc<DashMap<Uuid, (Uuid, mpsc::Sender<EngineEvent>)>>,
    /// Disabled skills (name -> true means disabled).
    pub disabled_skills: Arc<DashMap<String, bool>>,
    pub auth: Arc<dyn AuthPersistence>,
    pub reports: Arc<dyn ReportStorage>,
    pub verbose: bool,
    pub headless: bool,
}

impl AppState {
    pub fn new(
        verbose: bool,
        headless: bool,
        auth: Arc<dyn AuthPersistence>,
        reports: Arc<dyn ReportStorage>,
    ) -> Self {
        let sessions = Arc::new(DashMap::new());

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
            auth,
            reports,
            verbose,
            headless,
        }
    }

    /// Create an empty AppState with no sessions loaded from disk.
    #[cfg(test)]
    pub fn new_empty() -> Self {
        use crate::adapters::outbound::persistence::{CwdReportStorage, KeyringAuthPersistence};
        Self {
            sessions: Arc::new(DashMap::new()),
            event_senders: Arc::new(DashMap::new()),
            disabled_skills: Arc::new(DashMap::new()),
            auth: Arc::new(KeyringAuthPersistence),
            reports: Arc::new(CwdReportStorage::new_cwd()),
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
        persistence::save_session_meta(&session);
        session
    }

    /// Resolve LLM provider using the same logic as CLI.
    pub fn create_llm_provider(
        &self,
    ) -> anyhow::Result<std::sync::Arc<dyn dalang_application::application::ports::llm_port::LlmPort>>
    {
        let active_provider = self
            .auth
            .get_active_provider()
            .unwrap_or_else(|_| "gemini".to_string());
        let auth_method = self
            .auth
            .get_auth_method()
            .unwrap_or_else(|_| "apikey".to_string());
        let endpoint_mode = self
            .auth
            .get_endpoint_mode()
            .unwrap_or_else(|_| "openai_compat".to_string());

        let auth = resolve_auth_token(self.auth.as_ref(), &auth_method);
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
            self.auth
                .get_custom_base_url()
                .unwrap_or_else(|_| llm::get_default_base_url(&active_provider))
        } else {
            llm::get_default_base_url(&active_provider)
        };

        let (codeassist_ep, gcp_project) = if endpoint_mode == "cloudcode" {
            (
                self.auth.get_codeassist_endpoint().ok(),
                self.auth.get_gcp_project().ok(),
            )
        } else {
            (None, None)
        };

        let model = std::env::var("LLM_MODEL")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .or_else(|| {
                self.auth
                    .get_model_preference()
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

fn resolve_auth_token(store: &dyn AuthPersistence, auth_method: &str) -> AuthToken {
    if let Ok(token) = store.get_access_token() {
        return match auth_method {
            "bearer" | "copilot_oauth" => AuthToken::Bearer(token),
            _ => AuthToken::ApiKey(token),
        };
    }
    if let Ok(Some(key)) = store.get_api_key() {
        return AuthToken::ApiKey(key);
    }
    if let Ok(key) = std::env::var("LLM_API_KEY") {
        let key = key.trim();
        if !key.is_empty() {
            return AuthToken::ApiKey(key.to_string());
        }
    }
    if let Some(token) = crate::adapters::outbound::auth::cli_extractor::try_all_cli_extractors() {
        return AuthToken::Bearer(token);
    }
    AuthToken::None
}
