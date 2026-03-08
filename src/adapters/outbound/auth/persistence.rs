use anyhow::{Result, anyhow};
use keyring::Entry;

const SERVICE_NAME: &str = "dalang";
const ACCESS_TOKEN_KEY: &str = "access_token";
const REFRESH_TOKEN_KEY: &str = "refresh_token";
const MODEL_PREF_KEY: &str = "model_preference";
const ACTIVE_PROVIDER_KEY: &str = "active_provider";
const AUTH_METHOD_KEY: &str = "auth_method";
const GCP_PROJECT_KEY: &str = "gcp_project";
const ENDPOINT_MODE_KEY: &str = "endpoint_mode";
const CODEASSIST_ENDPOINT_KEY: &str = "codeassist_endpoint";
const CODEASSIST_TIER_KEY: &str = "codeassist_tier";
const OAUTH_CLIENT_ID_KEY: &str = "oauth_client_id";
const OAUTH_CLIENT_SECRET_KEY: &str = "oauth_client_secret";
const API_KEY_KEY: &str = "api_key";
const VERBOSE_KEY: &str = "verbose";
const CUSTOM_BASE_URL_KEY: &str = "custom_base_url";

pub fn save_tokens(access_token: &str, refresh_token: Option<&str>) -> Result<()> {
    let entry =
        Entry::new(SERVICE_NAME, ACCESS_TOKEN_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .set_password(access_token)
        .map_err(|e| anyhow!("Failed to save access token: {}", e))?;

    if let Some(refresh) = refresh_token {
        let entry = Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY)
            .map_err(|e| anyhow!("Keyring error: {}", e))?;
        entry
            .set_password(refresh)
            .map_err(|e| anyhow!("Failed to save refresh token: {}", e))?;
    }

    Ok(())
}

pub fn get_access_token() -> Result<String> {
    let entry =
        Entry::new(SERVICE_NAME, ACCESS_TOKEN_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No access token found: {}", e))
}

pub fn get_refresh_token() -> Result<String> {
    let entry =
        Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No refresh token found: {}", e))
}

pub fn delete_tokens() -> Result<()> {
    let entry =
        Entry::new(SERVICE_NAME, ACCESS_TOKEN_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    let _ = entry.delete_password();

    let entry =
        Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    let _ = entry.delete_password();

    Ok(())
}

pub fn save_model_preference(model: &str) -> Result<()> {
    let entry =
        Entry::new(SERVICE_NAME, MODEL_PREF_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .set_password(model)
        .map_err(|e| anyhow!("Failed to save model preference: {}", e))
}

pub fn get_model_preference() -> Result<String> {
    let entry =
        Entry::new(SERVICE_NAME, MODEL_PREF_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No model preference found: {}", e))
}

pub fn save_active_provider(provider: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, ACTIVE_PROVIDER_KEY)
        .map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .set_password(provider)
        .map_err(|e| anyhow!("Failed to save active provider: {}", e))
}

pub fn get_active_provider() -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, ACTIVE_PROVIDER_KEY)
        .map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No active provider found: {}", e))
}

pub fn save_auth_method(method: &str) -> Result<()> {
    let entry =
        Entry::new(SERVICE_NAME, AUTH_METHOD_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .set_password(method)
        .map_err(|e| anyhow!("Failed to save auth method: {}", e))
}

pub fn get_auth_method() -> Result<String> {
    let entry =
        Entry::new(SERVICE_NAME, AUTH_METHOD_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No auth method found: {}", e))
}

pub fn save_gcp_project(project: &str) -> Result<()> {
    let entry =
        Entry::new(SERVICE_NAME, GCP_PROJECT_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .set_password(project)
        .map_err(|e| anyhow!("Failed to save GCP project: {}", e))
}

pub fn get_gcp_project() -> Result<String> {
    let entry =
        Entry::new(SERVICE_NAME, GCP_PROJECT_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No GCP project found: {}", e))
}

// -- Endpoint mode ("openai_compat" | "cloudcode") --

pub fn save_endpoint_mode(mode: &str) -> Result<()> {
    let entry =
        Entry::new(SERVICE_NAME, ENDPOINT_MODE_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .set_password(mode)
        .map_err(|e| anyhow!("Failed to save endpoint mode: {}", e))
}

pub fn get_endpoint_mode() -> Result<String> {
    let entry =
        Entry::new(SERVICE_NAME, ENDPOINT_MODE_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No endpoint mode found: {}", e))
}

// -- CloudCode Assist active endpoint --

pub fn save_codeassist_endpoint(endpoint: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, CODEASSIST_ENDPOINT_KEY)
        .map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .set_password(endpoint)
        .map_err(|e| anyhow!("Failed to save codeassist endpoint: {}", e))
}

pub fn get_codeassist_endpoint() -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, CODEASSIST_ENDPOINT_KEY)
        .map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No codeassist endpoint found: {}", e))
}

// -- CloudCode Assist tier --

pub fn save_codeassist_tier(tier: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, CODEASSIST_TIER_KEY)
        .map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .set_password(tier)
        .map_err(|e| anyhow!("Failed to save codeassist tier: {}", e))
}

pub fn get_codeassist_tier() -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, CODEASSIST_TIER_KEY)
        .map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No codeassist tier found: {}", e))
}

// -- OAuth client credentials (for Gemini CLI OAuth flow) --

pub fn save_oauth_client_id(client_id: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, OAUTH_CLIENT_ID_KEY)
        .map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .set_password(client_id)
        .map_err(|e| anyhow!("Failed to save oauth client id: {}", e))
}

