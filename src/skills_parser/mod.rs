use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    pub tool_path: Option<String>,
    pub args: Option<Vec<String>>,
    pub requires_root: Option<bool>,
    #[serde(skip)]
    pub system_prompt: String,
    #[serde(skip)]
    pub role: Option<String>,
    #[serde(skip)]
    pub task: Option<String>,
    #[serde(skip)]
    pub constraints: Option<String>,
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

    // Extract specific sections if they exist
    definition.role = extract_section(&markdown_content, "Role");
    definition.task = extract_section(&markdown_content, "Task");
    definition.constraints = extract_section(&markdown_content, "Constraints");

    Ok(definition)
}

fn extract_section(content: &str, section_name: &str) -> Option<String> {
    let header = format!("# {}", section_name);
    let lines: Vec<&str> = content.lines().collect();

    let mut start_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().to_lowercase() == header.to_lowercase() {
            start_idx = Some(i + 1);
            break;
        }
    }

    let start = start_idx?;
    let mut section_lines = Vec::new();

    for line in &lines[start..] {
        if line.trim().starts_with('#') {
            break;
        }
        section_lines.push(*line);
    }

    let joined = section_lines.join("\n").trim().to_string();
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_skill_parsing_with_sections() {
        let content = r#"---
name: nmap_scanner
description: Port scan
tool_path: /usr/bin/nmap
args: ["-sV", "-T4", "{{target}}"]
requires_root: false
---
# Role
Pentester handal.

# Task
Lakukan scan.

# Constraints
Jangan merusak.
"#;
        let skill = parse_skill_content(content).unwrap();
        assert_eq!(skill.tool_path, Some("/usr/bin/nmap".to_string()));
        assert_eq!(
            skill.args,
            Some(vec![
                "-sV".to_string(),
                "-T4".to_string(),
                "{{target}}".to_string()
            ])
        );
        assert_eq!(skill.requires_root, Some(false));
        assert_eq!(skill.role, Some("Pentester handal.".to_string()));
        assert_eq!(skill.task, Some("Lakukan scan.".to_string()));
        assert_eq!(skill.constraints, Some("Jangan merusak.".to_string()));
    }

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
        assert_eq!(skill.role, None);
    }

    #[test]
    fn test_missing_frontmatter() {
        let content = r#"Kamu adalah ahli web"#;
        let result = parse_skill_content(content);
        assert!(result.is_err());
    }
}
