//! Integration tests for web API handlers.
//!
//! Uses axum's built-in test utilities via tower::ServiceExt.

#[cfg(test)]
mod tests {
    use crate::web::{build_router, state::AppState};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // oneshot

    fn test_state() -> AppState {
        AppState::new(false)
    }

    fn app() -> axum::Router {
        build_router(test_state())
    }

    #[tokio::test]
    async fn test_list_skills_returns_json_array() {
        let req = Request::builder()
            .uri("/api/skills")
            .body(Body::empty())
            .unwrap();

        let res = app().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array(), "Expected JSON array, got: {:?}", json);
    }

    #[tokio::test]
    async fn test_create_and_delete_session() {
        let state = test_state();
        let app = build_router(state.clone());

        // Create session
        let req = Request::builder()
            .uri("/api/sessions")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"target":"http://test.com","mode":"interactive"}"#,
            ))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        // Handler may return 200 or 201 for creation
        let status = res.status().as_u16();
        assert!(
            status == 200 || status == 201,
            "Expected 200 or 201 but got {status}"
        );

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let session_id = json["id"].as_str().expect("Expected session id");
        assert_eq!(json["target"].as_str().unwrap(), "http://test.com");
        assert_eq!(json["mode"].as_str().unwrap(), "interactive");
        assert_eq!(json["active"].as_bool().unwrap(), true);

        // Verify session exists in state
        assert!(state.sessions.len() > 0);

        // Delete session
        let req = Request::builder()
            .uri(format!("/api/sessions/{}", session_id))
            .method("DELETE")
            .body(Body::empty())
            .unwrap();

        let res = build_router(state.clone()).oneshot(req).await.unwrap();
        assert!(
            res.status() == StatusCode::OK || res.status() == StatusCode::NO_CONTENT
        );
    }

    #[tokio::test]
    async fn test_list_sessions_empty() {
        let req = Request::builder()
            .uri("/api/sessions")
            .body(Body::empty())
            .unwrap();

        let res = app().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_get_settings_returns_expected_shape() {
        let req = Request::builder()
            .uri("/api/settings")
            .body(Body::empty())
            .unwrap();

        let res = app().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json["provider"].is_string());
        assert!(json["model"].is_string());
        assert!(json["auth_method"].is_string());
        assert!(json["endpoint_mode"].is_string());
        assert!(json["auth_status"].is_string());
    }

    #[tokio::test]
    async fn test_get_nonexistent_skill_returns_404() {
        let req = Request::builder()
            .uri("/api/skills/definitely_not_a_skill_xyz")
            .body(Body::empty())
            .unwrap();

        let res = app().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_reports_returns_json() {
        let req = Request::builder()
            .uri("/api/reports")
            .body(Body::empty())
            .unwrap();

        let res = app().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array());
    }

    #[tokio::test]
    async fn test_static_fallback_serves_index_html() {
        let req = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let res = app().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(
            html.contains("<!DOCTYPE html>") || html.contains("<!doctype html>"),
            "Expected HTML content"
        );
    }

    #[tokio::test]
    async fn test_update_settings_model() {
        let req = Request::builder()
            .uri("/api/settings")
            .method("PUT")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"model":"gpt-4o-mini"}"#))
            .unwrap();

        let res = app().oneshot(req).await.unwrap();
        // May succeed or fail depending on keyring availability in CI,
        // but should not panic or return 404
        assert!(
            res.status() == StatusCode::OK
                || res.status() == StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_update_skill_toggle() {
        let state = test_state();
        let router = build_router(state.clone());

        // Disable a skill
        let req = Request::builder()
            .uri("/api/skills/nmap_scanner")
            .method("PUT")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"enabled":false}"#))
            .unwrap();

        let res = router.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["enabled"].as_bool(), Some(false));
        assert_eq!(json["name"].as_str(), Some("nmap_scanner"));

        // Verify it shows as disabled in the list
        let req = Request::builder()
            .uri("/api/skills")
            .body(Body::empty())
            .unwrap();
        let res = build_router(state.clone()).oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let skills = json.as_array().unwrap();
        let nmap = skills
            .iter()
            .find(|s| s["name"].as_str() == Some("nmap_scanner"));
        assert!(nmap.is_some(), "nmap_scanner should be in the list");
        assert_eq!(nmap.unwrap()["enabled"].as_bool(), Some(false));

        // Re-enable
        let req = Request::builder()
            .uri("/api/skills/nmap_scanner")
            .method("PUT")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"enabled":true}"#))
            .unwrap();
        let res = build_router(state.clone()).oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["enabled"].as_bool(), Some(true));
    }

    #[tokio::test]
    async fn test_update_nonexistent_skill_returns_404() {
        let req = Request::builder()
            .uri("/api/skills/no_such_skill_xyz")
            .method("PUT")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"enabled":false}"#))
            .unwrap();

        let res = app().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_settings_has_api_key_and_verbose_fields() {
        let req = Request::builder()
            .uri("/api/settings")
            .body(Body::empty())
            .unwrap();

        let res = app().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // New fields should exist
        assert!(json["has_api_key"].is_boolean(), "Expected has_api_key boolean");
        assert!(json["verbose"].is_boolean(), "Expected verbose boolean");
    }

    #[tokio::test]
    async fn test_test_connection_endpoint_exists() {
        let req = Request::builder()
            .uri("/api/settings/test-connection")
            .method("POST")
            .body(Body::empty())
            .unwrap();

        let res = app().oneshot(req).await.unwrap();
        // Should return a JSON response (may fail connection but shouldn't 404)
        assert_ne!(res.status(), StatusCode::NOT_FOUND);
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["success"].is_boolean());
        assert!(json["message"].is_string());
        assert!(json["latency_ms"].is_number());
    }
}
