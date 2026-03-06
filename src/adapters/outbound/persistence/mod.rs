//! Persistence adapter — implements `AuthPersistence` and `SessionStorage` ports.
//!
//! `KeyringAuthPersistence`: delegates to `auth::persistence` (keyring + config file).
//! `FileSessionStorage`: stub implementation — full migration in a follow-up sprint.

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
}

// ── Session Storage Adapter ───────────────────────────────────────────────────

/// Session storage adapter that persists to `~/.dalang/sessions/`.
///
/// TODO: The `web::persistence` module uses a Session struct with a different schema
/// (no message_count, event_count fields) and the EngineEvent is from `web::events`.
/// Full migration requires aligning these types. For now, this stub allows the port
/// trait to be registered and used without breaking compilation.
pub struct FileSessionStorage;

impl SessionStorage for FileSessionStorage {
    fn save_session_meta(&self, _meta: &SessionMeta) -> Result<()> {
        // TODO: migrate to web::persistence::save_session_meta once types are aligned
        Ok(())
    }

    fn save_events(&self, _session_id: Uuid, _events: &[EngineEvent]) -> Result<()> {
        // TODO: unify domain::models::EngineEvent and web::events::EngineEvent types
        // then delegate to crate::web::persistence::save_events
        Ok(())
    }

    fn load_all_sessions(&self) -> Result<Vec<SessionMeta>> {
        // TODO: align Session type with SessionMeta once web module is migrated
        Ok(vec![])
    }

    fn load_events(&self, _session_id: Uuid) -> Result<Vec<EngineEvent>> {
        // TODO: implement after aligning EngineEvent sources
        Ok(vec![])
    }

    fn load_memory(&self, _session_id: Uuid) -> Result<Vec<String>> {
        Ok(vec![])
    }

    fn save_memory(&self, _session_id: Uuid, _observations: &[String]) -> Result<()> {
        Ok(())
    }

    fn delete_session(&self, session_id: Uuid) -> Result<()> {
        crate::web::persistence::delete_session_dir(&session_id);
        Ok(())
    }
}
