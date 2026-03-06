//! WebSocket chat handler — bridges browser ↔ DalangOrchestrator via channels.

use crate::adapters::outbound::os_command::OsCommandExecutor;
use crate::application::ports::llm_port::LlmPort;
use crate::application::usecases::orchestrator::{DalangOrchestrator, OrchestratorConfig};
use crate::web::events::{ClientMessage, EngineEvent};
use crate::web::persistence;
use crate::web::state::{AppState, SessionMode};
use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use futures::SinkExt;
use futures::stream::StreamExt;
use std::sync::Arc;
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
    let conn_id = Uuid::new_v4();
    state
        .event_senders
        .insert(session_id, (conn_id, event_tx.clone()));

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
                    handle_client_message(text_str, session_id, &state_clone, &event_tx_clone)
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

    // Cleanup — only remove if this connection's sender is still the current one.
    state
        .event_senders
        .remove_if(&session_id, |_k, v| v.0 == conn_id);
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

/// Build a `DalangOrchestrator` from the current AppState and a wrapping LLM provider.
fn build_orchestrator(
    state: &AppState,
    provider: Box<dyn crate::llm::LlmProvider + Send + Sync>,
    cmd_timeout: u64,
) -> DalangOrchestrator {
    use crate::adapters::outbound::llm::new_shim;
    let disabled_skills: Vec<String> = state
        .disabled_skills
        .iter()
        .map(|e| e.key().clone())
        .collect();
    let llm: Arc<dyn LlmPort> = Arc::new(new_shim(provider));
    let executor = Arc::new(OsCommandExecutor);
    DalangOrchestrator::new(
        llm,
        executor,
        OrchestratorConfig {
            cmd_timeout,
            verbose: state.verbose,
            headless: state.headless,
            disabled_skills,
        },
    )
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

    let tx = event_tx.clone();
    let state_for_task = state.clone();

    // Get target and cmd_timeout from session
    let (target, cmd_timeout) = state
        .sessions
        .get(&session_id)
        .map(|s| (s.target.clone(), s.cmd_timeout))
        .unwrap_or_default();

    let orchestrator = build_orchestrator(state, provider, cmd_timeout);

    // Spawn orchestrator task: run_interactive_loop emits events via tx
    tokio::spawn(async move {
        match orchestrator
            .run_interactive_loop(&target, Some(tx.clone()))
            .await
        {
            Ok(()) => {
                // Persist any session state updates
                if let Some(session) = state_for_task.sessions.get(&session_id) {
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
    if let Some(mut session) = state.sessions.get_mut(&session_id) {
        session.cmd_timeout = cmd_timeout;
    } else {
        state.create_session(target.clone(), SessionMode::Scan, cmd_timeout);
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

    let orchestrator = build_orchestrator(state, provider, cmd_timeout);
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

        match orchestrator
            .run_autonomous_loop(&target, max_iter, Some(tx.clone()))
            .await
        {
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
    cmd_timeout: u64,
    state: &AppState,
    event_tx: &mpsc::Sender<EngineEvent>,
) {
    // Create session if not exists, or update cmd_timeout on existing
    if let Some(mut session) = state.sessions.get_mut(&session_id) {
        session.cmd_timeout = cmd_timeout;
    } else {
        state.create_session(target.clone(), SessionMode::Interactive, cmd_timeout);
    }

    let _ = event_tx
        .send(EngineEvent::Status {
            message: format!("Interactive session ready for target: {}", target),
        })
        .await;
}
