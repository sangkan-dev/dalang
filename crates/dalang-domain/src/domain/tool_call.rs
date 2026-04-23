use crate::domain::errors::DalangError;
use crate::domain::models::ToolCall;

#[derive(serde::Deserialize)]
struct RawToolResponse {
    tool: Option<String>,
    args: Option<serde_json::Value>,
}

fn parse_clean_tool_payload(clean: &str) -> Result<Vec<ToolCall>, DalangError> {
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

    // Support concatenated JSON objects (newline-delimited / streaming style), e.g.:
    // {"tool":"execute_skill",...}\n{"tool":"execute_skill",...}
    let mut stream_calls = Vec::new();
    let stream = serde_json::Deserializer::from_str(clean).into_iter::<RawToolResponse>();
    let mut had_stream_item = false;
    for item in stream {
        had_stream_item = true;
        if let Ok(parsed) = item {
            if let Some(name) = parsed.tool {
                stream_calls.push(ToolCall {
                    name,
                    arguments: parsed.args.unwrap_or(serde_json::Value::Null),
                });
            }
        } else {
            stream_calls.clear();
            break;
        }
    }
    if had_stream_item && !stream_calls.is_empty() {
        return Ok(stream_calls);
    }

    // Fallback to single object
    let parsed: RawToolResponse = serde_json::from_str(clean)
        .map_err(|e| DalangError::InvalidToolCallJson(format!("{e}; content: {clean}")))?;

    Ok(vec![ToolCall {
        name: parsed.tool.ok_or(DalangError::ToolCallMissingName)?,
        arguments: parsed.args.unwrap_or(serde_json::Value::Null),
    }])
}

fn strip_code_fence(content: &str) -> String {
    let mut clean = content.trim();
    if clean.starts_with("```json") {
        clean = &clean[7..];
    } else if clean.starts_with("```") {
        clean = &clean[3..];
    }
    if clean.ends_with("```") {
        clean = &clean[..clean.len() - 3];
    }
    clean.trim().to_string()
}

fn extract_json_fenced_blocks(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut cursor = content;

    loop {
        let Some(start) = cursor.find("```json") else {
            break;
        };
        let after = &cursor[start + 7..];
        let Some(end) = after.find("```") else {
            break;
        };
        blocks.push(after[..end].trim().to_string());
        cursor = &after[end + 3..];
    }

    blocks
}

fn extract_balanced_json_slice(s: &str, start: usize) -> Option<String> {
    let first = s[start..].chars().next()?;
    let close = match first {
        '{' => '}',
        '[' => ']',
        _ => return None,
    };

    let mut depth = 0u32;
    let mut in_string = false;
    let mut escape = false;

    for (off, ch) in s[start..].char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' | '[' => depth += 1,
            '}' | ']' => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
                if depth == 0 && ch == close {
                    let end = start + off + ch.len_utf8();
                    return Some(s[start..end].to_string());
                }
            }
            _ => {}
        }
    }

    None
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
pub fn parse_llm_tool_call(content: &str) -> Result<Vec<ToolCall>, DalangError> {
    let clean = strip_code_fence(content);
    if let Ok(calls) = parse_clean_tool_payload(&clean) {
        return Ok(calls);
    }

    for block in extract_json_fenced_blocks(content) {
        if let Ok(calls) = parse_clean_tool_payload(&block) {
            return Ok(calls);
        }
    }

    for (idx, ch) in content.char_indices() {
        if (ch == '{' || ch == '[')
            && let Some(snippet) = extract_balanced_json_slice(content, idx)
            && let Ok(calls) = parse_clean_tool_payload(snippet.trim())
        {
            return Ok(calls);
        }
    }

    Err(DalangError::ToolCallNotFound)
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

    #[test]
    fn test_parse_concatenated_tool_calls() {
        let json = r#"{"tool": "execute_skill", "args": {"skill_name": "nmap_scanner"}}
{"tool": "execute_skill", "args": {"skill_name": "nikto_scanner"}}"#;
        let calls = parse_llm_tool_call(json).unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "execute_skill");
        assert_eq!(calls[1].name, "execute_skill");
    }

    #[test]
    fn test_parse_tool_call_embedded_in_prose() {
        let mixed = r#"Baik, saya mulai dulu.
{"tool": "execute_skill", "args": {"skill_name": "nikto_scanner", "reasoning": "Initial recon"}}
Saya akan lanjut setelah ini."#;
        let calls = parse_llm_tool_call(mixed).unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "execute_skill");
    }
}
