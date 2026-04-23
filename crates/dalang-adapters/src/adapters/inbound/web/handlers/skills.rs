//! Skills management REST API handlers.

use crate::adapters::outbound::skills_parser;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use dalang_domain::domain::models::SkillDefinition;
use serde::{Deserialize, Serialize};

use crate::adapters::inbound::web::state::AppState;
use std::path::{Path as FsPath, PathBuf};

/// Resolve `skills/{name}.md` whether the process cwd is the repo root or `crates/dalang-adapters`.
fn skill_markdown_path(name: &str) -> PathBuf {
    let file = format!("{}.md", name);
    let cwd_candidate = FsPath::new("skills").join(&file);
    if cwd_candidate.exists() {
        return cwd_candidate;
    }
    let manifest_skills = FsPath::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../skills")
        .join(&file);
    if manifest_skills.exists() {
        return manifest_skills;
    }
    cwd_candidate
}

#[derive(Serialize)]
pub struct SkillSummary {
    pub name: String,
    pub description: String,
    pub tool_path: Option<String>,
    pub requires_root: bool,
    pub has_args: bool,
    pub enabled: bool,
    pub tool_available: bool,
}

#[derive(Serialize)]
pub struct SkillDetail {
    pub name: String,
    pub description: String,
    pub tool_path: Option<String>,
    pub args: Option<Vec<String>>,
    pub requires_root: bool,
    pub system_prompt: String,
    pub role: Option<String>,
    pub task: Option<String>,
    pub constraints: Option<String>,
    pub tool_available: bool,
}

impl From<&SkillDefinition> for SkillSummary {
    fn from(s: &SkillDefinition) -> Self {
        Self {
            name: s.name.clone(),
            description: s.description.clone(),
            tool_path: s.tool_path.clone(),
            requires_root: s.requires_root.unwrap_or(false),
            has_args: s.args.as_ref().is_some_and(|a| !a.is_empty()),
            enabled: true, // default — caller overrides from state
            tool_available: s.tool_available,
        }
    }
}

impl From<SkillDefinition> for SkillDetail {
    fn from(s: SkillDefinition) -> Self {
        Self {
            name: s.name,
            description: s.description,
            tool_path: s.tool_path,
            args: s.args,
            requires_root: s.requires_root.unwrap_or(false),
            system_prompt: s.system_prompt,
            role: s.role,
            task: s.task,
            constraints: s.constraints,
            tool_available: s.tool_available,
        }
    }
}

/// GET /api/skills — list all skills
pub async fn list_skills(State(state): State<AppState>) -> impl IntoResponse {
    match skills_parser::load_all_skills() {
        Ok(skills) => {
            let summaries: Vec<SkillSummary> = skills
                .iter()
                .map(|s| {
                    let mut summary = SkillSummary::from(s);
                    // Auto-disable if tool binary is not installed
                    if !summary.tool_available {
                        summary.enabled = false;
                    }
                    // Also check manually disabled skills
                    if state.disabled_skills.contains_key(&summary.name) {
                        summary.enabled = false;
                    }
                    summary
                })
                .collect();
            Ok(Json(summaries))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load skills: {}", e),
        )),
    }
}

/// GET /api/skills/:name — get skill detail
pub async fn get_skill(Path(name): Path<String>) -> impl IntoResponse {
    let path = skill_markdown_path(&name);

    if !path.exists() {
        return Err((StatusCode::NOT_FOUND, format!("Skill '{}' not found", name)));
    }

    match skills_parser::parse_skill(&path) {
        Ok(mut skill) => {
            // Check tool availability
            skill.tool_available = match &skill.tool_path {
                Some(tp) if tp != "null" && !tp.is_empty() => {
                    skills_parser::check_tool_available(tp)
                }
                _ => true,
            };
            Ok(Json(SkillDetail::from(skill)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse skill: {}", e),
        )),
    }
}

#[derive(Deserialize)]
pub struct UpdateSkillRequest {
    pub enabled: bool,
}

/// PUT /api/skills/:name — enable/disable a skill
pub async fn update_skill(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<UpdateSkillRequest>,
) -> impl IntoResponse {
    // Verify skill exists
    let skill_path = skill_markdown_path(&name);
    if !skill_path.exists() {
        return Err((StatusCode::NOT_FOUND, format!("Skill '{}' not found", name)));
    }

    if body.enabled {
        state.disabled_skills.remove(&name);
    } else {
        state.disabled_skills.insert(name.clone(), true);
    }

    Ok(Json(
        serde_json::json!({ "name": name, "enabled": body.enabled }),
    ))
}
