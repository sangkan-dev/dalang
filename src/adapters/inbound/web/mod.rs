//! Web UI server module — axum-based HTTP + WebSocket server.
//!
//! Serves the Svelte frontend (embedded via rust-embed) and provides
//! REST + WebSocket API for the chat interface.

pub mod embedded;
pub mod events;
pub mod handlers;
pub mod persistence;
pub mod state;
#[cfg(test)]
mod tests;

use axum::Router;
use axum::routing::{delete, get, post, put};
use state::AppState;
use tower_http::cors::{Any, CorsLayer};

/// Build the axum router with all API routes and static file fallback.
pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        // Sessions
        .route("/sessions", get(handlers::sessions::list_sessions))
        .route("/sessions", post(handlers::sessions::create_session))
        .route(
            "/sessions/{id}/messages",
            get(handlers::sessions::get_session_messages),
        )
        .route(
            "/sessions/{id}/events",
            get(handlers::sessions::get_session_events),
        )
        .route("/sessions/{id}", delete(handlers::sessions::delete_session))
        // WebSocket
        .route("/ws/{session_id}", get(handlers::chat::ws_handler))
        // Skills
        .route("/skills", get(handlers::skills::list_skills))
        .route(
            "/skills/{name}",
            get(handlers::skills::get_skill).put(handlers::skills::update_skill),
        )
        // Reports
        .route("/reports", get(handlers::reports::list_reports))
        .route("/reports/{filename}", get(handlers::reports::get_report))
        // Settings
        .route("/settings", get(handlers::settings::get_settings))
        .route("/settings", put(handlers::settings::update_settings))
        .route(
            "/settings/test-connection",
            post(handlers::settings::test_connection),
        );

    Router::new()
        .nest("/api", api_routes)
        .fallback(embedded::static_handler)
        .layer(cors)
        .with_state(state)
}

/// Start the web server on the given port.
pub async fn start_server(
    port: u16,
    open_browser: bool,
    verbose: bool,
    headless: bool,
) -> anyhow::Result<()> {
    let state = AppState::new(verbose, headless);
    let app = build_router(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("[*] Dalang Dashboard running at http://localhost:{}/dashboard", port);

    if open_browser {
        let url = format!("http://localhost:{}/dashboard", port);
        if open::that(&url).is_err() {
            println!("[!] Could not open browser. Please navigate to: {}", url);
        }
    }

    axum::serve(listener, app).await?;
    Ok(())
}
