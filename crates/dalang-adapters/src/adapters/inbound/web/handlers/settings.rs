//! Settings REST API handlers.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::adapters::inbound::web::state::AppState;
use crate::adapters::outbound::auth;
use crate::adapters::outbound::llm;

#[derive(Serialize)]
pub struct SettingsResponse {
    pub provider: String,
    pub model: String,
    pub auth_method: String,
    pub endpoint_mode: String,
    pub auth_status: String,
    pub has_api_key: bool,
    pub verbose: bool,
    pub custom_base_url: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub model: Option<String>,
    pub provider: Option<String>,
    pub endpoint_mode: Option<String>,
    pub api_key: Option<String>,
    pub verbose: Option<bool>,
    pub custom_base_url: Option<String>,
}

/// GET /api/settings — get current configuration
pub async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    let auth = state.auth.as_ref();
    let provider = auth
        .get_active_provider()
        .unwrap_or_else(|_| "gemini".to_string());
    let model = auth
        .get_model_preference()
        .unwrap_or_else(|_| llm::get_default_model(&provider));
    let auth_method = auth
        .get_auth_method()
        .unwrap_or_else(|_| "apikey".to_string());
    let endpoint_mode = auth
        .get_endpoint_mode()
        .unwrap_or_else(|_| "openai_compat".to_string());

    let auth_status = if auth.get_access_token().is_ok() {
        "authenticated".to_string()
    } else if std::env::var("LLM_API_KEY").is_ok() {
        "env_var".to_string()
    } else {
        "not_authenticated".to_string()
    };

    let has_api_key = auth.get_api_key().map(|o| o.is_some()).unwrap_or(false)
        || std::env::var("LLM_API_KEY").is_ok();
    let verbose = auth.get_verbose().unwrap_or(state.verbose);
    let custom_base_url = auth.get_custom_base_url().ok();

    Json(SettingsResponse {
        provider,
        model,
        auth_method,
        endpoint_mode,
        auth_status,
        has_api_key,
        verbose,
        custom_base_url,
    })
}

/// PUT /api/settings — update settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<UpdateSettingsRequest>,
) -> impl IntoResponse {
    let next_model = body.model.clone();
    let next_provider = body.provider.clone();

    if let Some(model) = body.model {
        match state.auth.save_model_preference(&model) {
            Ok(_) => {}
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to save model: {}", e),
                )
                    .into_response();
            }
        }
    }

    if let Some(provider) = body.provider {
        match state.auth.save_active_provider(&provider) {
            Ok(_) => {}
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to save provider: {}", e),
                )
                    .into_response();
            }
        }
    }

    if let Some(ref endpoint_mode) = body.endpoint_mode {
        match state.auth.save_endpoint_mode(endpoint_mode) {
            Ok(_) => {}
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to save endpoint mode: {}", e),
                )
                    .into_response();
            }
        }
    }

    // Auto-align endpoint_mode for providers/models that require it, unless user explicitly set endpoint_mode.
    if body.endpoint_mode.is_none() {
        if let Some(provider) = next_provider.as_deref()
            && provider.eq_ignore_ascii_case("copilot")
        {
            // gpt-5.3-codex requires CAPI messages endpoint.
            let mode = match next_model.as_deref() {
                Some(m) if m.eq_ignore_ascii_case("gpt-5.3-codex") => "copilot_capi",
                _ => "copilot",
            };
            let _ = state.auth.save_endpoint_mode(mode);
        }
    }

    if let Some(api_key) = body.api_key
        && !api_key.is_empty()
    {
        match state.auth.save_api_key(&api_key) {
            Ok(_) => {}
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to save API key: {}", e),
                )
                    .into_response();
            }
        }
    }

    if let Some(verbose) = body.verbose {
        let _ = state.auth.save_verbose(verbose);
    }

    if let Some(url) = body.custom_base_url {
        match state.auth.save_custom_base_url(&url) {
            Ok(_) => {}
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to save custom base url: {}", e),
                )
                    .into_response();
            }
        }
    }

    // Return updated settings
    let auth = state.auth.as_ref();
    let provider = auth
        .get_active_provider()
        .unwrap_or_else(|_| "gemini".to_string());
    let model = auth
        .get_model_preference()
        .unwrap_or_else(|_| llm::get_default_model(&provider));
    let auth_method = auth
        .get_auth_method()
        .unwrap_or_else(|_| "apikey".to_string());
    let endpoint_mode = auth
        .get_endpoint_mode()
        .unwrap_or_else(|_| "openai_compat".to_string());
    let auth_status = if auth.get_access_token().is_ok() {
        "authenticated"
    } else if std::env::var("LLM_API_KEY").is_ok() {
        "env_var"
    } else {
        "not_authenticated"
    };
    let has_api_key = auth.get_api_key().map(|o| o.is_some()).unwrap_or(false)
        || std::env::var("LLM_API_KEY").is_ok();
    let verbose = auth.get_verbose().unwrap_or(state.verbose);
    let custom_base_url = auth.get_custom_base_url().ok();

    Json(SettingsResponse {
        provider,
        model,
        auth_method,
        endpoint_mode,
        auth_status: auth_status.to_string(),
        has_api_key,
        verbose,
        custom_base_url,
    })
    .into_response()
}

