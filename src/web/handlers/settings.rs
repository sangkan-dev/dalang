//! Settings REST API handlers.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

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
}

#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub model: Option<String>,
}

/// GET /api/settings — get current configuration
pub async fn get_settings(State(_state): State<AppState>) -> impl IntoResponse {
    let provider =
        auth::persistence::get_active_provider().unwrap_or_else(|_| "gemini".to_string());
    let model = auth::persistence::get_model_preference()
        .unwrap_or_else(|_| llm::get_default_model(&provider));
    let auth_method =
        auth::persistence::get_auth_method().unwrap_or_else(|_| "apikey".to_string());
    let endpoint_mode =
        auth::persistence::get_endpoint_mode().unwrap_or_else(|_| "openai_compat".to_string());

    let auth_status = if auth::persistence::get_access_token().is_ok() {
        "authenticated".to_string()
    } else if std::env::var("LLM_API_KEY").is_ok() {
        "env_var".to_string()
    } else {
        "not_authenticated".to_string()
    };

    Json(SettingsResponse {
        provider,
        model,
        auth_method,
        endpoint_mode,
        auth_status,
    })
}

/// PUT /api/settings — update settings
pub async fn update_settings(
    State(_state): State<AppState>,
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
                    .into_response()
            }
        }
    }

    // Return updated settings
    let provider =
        auth::persistence::get_active_provider().unwrap_or_else(|_| "gemini".to_string());
    let model = auth::persistence::get_model_preference()
        .unwrap_or_else(|_| llm::get_default_model(&provider));
    let auth_method =
        auth::persistence::get_auth_method().unwrap_or_else(|_| "apikey".to_string());
    let endpoint_mode =
        auth::persistence::get_endpoint_mode().unwrap_or_else(|_| "openai_compat".to_string());
    let auth_status = if auth::persistence::get_access_token().is_ok() {
        "authenticated"
    } else {
        "not_authenticated"
    };

    Json(SettingsResponse {
        provider,
        model,
        auth_method,
        endpoint_mode,
        auth_status: auth_status.to_string(),
    })
    .into_response()
}
