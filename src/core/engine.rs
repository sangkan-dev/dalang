use crate::cdp::browser::DalangBrowser;
use crate::core::tool_call::{build_executor_args, parse_llm_tool_call};
use crate::executor::execute_safe_command;
use crate::llm::{LlmProvider, Message};
use crate::skills_parser::parse_skill;
use anyhow::{Result, anyhow};
use std::path::Path;

pub struct DalangEngine {
    llm: Box<dyn LlmProvider + Send + Sync>,
}

impl DalangEngine {
    pub fn new(llm: Box<dyn LlmProvider + Send + Sync>) -> Self {
        Self { llm }
    }

    pub async fn run_scan_loop(&self, target: &str, skill_names: &str) -> Result<()> {
        let skills: Vec<&str> = skill_names.split(',').map(|s| s.trim()).collect();
        if skills.is_empty() {
            return Err(anyhow!("No skills provided"));
        }

        // For sprint 2 MVP, we execute the first skill logic.
        // A full implementation would aggregate contexts or loop over them.
        let skill_name = skills[0];
        let skill_path = format!("skills/{}.md", skill_name);

        let skill_def = parse_skill(Path::new(&skill_path))?;
        println!(
            "[*] Loaded Skill: {} - {}",
            skill_def.name, skill_def.description
        );

        let mut messages = vec![
            Message::system(&skill_def.system_prompt),
            Message::user(&format!(
                "Tolong eksekusi tugas ini untuk target: {}. Keluarkan output JSON Tool Call berbentuk seperti:\n```json\n{{\"tool\": \"os-command\", \"args\": {{\"program\": \"echo\", \"args\": [\"halo\"]}}}}\n```\nAtau tool: `browser-navigate` (args: {{\"url\": \"<url>\"}}), `browser-evaluate-js` (args: {{\"script\": \"<js>\"}}), `browser-extract-dom` (args: {{}})\nBila sudah selesai, keluarkan respon final berupa teks biasa tanpa JSON.",
                target
            )),
        ];

        let browser = DalangBrowser::new().await?; // Initialize inside the engine run

        let max_iterations = 10;
        let mut i = 0;

        while i < max_iterations {
            i += 1;
            println!("\n[...] LLM is reasoning (Iteration {})...", i);

            // 1. Reason
            let response_text = self.llm.send_messages(&messages).await?;
            messages.push(Message::assistant(&response_text));

            // Check if it's a tool call (starts with json or ```json)
            if response_text.trim().starts_with('{') || response_text.trim().starts_with("```json")
            {
                // 2. Act
                match parse_llm_tool_call(&response_text) {
                    Ok(tool_call) => {
                        println!("[>] Tool Call Detected: {}", tool_call.name);

                        // Handle browser tools specifically
                        if tool_call.name.starts_with("browser-") {
                            let (success, resp) = match tool_call.name.as_str() {
                                "browser-navigate" => {
                                    let url = tool_call
                                        .arguments
                                        .get("url")
                                        .and_then(|v: &serde_json::Value| v.as_str())
                                        .unwrap_or("http://localhost");
                                    match browser.navigate(url).await {
                                        Ok(r) => (true, r),
                                        Err(e) => (false, e.to_string()),
                                    }
                                }
                                "browser-evaluate-js" => {
                                    let script = tool_call
                                        .arguments
                                        .get("script")
                                        .and_then(|v: &serde_json::Value| v.as_str())
                                        .unwrap_or("console.log('empty')");
                                    match browser.evaluate_js(script).await {
                                        Ok(r) => (true, r),
                                        Err(e) => (false, e.to_string()),
                                    }
                                }
                                "browser-extract-dom" => match browser.extract_dom().await {
                                    Ok(r) => (true, r),
                                    Err(e) => (false, e.to_string()),
                                },
                                _ => (false, "Unknown browser command".to_string()),
                            };

                            if success {
                                println!("[<] Browser Tool Success ({} bytes)", resp.len());
                                messages.push(Message::user(&format!("Observation:\n{}", resp)));
                            } else {
                                println!("[!] Browser Tool Failed: {}", resp);
                                messages.push(Message::user(&format!("Command Error:\n{}", resp)));
                            }
                            continue;
                        }

                        let args = build_executor_args(&tool_call);
                        if args.is_empty() {
                            messages.push(Message::user("Error: Invalid tool arguments"));
                            continue;
                        }

                        let program = &args[0];
                        let program_args: Vec<&str> =
                            args[1..].iter().map(|s: &String| s.as_str()).collect();

                        println!("    $ {} {}", program, program_args.join(" "));

                        // Observe
                        match execute_safe_command(program, &program_args, 30).await {
                            Ok((stdout, stderr)) => {
                                let mut obs = format!("STDOUT:\n{}\n", stdout);
                                if !stderr.is_empty() {
                                    obs.push_str(&format!("STDERR:\n{}\n", stderr));
                                }
                                println!("[<] Observation received ({} bytes)", obs.len());
                                messages.push(Message::user(&format!("Observation:\n{}", obs)));
                            }
                            Err(e) => {
                                println!("[!] Command execution failed: {}", e);
                                messages.push(Message::user(&format!("Command Error:\n{}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        println!("[!] Failed to parse tool call: {}", e);
                        println!("Raw output: {}", response_text);
                        messages.push(Message::user(
                            "Format JSON tool call salah. Tolong perbaiki sesuai format.",
                        ));
                    }
                }
            } else {
                // Final response
                println!("[✓] Final Response:\n{}", response_text);
                break;
            }
        }

        if i >= max_iterations {
            println!("[!] Reached maximum iterations ({})", max_iterations);
        }

        Ok(())
    }
}
