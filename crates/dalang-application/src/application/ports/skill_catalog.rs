//! Port for loading and describing security skills (markdown + frontmatter on disk or elsewhere).

use anyhow::Result;
use dalang_domain::domain::models::SkillDefinition;
use std::path::Path;

/// Abstraction over where skill definitions come from (filesystem, bundled assets, tests, etc.).
pub trait SkillCatalog: Send + Sync {
    fn load_all_skills(&self) -> Result<Vec<SkillDefinition>>;
    fn load_available_skills(&self) -> Result<(Vec<SkillDefinition>, Vec<String>)>;
    fn generate_catalog_prompt(&self, skills: &[SkillDefinition]) -> String;
    fn parse_skill_file(&self, path: &Path) -> Result<SkillDefinition>;
}
