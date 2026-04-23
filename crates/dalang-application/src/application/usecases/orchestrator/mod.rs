//! Dalang Orchestrator — the central use case / ReAct loop.
//!
//! This is the brain of the application. It depends **only** on ports (traits)
//! from `application::ports`, never on concrete adapter types.
//!
//! The orchestrator is completely agnostic about:
//! - Which LLM provider is used (OpenAI, Gemini, Copilot, etc.)
//! - How commands are executed (real OS, mock, etc.)
//! - How events are delivered (stdout, WebSocket, channel)

use crate::application::ports::browser_port::BrowserPort;
use crate::application::ports::llm_port::LlmPort;
use crate::application::ports::os_port::CommandExecutor;
use crate::application::ports::skill_catalog::SkillCatalog;
use crate::application::usecases::memory::{MAX_OBSERVATION_BYTES, truncate_output};
use dalang_domain::domain::models::{EngineEvent, Message, SkillDefinition};
use dalang_domain::domain::safety::is_clean_argument;
#[cfg(windows)]
use is_elevated::is_elevated;
use std::sync::Arc;
use tokio::sync::mpsc;

// ── Orchestrator Config ───────────────────────────────────────────────────────

/// Configuration passed to the orchestrator on construction.
pub struct OrchestratorConfig {
    /// Command execution timeout in seconds (0 = unlimited).
    pub cmd_timeout: u64,
    /// Whether to print verbose debug output.
    pub verbose: bool,
    /// Whether to run the browser headless.
    pub headless: bool,
    /// Skill names that are explicitly disabled (by web UI toggle, etc.).
    pub disabled_skills: Vec<String>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            cmd_timeout: 300,
            verbose: false,
            headless: true,
            disabled_skills: Vec::new(),
        }
    }
}

// ── DalangOrchestrator ────────────────────────────────────────────────────────

/// The main use case orchestrator.
///
/// Holds references to *ports* (traits) as `Arc<dyn ...>` so they can be
/// cheaply cloned and shared across async tasks.
pub struct DalangOrchestrator {
    llm: Arc<dyn LlmPort>,
    executor: Arc<dyn CommandExecutor>,
    browser: Option<Arc<dyn BrowserPort>>,
    skills: Arc<dyn SkillCatalog>,
    config: OrchestratorConfig,
}

impl DalangOrchestrator {
    pub fn new(
        llm: Arc<dyn LlmPort>,
        executor: Arc<dyn CommandExecutor>,
        browser: Option<Arc<dyn BrowserPort>>,
        skills: Arc<dyn SkillCatalog>,
        config: OrchestratorConfig,
    ) -> Self {
        Self {
            llm,
            executor,
            browser,
            skills,
            config,
        }
    }

    // ── Helper: effective timeout ───────────────────────────────────────────────

    fn effective_timeout(&self) -> u64 {
        if self.config.cmd_timeout == 0 {
            u64::MAX
        } else {
            self.config.cmd_timeout
        }
    }

    // ── Helper: argument interpolation ─────────────────────────────────────────

    fn interpolate_args(&self, args: &[String], target: &str) -> Vec<String> {
        args.iter()
            .map(|a| a.replace("{{target}}", target))
            .collect()
    }