#[derive(Serialize)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub message: String,
    pub latency_ms: u64,
}

/// POST /api/settings/test-connection — test LLM connection
pub async fn test_connection(State(state): State<AppState>) -> impl IntoResponse {
    let start = Instant::now();

    let provider = match state.create_llm_provider() {
        Ok(p) => p,
        Err(e) => {
            return Json(TestConnectionResponse {
                success: false,
                message: format!("Provider setup failed: {}", e),
                latency_ms: start.elapsed().as_millis() as u64,
            })
            .into_response();
        }
    };

    let messages = vec![
        llm::Message {
            role: "system".to_string(),
            content: "Reply with exactly: OK".to_string(),
        },
        llm::Message {
            role: "user".to_string(),
            content: "test".to_string(),
        },
    ];

    match provider.send_messages(&messages).await {
        Ok(resp) => {
            let latency = start.elapsed().as_millis() as u64;
            let preview = if resp.len() > 100 {
                &resp[..100]
            } else {
                &resp
            };
            Json(TestConnectionResponse {
                success: true,
                message: format!("Connected! Response: {}", preview),
                latency_ms: latency,
            })
            .into_response()
        }
        Err(e) => {
            let latency = start.elapsed().as_millis() as u64;
            Json(TestConnectionResponse {
                success: false,
                message: format!("Connection failed: {}", e),
                latency_ms: latency,
            })
            .into_response()
        }
    }
}

// ── OAuth / CLI Auth helpers for Web Settings ─────────────────────────────────

#[derive(Deserialize)]
pub struct AuthStartRequest {
    pub provider: String,
}

#[derive(Serialize)]
#[serde(tag = "kind")]
pub enum AuthStartResponse {
    #[serde(rename = "copilot_device")]
    CopilotDevice {
        verification_uri: String,
        user_code: String,
        expires_in: u64,
        device_code: String,
        interval: u64,
    },
    #[serde(rename = "cli_extract")]
    CliExtract { message: String },
}

/// POST /api/settings/auth/start — start an auth flow or attempt CLI extraction.
pub async fn auth_start(
    State(state): State<AppState>,
    Json(body): Json<AuthStartRequest>,
) -> impl IntoResponse {
    let provider = body.provider.to_lowercase();

    if provider == "copilot" || provider == "github" || provider == "github-copilot" {
        // Start GitHub device flow (step 1 only) and return verification URI + code.
        // We intentionally do NOT block/poll in HTTP request.
        let client = reqwest::Client::new();
        let resp = match client
            .post("https://github.com/login/device/code")
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("User-Agent", "GithubCopilot/1.155.0")
            .form(&[
                ("client_id", "01ab8ac9400c4e429b23"),
                ("scope", "read:user user:email"),
            ])
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    format!("Failed to contact GitHub: {}", e),
                )
                    .into_response();
            }
        };

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return (
                StatusCode::BAD_GATEWAY,
                format!("GitHub device code request failed ({}): {}", status, text),
            )
                .into_response();
        }

        #[derive(Deserialize)]
        struct DeviceCodeResponse {
            device_code: String,
            user_code: String,
            verification_uri: String,
            interval: u64,
            expires_in: u64,
        }

        let device: DeviceCodeResponse = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    format!("Invalid GitHub response: {}", e),
                )
                    .into_response();
            }
        };

        return Json(AuthStartResponse::CopilotDevice {
            verification_uri: device.verification_uri,
            user_code: device.user_code,
            expires_in: device.expires_in,
            device_code: device.device_code,
            interval: device.interval,
        })
        .into_response();
    }

    if provider == "gemini" {
        // Attempt to extract a bearer token from gcloud or gemini-cli on the host.
        // If found, persist it as a bearer token for Cloud Code Assist.
        match auth::cli_extractor::try_all_cli_extractors() {
            Some(token) => {
                if let Err(e) = state.auth.save_tokens(&token, None) {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to persist token: {}", e),
                    )
                        .into_response();
                }
                let _ = state.auth.save_auth_method("bearer");
                let _ = state.auth.save_endpoint_mode("cloudcode");
                let _ = state.auth.save_active_provider("gemini");
                Json(AuthStartResponse::CliExtract {
                    message:
                        "Sesi ditemukan dari gcloud/gemini-cli dan disimpan. Klik “Muat ulang status” untuk memastikan."
                            .to_string(),
                })
                .into_response()
            }
            None => Json(AuthStartResponse::CliExtract {
                message: "Belum ditemukan sesi CLI. Login dulu di terminal, mis. `dalang login --provider gemini` (atau `gcloud auth login` / `gemini auth login`), lalu coba lagi."
                    .to_string(),
            })
            .into_response(),
        }
    } else {
        Json(AuthStartResponse::CliExtract {
            message: "Untuk penyedia ini, gunakan API key atau login manual via `dalang login --provider <nama>`."
                .to_string(),
        })
        .into_response()
    }
}

