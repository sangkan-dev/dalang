pub mod auth;
pub mod cdp;
mod cli;
pub mod core;
pub mod executor;
pub mod llm;
pub mod skills_parser;

use anyhow::Result;
use clap::Parser;
use cli::{Commands, DalangArgs};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = DalangArgs::parse();

    match args.command {
        Commands::Init => {
            println!("Initializing Dalang environment...");
            let skills_dir = std::path::Path::new("skills");
            if !skills_dir.exists() {
                std::fs::create_dir_all(skills_dir)?;
                println!("[+] Created skills/ directory.");
            }

            let example_skill = skills_dir.join("example-nmap.md");
            if !example_skill.exists() {
                let content = r#"---
name: nmap_scanner
description: Basic port scanning using Nmap.
tool_path: nmap
args:
  - "-sV"
  - "{{target}}"
---

### ROLE
You are a Security Auditor.

### TASK
Identify open ports and services on the target.

### CONSTRAINTS
- Sanctioned audit environment.
"#;
                std::fs::write(&example_skill, content)?;
                println!("[+] Created example skill: skills/example-nmap.md");
            }
            println!("[✓] Initialization complete!");
        }
        Commands::Login { provider } => {
            let provider = auth::AuthProvider::from_str(&provider)?;
            println!("Logging in to {}...", provider.as_str());

            match provider {
                auth::AuthProvider::Gemini => {
                    let config = auth::oauth::OauthConfig::gemini_default();
                    let url = auth::oauth::build_auth_url(&config);
                    println!("[*] Opening browser for authentication...");
                    open::that(&url)?;

                    let code = auth::oauth::run_callback_server(38343)?;
                    let token_data = auth::oauth::perform_token_exchange(&config, &code).await?;

                    if let Some(access) = token_data.get("access_token").and_then(|v| v.as_str()) {
                        let refresh = token_data.get("refresh_token").and_then(|v| v.as_str());
                        auth::persistence::save_tokens(access, refresh)?;
                        println!("[+] Login successful and tokens saved to keyring!");

                        // SPRINT 12: Interactive Model Selection
                        println!("[*] Fetching available models for {}...", provider.as_str());
                        let auth_token = llm::AuthToken::Bearer(access.to_string());
                        let base_url = std::env::var("LLM_BASE_URL").unwrap_or_else(|_| {
                            "https://generativelanguage.googleapis.com/v1beta".to_string()
                        });

                        // We use a dummy model just to instantiate the provider for fetching
                        if let Ok(llm_provider) = llm::openai::OpenAiCompatibleProvider::new(
                            base_url,
                            "gemini-1.5-pro".to_string(),
                            auth_token,
                        ) {
                            use crate::llm::LlmProvider;
                            if let Ok(models) = llm_provider.get_available_models().await {
                                if !models.is_empty() {
                                    use dialoguer::{Select, theme::ColorfulTheme};
                                    let selection = Select::with_theme(&ColorfulTheme::default())
                                        .with_prompt("Select your preferred AI Model")
                                        .default(0)
                                        .items(&models)
                                        .interact()
                                        .unwrap_or(0);

                                    let chosen_model = &models[selection];
                                    if auth::persistence::save_model_preference(chosen_model)
                                        .is_ok()
                                    {
                                        println!("[+] Default model set to: {}", chosen_model);
                                    }
                                } else {
                                    println!("[-] No models returned from provider API.");
                                }
                            } else {
                                println!("[-] Failed to fetch models from provider API.");
                            }
                        }
                    }
                }
                _ => {
                    println!(
                        "[!] Provider {} OAuth not yet fully implemented in MVP.",
                        provider.as_str()
                    );
                }
            }
        }
        Commands::Scan {
            target,
            skills,
            auto,
        } => {
            println!("Starting automated scan...");
            println!("Target: {}", target);
            if auto {
                println!("Mode: Autonomous Auto-Pilot");
            } else {
                println!("Skills: {}", skills.as_deref().unwrap_or("none"));
            }

            // Try to find auth: CLI Extractor -> Keyring -> Env -> None
            let auth = if let Some(token) = auth::cli_extractor::try_all_cli_extractors() {
                llm::AuthToken::Bearer(token)
            } else if let Ok(token) = auth::persistence::get_access_token() {
                println!("[+] Using stored session from keyring");
                llm::AuthToken::Bearer(token)
            } else if let Ok(key) = std::env::var("LLM_API_KEY") {
                llm::AuthToken::ApiKey(key)
            } else {
                println!(
                    "[!] No active session found. Please run 'dalang login' or set LLM_API_KEY"
                );
                llm::AuthToken::None
            };

            let base_url = std::env::var("LLM_BASE_URL")
                .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta".to_string());

            // SPRINT 12: Load preferred model from ENV -> Persistence -> Default
            let model = std::env::var("LLM_MODEL")
                .or_else(|_| auth::persistence::get_model_preference())
                .unwrap_or_else(|_| "gemini-1.5-pro".to_string());

            println!("[*] Using Model: {}", model);

            let provider = llm::openai::OpenAiCompatibleProvider::new(base_url, model, auth)?;
            let engine = core::engine::DalangEngine::new(Box::new(provider));

            if auto {
                engine.run_autonomous_loop(&target).await?;
            } else {
                let skills_list = skills
                    .ok_or_else(|| anyhow::anyhow!("Either specify --skills or use --auto"))?;
                engine.run_scan_loop(&target, &skills_list).await?;
            }
        }
        Commands::Interact { target } => {
            println!("Starting interactive session...");
            println!("Target: {}", target);

            let auth_str = auth::cli_extractor::try_all_cli_extractors()
                .or_else(|| auth::persistence::get_access_token().ok())
                .or_else(|| std::env::var("LLM_API_KEY").ok())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "No API key found. Please run 'dalang login' or set LLM_API_KEY."
                    )
                })?;

            let auth = if auth_str.len() > 64 {
                llm::AuthToken::Bearer(auth_str)
            } else {
                llm::AuthToken::ApiKey(auth_str)
            };

            let base_url = std::env::var("LLM_BASE_URL")
                .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta".to_string());

            // SPRINT 12: Load preferred model from ENV -> Persistence -> Default
            let model = std::env::var("LLM_MODEL")
                .or_else(|_| auth::persistence::get_model_preference())
                .unwrap_or_else(|_| "gemini-1.5-pro".to_string());

            println!("[*] Using Model: {}", model);

            let provider = llm::openai::OpenAiCompatibleProvider::new(base_url, model, auth)?;
            let engine = core::engine::DalangEngine::new(Box::new(provider));

            engine.run_interactive_loop(&target).await?;
        }
    }

    Ok(())
}
