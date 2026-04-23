//! Default [`SkillCatalog`](crate::application::ports::skill_catalog::SkillCatalog) backed by the repo `skills/` directory (or workspace fallback).

use crate::application::ports::skill_catalog::SkillCatalog;
use anyhow::Result;
use dalang_domain::domain::models::SkillDefinition;
use std::path::Path;

/// Loads skills from disk using the same resolution rules as `skills_parser` free functions.
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemSkillCatalog;

impl SkillCatalog for FileSystemSkillCatalog {
    fn load_all_skills(&self) -> Result<Vec<SkillDefinition>> {
        dalang_skills::load_all_skills()
    }

    fn load_available_skills(&self) -> Result<(Vec<SkillDefinition>, Vec<String>)> {
        dalang_skills::load_available_skills()
    }

    fn generate_catalog_prompt(&self, skills: &[SkillDefinition]) -> String {
        dalang_skills::generate_skills_catalog_prompt(skills)
    }

    fn parse_skill_file(&self, path: &Path) -> Result<SkillDefinition> {
        dalang_skills::parse_skill(path)
    }
}
