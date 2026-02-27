use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value, // Either Object or String depending on implementation
}

/// Parses a JSON output from the LLM expecting a specific tool call structure.
/// In this sprint, we assume the LLM outputs a JSON string describing the tool name and args.
/// ```json
/// {
///   "tool": "os-command",
///   "args": { "cmd": "ping", "target": "127.0.0.1" }
/// }
/// ```
#[derive(Debug, Deserialize)]
struct SimpleToolResponse {
    tool: Option<String>,
    args: Option<serde_json::Value>,
}

pub fn parse_llm_tool_call(content: &str) -> Result<ToolCall> {
    // LLMs sometimes wrap JSON in markdown blocks like ```json\n...\n```
    let mut clean_content = content.trim();
    if clean_content.starts_with("```json") {
        clean_content = &clean_content[7..];
    } else if clean_content.starts_with("```") {
        clean_content = &clean_content[3..];
    }
    if clean_content.ends_with("```") {
        clean_content = &clean_content[..clean_content.len() - 3];
    }
    clean_content = clean_content.trim();

    // Parse JSON
    let parsed: SimpleToolResponse = serde_json::from_str(clean_content).map_err(|e| {
        anyhow!(
            "Failed to parse JSON tool call: {}. Content: {}",
            e,
            clean_content
        )
    })?;

    let name = parsed
        .tool
        .ok_or_else(|| anyhow!("Missing 'tool' field in JSON"))?;
    let arguments = parsed.args.unwrap_or(serde_json::Value::Null);

    Ok(ToolCall { name, arguments })
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
    fn test_parse_tool_call() {
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
        let call = parse_llm_tool_call(json).unwrap();
        assert_eq!(call.name, "os-command");

        let exec_args = build_executor_args(&call);
        assert_eq!(exec_args, vec!["nmap", "-sV", "localhost"]);
    }
}
