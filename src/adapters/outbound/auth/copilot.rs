use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

// ─── Constants ──────────────────────────────────────────────

/// GitHub Copilot CLI OAuth App client_id (extracted from @github/copilot v0.0.420)
pub const COPILOT_CLIENT_ID: &str = "Ov23ctDVkRmgkPke0Mmm";

/// Default OAuth scopes for Copilot device flow
pub const COPILOT_SCOPES: &str = "read:user,read:org,repo,gist";

/// Default GitHub host
pub const GITHUB_HOST: &str = "https://github.com";

/// Copilot CLI keychain service name (as found in the Copilot CLI source)
const COPILOT_CLI_KEYCHAIN_SERVICE: &str = "copilot-cli";

// ─── Data Structures ────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CopilotLoginResult {
    pub host: String,
    pub login: String,
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct CopilotSessionToken {
    pub token: String,
    pub expires_at: i64,
}

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[allow(dead_code)]
    expires_in: u64,
    interval: u64,
}

#[derive(Deserialize)]
struct AccessTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
    interval: Option<u64>,
}

#[derive(Deserialize)]
struct CopilotTokenResponse {
    token: String,
    expires_at: i64,
}

#[derive(Deserialize)]
struct CopilotUserResponse {
    pub login: Option<String>,
    #[allow(dead_code)]
    pub copilot_plan: Option<String>,
}

#[derive(Deserialize)]
struct CopilotConfigUser {
    host: Option<String>,
    login: Option<String>,
}

#[derive(Deserialize)]
struct CopilotConfig {
    last_logged_in_user: Option<CopilotConfigUser>,
}

// ─── Token Extraction ───────────────────────────────────────

/// Try to extract a GitHub token from environment variables.
/// Checks COPILOT_GITHUB_TOKEN → GH_TOKEN → GITHUB_TOKEN in priority order.
/// Rejects classic PATs (ghp_ prefix) as they are not supported.
pub fn extract_github_env_token() -> Result<String> {
    let env_vars = ["COPILOT_GITHUB_TOKEN", "GH_TOKEN", "GITHUB_TOKEN"];

    for var in &env_vars {
        if let Ok(token) = std::env::var(var) {
            let token = token.trim().to_string();
            if !token.is_empty() {
                if token.starts_with("ghp_") {
                    return Err(anyhow!(
                        "Classic PATs (ghp_) are not supported for Copilot. \
                         Use a fine-grained PAT or OAuth token instead."
                    ));
                }
                return Ok(token);
            }
        }
    }

    Err(anyhow!(
        "No GitHub token found in environment variables (COPILOT_GITHUB_TOKEN, GH_TOKEN, GITHUB_TOKEN)"
    ))
}

/// Try to extract the Copilot CLI token from the OS keychain.
/// The Copilot CLI stores tokens under service "copilot-cli" with account "{host}:{login}".
pub fn extract_copilot_keychain_token() -> Result<String> {
    // Read the copilot config to get the logged-in user
    let config = read_copilot_config()?;
    let user = config
        .last_logged_in_user
        .ok_or_else(|| anyhow!("No logged-in user found in copilot config"))?;

    let host = user.host.unwrap_or_else(|| GITHUB_HOST.to_string());
    let login = user
        .login
        .ok_or_else(|| anyhow!("No login found in copilot config"))?;

    let account = format!("{}:{}", host, login);

    let entry = keyring::Entry::new(COPILOT_CLI_KEYCHAIN_SERVICE, &account)
        .map_err(|e| anyhow!("Keyring error: {}", e))?;

    entry
        .get_password()
        .map_err(|e| anyhow!("Could not read Copilot CLI token from keychain: {}", e))
}

/// Try to extract a GitHub token from the `gh` CLI.
pub fn extract_gh_cli_token() -> Result<String> {
    let output = Command::new("gh").args(["auth", "token"]).output()?;

    if output.status.success() {
        let token = String::from_utf8(output.stdout)?.trim().to_string();
        if !token.is_empty() {
            if token.starts_with("ghp_") {
                return Err(anyhow!(
                    "Classic PATs (ghp_) are not supported. Use: gh auth login"
                ));
            }
            return Ok(token);
        }
    }

    Err(anyhow!("Could not extract token from gh CLI"))
}

/// Try all available methods to extract a GitHub/Copilot token.
/// Priority: env var → keychain → gh CLI
pub fn try_extract_copilot_token() -> Option<String> {
    if let Ok(token) = extract_github_env_token() {
        println!("[+] Found GitHub token from environment variable");
        return Some(token);
    }

    if let Ok(token) = extract_copilot_keychain_token() {
        println!("[+] Found token from Copilot CLI keychain");
        return Some(token);
    }

    if let Ok(token) = extract_gh_cli_token() {
        println!("[+] Found token from gh CLI");
        return Some(token);
    }

    None
}

// ─── Copilot Token Exchange ─────────────────────────────────

/// Exchange a GitHub OAuth token for a short-lived Copilot session token.
/// GET https://api.github.com/copilot_internal/v2/token
pub async fn exchange_copilot_session_token(github_token: &str) -> Result<CopilotSessionToken> {
    let client = reqwest::Client::new();

    let resp = client
        .get("https://api.github.com/copilot_internal/v2/token")
        .header("Authorization", format!("token {}", github_token))
        .header("User-Agent", "GithubCopilot/1.155.0")
        .header("Accept", "application/json")
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "Copilot token exchange failed ({}): {}",
            status,
            body
        ));
    }

    let token_resp: CopilotTokenResponse = resp.json().await?;
    Ok(CopilotSessionToken {
        token: token_resp.token,
        expires_at: token_resp.expires_at,
    })
}

