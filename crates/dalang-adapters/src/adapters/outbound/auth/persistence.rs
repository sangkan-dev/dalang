use anyhow::{Result, anyhow};
use keyring::Entry;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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

// ── Storage Abstraction ──────────────────────────────────────────────────────

trait AuthStorage {
    fn set(&self, key: &str, value: &str) -> Result<()>;
    fn get(&self, key: &str) -> Result<String>;
    fn delete(&self, key: &str) -> Result<()>;
}

struct KeyringStorage;

impl AuthStorage for KeyringStorage {
    fn set(&self, key: &str, value: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key).map_err(|e| anyhow!("Keyring error: {}", e))?;
        entry
            .set_password(value)
            .map_err(|e| anyhow!("Failed to save {}: {}", key, e))
    }

    fn get(&self, key: &str) -> Result<String> {
        let entry = Entry::new(SERVICE_NAME, key).map_err(|e| anyhow!("Keyring error: {}", e))?;
        entry
            .get_password()
            .map_err(|e| anyhow!("No {} found: {}", key, e))
    }

    fn delete(&self, key: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key).map_err(|e| anyhow!("Keyring error: {}", e))?;
        let _ = entry.delete_password();
        Ok(())
    }
}

struct FileStorage {
    path: PathBuf,
}

impl FileStorage {
    fn new() -> Self {
        let path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".dalang")
            .join("auth.json");
        Self { path }
    }

    fn load(&self) -> HashMap<String, String> {
        fs::read_to_string(&self.path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self, map: &HashMap<String, String>) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(map)?;
        fs::write(&self.path, json)?;

        // Set restrictive permissions on Linux
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.path)?.permissions();
            perms.set_mode(0o600);
            let _ = fs::set_permissions(&self.path, perms);
        }
        Ok(())
    }
}

impl AuthStorage for FileStorage {
    fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut map = self.load();
        map.insert(key.to_string(), value.to_string());
        self.save(&map)
    }

    fn get(&self, key: &str) -> Result<String> {
        let map = self.load();
        map.get(key)
            .cloned()
            .ok_or_else(|| anyhow!("No {} found in file storage", key))
    }

    fn delete(&self, key: &str) -> Result<()> {
        let mut map = self.load();
        map.remove(key);
        self.save(&map)
    }
}

lazy_static::lazy_static! {
    static ref STORAGE: Box<dyn AuthStorage + Send + Sync> = {
        let is_docker = std::env::var("DALANG_DOCKER").map(|v| v == "true").unwrap_or(false);

        if is_docker {
            println!("[*] Docker environment detected, using file-based auth storage.");
            return Box::new(FileStorage::new());
        }

        // Try to initialize keyring, fallback to file if it fails
        match Entry::new(SERVICE_NAME, "test") {
            Ok(_) => Box::new(KeyringStorage),
            Err(e) => {
                println!("[!] Keyring unavailable ({}), falling back to file-based auth storage.", e);
                Box::new(FileStorage::new())
            }
        }
    };
}

// ── Public API ───────────────────────────────────────────────────────────────

pub fn save_tokens(access_token: &str, refresh_token: Option<&str>) -> Result<()> {
    STORAGE.set(ACCESS_TOKEN_KEY, access_token)?;
    if let Some(refresh) = refresh_token {
        STORAGE.set(REFRESH_TOKEN_KEY, refresh)?;
    }
    Ok(())
}

pub fn get_access_token() -> Result<String> {
    STORAGE.get(ACCESS_TOKEN_KEY)
}

pub fn get_refresh_token() -> Result<String> {
    STORAGE.get(REFRESH_TOKEN_KEY)
}

pub fn delete_tokens() -> Result<()> {
    STORAGE.delete(ACCESS_TOKEN_KEY)?;
    STORAGE.delete(REFRESH_TOKEN_KEY)?;
    Ok(())
}

