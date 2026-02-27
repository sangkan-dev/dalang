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

            if let Err(e) = auth::persistence::save_active_provider(provider.as_str()) {
                println!("[-] Failed to save active provider: {}", e);
            }

            use dialoguer::{Input, Password, Select, theme::ColorfulTheme};

            match provider {
                auth::AuthProvider::Gemini => {
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
                            // ── API Key ──
                            let api_key = Password::with_theme(&ColorfulTheme::default())
                                .with_prompt("Enter your Gemini API Key")
                                .interact()?;
                            if api_key.trim().is_empty() {
                                return Err(anyhow::anyhow!("API Key cannot be empty."));
                            }
                            auth::persistence::save_tokens(api_key.trim(), None)?;
                            let _ = auth::persistence::save_auth_method("apikey");
                            println!("[+] API Key saved to keyring!");

                            let base_url = llm::get_default_base_url("gemini");
                            interactive_model_selection(
                                "gemini",
                                &base_url,
                                api_key.trim(),
                                "apikey",
                            )
                            .await;
                        }
                        1 => {
                            // ── Gemini CLI token extraction ──
                            println!("[*] Looking for Gemini CLI credentials...");
                            match auth::cli_extractor::extract_gemini_cli_token() {
                                Ok(token) => {
                                    auth::persistence::save_tokens(&token, None)?;
                                    let _ = auth::persistence::save_auth_method("bearer");
                                    println!("[+] Gemini CLI token extracted and saved!");

                                    // Prompt for GCP project ID (required for Vertex AI)
                                    println!("\n[!] Vertex AI endpoint requires a GCP project ID.");
                                    let project: String =
                                        Input::with_theme(&ColorfulTheme::default())
                                            .with_prompt("Enter your GCP Project ID")
                                            .interact_text()?;

                                    if project.trim().is_empty() {
                                        return Err(anyhow::anyhow!(
                                            "GCP Project ID cannot be empty."
                                        ));
                                    }
                                    let _ = auth::persistence::save_gcp_project(project.trim());
                                    println!("[+] GCP Project saved: {}", project.trim());

                                    let base_url =
                                        llm::get_vertex_base_url(project.trim(), "us-central1");
                                    interactive_model_selection(
                                        "gemini", &base_url, &token, "bearer",
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
                            // ── gcloud ADC ──
                            println!("[*] Extracting token from gcloud...");
                            match auth::cli_extractor::extract_gcloud_token() {
                                Ok(token) => {
                                    auth::persistence::save_tokens(&token, None)?;
                                    let _ = auth::persistence::save_auth_method("bearer");
                                    println!("[+] gcloud ADC token extracted and saved!");
                                    println!(
                                        "[!] Note: ADC tokens expire. Re-run login to refresh."
                                    );

                                    // Prompt for GCP project ID
                                    println!("\n[!] Vertex AI endpoint requires a GCP project ID.");
                                    let project: String =
                                        Input::with_theme(&ColorfulTheme::default())
                                            .with_prompt("Enter your GCP Project ID")
                                            .interact_text()?;

                                    if project.trim().is_empty() {
                                        return Err(anyhow::anyhow!(
                                            "GCP Project ID cannot be empty."
                                        ));
                                    }
                                    let _ = auth::persistence::save_gcp_project(project.trim());
                                    println!("[+] GCP Project saved: {}", project.trim());

                                    let base_url =
                                        llm::get_vertex_base_url(project.trim(), "us-central1");
                                    interactive_model_selection(
                                        "gemini", &base_url, &token, "bearer",
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

                    let base_url = llm::get_default_base_url(provider.as_str());

                    match selection {
                        0 => {
                            let api_key = Password::with_theme(&ColorfulTheme::default())
                                .with_prompt(format!("Enter your {} API Key", provider.as_str()))
                                .interact()?;
                            if api_key.trim().is_empty() {
                                return Err(anyhow::anyhow!("API Key cannot be empty."));
                            }
                            auth::persistence::save_tokens(api_key.trim(), None)?;
                            let _ = auth::persistence::save_auth_method("apikey");
                            println!("[+] API Key saved to keyring!");
                            interactive_model_selection(
                                provider.as_str(),
                                &base_url,
                                api_key.trim(),
                                "apikey",
                            )
                            .await;
                        }
                        1 => match std::env::var("LLM_API_KEY") {
                            Ok(key) if !key.trim().is_empty() => {
                                auth::persistence::save_tokens(key.trim(), None)?;
                                let _ = auth::persistence::save_auth_method("apikey");
                                println!("[+] LLM_API_KEY imported and saved to keyring!");
                                interactive_model_selection(
                                    provider.as_str(),
                                    &base_url,
                                    key.trim(),
                                    "apikey",
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

            let (auth, base_url, model) = resolve_runtime_config();

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

            let (auth, base_url, model) = resolve_runtime_config();
            if matches!(auth, llm::AuthToken::None) {
                return Err(anyhow::anyhow!(
                    "No API key found. Please run 'dalang login' or set LLM_API_KEY."
                ));
            }

            let provider = llm::openai::OpenAiCompatibleProvider::new(base_url, model, auth)?;
            let engine = core::engine::DalangEngine::new(Box::new(provider));

            engine.run_interactive_loop(&target).await?;
        }
    }

    Ok(())
}

/// Resolve auth token, base URL, and model based on stored provider + auth method.
fn resolve_runtime_config() -> (llm::AuthToken, String, String) {
    let active_provider =
        auth::persistence::get_active_provider().unwrap_or_else(|_| "gemini".to_string());
    let auth_method = auth::persistence::get_auth_method().unwrap_or_else(|_| "apikey".to_string());

    // Resolve auth token
    let auth = resolve_auth_token(&auth_method);

    // Resolve base URL — Vertex AI for bearer, standard for API Key
    let base_url = if auth_method == "bearer"
        && (active_provider == "gemini" || active_provider == "google")
    {
        if let Ok(project) = auth::persistence::get_gcp_project() {
            llm::get_vertex_base_url(&project, "us-central1")
        } else {
            println!("[!] GCP project not set. Run 'dalang login --provider gemini' to configure.");
            llm::get_default_base_url(&active_provider)
        }
    } else {
        std::env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| llm::get_default_base_url(&active_provider))
    };

    // Resolve model
    let model = std::env::var("LLM_MODEL")
        .or_else(|_| auth::persistence::get_model_preference())
        .unwrap_or_else(|_| llm::get_default_model(&active_provider));

    println!(
        "[*] Using Provider: {} | Model: {} | Auth: {}",
        active_provider, model, auth_method
    );

    (auth, base_url, model)
}

fn resolve_auth_token(auth_method: &str) -> llm::AuthToken {
    // Priority: Keyring -> Env -> CLI Extractor -> None
    if let Ok(token) = auth::persistence::get_access_token() {
        println!("[+] Using stored session from keyring");
        return match auth_method {
            "bearer" => llm::AuthToken::Bearer(token),
            _ => llm::AuthToken::ApiKey(token),
        };
    }

    if let Ok(key) = std::env::var("LLM_API_KEY") {
        return llm::AuthToken::ApiKey(key);
    }

    if let Some(token) = auth::cli_extractor::try_all_cli_extractors() {
        return llm::AuthToken::Bearer(token);
    }

    println!("[!] No active session found. Please run 'dalang login' or set LLM_API_KEY");
    llm::AuthToken::None
}

/// Interactive model selection after login. Falls back to curated list if API fetch fails.
async fn interactive_model_selection(
    provider_name: &str,
    base_url: &str,
    token: &str,
    auth_method: &str,
) {
    println!("[*] Fetching available models for {}...", provider_name);

    let auth_token = match auth_method {
        "bearer" => llm::AuthToken::Bearer(token.to_string()),
        _ => llm::AuthToken::ApiKey(token.to_string()),
    };

    let mut models: Vec<String> = Vec::new();

    if let Ok(llm_provider) = llm::openai::OpenAiCompatibleProvider::new(
        base_url.to_string(),
        "dummy".to_string(),
        auth_token,
    ) {
        use crate::llm::LlmProvider;
        if let Ok(fetched) = llm_provider.get_available_models().await {
            models = fetched;
        }
    }

    // Fallback: curated model list when API fetch fails
    if models.is_empty() {
        println!("[!] Could not fetch models from API. Showing curated list.");
        models = get_fallback_models(provider_name);
    }

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
    }
}

fn get_fallback_models(provider: &str) -> Vec<String> {
    match provider {
        "gemini" | "google" => vec![
            "gemini-3.1-pro-preview".to_string(),
            "gemini-3-pro-preview".to_string(),
            "gemini-2.5-flash".to_string(),
            "gemini-2.5-pro-preview-05-06".to_string(),
            "gemini-2.5-flash-preview-05-20".to_string(),
            "gemini-2.0-flash".to_string(),
            "gemini-2.0-flash-lite".to_string(),
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
        ],
        "openai" => vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "o1".to_string(),
            "o1-mini".to_string(),
        ],
        "anthropic" => vec![
            "claude-sonnet-4-20250514".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
        ],
        _ => vec!["auto".to_string()],
    }
}
