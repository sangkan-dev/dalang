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
            // TODO: Implement init logic
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
            let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "gemini-1.5-pro".to_string());

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
            // TODO: Implement interactive logic
        }
    }

    Ok(())
}
