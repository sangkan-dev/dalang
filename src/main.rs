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

            // FIX-04: Use dynamic provider-aware base URL
            let base_url = std::env::var("LLM_BASE_URL")
                .unwrap_or_else(|_| llm::get_default_base_url(provider.as_str()));

            // SPRINT 13: Save active provider immediately
            if let Err(e) = auth::persistence::save_active_provider(provider.as_str()) {
                println!("[-] Failed to save active provider: {}", e);
            }

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

                        // Interactive Model Selection
                        interactive_model_selection(provider.as_str(), &base_url, access).await;
                    }
                }
                // FIX-01: API Key login for OpenAI and Anthropic
                auth::AuthProvider::OpenAi | auth::AuthProvider::Anthropic => {
                    use dialoguer::{Password, theme::ColorfulTheme};

                    let api_key = Password::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!("Enter your {} API Key", provider.as_str()))
                        .interact()?;

                    if api_key.is_empty() {
                        return Err(anyhow::anyhow!("API Key cannot be empty."));
                    }

                    // Save API key as access token in keyring
                    auth::persistence::save_tokens(&api_key, None)?;
                    println!("[+] API Key saved to keyring!");

                    // Interactive Model Selection
                    interactive_model_selection(provider.as_str(), &base_url, &api_key).await;
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
            let auth = resolve_auth_token();

            // SPRINT 13: Get active provider and resolve defaults
            let active_provider =
                auth::persistence::get_active_provider().unwrap_or_else(|_| "gemini".to_string());
            let default_base_url = llm::get_default_base_url(&active_provider);
            let default_model = llm::get_default_model(&active_provider);

            let base_url = std::env::var("LLM_BASE_URL").unwrap_or(default_base_url);

            let model = std::env::var("LLM_MODEL")
                .or_else(|_| auth::persistence::get_model_preference())
                .unwrap_or(default_model);

            println!("[*] Using Provider: {} | Model: {}", active_provider, model);

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

            let auth = resolve_auth_token();
            if matches!(auth, llm::AuthToken::None) {
                return Err(anyhow::anyhow!(
                    "No API key found. Please run 'dalang login' or set LLM_API_KEY."
                ));
            }

            // SPRINT 13: Get active provider and resolve defaults
            let active_provider =
                auth::persistence::get_active_provider().unwrap_or_else(|_| "gemini".to_string());
            let default_base_url = llm::get_default_base_url(&active_provider);
            let default_model = llm::get_default_model(&active_provider);

            let base_url = std::env::var("LLM_BASE_URL").unwrap_or(default_base_url);

            let model = std::env::var("LLM_MODEL")
                .or_else(|_| auth::persistence::get_model_preference())
                .unwrap_or(default_model);

            println!("[*] Using Provider: {} | Model: {}", active_provider, model);

            let provider = llm::openai::OpenAiCompatibleProvider::new(base_url, model, auth)?;
            let engine = core::engine::DalangEngine::new(Box::new(provider));

            engine.run_interactive_loop(&target).await?;
        }
    }

    Ok(())
}

/// FIX-11: Resolve auth token using provider-aware detection instead of string length heuristic.
fn resolve_auth_token() -> llm::AuthToken {
    // Priority: CLI Extractor -> Keyring -> Env -> None
    if let Some(token) = auth::cli_extractor::try_all_cli_extractors() {
        return llm::AuthToken::Bearer(token);
    }

    if let Ok(token) = auth::persistence::get_access_token() {
        println!("[+] Using stored session from keyring");
        // Determine token type based on active provider
        let provider = auth::persistence::get_active_provider().unwrap_or_default();
        return match provider.as_str() {
            // OpenAI/Anthropic use API Keys (stored as access_token in keyring)
            "openai" | "anthropic" => llm::AuthToken::ApiKey(token),
            // Gemini/Google use Bearer tokens from OAuth
            _ => llm::AuthToken::Bearer(token),
        };
    }

    if let Ok(key) = std::env::var("LLM_API_KEY") {
        return llm::AuthToken::ApiKey(key);
    }

    println!("[!] No active session found. Please run 'dalang login' or set LLM_API_KEY");
    llm::AuthToken::None
}

/// Interactive model selection after successful login.
async fn interactive_model_selection(provider_name: &str, base_url: &str, token: &str) {
    println!("[*] Fetching available models for {}...", provider_name);

    // Determine auth token type based on provider
    let auth_token = match provider_name {
        "openai" | "anthropic" => llm::AuthToken::ApiKey(token.to_string()),
        _ => llm::AuthToken::Bearer(token.to_string()),
    };

    if let Ok(llm_provider) = llm::openai::OpenAiCompatibleProvider::new(
        base_url.to_string(),
        "dummy".to_string(),
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
                if auth::persistence::save_model_preference(chosen_model).is_ok() {
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
