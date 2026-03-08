//! Persistence adapter — implements `AuthPersistence` and `SessionStorage` ports.
//!
//! `KeyringAuthPersistence`: delegates to `auth::persistence` (keyring + config file).
//! `FileSessionStorage`: delegates to `web::persistence` for session file I/O.

use crate::application::ports::storage_port::{AuthPersistence, SessionMeta, SessionStorage};
use crate::domain::models::EngineEvent;
use anyhow::Result;
use uuid::Uuid;

// ── Auth Persistence Adapter ──────────────────────────────────────────────────

/// Thin wrapper around `auth::persistence` functions, implementing `AuthPersistence`.
pub struct KeyringAuthPersistence;

impl AuthPersistence for KeyringAuthPersistence {
    fn save_tokens(&self, access_token: &str, refresh_token: Option<&str>) -> Result<()> {
        crate::auth::persistence::save_tokens(access_token, refresh_token)
    }
    fn get_access_token(&self) -> Result<String> {
        crate::auth::persistence::get_access_token()
    }
    fn get_refresh_token(&self) -> Result<Option<String>> {
        crate::auth::persistence::get_refresh_token().map(Some)
    }
    fn save_active_provider(&self, provider: &str) -> Result<()> {
        crate::auth::persistence::save_active_provider(provider)
    }
    fn get_active_provider(&self) -> Result<String> {
        crate::auth::persistence::get_active_provider()
    }
    fn save_auth_method(&self, method: &str) -> Result<()> {
        crate::auth::persistence::save_auth_method(method)
    }
    fn get_auth_method(&self) -> Result<String> {
        crate::auth::persistence::get_auth_method()
    }
    fn save_endpoint_mode(&self, mode: &str) -> Result<()> {
        crate::auth::persistence::save_endpoint_mode(mode)
    }
    fn get_endpoint_mode(&self) -> Result<String> {
        crate::auth::persistence::get_endpoint_mode()
    }
    fn save_model_preference(&self, model: &str) -> Result<()> {
        crate::auth::persistence::save_model_preference(model)
    }
    fn get_model_preference(&self) -> Result<String> {
        crate::auth::persistence::get_model_preference()
    }
    fn save_api_key(&self, key: &str) -> Result<()> {
        crate::auth::persistence::save_api_key(key)
    }
    fn get_api_key(&self) -> Result<Option<String>> {
        crate::auth::persistence::get_api_key().map(Some)
    }
    fn save_verbose(&self, verbose: bool) -> Result<()> {
        crate::auth::persistence::save_verbose(verbose)
    }
    fn get_verbose(&self) -> Result<bool> {
        crate::auth::persistence::get_verbose()
    }
    fn save_codeassist_endpoint(&self, endpoint: &str) -> Result<()> {
        crate::auth::persistence::save_codeassist_endpoint(endpoint)
    }
    fn get_codeassist_endpoint(&self) -> Result<String> {
        crate::auth::persistence::get_codeassist_endpoint()
    }
    fn save_gcp_project(&self, project_id: &str) -> Result<()> {
        crate::auth::persistence::save_gcp_project(project_id)
    }
    fn get_gcp_project(&self) -> Result<String> {
        crate::auth::persistence::get_gcp_project()
    }
    fn save_custom_base_url(&self, url: &str) -> Result<()> {
        crate::adapters::outbound::auth::persistence::save_custom_base_url(url)
    }
    fn get_custom_base_url(&self) -> Result<String> {
        crate::adapters::outbound::auth::persistence::get_custom_base_url()
    }
}

// ── Session Storage Adapter ───────────────────────────────────────────────────

/// Session storage adapter that persists to `~/.dalang/sessions/`.
///
/// Delegates to `web::persistence` for all file I/O, which already handles
/// `~/.dalang/sessions/{uuid}/` with `session.json`, `events.json`, and `MEMORY.md`.
pub struct FileSessionStorage;

impl SessionStorage for FileSessionStorage {
    fn save_session_meta(&self, meta: &SessionMeta) -> Result<()> {
        // Build a minimal web::state::Session from the port-level SessionMeta.
        // Only fields present in SessionMeta are populated; events/messages start empty.
        use crate::web::state::{Session, SessionMode};
        let mode = match meta.mode.as_str() {
            "scan" => SessionMode::Scan,
            _ => SessionMode::Interactive,
        };
        let session = Session {
            id: meta.id,
            target: meta.target.clone(),
            mode,
            messages: vec![],
            events: vec![],
            created_at: meta.created_at.clone(),
            active: meta.active,
            cmd_timeout: 300,
        };
        crate::web::persistence::save_session_meta(&session);
        Ok(())
    }

    fn save_events(&self, session_id: Uuid, events: &[EngineEvent]) -> Result<()> {
        crate::web::persistence::save_events(&session_id, events);
        Ok(())
    }

    fn load_all_sessions(&self) -> Result<Vec<SessionMeta>> {
        let raw = crate::web::persistence::load_all_sessions();
        let metas = raw
            .into_iter()
            .map(|(session, _events)| {
                use crate::web::state::SessionMode;
                let mode = match &session.mode {
                    SessionMode::Scan => "scan",
                    SessionMode::Interactive => "interactive",
                }
                .to_string();
                // Count messages and events from the session struct
                let message_count = session.messages.len();
                let event_count = session.events.len();
                SessionMeta {
                    id: session.id,
                    target: session.target,
                    mode,
                    created_at: session.created_at,
                    active: session.active,
                    message_count,
                    event_count,
                }
            })
            .collect();
        Ok(metas)
    }

    fn load_events(&self, session_id: Uuid) -> Result<Vec<EngineEvent>> {
        let events = crate::web::persistence::load_events(&session_id).unwrap_or_default();
        Ok(events)
    }

    fn load_memory(&self, session_id: Uuid) -> Result<Vec<String>> {
        if let Some(ctx) = crate::web::persistence::load_memory(&session_id) {
            Ok(ctx.observations().to_vec())
        } else {
            Ok(vec![])
        }
    }

    fn save_memory(&self, session_id: Uuid, observations: &[String]) -> Result<()> {
        // Build a minimal ContextManager from observations and delegate to persistence.
        use crate::application::usecases::memory::ContextManager;
        let ctx = ContextManager::from_observations(observations.to_vec());
        // We need a target for the MEMORY.md header; use a placeholder if unknown.
        let target = "unknown";
        crate::web::persistence::save_memory(&session_id, target, &ctx);
        Ok(())
    }

    fn delete_session(&self, session_id: Uuid) -> Result<()> {
        crate::web::persistence::delete_session_dir(&session_id);
        Ok(())
    }
}
