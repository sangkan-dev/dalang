//! Session management REST API handlers.

use crate::web::state::{AppState, Session, SessionMode};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub target: String,
    #[serde(default = "default_mode")]
    pub mode: SessionMode,
}

fn default_mode() -> SessionMode {
    SessionMode::Interactive
}

/// POST /api/sessions — create a new session
pub async fn create_session(
    State(state): State<AppState>,
    Json(body): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    let session = state.create_session(body.target, body.mode);
    (StatusCode::CREATED, Json(session))
}

/// GET /api/sessions — list all sessions
pub async fn list_sessions(State(state): State<AppState>) -> impl IntoResponse {
    let sessions: Vec<Session> = state
        .sessions
        .iter()
        .map(|entry| entry.value().clone())
        .collect();
    Json(sessions)
}

/// GET /api/sessions/:id/messages — get chat history for a session
pub async fn get_session_messages(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.sessions.get(&id) {
        Some(session) => Ok(Json(session.messages.clone())),
        None => Err((StatusCode::NOT_FOUND, "Session not found")),
    }
}

/// DELETE /api/sessions/:id — delete a session
pub async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    if state.sessions.remove(&id).is_some() {
        state.event_senders.remove(&id);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
