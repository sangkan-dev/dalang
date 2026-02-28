//! WebSocket chat handler — bridges browser ↔ DalangEngine via channels.

use crate::core::engine::DalangEngine;
use crate::web::events::{ClientMessage, EngineEvent};
use crate::web::persistence;
use crate::web::state::{AppState, SessionMode};
use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use futures::stream::StreamExt;
use futures::SinkExt;
use tokio::sync::mpsc;
use uuid::Uuid;

/// WebSocket upgrade handler at `/api/ws/{session_id}`
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(session_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, session_id, state))
}

async fn handle_socket(socket: WebSocket, session_id: Uuid, state: AppState) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Channel for engine events → WebSocket
    let (event_tx, mut event_rx) = mpsc::channel::<EngineEvent>(256);
    state.event_senders.insert(session_id, event_tx.clone());

    // Task: forward engine events to WebSocket AND persist to session + disk
    let persist_state = state.clone();
    let persist_sid = session_id;
    let send_task = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            // Persist event to in-memory session and disk
            {
                if let Some(mut session) = persist_state.sessions.get_mut(&persist_sid) {
                    session.events.push(event.clone());
                    persistence::save_events(&persist_sid, &session.events);
                }
            }

            let json = match serde_json::to_string(&event) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("[web] Failed to serialize event: {}", e);
                    continue;
                }
            };
            if ws_sender.send(WsMessage::Text(json.into())).await.is_err() {
                break; // Client disconnected
            }
        }
    });

    // Task: receive client messages from WebSocket
    let state_clone = state.clone();
    let event_tx_clone = event_tx.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                WsMessage::Text(text) => {
                    let text_str: &str = &text;
                    handle_client_message(
                        text_str,
                        session_id,
                        &state_clone,
                        &event_tx_clone,
                    )
                    .await;
                }
                WsMessage::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    // Cleanup
    state.event_senders.remove(&session_id);
}

async fn handle_client_message(
    text: &str,
    session_id: Uuid,
    state: &AppState,
    event_tx: &mpsc::Sender<EngineEvent>,
) {
    let client_msg: ClientMessage = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(e) => {
            let _ = event_tx
                .send(EngineEvent::Error {
                    message: format!("Invalid message format: {}", e),
                })
                .await;
            return;
        }
    };

    match client_msg {
        ClientMessage::Chat { message } => {
            handle_chat_message(session_id, message, state, event_tx).await;
        }
        ClientMessage::StartScan {
            target,
            max_iter,
            cmd_timeout,
        } => {
            handle_start_scan(session_id, target, max_iter, cmd_timeout, state, event_tx).await;
        }
        ClientMessage::StartInteractive {
            target,
            cmd_timeout,
        } => {
            handle_start_interactive(session_id, target, cmd_timeout, state, event_tx).await;
        }
    }
}

async fn handle_chat_message(
    session_id: Uuid,
    message: String,
    state: &AppState,
    event_tx: &mpsc::Sender<EngineEvent>,
) {
    // Add user message to session and persist
    if let Some(mut session) = state.sessions.get_mut(&session_id) {
        session.messages.push(crate::llm::Message::user(&message));
        persistence::save_messages(&session_id, &session.messages);
    }

    // Create LLM provider
    let provider = match state.create_llm_provider() {
        Ok(p) => p,
        Err(e) => {
            let _ = event_tx
                .send(EngineEvent::Error {
                    message: format!("LLM provider error: {}", e),
                })
                .await;
            return;
        }
    };

    let engine = DalangEngine::new(provider, 300, state.verbose);
    let tx = event_tx.clone();
    let state_for_task = state.clone();

    // Get target from session
    let target = state
        .sessions
        .get(&session_id)
        .map(|s| s.target.clone())
        .unwrap_or_default();

    // Get existing messages
    let messages = state
        .sessions
        .get(&session_id)
        .map(|s| s.messages.clone())
        .unwrap_or_default();

    // Spawn engine task
    tokio::spawn(async move {
        match engine
            .run_interactive_ws(&target, &messages, Some(session_id), tx.clone())
            .await
        {
            Ok(response_msgs) => {
                // Write response messages back to session and persist
                if let Some(mut session) = state_for_task.sessions.get_mut(&session_id) {
                    session.messages = response_msgs;
                    persistence::save_messages(&session_id, &session.messages);
                }
                let _ = tx.send(EngineEvent::Done {
                    reason: "Chat response complete".to_string(),
                });
            }
            Err(e) => {
                let _ = tx
                    .send(EngineEvent::Error {
                        message: format!("Engine error: {}", e),
                    })
                    .await;
            }
        }
    });
}

async fn handle_start_scan(
    session_id: Uuid,
    target: String,
    max_iter: u32,
    cmd_timeout: u64,
    state: &AppState,
    event_tx: &mpsc::Sender<EngineEvent>,
) {
    // Create/update session
    if !state.sessions.contains_key(&session_id) {
        state.create_session(target.clone(), SessionMode::Scan);
    }

    let provider = match state.create_llm_provider() {
        Ok(p) => p,
        Err(e) => {
            let _ = event_tx
                .send(EngineEvent::Error {
                    message: format!("LLM provider error: {}", e),
                })
                .await;
            return;
        }
    };

    let engine = DalangEngine::new(provider, cmd_timeout, state.verbose);
    let tx = event_tx.clone();
    let state_for_task = state.clone();

    tokio::spawn(async move {
        let _ = tx
            .send(EngineEvent::Status {
                message: format!(
                    "Starting autonomous scan on {} (max_iter: {})",
                    target, max_iter
                ),
            })
            .await;

        match engine.run_autonomous_ws(&target, max_iter, Some(session_id), tx.clone()).await {
            Ok(_) => {
                let _ = tx.send(EngineEvent::Done {
                    reason: "Scan complete".to_string(),
                });
            }
            Err(e) => {
                let _ = tx
                    .send(EngineEvent::Error {
                        message: format!("Scan error: {}", e),
                    })
                    .await;
            }
        }

        // Mark session as inactive and persist
        if let Some(mut session) = state_for_task.sessions.get_mut(&session_id) {
            session.active = false;
            persistence::save_session_meta(&session);
        }
    });
}

async fn handle_start_interactive(
    session_id: Uuid,
    target: String,
    _cmd_timeout: u64,
    state: &AppState,
    event_tx: &mpsc::Sender<EngineEvent>,
) {
    // Create session if not exists
    if !state.sessions.contains_key(&session_id) {
        state.create_session(target.clone(), SessionMode::Interactive);
    }

    let _ = event_tx
        .send(EngineEvent::Status {
            message: format!("Interactive session ready for target: {}", target),
        })
        .await;
}
