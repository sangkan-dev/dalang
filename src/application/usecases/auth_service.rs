//! Auth Service Use Case.
//!
//! Encapsulates all authentication-related business logic:
//! token retrieval, provider selection, and config resolution.
//!
//! Depends on the `AuthPersistence` port — no direct keyring/filesystem access here.

use crate::application::ports::storage_port::AuthPersistence;
use crate::domain::errors::DalangError;
use crate::domain::models::AuthToken;
use anyhow::Result;
use std::sync::Arc;

/// Service responsible for resolving the active authentication configuration.
pub struct AuthService {
    persistence: Arc<dyn AuthPersistence>,
}

impl AuthService {
    pub fn new(persistence: Arc<dyn AuthPersistence>) -> Self {
        Self { persistence }
    }

    /// Return the active LLM provider name (e.g. `"gemini"`, `"anthropic"`).
    /// Falls back to `"gemini"` if not configured.
    pub fn active_provider(&self) -> String {
        self.persistence
            .get_active_provider()
            .unwrap_or_else(|_| "gemini".to_string())
    }

    /// Resolve the authentication token to use for the active provider.
    ///
    /// Priority (highest → lowest):
    /// 1. `LLM_API_KEY` environment variable
    /// 2. GitHub Copilot token (if provider == "copilot")
    /// 3. Stored API key in keyring
    /// 4. Stored access/refresh token (OAuth for Gemini, etc.)
    pub fn resolve_auth_token(&self) -> AuthToken {
        let provider = self.active_provider();

        // 1. LLM_API_KEY env var always wins
        if let Ok(key) = std::env::var("LLM_API_KEY")
            && !key.is_empty() {
                return AuthToken::ApiKey(key);
            }

        // 2. Copilot: use stored bearer token
        if provider == "copilot"
            && let Ok(token) = self.persistence.get_access_token() {
                return AuthToken::Bearer(token);
            }

        // 3. Stored API key
        if let Ok(Some(key)) = self.persistence.get_api_key()
            && !key.is_empty() {
                return AuthToken::ApiKey(key);
            }

        // 4. OAuth access token (Gemini, etc.)
        if let Ok(Some(token)) = self.persistence.get_refresh_token() {
            return AuthToken::Bearer(token);
        }

        AuthToken::None
    }

    /// Return the preferred model name for the current provider.
    /// Falls back to the default model for that provider.
    pub fn resolve_model(&self) -> String {
        let provider = self.active_provider();
        self.persistence
            .get_model_preference()
            .unwrap_or_else(|_| default_model_for(&provider))
    }

    /// Validate that authentication is available, returning an error if not.
    pub fn require_auth(&self) -> Result<AuthToken> {
        let token = self.resolve_auth_token();
        if matches!(token, AuthToken::None) {
            return Err(DalangError::NoCredentials.into());
        }
        Ok(token)
    }

    /// Save a new API key for the active provider.
    pub fn save_api_key(&self, key: &str) -> Result<()> {
        self.persistence.save_api_key(key)
    }

    /// Save a new active provider selection.
    pub fn save_provider(&self, provider: &str) -> Result<()> {
        self.persistence.save_active_provider(provider)
    }

    /// Save the preferred model.
    pub fn save_model(&self, model: &str) -> Result<()> {
        self.persistence.save_model_preference(model)
    }

    /// Return whether verbose logging is enabled.
    pub fn verbose(&self) -> bool {
        self.persistence.get_verbose().unwrap_or(false)
    }
}

/// Default model name for a given provider.
fn default_model_for(provider: &str) -> String {
    match provider {
        "anthropic" => "claude-3-5-sonnet-20241022".to_string(),
        "openai" => "gpt-4o".to_string(),
        "copilot" => "gpt-4o".to_string(),
        "cloudcode" => "gemini-2.5-pro-preview-03-25".to_string(),
        _ => "gemini-2.0-flash".to_string(), // gemini default
    }
}
