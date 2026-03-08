//! Settings REST API handlers.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::auth;
use crate::llm;
use crate::web::state::AppState;

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
    let provider =
        auth::persistence::get_active_provider().unwrap_or_else(|_| "gemini".to_string());
    let model = auth::persistence::get_model_preference()
        .unwrap_or_else(|_| llm::get_default_model(&provider));
    let auth_method = auth::persistence::get_auth_method().unwrap_or_else(|_| "apikey".to_string());
    let endpoint_mode =
        auth::persistence::get_endpoint_mode().unwrap_or_else(|_| "openai_compat".to_string());

    let auth_status = if auth::persistence::get_access_token().is_ok() {
        "authenticated".to_string()
    } else if std::env::var("LLM_API_KEY").is_ok() {
        "env_var".to_string()
    } else {
        "not_authenticated".to_string()
    };

    let has_api_key =
        auth::persistence::get_api_key().is_ok() || std::env::var("LLM_API_KEY").is_ok();
    let verbose = auth::persistence::get_verbose().unwrap_or(state.verbose);
    let custom_base_url = auth::persistence::get_custom_base_url().ok();

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
    if let Some(model) = body.model {
        match auth::persistence::save_model_preference(&model) {
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
        match auth::persistence::save_active_provider(&provider) {
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

    if let Some(endpoint_mode) = body.endpoint_mode {
        match auth::persistence::save_endpoint_mode(&endpoint_mode) {
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

    if let Some(api_key) = body.api_key
        && !api_key.is_empty()
    {
        match auth::persistence::save_api_key(&api_key) {
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
        let _ = auth::persistence::save_verbose(verbose);
    }

    if let Some(url) = body.custom_base_url {
        match auth::persistence::save_custom_base_url(&url) {
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
    let provider =
        auth::persistence::get_active_provider().unwrap_or_else(|_| "gemini".to_string());
    let model = auth::persistence::get_model_preference()
        .unwrap_or_else(|_| llm::get_default_model(&provider));
    let auth_method = auth::persistence::get_auth_method().unwrap_or_else(|_| "apikey".to_string());
    let endpoint_mode =
        auth::persistence::get_endpoint_mode().unwrap_or_else(|_| "openai_compat".to_string());
    let auth_status = if auth::persistence::get_access_token().is_ok() {
        "authenticated"
    } else if std::env::var("LLM_API_KEY").is_ok() {
        "env_var"
    } else {
        "not_authenticated"
    };
    let has_api_key =
        auth::persistence::get_api_key().is_ok() || std::env::var("LLM_API_KEY").is_ok();
    let verbose = auth::persistence::get_verbose().unwrap_or(state.verbose);
    let custom_base_url = auth::persistence::get_custom_base_url().ok();

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
