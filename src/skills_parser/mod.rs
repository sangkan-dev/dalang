use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    #[serde(skip)]
    pub system_prompt: String,
}

/// Parse a markdown file with YAML frontmatter to extract the skill definition
pub fn parse_skill(path: &Path) -> Result<SkillDefinition> {
    let content = fs::read_to_string(path)?;
    parse_skill_content(&content)
}

/// Parse skill directly from string content
pub fn parse_skill_content(content: &str) -> Result<SkillDefinition> {
    // Basic splitting logic for YAML frontmatter
    let mut lines = content.lines();

    // Ensure the file starts with '---'
    if let Some(first_line) = lines.next() {
        if first_line.trim() != "---" {
            return Err(anyhow!("File does not start with YAML frontmatter '---'"));
        }
    } else {
        return Err(anyhow!("Empty file"));
    }

    let mut yaml_content = String::new();
    let mut markdown_content = String::new();
    let mut in_yaml = true;

    for line in lines {
        if in_yaml {
            if line.trim() == "---" {
                in_yaml = false;
            } else {
                yaml_content.push_str(line);
                yaml_content.push('\n');
            }
        } else {
            markdown_content.push_str(line);
            markdown_content.push('\n');
        }
    }

    if in_yaml {
        return Err(anyhow!("Unclosed YAML frontmatter"));
    }

    // Parse the YAML block
    let mut definition: SkillDefinition = serde_yaml::from_str(&yaml_content)?;

    // The rest is the system prompt
    definition.system_prompt = markdown_content.trim().to_string();

    Ok(definition)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_skill_parsing() {
        let content = r#"---
name: web-crawl
description: Mengambil data dari website
---
Kamu adalah ahli dalam mengambil data dari halaman web.
Silahkan ambil informasi dari URL yang diberikan.
"#;
        let skill = parse_skill_content(content).unwrap();
        assert_eq!(skill.name, "web-crawl");
        assert_eq!(skill.description, "Mengambil data dari website");
        assert_eq!(
            skill.system_prompt,
            "Kamu adalah ahli dalam mengambil data dari halaman web.\nSilahkan ambil informasi dari URL yang diberikan."
        );
    }

    #[test]
    fn test_missing_frontmatter() {
        let content = r#"Kamu adalah ahli web"#;
        let result = parse_skill_content(content);
        assert!(result.is_err());
    }
}
