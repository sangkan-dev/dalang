use anyhow::{Result, anyhow};
use keyring::Entry;

const SERVICE_NAME: &str = "dalang";
const ACCESS_TOKEN_KEY: &str = "access_token";
const REFRESH_TOKEN_KEY: &str = "refresh_token";
const MODEL_PREF_KEY: &str = "model_preference";
const ACTIVE_PROVIDER_KEY: &str = "active_provider";

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
