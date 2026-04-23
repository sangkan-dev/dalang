use anyhow::{Result, anyhow};
use dalang_domain::domain::models::SkillDefinition;
use std::fs;
use std::path::{Path, PathBuf};

/// Check if a binary exists on the system PATH.
pub fn check_tool_available(tool_path: &str) -> bool {
    let bin_name = std::path::Path::new(tool_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(tool_path);

    if std::path::Path::new(tool_path).is_file() {
        return true;
    }

    std::process::Command::new("which")
        .arg(bin_name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Prefer `./skills` (repo / process cwd); fall back to `../../skills` from this crate (workspace layout).
fn resolve_skills_dir() -> Option<PathBuf> {
    let cwd = PathBuf::from("skills");
    if cwd.exists() {
        return Some(cwd);
    }
    let from_manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    if from_manifest.exists() {
        return Some(from_manifest);
    }
    None
}

/// Load all skills from the resolved `skills` directory.
pub fn load_all_skills() -> anyhow::Result<Vec<SkillDefinition>> {
    let mut skills = Vec::new();
    let Some(skills_dir) = resolve_skills_dir() else {
        return Ok(skills);
    };

    for entry in std::fs::read_dir(skills_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(mut skill) = parse_skill_content(&content) {
                skill.tool_available = match &skill.tool_path {
                    Some(tp) if tp != "null" && !tp.is_empty() => check_tool_available(tp),
                    _ => true,
                };
                skills.push(skill);
            }
        }
    }

    Ok(skills)
}

/// Load only skills whose tools are installed on the system.
pub fn load_available_skills() -> anyhow::Result<(Vec<SkillDefinition>, Vec<String>)> {
    let all = load_all_skills()?;
    let mut available = Vec::new();
    let mut unavailable_names = Vec::new();

    for skill in all {
        if skill.tool_available {
            available.push(skill);
        } else {
            unavailable_names.push(format!(
                "{} (requires: {})",
                skill.name,
                skill.tool_path.as_deref().unwrap_or("unknown")
            ));
        }
    }

    Ok((available, unavailable_names))
}

pub fn generate_skills_catalog_prompt(skills: &[SkillDefinition]) -> String {
    let mut prompt = String::from("AVAILABLE SKILLS (TOOLS) IN YOUR ARSENAL:\n\n");

    for (i, skill) in skills.iter().enumerate() {
        prompt.push_str(&format!(
            "{}. `{}`: {}\n",
            i + 1,
            skill.name,
            skill.description
        ));
    }

    prompt.push_str("\nYou can invoke these tools using the `execute_skill` function with the `skill_name` parameter.");
    prompt
}

pub fn parse_skill(path: &Path) -> Result<SkillDefinition> {
    let content = fs::read_to_string(path)?;
    parse_skill_content(&content)
}

pub fn parse_skill_content(content: &str) -> Result<SkillDefinition> {
    let mut lines = content.lines();

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

    let mut definition: SkillDefinition = serde_yaml::from_str(&yaml_content)?;

    definition.system_prompt = markdown_content.trim().to_string();

    definition.role = extract_section(&markdown_content, "Role");
    definition.task = extract_section(&markdown_content, "Task");
    definition.constraints = extract_section(&markdown_content, "Constraints");

    Ok(definition)
}

fn extract_section(content: &str, section_name: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let target = section_name.to_lowercase();

    let mut start_idx = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let header_text = trimmed.trim_start_matches('#').trim().to_lowercase();
            if header_text == target {
                start_idx = Some(i + 1);
                break;
            }
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