#[derive(Deserialize)]
pub struct CopilotPollRequest {
    pub device_code: String,
}

#[derive(Serialize)]
#[serde(tag = "kind")]
pub enum CopilotPollResponse {
    #[serde(rename = "pending")]
    Pending { message: String },
    #[serde(rename = "authenticated")]
    Authenticated { message: String },
}

/// POST /api/settings/auth/copilot/poll — poll once to exchange device_code for token.
pub async fn copilot_poll(
    State(state): State<AppState>,
    Json(body): Json<CopilotPollRequest>,
) -> impl IntoResponse {
    let client = reqwest::Client::new();
    let poll_resp = match client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", "GithubCopilot/1.155.0")
        .form(&[
            ("client_id", "01ab8ac9400c4e429b23"),
            ("device_code", body.device_code.as_str()),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("Failed to contact GitHub: {}", e),
            )
                .into_response();
        }
    };

    if !poll_resp.status().is_success() {
        return Json(CopilotPollResponse::Pending {
            message: "Menunggu otorisasi…".to_string(),
        })
        .into_response();
    }

    #[derive(Deserialize)]
    struct AccessTokenResponse {
        access_token: Option<String>,
        error: Option<String>,
        error_description: Option<String>,
        interval: Option<u64>,
    }

    let token_resp: AccessTokenResponse = match poll_resp.json().await {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("Invalid GitHub response: {}", e),
            )
                .into_response();
        }
    };

    if let Some(access_token) = token_resp.access_token {
        // Persist similarly to CLI login.
        if let Err(e) = state.auth.save_tokens(&access_token, None) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to persist token: {}", e),
            )
                .into_response();
        }
        let _ = state.auth.save_auth_method("copilot_oauth");
        // Some Copilot models require CAPI model identifiers (eg gpt-5.3-codex).
        let model_pref = state.auth.get_model_preference().unwrap_or_default();
        let endpoint_mode = if model_pref.eq_ignore_ascii_case("gpt-5.3-codex") {
            "copilot_capi"
        } else {
            "copilot"
        };
        let _ = state.auth.save_endpoint_mode(endpoint_mode);
        let _ = state.auth.save_active_provider("copilot");
        return Json(CopilotPollResponse::Authenticated {
            message: "Login Copilot berhasil. Klik “Muat ulang status”.".to_string(),
        })
        .into_response();
    }

    match token_resp.error.as_deref() {
        Some("authorization_pending") => Json(CopilotPollResponse::Pending {
            message: "Menunggu otorisasi…".to_string(),
        })
        .into_response(),
        Some("slow_down") => Json(CopilotPollResponse::Pending {
            message: format!(
                "Terlalu cepat. Coba lagi beberapa detik lagi{}",
                token_resp
                    .interval
                    .map(|v| format!(" (interval disarankan: {}s)", v))
                    .unwrap_or_default()
            ),
        })
        .into_response(),
        Some("expired_token") => (
            StatusCode::BAD_REQUEST,
            "Kode perangkat kadaluarsa. Mulai login lagi.".to_string(),
        )
            .into_response(),
        Some("access_denied") => (
            StatusCode::BAD_REQUEST,
            "Login ditolak. Mulai login lagi jika diperlukan.".to_string(),
        )
            .into_response(),
        Some(err) => (
            StatusCode::BAD_REQUEST,
            format!(
                "OAuth error: {} - {}",
                err,
                token_resp
                    .error_description
                    .unwrap_or_else(|| "Unknown error".to_string())
            ),
        )
            .into_response(),
        None => Json(CopilotPollResponse::Pending {
            message: "Menunggu otorisasi…".to_string(),
        })
        .into_response(),
    }
}
