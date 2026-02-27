use anyhow::{Result, anyhow};
use keyring::Entry;

const SERVICE_NAME: &str = "dalang";
const ACCESS_TOKEN_KEY: &str = "access_token";
const REFRESH_TOKEN_KEY: &str = "refresh_token";

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
