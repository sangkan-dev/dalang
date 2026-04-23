//! Composition root for wiring ports into `DalangOrchestrator`.

use dalang_adapters::adapters::outbound::browser_cdp::LazyBrowserAdapter;
use dalang_adapters::adapters::outbound::llm;
use dalang_adapters::adapters::outbound::os_command::OsCommandExecutor;
use dalang_application::application::ports::llm_port::LlmPort;
use dalang_application::application::usecases::orchestrator::{
    DalangOrchestrator, OrchestratorConfig,
};
use dalang_application::skills_parser::FileSystemSkillCatalog;
use std::sync::Arc;

use crate::runtime::ResolvedLlmRuntime;

pub fn wire_orchestrator(
    resolved: &ResolvedLlmRuntime,
    config: OrchestratorConfig,
    headed: bool,
) -> anyhow::Result<DalangOrchestrator> {
    let provider = llm::create_provider(
        &resolved.endpoint_mode,
        resolved.base_url.clone(),
        resolved.model.clone(),
        resolved.auth.clone(),
        resolved.codeassist_endpoint.clone(),
        resolved.gcp_project.clone(),
    )?;
    let llm_adapter: Arc<dyn LlmPort> = provider;
    let executor: Arc<dyn dalang_application::application::ports::os_port::CommandExecutor> =
        Arc::new(OsCommandExecutor);
    let browser: Arc<dyn dalang_application::application::ports::browser_port::BrowserPort> =
        Arc::new(LazyBrowserAdapter::new(!headed));

    Ok(DalangOrchestrator::new(
        llm_adapter,
        executor,
        Some(browser),
        Arc::new(FileSystemSkillCatalog),
        config,
    ))
}