pub fn save_model_preference(model: &str) -> Result<()> {
    STORAGE.set(MODEL_PREF_KEY, model)
}

pub fn get_model_preference() -> Result<String> {
    STORAGE.get(MODEL_PREF_KEY)
}

pub fn save_active_provider(provider: &str) -> Result<()> {
    STORAGE.set(ACTIVE_PROVIDER_KEY, provider)
}

pub fn get_active_provider() -> Result<String> {
    STORAGE.get(ACTIVE_PROVIDER_KEY)
}

pub fn save_auth_method(method: &str) -> Result<()> {
    STORAGE.set(AUTH_METHOD_KEY, method)
}

pub fn get_auth_method() -> Result<String> {
    STORAGE.get(AUTH_METHOD_KEY)
}

pub fn save_gcp_project(project: &str) -> Result<()> {
    STORAGE.set(GCP_PROJECT_KEY, project)
}

pub fn get_gcp_project() -> Result<String> {
    STORAGE.get(GCP_PROJECT_KEY)
}

pub fn save_endpoint_mode(mode: &str) -> Result<()> {
    STORAGE.set(ENDPOINT_MODE_KEY, mode)
}

pub fn get_endpoint_mode() -> Result<String> {
    STORAGE.get(ENDPOINT_MODE_KEY)
}

pub fn save_codeassist_endpoint(endpoint: &str) -> Result<()> {
    STORAGE.set(CODEASSIST_ENDPOINT_KEY, endpoint)
}

pub fn get_codeassist_endpoint() -> Result<String> {
    STORAGE.get(CODEASSIST_ENDPOINT_KEY)
}

pub fn save_codeassist_tier(tier: &str) -> Result<()> {
    STORAGE.set(CODEASSIST_TIER_KEY, tier)
}

pub fn get_codeassist_tier() -> Result<String> {
    STORAGE.get(CODEASSIST_TIER_KEY)
}

pub fn save_oauth_client_id(client_id: &str) -> Result<()> {
    STORAGE.set(OAUTH_CLIENT_ID_KEY, client_id)
}

pub fn get_oauth_client_id() -> Result<String> {
    STORAGE.get(OAUTH_CLIENT_ID_KEY)
}

pub fn save_oauth_client_secret(secret: &str) -> Result<()> {
    STORAGE.set(OAUTH_CLIENT_SECRET_KEY, secret)
}

pub fn get_oauth_client_secret() -> Result<String> {
    STORAGE.get(OAUTH_CLIENT_SECRET_KEY)
}

pub fn delete_oauth_credentials() -> Result<()> {
    STORAGE.delete(OAUTH_CLIENT_ID_KEY)?;
    STORAGE.delete(OAUTH_CLIENT_SECRET_KEY)?;
    Ok(())
}

pub fn save_api_key(key: &str) -> Result<()> {
    STORAGE.set(API_KEY_KEY, key.trim())
}

pub fn get_api_key() -> Result<String> {
    let key = STORAGE.get(API_KEY_KEY)?;
    let trimmed = key.trim().to_string();
    if trimmed.is_empty() {
        Err(anyhow!("Stored API key is empty"))
    } else {
        Ok(trimmed)
    }
}

pub fn save_verbose(verbose: bool) -> Result<()> {
    STORAGE.set(VERBOSE_KEY, if verbose { "true" } else { "false" })
}

pub fn get_verbose() -> Result<bool> {
    STORAGE.get(VERBOSE_KEY).map(|v| v == "true")
}

pub fn save_custom_base_url(url: &str) -> Result<()> {
    STORAGE.set(CUSTOM_BASE_URL_KEY, url.trim())
}

pub fn get_custom_base_url() -> Result<String> {
    let url = STORAGE.get(CUSTOM_BASE_URL_KEY)?;
    let trimmed = url.trim().to_string();
    if trimmed.is_empty() {
        Err(anyhow!("Stored custom base URL is empty"))
    } else {
        Ok(trimmed)
    }
}