/// Validate a GitHub token by calling the Copilot user endpoint.
/// Returns the user's login name.
pub async fn validate_github_token(token: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let resp = client
        .get("https://api.github.com/copilot_internal/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "GithubCopilot/1.155.0")
        .header("Accept", "application/json")
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "GitHub token validation failed ({}): {}. \
             Make sure you have an active Copilot subscription.",
            status,
            body
        ));
    }

    let user: CopilotUserResponse = resp.json().await?;
    Ok(user.login.unwrap_or_else(|| "unknown".to_string()))
}

// ─── GitHub Device Flow OAuth ───────────────────────────────

/// Perform the GitHub Device Flow OAuth login.
/// 1. Request device + user codes
/// 2. Show user the verification URL and code
/// 3. Poll for authorization
/// 4. Return the access token
pub async fn login_copilot_device_flow() -> Result<CopilotLoginResult> {
    let host = GITHUB_HOST;
    let client = reqwest::Client::new();

    // Step 1: Request device code
    println!("[*] Requesting device code from GitHub...");
    let resp = client
        .post(format!("{}/login/device/code", host))
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", "GithubCopilot/1.155.0")
        .form(&[("client_id", COPILOT_CLIENT_ID), ("scope", COPILOT_SCOPES)])
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "Failed to request device code ({}): {}",
            status,
            body
        ));
    }

    let device: DeviceCodeResponse = resp.json().await?;

    // Step 2: Show user the URL and code
    println!();
    println!("┌─────────────────────────────────────────────────┐");
    println!("│  GitHub Device Authorization                    │");
    println!("├─────────────────────────────────────────────────┤");
    println!(
        "│  Open:  {}{}│",
        device.verification_uri,
        " ".repeat(49usize.saturating_sub(8 + device.verification_uri.len()))
    );
    println!(
        "│  Code:  {}{}│",
        device.user_code,
        " ".repeat(49usize.saturating_sub(8 + device.user_code.len()))
    );
    println!("└─────────────────────────────────────────────────┘");
    println!();

    // Try to open browser automatically
    if open::that(&device.verification_uri).is_err() {
        println!("[*] Could not open browser. Please visit the URL manually.");
    }

    // Try to copy code to clipboard
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(device.user_code.as_bytes())?;
                }
                child.wait()
            });
    }

    println!(
        "[*] Waiting for authorization (polling every {}s)...",
        device.interval
    );

    // Step 3: Poll for access token
    let mut interval = std::cmp::max(device.interval, 5);
    let max_attempts = 60; // ~5 minutes with 5s interval

    for attempt in 1..=max_attempts {
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;

        let poll_resp = client
            .post(format!("{}/login/oauth/access_token", host))
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("User-Agent", "GithubCopilot/1.155.0")
            .form(&[
                ("client_id", COPILOT_CLIENT_ID),
                ("device_code", device.device_code.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await?;

        if !poll_resp.status().is_success() {
            continue;
        }

        let token_resp: AccessTokenResponse = poll_resp.json().await?;

        if let Some(access_token) = token_resp.access_token {
            println!("\n[+] Authorization successful!");

            // Validate and get login
            let login = match validate_github_token(&access_token).await {
                Ok(l) => l,
                Err(_) => "unknown".to_string(),
            };

            return Ok(CopilotLoginResult {
                host: host.to_string(),
                login,
                token: access_token,
            });
        }

        match token_resp.error.as_deref() {
            Some("authorization_pending") => {
                if attempt % 6 == 0 {
                    println!(
                        "[*] Still waiting for authorization... (attempt {}/{})",
                        attempt, max_attempts
                    );
                }
            }
            Some("slow_down") => {
                if let Some(new_interval) = token_resp.interval {
                    interval = new_interval + 2;
                } else {
                    interval += 5;
                }
            }
            Some("expired_token") => {
                return Err(anyhow!("Device code expired. Please try again."));
            }
            Some("access_denied") => {
                return Err(anyhow!("Authorization was denied by the user."));
            }
            Some(err) => {
                let desc = token_resp
                    .error_description
                    .unwrap_or_else(|| "Unknown error".to_string());
                return Err(anyhow!("OAuth error: {} - {}", err, desc));
            }
            None => {}
        }
    }

    Err(anyhow!(
        "Timed out waiting for GitHub authorization after {} attempts",
        max_attempts
    ))
}

// ─── Helpers ────────────────────────────────────────────────

/// Read the Copilot CLI config file at ~/.config/.copilot/config.json
fn read_copilot_config() -> Result<CopilotConfig> {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| anyhow!("Could not determine home directory"))?;

    let config_path = home.join(".config/.copilot/config.json");

    if !config_path.exists() {
        return Err(anyhow!(
            "Copilot CLI config not found at {}",
            config_path.display()
        ));
    }

    let content = std::fs::read_to_string(&config_path)?;
    let config: CopilotConfig = serde_json::from_str(&content)?;
    Ok(config)
}

/// Persist a Copilot login result to the Dalang keyring.
pub fn persist_copilot_login(result: &CopilotLoginResult) -> Result<()> {
    super::persistence::save_tokens(&result.token, None)?;
    let _ = super::persistence::save_auth_method("copilot_oauth");
    let _ = super::persistence::save_endpoint_mode("copilot");
    let _ = super::persistence::save_active_provider("copilot");
    Ok(())
}
