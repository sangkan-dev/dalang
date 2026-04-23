//! Embedded static file serving for the Svelte frontend.

use axum::body::Body;
use axum::http::{Request, Response, StatusCode, header};
use axum::response::IntoResponse;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../../web2/build-dashboard/"]
pub struct WebAssets;

/// Serve embedded static files. Falls back to index.html for SPA routing.
pub async fn static_handler(req: Request<Body>) -> impl IntoResponse {
    let path = req.uri().path().trim_start_matches('/');

    // Rust runtime only serves dashboard + static assets.
    // Marketing landing is deployed separately to Cloudflare Pages.
    if path.is_empty() || path == "index.html" {
        return Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(header::LOCATION, "/dashboard")
            .body(Body::empty())
            .unwrap();
    }

    let is_dashboard_path = path == "dashboard" || path.starts_with("dashboard/");
    let is_static_asset = path.starts_with("_app/")
        || path.starts_with("favicon")
        || path.starts_with("icons/")
        || path.starts_with("fonts/");

    if !is_dashboard_path && !is_static_asset {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap();
    }

    // Try the exact path first
    if let Some(content) = WebAssets::get(path) {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime)
            .header(header::CACHE_CONTROL, "public, max-age=3600")
            .body(Body::from(content.data.to_vec()))
            .unwrap();
    }

    // Serve route index files from prerendered directories.
    if !path.contains('.') {
        let route_index = format!("{}/index.html", path.trim_end_matches('/'));
        if let Some(content) = WebAssets::get(&route_index) {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(content.data.to_vec()))
                .unwrap();
        }
    }

    // Dashboard fallback keeps deep links working under /dashboard/*.
    if is_dashboard_path && let Some(content) = WebAssets::get("dashboard/index.html") {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(content.data.to_vec()))
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap()
}
