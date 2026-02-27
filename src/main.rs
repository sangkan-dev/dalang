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
    let args = DalangArgs::parse();

    match args.command {
        Commands::Init => {
            println!("Initializing Dalang environment...");
            let skills_dir = std::path::Path::new("skills");
            if !skills_dir.exists() {
                std::fs::create_dir_all(skills_dir)?;
                println!("[+] Created skills/ directory.");
            }

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
        Commands::Login { provider } => {
            let provider = auth::AuthProvider::from_str(&provider)?;
            println!("Logging in to {}...\n", provider.as_str());

            let base_url = std::env::var("LLM_BASE_URL")
                .unwrap_or_else(|_| llm::get_default_base_url(provider.as_str()));

            if let Err(e) = auth::persistence::save_active_provider(provider.as_str()) {
                println!("[-] Failed to save active provider: {}", e);
            }

            use dialoguer::{Password, Select, theme::ColorfulTheme};

            match provider {
                auth::AuthProvider::Gemini => {
                    // ── Gemini: 3 auth methods ──
                    let methods = vec![
                        "API Key (recommended) — paste from https://aistudio.google.com/apikey",
                        "Gemini CLI — auto-extract from existing gemini-cli session",
                        "Google Cloud ADC — use gcloud Application Default Credentials",
                    ];

                    let selection = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Select authentication method")
                        .default(0)
                        .items(&methods)
                        .interact()?;

                    match selection {
                        0 => {
                            let api_key = Password::with_theme(&ColorfulTheme::default())
                                .with_prompt("Enter your Gemini API Key")
                                .interact()?;
                            if api_key.trim().is_empty() {
                                return Err(anyhow::anyhow!("API Key cannot be empty."));
                            }
                            auth::persistence::save_tokens(api_key.trim(), None)?;
                            println!("[+] API Key saved to keyring!");
                            interactive_model_selection(
                                provider.as_str(),
                                &base_url,
                                api_key.trim(),
                            )
                            .await;
                        }
                        1 => {
                            println!("[*] Looking for Gemini CLI credentials...");
                            match auth::cli_extractor::extract_gemini_cli_token() {
                                Ok(token) => {
                                    auth::persistence::save_tokens(&token, None)?;
                                    println!("[+] Gemini CLI token extracted and saved!");
                                    interactive_model_selection(
                                        provider.as_str(),
                                        &base_url,
                                        &token,
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    println!("[!] Failed: {}", e);
                                    println!(
                                        "    Make sure gemini-cli is installed and logged in."
                                    );
                                    println!(
                                        "    Try: gemini auth login, then re-run dalang login."
                                    );
                                }
                            }
                        }
                        2 => {
                            println!("[*] Extracting token from gcloud...");
                            match auth::cli_extractor::extract_gcloud_token() {
                                Ok(token) => {
                                    auth::persistence::save_tokens(&token, None)?;
                                    println!("[+] gcloud ADC token extracted and saved!");
                                    println!(
                                        "[!] Note: ADC tokens expire. Re-run login to refresh."
                                    );
                                    interactive_model_selection(
                                        provider.as_str(),
                                        &base_url,
                                        &token,
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    println!("[!] Failed: {}", e);
                                    println!("    Make sure gcloud is installed and logged in.");
                                    println!(
                                        "    Try: gcloud auth login, then re-run dalang login."
                                    );
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                auth::AuthProvider::OpenAi | auth::AuthProvider::Anthropic => {
                    // ── OpenAI / Anthropic: 2 auth methods ──
                    let key_url = match provider {
                        auth::AuthProvider::OpenAi => "https://platform.openai.com/api-keys",
                        auth::AuthProvider::Anthropic => {
                            "https://console.anthropic.com/settings/keys"
                        }
                        _ => unreachable!(),
                    };
                    let methods = vec![
                        format!("API Key (recommended) — paste from {}", key_url),
                        "Environment Variable — use existing LLM_API_KEY".to_string(),
                    ];

                    let selection = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Select authentication method")
                        .default(0)
                        .items(&methods)
                        .interact()?;

                    match selection {
                        0 => {
                            let api_key = Password::with_theme(&ColorfulTheme::default())
                                .with_prompt(format!("Enter your {} API Key", provider.as_str()))
                                .interact()?;
                            if api_key.trim().is_empty() {
                                return Err(anyhow::anyhow!("API Key cannot be empty."));
                            }
                            auth::persistence::save_tokens(api_key.trim(), None)?;
                            println!("[+] API Key saved to keyring!");
                            interactive_model_selection(
                                provider.as_str(),
                                &base_url,
                                api_key.trim(),
                            )
                            .await;
                        }
                        1 => match std::env::var("LLM_API_KEY") {
                            Ok(key) if !key.trim().is_empty() => {
                                auth::persistence::save_tokens(key.trim(), None)?;
                                println!("[+] LLM_API_KEY imported and saved to keyring!");
                                interactive_model_selection(
                                    provider.as_str(),
                                    &base_url,
                                    key.trim(),
                                )
                                .await;
                            }
                            _ => {
                                println!("[!] LLM_API_KEY environment variable not set or empty.");
                                println!("    Set it with: export LLM_API_KEY=\"your-key\"");
                            }
                        },
                        _ => unreachable!(),
                    }
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

            let auth = resolve_auth_token();

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

fn resolve_auth_token() -> llm::AuthToken {
    if let Some(token) = auth::cli_extractor::try_all_cli_extractors() {
        return llm::AuthToken::Bearer(token);
    }

    if let Ok(token) = auth::persistence::get_access_token() {
        println!("[+] Using stored session from keyring");
        return llm::AuthToken::ApiKey(token);
    }

    if let Ok(key) = std::env::var("LLM_API_KEY") {
        return llm::AuthToken::ApiKey(key);
    }

    println!("[!] No active session found. Please run 'dalang login' or set LLM_API_KEY");
    llm::AuthToken::None
}

async fn interactive_model_selection(provider_name: &str, base_url: &str, token: &str) {
    println!("[*] Fetching available models for {}...", provider_name);

    let auth_token = match provider_name {
        "openai" | "anthropic" => llm::AuthToken::ApiKey(token.to_string()),
        _ => llm::AuthToken::ApiKey(token.to_string()),
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
