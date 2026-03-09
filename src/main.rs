// ── Hexagonal Architecture layers ───────────────────────────────────────────
pub mod adapters;
pub mod application;
pub mod domain;

use adapters::inbound::cli::{Commands, DalangArgs};
use adapters::inbound::web;
use adapters::outbound::{auth, llm, skills_parser};
use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = DalangArgs::parse();
    let verbose = args.verbose;

    match args.command {
        Commands::Init => {
            println!("Initializing Dalang environment...");
            let skills_dir = std::path::Path::new("skills");
            if !skills_dir.exists() {
                std::fs::create_dir_all(skills_dir)?;
                println!("[+] Created skills/ directory.");
            }

            let mut installed = 0;
            let mut skipped = 0;

            for file in skills_parser::bundled::BUNDLED_SKILLS.files() {
                let filename = file.path().to_str().unwrap_or_default();
                let skill_path = skills_dir.join(filename);
                if skill_path.exists() {
                    skipped += 1;
                } else {
                    std::fs::write(&skill_path, file.contents())?;
                    println!("[+] Installed skill: {}", filename);
                    installed += 1;
                }
            }

            println!(
                "[✓] Initialization complete! {} skills installed, {} already existed.",
                installed, skipped
            );
        }
        Commands::Login { provider } => {
            let provider = auth::AuthProvider::from_name(&provider)?;
            println!("Logging in to {}...\n", provider.as_str());

            if let Err(e) = auth::persistence::save_active_provider(provider.as_str()) {
                println!("[-] Failed to save active provider: {}", e);
            }

            use dialoguer::{Password, Select, theme::ColorfulTheme};

            match provider {
                auth::AuthProvider::Gemini => {
                    let methods = vec![
                        "API Key (recommended) — paste from https://aistudio.google.com/apikey",
                        "Gemini CLI OAuth — full Cloud Code Assist flow (no GCP project needed)",
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
                            let _ = auth::persistence::save_endpoint_mode("openai_compat");
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
                            // ── Gemini CLI OAuth (full Cloud Code Assist flow) ──
                            println!("\n[!] Account safety caution:");
                            println!(
                                "    This is an unofficial integration and is not endorsed by Google."
                            );
                            println!(
                                "    Some users have reported account restrictions after using"
                            );
                            println!("    third-party Gemini CLI OAuth clients.");
                            println!("    Proceed only if you understand and accept this risk.\n");

                            let confirm = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
                                .with_prompt("Continue with Gemini CLI OAuth?")
                                .default(false)
                                .interact()?;

                            if !confirm {
                                println!("[*] Skipped Gemini CLI OAuth setup.");
                                return Ok(());
                            }

                            match auth::gemini_codeassist::login_gemini_cli_oauth().await {
                                Ok(result) => {
                                    auth::gemini_codeassist::persist_oauth_result(&result)?;
                                    println!("[+] Gemini CLI OAuth login successful!");
                                    println!("[+] Project: {}", result.project_id);
                                    println!("[+] Endpoint: {}", result.active_endpoint);

                                    // Use the standard Gemini base URL for model selection
                                    let base_url = llm::get_default_base_url("gemini");
                                    interactive_model_selection(
                                        "gemini",
                                        &base_url,
                                        &result.access_token,
                                        "bearer",
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    println!("[!] Gemini CLI OAuth failed: {}", e);
                                    println!("    You can try API key mode instead.");
                                }
                            }
                        }
                        2 => {
                            // ── Gemini CLI token extraction (legacy) ──
                            println!("[*] Looking for Gemini CLI credentials...");
                            match auth::cli_extractor::extract_gemini_cli_token() {
                                Ok(token) => {
                                    auth::persistence::save_tokens(&token, None)?;
                                    let _ = auth::persistence::save_auth_method("bearer");
                                    let _ = auth::persistence::save_endpoint_mode("openai_compat");
                                    println!("[+] Gemini CLI token extracted and saved!");

                                    let base_url = llm::get_default_base_url("gemini");
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
                        3 => {
                            // ── gcloud ADC ──
                            println!("[*] Extracting token from gcloud...");
                            match auth::cli_extractor::extract_gcloud_token() {
                                Ok(token) => {
                                    auth::persistence::save_tokens(&token, None)?;
                                    let _ = auth::persistence::save_auth_method("bearer");
                                    let _ = auth::persistence::save_endpoint_mode("openai_compat");
                                    println!("[+] gcloud ADC token extracted and saved!");
                                    println!(
                                        "[!] Note: ADC tokens expire. Re-run login to refresh."
                                    );

                                    let base_url = llm::get_default_base_url("gemini");
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
                auth::AuthProvider::Copilot => {
                    println!("[!] DISCLAIMER: This is an unofficial integration.");
                    println!(
                        "    Dalang uses the GitHub Copilot API through reverse-engineered endpoints."
                    );
                    println!("    This is NOT endorsed by GitHub/Microsoft.");
                    println!("    Use at your own risk — your account, your responsibility.\n");

                    let methods = vec![
                        "Device Flow OAuth (recommended) — authenticate via browser",
                        "Copilot CLI keychain — extract from existing @github/copilot session",
                        "Environment Variable — use COPILOT_GITHUB_TOKEN / GH_TOKEN / GITHUB_TOKEN",
                        "Manual PAT — paste a GitHub Personal Access Token (fine-grained)",
                    ];

                    let selection = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Select authentication method")
                        .default(0)
                        .items(&methods)
                        .interact()?;

                    match selection {
                        0 => {
                            // ── Device Flow OAuth ──
                            let confirm = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
                                .with_prompt(
                                    "This will open your browser for GitHub login. Continue?",
                                )
                                .default(true)
                                .interact()?;

                            if !confirm {
                                println!("[*] Skipped Copilot Device Flow setup.");
                                return Ok(());
                            }

                            match auth::copilot::login_copilot_device_flow().await {
                                Ok(result) => {
                                    auth::copilot::persist_copilot_login(&result)?;
                                    println!("[+] GitHub Copilot login successful!");
                                    println!("[+] User: {}", result.login);

                                    copilot_model_selection().await;
                                }
                                Err(e) => {
                                    println!("[!] Copilot Device Flow failed: {}", e);
                                    println!(
                                        "    You can try keychain extraction or manual PAT instead."
                                    );
                                }
                            }
                        }
                        1 => {
                            // ── Copilot CLI keychain extraction ──
                            println!("[*] Looking for Copilot CLI credentials...");
                            match auth::copilot::try_extract_copilot_token() {
                                Some(token) => {
                                    // Validate the token
                                    match auth::copilot::validate_github_token(&token).await {
                                        Ok(login) => {
                                            auth::persistence::save_tokens(&token, None)?;
                                            let _ = auth::persistence::save_auth_method(
                                                "copilot_oauth",
                                            );
                                            let _ =
                                                auth::persistence::save_endpoint_mode("copilot");
                                            println!(
                                                "[+] Copilot CLI token extracted! User: {}",
                                                login
                                            );

                                            copilot_model_selection().await;
                                        }
                                        Err(e) => {
                                            println!(
                                                "[!] Token found but validation failed: {}",
                                                e
                                            );
                                            println!(
                                                "    The token may have expired. Try Device Flow instead."
                                            );
                                        }
                                    }
                                }
                                None => {
                                    println!("[!] No Copilot CLI credentials found.");
                                    println!(
                                        "    Install @github/copilot and run: github-copilot login"
                                    );
                                    println!("    Or use Device Flow instead.");
                                }
                            }
                        }
                        2 => {
                            // ── Environment variable ──
                            match auth::copilot::extract_github_env_token() {
                                Ok(token) => {
                                    match auth::copilot::validate_github_token(&token).await {
                                        Ok(login) => {
                                            auth::persistence::save_tokens(&token, None)?;
                                            let _ = auth::persistence::save_auth_method(
                                                "copilot_oauth",
                                            );
                                            let _ =
                                                auth::persistence::save_endpoint_mode("copilot");
                                            println!("[+] GitHub token imported! User: {}", login);

                                            copilot_model_selection().await;
                                        }
                                        Err(e) => {
                                            println!(
                                                "[!] Token found but validation failed: {}",
                                                e
                                            );
                                            println!(
                                                "    Ensure COPILOT_GITHUB_TOKEN, GH_TOKEN, or GITHUB_TOKEN is valid."
                                            );
                                        }
                                    }
                                }
                                Err(_) => {
                                    println!("[!] No GitHub token found in environment.");
                                    println!(
                                        "    Set one of: COPILOT_GITHUB_TOKEN, GH_TOKEN, GITHUB_TOKEN"
                                    );
                                    println!(
                                        "    Note: Classic PATs (ghp_) are not supported for Copilot API."
                                    );
                                }
                            }
                        }
                        3 => {
                            // ── Manual PAT ──
                            println!("[*] Enter a GitHub fine-grained Personal Access Token.");
                            println!(
                                "    Classic tokens (ghp_) are NOT supported for Copilot API."
                            );
                            println!(
                                "    Create one at: https://github.com/settings/tokens?type=beta\n"
                            );

                            let pat = Password::with_theme(&ColorfulTheme::default())
                                .with_prompt("Enter your GitHub PAT")
                                .interact()?;

                            if pat.trim().is_empty() {
                                return Err(anyhow::anyhow!("Token cannot be empty."));
                            }

                            if pat.trim().starts_with("ghp_") {
                                println!(
                                    "[!] Classic PATs (ghp_) are not supported for Copilot API."
                                );
                                println!("    Please use a fine-grained token instead.");
                                return Ok(());
                            }

                            match auth::copilot::validate_github_token(pat.trim()).await {
                                Ok(login) => {
                                    auth::persistence::save_tokens(pat.trim(), None)?;
                                    let _ = auth::persistence::save_auth_method("copilot_oauth");
                                    let _ = auth::persistence::save_endpoint_mode("copilot");
                                    println!("[+] GitHub PAT saved! User: {}", login);

                                    copilot_model_selection().await;
                                }
                                Err(e) => {
                                    println!("[!] Token validation failed: {}", e);
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                auth::AuthProvider::Custom => {
                    let base_url =
                        dialoguer::Input::<String>::with_theme(&ColorfulTheme::default())
                            .with_prompt("Enter custom OpenAI-compatible Base URL")
                            .interact()?;

                    let api_key = Password::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter your API Key")
                        .interact()?;

                    if api_key.trim().is_empty() {
                        return Err(anyhow::anyhow!("API Key cannot be empty."));
                    }

                    auth::persistence::save_tokens(api_key.trim(), None)?;
                    auth::persistence::save_custom_base_url(base_url.trim())?;
                    let _ = auth::persistence::save_auth_method("apikey");
                    let _ = auth::persistence::save_endpoint_mode("openai_compat");
                    println!("[+] Custom configuration saved!");

                    interactive_model_selection(
                        "custom",
                        base_url.trim(),
                        api_key.trim(),
                        "apikey",
                    )
                    .await;
                }
            }
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

            let (auth, base_url, model, endpoint_mode, codeassist_ep, gcp_project) =
                resolve_runtime_config();

            // ── DI wiring: construct ports & orchestrator ──────────────────
            use crate::adapters::outbound::os_command::OsCommandExecutor;
            use crate::application::ports::llm_port::LlmPort;
            use crate::application::usecases::orchestrator::{
                DalangOrchestrator, OrchestratorConfig,
            };
            use std::sync::Arc;

            let provider = llm::create_provider(
                &endpoint_mode,
                base_url,
                model,
                auth,
                codeassist_ep,
                gcp_project,
            )?;
            let llm_adapter: Arc<dyn LlmPort> = provider;
            let executor: Arc<dyn crate::application::ports::os_port::CommandExecutor> =
                Arc::new(OsCommandExecutor);
            let browser: Arc<dyn crate::application::ports::browser_port::BrowserPort> =
                Arc::new(crate::adapters::outbound::browser_cdp::LazyBrowserAdapter::new(!headed));
            let orchestrator = DalangOrchestrator::new(
                llm_adapter,
                executor,
                Some(browser),
                OrchestratorConfig {
                    cmd_timeout,
                    verbose,
                    headless: !headed,
                    disabled_skills: vec![],
                },
            );

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

            let (auth, base_url, model, endpoint_mode, codeassist_ep, gcp_project) =
                resolve_runtime_config();
            if matches!(auth, llm::AuthToken::None) {
                return Err(anyhow::anyhow!(
                    "No API key found. Please run 'dalang login' or set LLM_API_KEY."
                ));
            }

            // ── DI wiring ────────────────────────────────────────────────
            use crate::adapters::outbound::os_command::OsCommandExecutor;
            use crate::application::ports::llm_port::LlmPort;
            use crate::application::usecases::orchestrator::{
                DalangOrchestrator, OrchestratorConfig,
            };
            use std::sync::Arc;

            let provider = llm::create_provider(
                &endpoint_mode,
                base_url,
                model,
                auth,
                codeassist_ep,
                gcp_project,
            )?;
            let llm_adapter: Arc<dyn LlmPort> = provider;
            let executor: Arc<dyn crate::application::ports::os_port::CommandExecutor> =
                Arc::new(OsCommandExecutor);
            let browser: Arc<dyn crate::application::ports::browser_port::BrowserPort> =
                Arc::new(crate::adapters::outbound::browser_cdp::LazyBrowserAdapter::new(!headed));
            let orchestrator = DalangOrchestrator::new(
                llm_adapter,
                executor,
                Some(browser),
                OrchestratorConfig {
                    cmd_timeout,
                    verbose,
                    headless: !headed,
                    disabled_skills: vec![],
                },
            );

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
                // Direct set via --set flag
                auth::persistence::save_model_preference(&model_name)?;
                println!("[+] Model switched to: {}", model_name);
            } else {
                // Interactive picker
                use dialoguer::{Select, theme::ColorfulTheme};

                let mut models = get_fallback_models(&active_provider);

                // If current model isn't in the list, prepend it
                if !models.contains(&current_model) {
                    models.insert(0, current_model.clone());
                }

                // Find current model index for default selection
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

/// Resolve auth token, base URL, model, endpoint mode, and optional codeassist endpoint.
fn resolve_runtime_config() -> (
    llm::AuthToken,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
) {
    let active_provider =
        auth::persistence::get_active_provider().unwrap_or_else(|_| "gemini".to_string());
    let auth_method = auth::persistence::get_auth_method().unwrap_or_else(|_| "apikey".to_string());
    let endpoint_mode =
        auth::persistence::get_endpoint_mode().unwrap_or_else(|_| "openai_compat".to_string());

    // Resolve auth token
    let auth = resolve_auth_token(&auth_method);

    // Resolve base URL — for cloudcode mode, the factory handles it internally;
    // for openai_compat, use LLM_BASE_URL, stored custom URL, or provider default
    let base_url = std::env::var("LLM_BASE_URL").unwrap_or_else(|_| {
        if endpoint_mode == "openai_compat" {
            auth::persistence::get_custom_base_url()
                .unwrap_or_else(|_| llm::get_default_base_url(&active_provider))
        } else {
            llm::get_default_base_url(&active_provider)
        }
    });

    // Resolve codeassist endpoint and GCP project (only relevant for cloudcode mode)
    let (codeassist_ep, gcp_project) = if endpoint_mode == "cloudcode" {
        (
            auth::persistence::get_codeassist_endpoint().ok(),
            auth::persistence::get_gcp_project().ok(),
        )
    } else {
        (None, None)
    };

    // Resolve model
    let model = std::env::var("LLM_MODEL")
        .or_else(|_| auth::persistence::get_model_preference())
        .unwrap_or_else(|_| llm::get_default_model(&active_provider));

    println!(
        "[*] Using Provider: {} | Model: {} | Auth: {} | Mode: {}",
        active_provider, model, auth_method, endpoint_mode
    );

    (
        auth,
        base_url,
        model,
        endpoint_mode,
        codeassist_ep,
        gcp_project,
    )
}

fn resolve_auth_token(auth_method: &str) -> llm::AuthToken {
    // Priority: Keyring -> Env -> CLI Extractor -> None
    if let Ok(token) = auth::persistence::get_access_token() {
        println!("[+] Using stored session from keyring");
        return match auth_method {
            "bearer" | "copilot_oauth" => llm::AuthToken::Bearer(token),
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

/// Copilot-specific model selection (uses curated list since Copilot API doesn't expose /models).
async fn copilot_model_selection() {
    let models = get_fallback_models("copilot");

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
            // Source: @google/gemini-cli-core config/models.js VALID_GEMINI_MODELS
            "gemini-2.5-flash".to_string(),
            "gemini-2.5-pro".to_string(),
            "gemini-2.5-flash-lite".to_string(),
            "gemini-3-pro-preview".to_string(),
            "gemini-3-flash-preview".to_string(),
            "gemini-3.1-pro-preview".to_string(),
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
        "copilot" | "github" | "github-copilot" => vec![
            "claude-sonnet-4.5".to_string(),
            "claude-sonnet-4.6".to_string(),
            "claude-haiku-4.5".to_string(),
            "claude-opus-4.6".to_string(),
            "claude-opus-4.6-fast".to_string(),
            "claude-opus-4.5".to_string(),
            "claude-sonnet-4".to_string(),
            "gemini-3-pro-preview".to_string(),
            "gpt-5.2".to_string(),
            "gpt-5.1".to_string(),
            "gpt-5-mini".to_string(),
            "gpt-4.1".to_string(),
        ],
        _ => vec!["auto".to_string()],
    }
}
