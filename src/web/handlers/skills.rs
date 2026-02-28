//! Skills management REST API handlers.

use crate::skills_parser::{self, SkillDefinition};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct SkillSummary {
    pub name: String,
    pub description: String,
    pub tool_path: Option<String>,
    pub requires_root: bool,
    pub has_args: bool,
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
pub async fn list_skills() -> impl IntoResponse {
    match skills_parser::load_all_skills() {
        Ok(skills) => {
            let summaries: Vec<SkillSummary> = skills.iter().map(SkillSummary::from).collect();
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
