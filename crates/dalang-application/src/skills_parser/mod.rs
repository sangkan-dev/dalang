//! Skill parsing from `dalang-skills` plus [`FileSystemSkillCatalog`](crate::skills_parser::FileSystemSkillCatalog).

mod fs_catalog;

pub use dalang_skills::*;
pub use fs_catalog::FileSystemSkillCatalog;