pub fn get_oauth_client_id() -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, OAUTH_CLIENT_ID_KEY)
        .map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No oauth client id found: {}", e))
}

pub fn save_oauth_client_secret(secret: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, OAUTH_CLIENT_SECRET_KEY)
        .map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .set_password(secret)
        .map_err(|e| anyhow!("Failed to save oauth client secret: {}", e))
}

pub fn get_oauth_client_secret() -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, OAUTH_CLIENT_SECRET_KEY)
        .map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No oauth client secret found: {}", e))
}

pub fn delete_oauth_credentials() -> Result<()> {
    if let Ok(entry) = Entry::new(SERVICE_NAME, OAUTH_CLIENT_ID_KEY) {
        let _ = entry.delete_password();
    }
    if let Ok(entry) = Entry::new(SERVICE_NAME, OAUTH_CLIENT_SECRET_KEY) {
        let _ = entry.delete_password();
    }
    Ok(())
}

pub fn save_api_key(key: &str) -> Result<()> {
    let entry =
        Entry::new(SERVICE_NAME, API_KEY_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .set_password(key)
        .map_err(|e| anyhow!("Failed to save API key: {}", e))
}

pub fn get_api_key() -> Result<String> {
    let entry =
        Entry::new(SERVICE_NAME, API_KEY_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No API key found: {}", e))
}

pub fn save_verbose(verbose: bool) -> Result<()> {
    let entry =
        Entry::new(SERVICE_NAME, VERBOSE_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .set_password(if verbose { "true" } else { "false" })
        .map_err(|e| anyhow!("Failed to save verbose: {}", e))
}

pub fn get_verbose() -> Result<bool> {
    let entry =
        Entry::new(SERVICE_NAME, VERBOSE_KEY).map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map(|v| v == "true")
        .map_err(|e| anyhow!("No verbose setting found: {}", e))
}

pub fn save_custom_base_url(url: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, CUSTOM_BASE_URL_KEY)
        .map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .set_password(url)
        .map_err(|e| anyhow!("Failed to save custom base url: {}", e))
}

pub fn get_custom_base_url() -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, CUSTOM_BASE_URL_KEY)
        .map_err(|e| anyhow!("Keyring error: {}", e))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No custom base url found: {}", e))
}
