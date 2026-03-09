use crate::domain::models::ToolCall;
use anyhow::{Result, anyhow};

#[derive(serde::Deserialize)]
struct RawToolResponse {
    tool: Option<String>,
    args: Option<serde_json::Value>,
}

/// Parses JSON output from the LLM expecting tool call structure(s).
///
/// Supports both a single object and a JSON array of objects:
/// ```json
/// {"tool": "os-command", "args": {"program": "nmap", "args": ["-sV", "localhost"]}}
/// ```
/// or:
/// ```json
/// [{"tool": "execute_skill", "args": {...}}, {"tool": "execute_skill", "args": {...}}]
/// ```
pub fn parse_llm_tool_call(content: &str) -> Result<Vec<ToolCall>> {
    let mut clean = content.trim();
    if clean.starts_with("```json") {
        clean = &clean[7..];
    } else if clean.starts_with("```") {
        clean = &clean[3..];
    }
    if clean.ends_with("```") {
        clean = &clean[..clean.len() - 3];
    }
    clean = clean.trim();

    // Try parsing as array first
    if let Ok(arr) = serde_json::from_str::<Vec<RawToolResponse>>(clean) {
        let mut calls = Vec::new();
        for parsed in arr {
            if let Some(name) = parsed.tool {
                calls.push(ToolCall {
                    name,
                    arguments: parsed.args.unwrap_or(serde_json::Value::Null),
                });
            }
        }
        if !calls.is_empty() {
            return Ok(calls);
        }
    }

    // Fallback to single object
    let parsed: RawToolResponse = serde_json::from_str(clean)
        .map_err(|e| anyhow!("Failed to parse JSON tool call: {}. Content: {}", e, clean))?;

    Ok(vec![ToolCall {
        name: parsed.tool.ok_or_else(|| anyhow!("Missing 'tool' field"))?,
        arguments: parsed.args.unwrap_or(serde_json::Value::Null),
    }])
}

/// Converts the ToolCall arguments into a list of strings array that can be passed to the executor
pub fn build_executor_args(tool_call: &ToolCall) -> Vec<String> {
    let mut args = Vec::new();

    // Custom mapping rules based on tool name
    match tool_call.name.as_str() {
        "os-command" => {
            // e.g. {"program": "nmap", "args": ["-p", "80", "localhost"]}
            if let serde_json::Value::Object(map) = &tool_call.arguments {
                if let Some(serde_json::Value::String(program)) = map.get("program") {
                    args.push(program.clone());
                }

                if let Some(serde_json::Value::Array(arr)) = map.get("args") {
                    for item in arr {
                        if let serde_json::Value::String(s) = item {
                            args.push(s.clone());
                        }
                    }
                }
            }
        }
        _ => {
            // General fallback: pass arguments as JSON string
            args.push(tool_call.arguments.to_string());
        }
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_tool_call() {
        let json = r#"
        ```json
        {
            "tool": "os-command",
            "args": {
                "program": "nmap",
                "args": ["-sV", "localhost"]
            }
        }
        ```
        "#;
        let calls = parse_llm_tool_call(json).unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "os-command");

        let exec_args = build_executor_args(&calls[0]);
        assert_eq!(exec_args, vec!["nmap", "-sV", "localhost"]);
    }

    #[test]
    fn test_parse_array_tool_calls() {
        let json = r#"[
            {"tool": "execute_skill", "args": {"skill_name": "nmap_scanner"}},
            {"tool": "execute_skill", "args": {"skill_name": "nikto_scanner"}}
        ]"#;
        let calls = parse_llm_tool_call(json).unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "execute_skill");
        assert_eq!(calls[1].name, "execute_skill");
    }
}
