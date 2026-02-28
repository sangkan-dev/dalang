//! Skills management REST API handlers.

use crate::skills_parser::{self, SkillDefinition};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::web::state::AppState;

#[derive(Serialize)]
pub struct SkillSummary {
    pub name: String,
    pub description: String,
    pub tool_path: Option<String>,
    pub requires_root: bool,
    pub has_args: bool,
    pub enabled: bool,
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
    let skill_path = format!("skills/{}.md", name);
    let path = std::path::Path::new(&skill_path);

    if !path.exists() {
        return Err((StatusCode::NOT_FOUND, format!("Skill '{}' not found", name)));
    }

    match skills_parser::parse_skill(path) {
        Ok(skill) => Ok(Json(SkillDetail::from(skill))),
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
    let skill_path = format!("skills/{}.md", name);
    if !std::path::Path::new(&skill_path).exists() {
        return Err((StatusCode::NOT_FOUND, format!("Skill '{}' not found", name)));
    }

    if body.enabled {
        state.disabled_skills.remove(&name);
    } else {
        state.disabled_skills.insert(name.clone(), true);
    }

    Ok(Json(serde_json::json!({ "name": name, "enabled": body.enabled })))
}
