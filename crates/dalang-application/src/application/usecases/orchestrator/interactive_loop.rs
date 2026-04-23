use super::DalangOrchestrator;
use crate::application::usecases::memory::{
    MAX_OBSERVATION_BYTES, compact_messages, truncate_output,
};
use anyhow::Result;
use dalang_domain::domain::models::{EngineEvent, Message};
use dalang_domain::domain::tool_call::{build_executor_args, parse_llm_tool_call};
use tokio::sync::mpsc;

impl DalangOrchestrator {
    // ── Public: Interactive Loop ───────────────────────────────────────────────

    /// Human-in-the-loop interactive session (CLI mode only).
    /// Used by `dalang interact --target <url>`.
    ///
    /// For WebSocket mode, use `process_chat_message` instead.
    pub async fn run_interactive_loop(
        &self,
        target: &str,
        _tx: Option<mpsc::Sender<EngineEvent>>,
    ) -> Result<()> {
        let (raw_skills, raw_unavailable) = self.skills.load_available_skills()?;
        let (skills, unavailable) = self.filter_disabled_skills(raw_skills, raw_unavailable);

        println!("[*] Interactive session started. Target: {}", target);
        println!(
            "[*] Loaded {} skills. Type your task below (empty line = quit).",
            skills.len()
        );
        if !unavailable.is_empty() {
            println!(
                "[!] {} skills disabled: {}",
                unavailable.len(),
                unavailable.join(", ")
            );
        }

        let skills_catalog = self.skills.generate_catalog_prompt(&skills);
        let system_prompt = format!(
            "[AUTHORIZED AUDIT ENVIRONMENT - INTERACTIVE MODE]\n\
            You are a Senior Security Auditor assistant for a sanctioned pentest of: {target}.\n\n\
            {skills_catalog}\n\n\
            Assist the pentester with concise, evidence-first responses. When asked to run a tool, use the JSON tool call format:\n\
            ```json\n\
            {{\"tool\": \"execute_skill\", \"args\": {{\"skill_name\": \"<name>\", \"target_url\": \"<optional-url>\", \"reasoning\": \"<why>\", \"custom_args\": [], \"args_override\": []}}}}\n\
            ```\n\
            RULES:\n\
            - Never claim a tool is queued/completed before observation exists.\n\
            - Prefer at most 1-2 tool calls per response.\n\
            - If executing tools, output ONLY JSON (no narrative text).\n\
            For each finding, include the exact URL, parameter, PoC, and severity.\n\n\
            FINAL REPORT: When the user asks for a report or you conclude the assessment, output markdown with title line \
`VULNERABILITY REPORT` or `LAPORAN KERENTANAN`, then mandatory section `## Ringkasan untuk pihak non-teknis` \
(3–6 bullets in simple Bahasa Indonesia for non-technical readers: serious issues yes/no, rough impact, priority fixes), \
then `## Findings` with technical detail (URLs, PoC, severity) as usual."
        );

        let mut messages = vec![Message::system(&system_prompt)];

        loop {
            use std::io::{self, BufRead, Write};
            print!("\n[You] > ");
            io::stdout().flush().unwrap();

            let stdin = io::stdin();
            let mut input = String::new();
            stdin.lock().read_line(&mut input).ok();
            let input = input.trim().to_string();

            if input.is_empty() {
                println!("[*] Ending interactive session.");
                break;
            }

            messages.push(Message::user(&input));
            compact_messages(&mut messages);

            let response = self.llm.send_messages(&messages).await?;
            messages.push(Message::assistant(&response));
            println!("\n[Dalang]\n{}", response);

            if Self::is_final_report(&response) {
                if let Some(filename) = Self::save_report(target, &response) {
                    println!("[+] Interactive report saved: {}", filename);
                }
                break;
            }

            // Handle tool calls if present
            if let Ok(tool_calls) = parse_llm_tool_call(&response) {
                for tool_call in tool_calls {
                    if tool_call.name == "execute_skill" {
                        let skill_name = tool_call
                            .arguments
                            .get("skill_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if let Some(skill_def) = Self::find_skill_by_name(&skills, &skill_name) {
                            self.execute_skill_native(
                                skill_def,
                                target,
                                Some(&tool_call.arguments),
                                &mut messages,
                                None,
                            )
                            .await;
                        } else {
                            println!("[!] Skill '{}' not found.", skill_name);
                        }
                    } else if tool_call.name.starts_with("browser-") {
                        if let Some(browser) = &self.browser {
                            let output =
                                crate::application::browser_tool_dispatch::dispatch_browser_tool(
                                    &**browser, &tool_call,
                                )
                                .await;
                            let obs = truncate_output(&output, MAX_OBSERVATION_BYTES);
                            println!("    [B] {}: {}", tool_call.name, &obs[..obs.len().min(200)]);
                            messages.push(Message::user(&format!(
                                "Browser Observation ({}):\n{}",
                                tool_call.name, obs
                            )));
                        } else {
                            println!("[!] No browser session attached.");
                        }
                    } else {
                        let args = build_executor_args(&tool_call);
                        if !args.is_empty() {
                            let program = &args[0];
                            let prog_args: Vec<&str> =
                                args[1..].iter().map(|s| s.as_str()).collect();
                            println!("    $ {} {}", program, prog_args.join(" "));
                            match self
                                .executor
                                .execute(program, &prog_args, self.effective_timeout())
                                .await
                            {
                                Ok((stdout, stderr)) => {
                                    let mut obs = format!("STDOUT:\n{}\n", stdout);
                                    if !stderr.is_empty() {
                                        obs.push_str(&format!("STDERR:\n{}\n", stderr));
                                    }
                                    messages.push(Message::user(&format!(
                                        "Observation ({}):\n{}",
                                        program, obs
                                    )));
                                }
                                Err(e) => {
                                    messages.push(Message::user(&format!("Command Error: {}", e)));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
