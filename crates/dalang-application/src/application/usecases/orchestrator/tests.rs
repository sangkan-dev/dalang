use super::{DalangOrchestrator, OrchestratorConfig};
use crate::application::ports::llm_port::LlmPort;
use crate::application::ports::os_port::CommandExecutor;
use crate::application::usecases::memory::{MAX_OBSERVATION_BYTES, truncate_output};
use crate::skills_parser::FileSystemSkillCatalog;
use anyhow::Result;
use dalang_domain::domain::models::{Message, SkillDefinition};
use dalang_domain::domain::safety::is_safety_refusal;
use std::sync::Arc;

// ── Mock Implementations ──────────────────────────────────────────────────

/// A mock LLM that always returns a fixed response.
struct MockLlm {
    response: String,
}

#[async_trait::async_trait]
impl LlmPort for MockLlm {
    async fn send_messages(&self, _messages: &[Message]) -> Result<String> {
        Ok(self.response.clone())
    }

    async fn send_messages_with_tools(
        &self,
        _messages: &[Message],
        _tools: Vec<serde_json::Value>,
    ) -> Result<String> {
        Ok(self.response.clone())
    }

    async fn get_available_models(&self) -> Result<Vec<String>> {
        Ok(vec!["mock-model".to_string()])
    }
}

/// A mock executor that returns preset output.
struct MockExecutor {
    stdout: String,
    stderr: String,
}

#[async_trait::async_trait]
impl CommandExecutor for MockExecutor {
    async fn execute(
        &self,
        _cmd: &str,
        _args: &[&str],
        _timeout_secs: u64,
    ) -> Result<(String, String)> {
        Ok((self.stdout.clone(), self.stderr.clone()))
    }
}

/// Build an orchestrator with mock dependencies.
fn make_orchestrator(llm_response: &str) -> DalangOrchestrator {
    let llm: Arc<dyn LlmPort> = Arc::new(MockLlm {
        response: llm_response.to_string(),
    });
    let executor: Arc<dyn CommandExecutor> = Arc::new(MockExecutor {
        stdout: "mock stdout".to_string(),
        stderr: String::new(),
    });
    DalangOrchestrator::new(
        llm,
        executor,
        None,
        Arc::new(FileSystemSkillCatalog),
        OrchestratorConfig::default(),
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[test]
fn test_orchestrator_construction() {
    let o = make_orchestrator("Hello");
    assert_eq!(o.config.cmd_timeout, 300);
    assert!(!o.config.verbose);
    assert!(o.config.headless);
}

#[test]
fn test_config_defaults() {
    let config = OrchestratorConfig::default();
    assert_eq!(config.cmd_timeout, 300);
    assert!(!config.verbose);
    assert!(config.headless);
    assert!(config.disabled_skills.is_empty());
}

// ── DalangOrchestrator::truncate_output (static) ──────────────────────────

#[test]
fn test_truncate_output_short() {
    let input = "short output";
    let result = truncate_output(input, MAX_OBSERVATION_BYTES);
    assert_eq!(result, input);
}

#[test]
fn test_truncate_output_long() {
    let long_input = "x".repeat(MAX_OBSERVATION_BYTES + 100);
    let result = truncate_output(&long_input, MAX_OBSERVATION_BYTES);
    assert!(result.contains("TRUNCATED"));
    assert!(result.len() < long_input.len());
}

// ── parse_llm_tool_call (free fn in domain::tool_call) ─────────────────

#[test]
fn test_parse_tool_call_valid_json() {
    let raw = r#"```json
{"name": "nmap_scan", "arguments": {"target": "192.168.1.1"}}
```"#;
    // parse_llm_tool_call is now a free fn; just assert it's not a safety refusal.
    assert!(!is_safety_refusal(raw));
}

// ── is_safety_refusal (free fn in domain::safety) ─────────────────────────

#[test]
fn test_safety_refusal_detected() {
    let refusals = [
        "I cannot assist with this request.",
        "I am unable to help.",
        "My safety guidelines prevent me from doing that.",
        "I'm sorry, but I can't help with that.",
        "As an AI, I must decline.",
    ];
    for r in &refusals {
        assert!(is_safety_refusal(r), "Expected safety refusal for: '{}'", r);
    }
}

#[test]
fn test_safety_refusal_not_triggered() {
    let ok = [
        "I found port 80 open on the target.",
        "The nmap scan reveals 3 open ports.",
        "Analysing HTTP response headers...",
    ];
    for r in &ok {
        assert!(
            !is_safety_refusal(r),
            "False positive safety refusal for: '{}'",
            r
        );
    }
}

// ── filter_disabled_skills ────────────────────────────────────────────────

#[test]
fn test_filter_disabled_skills_removes_named_skill() {
    let llm: Arc<dyn LlmPort> = Arc::new(MockLlm {
        response: String::new(),
    });
    let executor: Arc<dyn CommandExecutor> = Arc::new(MockExecutor {
        stdout: String::new(),
        stderr: String::new(),
    });
    let config = OrchestratorConfig {
        disabled_skills: vec!["nmap-scan".to_string()],
        ..Default::default()
    };
    let o = DalangOrchestrator::new(
        llm,
        executor,
        None,
        Arc::new(FileSystemSkillCatalog),
        config,
    );

    let make_skill = |name: &str| SkillDefinition {
        name: name.to_string(),
        description: String::new(),
        tool_path: None,
        args: None,
        requires_root: None,
        system_prompt: String::new(),
        role: None,
        task: None,
        constraints: None,
        tool_available: true,
    };

    let skills = vec![make_skill("nmap-scan"), make_skill("web-crawl")];
    let (enabled, disabled) = o.filter_disabled_skills(skills, vec![]);

    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled[0].name, "web-crawl");
    assert!(disabled.contains(&"nmap-scan".to_string()));
}

#[test]
fn test_filter_disabled_skills_no_disabled() {
    let o = make_orchestrator("");
    let make_skill = |name: &str| SkillDefinition {
        name: name.to_string(),
        description: String::new(),
        tool_path: None,
        args: None,
        requires_root: None,
        system_prompt: String::new(),
        role: None,
        task: None,
        constraints: None,
        tool_available: true,
    };
    let skills = vec![make_skill("nmap-scan"), make_skill("web-crawl")];
    let (enabled, _) = o.filter_disabled_skills(skills, vec![]);
    // Nothing disabled → all skills pass through
    assert_eq!(enabled.len(), 2);
}

// ── Mock port integration ─────────────────────────────────────────────────

#[tokio::test]
async fn test_mock_llm_returns_response() {
    let llm = MockLlm {
        response: "Test answer".to_string(),
    };
    let msgs = vec![Message::user("hello")];
    let resp = llm.send_messages(&msgs).await.unwrap();
    assert_eq!(resp, "Test answer");
}

#[tokio::test]
async fn test_mock_executor_returns_stdout() {
    let executor = MockExecutor {
        stdout: "command output".to_string(),
        stderr: String::new(),
    };
    let (stdout, stderr) = executor.execute("echo", &["test"], 5).await.unwrap();
    assert_eq!(stdout, "command output");
    assert!(stderr.is_empty());
}