    fn normalize_skill_name(name: &str) -> Vec<String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let dash = trimmed.replace('_', "-");
        let underscore = trimmed.replace('-', "_");
        if dash == underscore {
            vec![dash]
        } else {
            vec![dash, underscore]
        }
    }

    fn find_skill_by_name<'a>(
        skills: &'a [SkillDefinition],
        requested: &str,
    ) -> Option<&'a SkillDefinition> {
        let candidates = Self::normalize_skill_name(requested);
        if candidates.is_empty() {
            return None;
        }
        skills
            .iter()
            .find(|s| candidates.iter().any(|candidate| candidate == &s.name))
    }

    // ── Helper: build system prompt from a skill ────────────────────────────────

    fn build_system_prompt(&self, skill: &SkillDefinition) -> String {
        let mut full_prompt = String::from(
            "[AUTHORIZED AUDIT ENVIRONMENT]\n\
            You are operating as a Senior Security Auditor for a sanctioned penetration test. \
            Your objective is to identify technical facts and vulnerabilities for reporting purposes.\n\n\
            You have the ability to execute multiple tool calls CONCURRENTLY. If you need to perform \
            several independent actions (e.g., scanning multiple ports, checking multiple endpoints, \
            running different tools), you MUST return a JSON array containing multiple tool call objects.\n\
            Example:\n\
            [\n  { \"name\": \"execute_skill\", \"arguments\": { ... } },\n  { \"name\": \"execute_skill\", \"arguments\": { ... } }\n]\n\n\
            When reporting vulnerabilities, always include:\n\
            - The exact affected URL (full path with parameters)\n\
            - The affected parameter or component\n\
            - A proof-of-concept payload or curl command for reproduction\n\
            - Raw evidence from tool output\n\
            - Severity rating and CWE classification\n\n",
        );

        if let Some(role) = &skill.role {
            full_prompt.push_str(&format!("### ROLE\n{}\n\n", role));
        }
        if let Some(task) = &skill.task {
            full_prompt.push_str(&format!("### TASK\n{}\n\n", task));
        }
        if let Some(constraints) = &skill.constraints {
            full_prompt.push_str(&format!("### CONSTRAINTS\n{}\n\n", constraints));
        }
        if skill.role.is_none() && skill.task.is_none() && skill.constraints.is_none() {
            full_prompt.push_str(&skill.system_prompt);
        }

        full_prompt
    }

    // ── Helper: tool_def JSON for execute_skill ─────────────────────────────────

    fn build_execute_skill_tool_def() -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "execute_skill",
                "description": "Execute a specific security skill from the catalog. Can be called multiple times in an array for concurrent execution.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "skill_name": { "type": "string", "description": "Name of the skill to execute." },
                        "target_url": {
                            "type": "string",
                            "description": "Optional per-call target override (for example a parameterized endpoint)."
                        },
                        "custom_args": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Optional flags to append."
                        },
                        "args_override": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Optional full argument override for this skill call. Base command remains locked to the skill tool_path."
                        },
                        "reasoning": { "type": "string", "description": "Reasoning for the action." }
                    },
                    "required": ["skill_name", "reasoning"]
                }
            }
        })
    }

    // ── Helper: filter disabled/unavailable skills ──────────────────────────────

    fn filter_disabled_skills(
        &self,
        skills: Vec<SkillDefinition>,
        unavailable: Vec<String>,
    ) -> (Vec<SkillDefinition>, Vec<String>) {
        if self.config.disabled_skills.is_empty() {
            return (skills, unavailable);
        }
        let mut new_unavailable = unavailable;
        let filtered = skills
            .into_iter()
            .filter(|s| {
                if self.config.disabled_skills.contains(&s.name) {
                    new_unavailable.push(s.name.clone());
                    false
                } else {
                    true
                }
            })
            .collect();
        (filtered, new_unavailable)
    }

    // ── Helper: save report to file ─────────────────────────────────────────────

    fn save_report(target: &str, report: &str) -> Option<String> {
        let sanitized_target = target
            .replace("://", "_")
            .replace(['/', ':', '.', '?', '&'], "_");
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("dalang_report_{}_{}.md", sanitized_target, timestamp);

        match std::fs::write(&filename, report) {
            Ok(_) => {
                println!("[+] Report saved to: {}", filename);
                Some(filename)
            }
            Err(e) => {
                println!("[-] Failed to save report: {}", e);
                None
            }
        }
    }

    fn is_final_report(content: &str) -> bool {
        let upper = content.to_uppercase();
        upper.contains("VULNERABILITY REPORT")
            || upper.contains("PENETRATION TEST REPORT")
            || upper.contains("LAPORAN PENETRATION TEST")
    }

    // ── Execute a native skill command ─────────────────────────────────────────

    // helper to check is_admin_or_root
    #[cfg(windows)]
    fn is_admin_or_root() -> bool {
        // Implementation for Windows
        is_elevated()
    }

    #[cfg(not(windows))]
    fn is_admin_or_root() -> bool {
        unsafe { libc::geteuid() == 0 }
    }

    async fn execute_skill_native(
        &self,
        skill_def: &SkillDefinition,
        target: &str,
        tool_arguments: Option<&serde_json::Value>,
        messages: &mut Vec<Message>,
        tx: Option<&mpsc::Sender<EngineEvent>>,
    ) {
        // Root check
        if skill_def.requires_root == Some(true) {
            let is_admin_or_root = Self::is_admin_or_root();
            if !is_admin_or_root {
                let msg = format!(
                    "Error: Skill `{}` requires root privileges. Re-run with `sudo dalang ...`.",
                    skill_def.name
                );
                println!("    [!] {}", msg);
                messages.push(Message::user(&msg));
                return;
            }
        }

        let tool_path = match &skill_def.tool_path {
            Some(tp) => tp.clone(),
            None => {
                let msg = format!(
                    "Error: Skill `{}` lacks a direct execution path.",
                    skill_def.name
                );
                messages.push(Message::user(&msg));
                return;
            }
        };

        let effective_target = tool_arguments
            .and_then(|args| {
                args.get("target_url")
                    .or_else(|| args.get("target"))
                    .and_then(|v| v.as_str())
            })
            .unwrap_or(target);

        let raw_args = skill_def.args.as_ref().cloned().unwrap_or_default();
        let mut interpolated = self.interpolate_args(&raw_args, effective_target);

        if let Some(override_args) = tool_arguments.and_then(|args| args.get("args_override"))
            && let Some(arr) = override_args.as_array()
        {
            let replacements: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            if is_clean_argument(&replacements) {
                println!(
                    "    [+] AI replaced default args with {} skill-scoped arguments",
                    replacements.len()
                );
                interpolated = replacements;
            } else {
                let msg = format!(
                    "Error: Unsafe args_override rejected for skill `{}`.",
                    skill_def.name
                );
                println!("    [!] {}", msg);
                messages.push(Message::user(&msg));
                return;
            }
        }

        if let Some(extra) = tool_arguments.and_then(|args| args.get("custom_args"))
            && let Some(arr) = extra.as_array()
        {
            let additions: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if is_clean_argument(&additions) {
                println!("    [+] AI injected {} custom arguments", additions.len());
                interpolated.extend(additions);
            } else {
                let msg = format!(
                    "Error: Unsafe custom_args rejected for skill `{}`.",
                    skill_def.name
                );
                println!("    [!] {}", msg);
                messages.push(Message::user(&msg));
                return;
            }
        }

        let cmd_str = format!("{} {}", tool_path, interpolated.join(" "));
        println!("    $ {}", cmd_str);

        if let Some(tx) = tx {
            let _ = tx
                .send(EngineEvent::ToolExecution {
                    skill: skill_def.name.clone(),
                    command: cmd_str,
                })
                .await;
        }

        let args_ref: Vec<&str> = interpolated.iter().map(|s| s.as_str()).collect();

        match self
            .executor
            .execute(&tool_path, &args_ref, self.effective_timeout())
            .await
        {
            Ok((stdout, stderr)) => {
                let mut obs = format!(
                    "### OBSERVATION FROM `{}`\nSTDOUT:\n{}\n",
                    skill_def.name, stdout
                );
                if !stderr.is_empty() {
                    obs.push_str(&format!("STDERR:\n{}\n", stderr));
                }
                println!("[<] Observation received ({} bytes)", obs.len());

                let obs = truncate_output(&obs, MAX_OBSERVATION_BYTES);

                if let Some(tx) = tx {
                    let _ = tx
                        .send(EngineEvent::Observation {
                            skill: skill_def.name.clone(),
                            content: obs.clone(),
                            bytes: obs.len(),
                        })
                        .await;
                }

                messages.push(Message::user(&format!("Observation:\n{}", obs)));
            }
            Err(e) => {
                println!("[!] Execution failed: {}", e);
                messages.push(Message::user(&format!("Tool Error: {}\n", e)));
            }
        }
    }
}

mod autonomous_loop;
mod chat;
mod interactive_loop;
mod scan_loop;

#[cfg(test)]
mod tests;
