//! Skill markdown parsing lives in `dalang-application`; bundled assets stay here.

pub mod bundled;

pub use dalang_domain::domain::models::SkillDefinition;
pub use dalang_skills::{
    check_tool_available, generate_skills_catalog_prompt, load_all_skills, load_available_skills,
    parse_skill, parse_skill_content,
};
