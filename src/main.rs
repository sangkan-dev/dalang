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

            // Install all bundled skills
            let bundled = skills_parser::bundled::BUNDLED_SKILLS;
            let mut installed = 0;
            let mut skipped = 0;

            for skill in bundled {
                let skill_path = skills_dir.join(skill.filename);
                if skill_path.exists() {
                    skipped += 1;
                } else {
                    std::fs::write(&skill_path, skill.content)?;
                    println!("[+] Installed skill: {}", skill.filename);
                    installed += 1;
                }
            }

            println!(
                "[✓] Initialization complete! {} skills installed, {} already existed.",
                installed, skipped
            );
        }
        Commands::Login { provider, oauth } => {
            let provider = auth::AuthProvider::from_str(&provider)?;
            println!("Logging in to {}...", provider.as_str());

            // Use dynamic provider-aware base URL
            let base_url = std::env::var("LLM_BASE_URL")
                .unwrap_or_else(|_| llm::get_default_base_url(provider.as_str()));

            // Save active provider immediately
            if let Err(e) = auth::persistence::save_active_provider(provider.as_str()) {
                println!("[-] Failed to save active provider: {}", e);
            }

            if oauth {
                // ── OAuth Flow (--oauth flag) ──
                match provider {
                    auth::AuthProvider::Gemini => {
                        let config = auth::oauth::OauthConfig::gemini_default();
                        let url = auth::oauth::build_auth_url(&config);
                        println!("[*] Opening browser for OAuth authentication...");
                        println!("[!] Note: OAuth requires a configured client_secret.");
                        println!(
                            "[!] If this fails, use API Key login instead: dalang login --provider gemini"
                        );
                        open::that(&url)?;

                        let code = auth::oauth::run_callback_server(38343)?;
                        let token_data =
                            auth::oauth::perform_token_exchange(&config, &code).await?;

                        if let Some(access) =
                            token_data.get("access_token").and_then(|v| v.as_str())
                        {
                            let refresh = token_data.get("refresh_token").and_then(|v| v.as_str());
                            auth::persistence::save_tokens(access, refresh)?;
                            println!("[+] OAuth login successful!");
                            interactive_model_selection(provider.as_str(), &base_url, access).await;
                        }
                    }
                    _ => {
                        println!(
                            "[!] OAuth is only available for Gemini/Google. Use API Key instead."
                        );
                        println!("    dalang login --provider {}", provider.as_str());
                    }
                }
            } else {
                // ── API Key Flow (default for all providers) ──
                use dialoguer::{Password, theme::ColorfulTheme};

                let prompt_text = match provider {
                    auth::AuthProvider::Gemini => {
                        "Enter your Gemini API Key (from https://aistudio.google.com/apikey)"
                    }
                    auth::AuthProvider::OpenAi => {
                        "Enter your OpenAI API Key (from https://platform.openai.com/api-keys)"
                    }
                    auth::AuthProvider::Anthropic => {
                        "Enter your Anthropic API Key (from https://console.anthropic.com/settings/keys)"
                    }
                };

                let api_key = Password::with_theme(&ColorfulTheme::default())
                    .with_prompt(prompt_text)
                    .interact()?;

                if api_key.trim().is_empty() {
                    return Err(anyhow::anyhow!("API Key cannot be empty."));
                }

                auth::persistence::save_tokens(api_key.trim(), None)?;
                println!("[+] API Key saved to keyring!");

                // Interactive Model Selection
                interactive_model_selection(provider.as_str(), &base_url, api_key.trim()).await;
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
        // CLI-extracted tokens (gcloud, gemini-cli) are always Bearer tokens
        return llm::AuthToken::Bearer(token);
    }

    if let Ok(token) = auth::persistence::get_access_token() {
        println!("[+] Using stored session from keyring");
        // Since all providers now default to API Key login,
        // stored tokens are API keys unless from OAuth
        return llm::AuthToken::ApiKey(token);
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
