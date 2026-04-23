mod init_env;
mod login_flow;
mod orchestrator_wiring;
mod runtime;

use anyhow::Result;
use clap::Parser;
use dalang_adapters::adapters::inbound::cli::{Commands, DalangArgs};
use dalang_adapters::adapters::inbound::web;
use dalang_adapters::adapters::outbound::{auth, llm};
use dalang_application::application::usecases::orchestrator::OrchestratorConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let args = DalangArgs::parse();
    let verbose = args.verbose;

    match args.command {
        Commands::Init => init_env::run()?,
        Commands::Login { provider } => {
            let provider = auth::AuthProvider::from_name(&provider)?;
            login_flow::run(provider).await?;
        }
        Commands::Scan {
            target,
            skills,
            auto,
            max_iter,
            cmd_timeout,
            headed,
        } => {
            println!("Starting automated scan...");
            println!("Target: {}", target);
            if auto {
                println!("Mode: Autonomous Auto-Pilot");
            } else {
                println!("Skills: {}", skills.as_deref().unwrap_or("none"));
            }

            let resolved = runtime::resolve_runtime_config();
            let orchestrator = orchestrator_wiring::wire_orchestrator(
                &resolved,
                OrchestratorConfig {
                    cmd_timeout,
                    verbose,
                    headless: !headed,
                    disabled_skills: vec![],
                },
                headed,
            )?;

            if auto {
                orchestrator
                    .run_autonomous_loop(&target, max_iter, None)
                    .await?;
            } else {
                let skills_list = skills
                    .ok_or_else(|| anyhow::anyhow!("Either specify --skills or use --auto"))?;
                orchestrator.run_scan_loop(&target, &skills_list).await?;
            }
        }
        Commands::Interact {
            target,
            cmd_timeout,
            headed,
        } => {
            println!("Starting interactive session...");
            println!("Target: {}", target);

            let resolved = runtime::resolve_runtime_config();
            if matches!(resolved.auth, llm::AuthToken::None) {
                return Err(anyhow::anyhow!(
                    "No API key found. Please run 'dalang login' or set LLM_API_KEY."
                ));
            }

            let orchestrator = orchestrator_wiring::wire_orchestrator(
                &resolved,
                OrchestratorConfig {
                    cmd_timeout,
                    verbose,
                    headless: !headed,
                    disabled_skills: vec![],
                },
                headed,
            )?;

            orchestrator.run_interactive_loop(&target, None).await?;
        }
        Commands::Model { set } => {
            let active_provider =
                auth::persistence::get_active_provider().unwrap_or_else(|_| "gemini".to_string());
            let current_model = auth::persistence::get_model_preference()
                .unwrap_or_else(|_| llm::get_default_model(&active_provider));

            println!("[*] Provider: {}", active_provider);
            println!("[*] Current model: {}", current_model);

            if let Some(model_name) = set {
                auth::persistence::save_model_preference(&model_name)?;
                println!("[+] Model switched to: {}", model_name);
            } else {
                use dialoguer::{Select, theme::ColorfulTheme};

                let mut models = runtime::get_fallback_models(&active_provider);

                if !models.contains(&current_model) {
                    models.insert(0, current_model.clone());
                }

                let default_idx = models.iter().position(|m| m == &current_model).unwrap_or(0);

                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select AI model")
                    .default(default_idx)
                    .items(&models)
                    .interact()?;

                let chosen = &models[selection];
                auth::persistence::save_model_preference(chosen)?;
                println!("[+] Model switched to: {}", chosen);
            }
        }
        Commands::Web { port, open, headed } => {
            web::start_server(port, open, verbose, !headed).await?;
        }
    }

    Ok(())
}
