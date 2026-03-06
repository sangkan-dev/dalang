//! Reports management REST API handlers.

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize)]
pub struct ReportSummary {
    pub filename: String,
    pub size: u64,
    pub created: String,
}

#[derive(Deserialize)]
pub struct ReportQuery {
    pub format: Option<String>,
}

/// GET /api/reports — list all saved reports
pub async fn list_reports() -> impl IntoResponse {
    let mut reports = Vec::new();

    let entries = match fs::read_dir(".") {
        Ok(e) => e,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read directory: {}", e),
            ))
        }
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("dalang_report_") && name.ends_with(".md")
            && let Ok(meta) = entry.metadata() {
                let created = meta
                    .modified()
                    .ok()
                    .and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .ok()
                            .map(|d| {
                                chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_default()
                            })
                    })
                    .unwrap_or_default();

                reports.push(ReportSummary {
                    filename: name,
                    size: meta.len(),
                    created,
                });
            }
    }

    reports.sort_by(|a, b| b.filename.cmp(&a.filename));
    Ok(Json(reports))
}

/// GET /api/reports/:filename — get report content
pub async fn get_report(
    Path(filename): Path<String>,
    Query(query): Query<ReportQuery>,
) -> impl IntoResponse {
    // Sanitize: only allow dalang_report_*.md files
    if !filename.starts_with("dalang_report_") || !filename.ends_with(".md") {
        return Err((StatusCode::BAD_REQUEST, "Invalid report filename".to_string()));
    }

    let content = match fs::read_to_string(&filename) {
        Ok(c) => c,
        Err(_) => return Err((StatusCode::NOT_FOUND, "Report not found".to_string())),
    };

    match query.format.as_deref() {
        Some("html") => {
            let html = markdown_to_html(&content);
            let full_html = format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Dalang Report - {}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 900px; margin: 0 auto; padding: 2rem; background: #0d1117; color: #c9d1d9; }}
        h1, h2, h3 {{ color: #58a6ff; }}
        code {{ background: #161b22; padding: 0.2em 0.4em; border-radius: 6px; font-size: 85%; }}
        pre {{ background: #161b22; padding: 1rem; border-radius: 6px; overflow-x: auto; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #30363d; padding: 0.5rem; text-align: left; }}
        th {{ background: #161b22; }}
        strong {{ color: #f0883e; }}
        a {{ color: #58a6ff; }}
        .severity-critical {{ color: #f85149; font-weight: bold; }}
        .severity-high {{ color: #f0883e; font-weight: bold; }}
        .severity-medium {{ color: #d29922; font-weight: bold; }}
        .severity-low {{ color: #3fb950; font-weight: bold; }}
    </style>
</head>
<body>{}</body>
</html>"#,
                filename, html
            );
            Ok(Html(full_html).into_response())
        }
        _ => Ok(Json(serde_json::json!({
            "filename": filename,
            "content": content,
        }))
        .into_response()),
    }
}

/// Convert markdown to HTML using pulldown-cmark.
fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};
    let options = Options::all();
    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
