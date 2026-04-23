//! Interactive `dalang login` flows (provider-specific).

use anyhow::Result;
use dalang_adapters::adapters::outbound::{auth, llm};
use dialoguer::{Password, Select, theme::ColorfulTheme};

pub async fn run(provider: auth::AuthProvider) -> Result<()> {
    println!("Logging in to {}...\n", provider.as_str());

    if let Err(e) = auth::persistence::save_active_provider(provider.as_str()) {
        println!("[-] Failed to save active provider: {}", e);
    }

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
                    crate::runtime::interactive_model_selection(
                        "gemini",
                        &base_url,
                        api_key.trim(),
                        "apikey",
                    )
                    .await;
                }
                1 => {
                    println!("\n[!] Account safety caution:");
                    println!(
                        "    This is an unofficial integration and is not endorsed by Google."
                    );
                    println!("    Some users have reported account restrictions after using");
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

                            let base_url = llm::get_default_base_url("gemini");
                            crate::runtime::interactive_model_selection(
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
                    println!("[*] Looking for Gemini CLI credentials...");
                    match auth::cli_extractor::extract_gemini_cli_token() {
                        Ok(token) => {
                            auth::persistence::save_tokens(&token, None)?;
                            let _ = auth::persistence::save_auth_method("bearer");
                            let _ = auth::persistence::save_endpoint_mode("openai_compat");
                            println!("[+] Gemini CLI token extracted and saved!");

                            let base_url = llm::get_default_base_url("gemini");
                            crate::runtime::interactive_model_selection(
                                "gemini", &base_url, &token, "bearer",
                            )
                            .await;
                        }
                        Err(e) => {
                            println!("[!] Failed: {}", e);
                            println!("    Make sure gemini-cli is installed and logged in.");
                            println!("    Try: gemini auth login, then re-run dalang login.");
                        }
                    }
                }
                3 => {
                    println!("[*] Extracting token from gcloud...");
                    match auth::cli_extractor::extract_gcloud_token() {
                        Ok(token) => {
                            auth::persistence::save_tokens(&token, None)?;
                            let _ = auth::persistence::save_auth_method("bearer");
                            let _ = auth::persistence::save_endpoint_mode("openai_compat");
                            println!("[+] gcloud ADC token extracted and saved!");
                            println!("[!] Note: ADC tokens expire. Re-run login to refresh.");

                            let base_url = llm::get_default_base_url("gemini");
                            crate::runtime::interactive_model_selection(
                                "gemini", &base_url, &token, "bearer",
                            )
                            .await;
                        }
                        Err(e) => {
                            println!("[!] Failed: {}", e);
                            println!("    Make sure gcloud is installed and logged in.");
                            println!("    Try: gcloud auth login, then re-run dalang login.");
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        auth::AuthProvider::OpenAi | auth::AuthProvider::Anthropic => {
            let key_url = match provider {
                auth::AuthProvider::OpenAi => "https://platform.openai.com/api-keys",
                auth::AuthProvider::Anthropic => "https://console.anthropic.com/settings/keys",
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
                    crate::runtime::interactive_model_selection(
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
                        crate::runtime::interactive_model_selection(
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
                    let confirm = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("This will open your browser for GitHub login. Continue?")
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

                            crate::runtime::copilot_model_selection().await;
                        }
                        Err(e) => {
                            println!("[!] Copilot Device Flow failed: {}", e);
                            println!("    You can try keychain extraction or manual PAT instead.");
                        }
                    }
                }
                1 => {
                    println!("[*] Looking for Copilot CLI credentials...");
                    match auth::copilot::try_extract_copilot_token() {
                        Some(token) => match auth::copilot::validate_github_token(&token).await {
                            Ok(login) => {
                                auth::persistence::save_tokens(&token, None)?;
                                let _ = auth::persistence::save_auth_method("copilot_oauth");
                                let _ = auth::persistence::save_endpoint_mode("copilot");
                                println!("[+] Copilot CLI token extracted! User: {}", login);

                                crate::runtime::copilot_model_selection().await;
                            }
                            Err(e) => {
                                println!("[!] Token found but validation failed: {}", e);
                                println!(
                                    "    The token may have expired. Try Device Flow instead."
                                );
                            }
                        },
                        None => {
                            println!("[!] No Copilot CLI credentials found.");
                            println!("    Install @github/copilot and run: github-copilot login");
                            println!("    Or use Device Flow instead.");
                        }
                    }
                }
                2 => match auth::copilot::extract_github_env_token() {
                    Ok(token) => match auth::copilot::validate_github_token(&token).await {
                        Ok(login) => {
                            auth::persistence::save_tokens(&token, None)?;
                            let _ = auth::persistence::save_auth_method("copilot_oauth");
                            let _ = auth::persistence::save_endpoint_mode("copilot");
                            println!("[+] GitHub token imported! User: {}", login);

                            crate::runtime::copilot_model_selection().await;
                        }
                        Err(e) => {
                            println!("[!] Token found but validation failed: {}", e);
                            println!(
                                "    Ensure COPILOT_GITHUB_TOKEN, GH_TOKEN, or GITHUB_TOKEN is valid."
                            );
                        }
                    },
                    Err(_) => {
                        println!("[!] No GitHub token found in environment.");
                        println!("    Set one of: COPILOT_GITHUB_TOKEN, GH_TOKEN, GITHUB_TOKEN");
                        println!(
                            "    Note: Classic PATs (ghp_) are not supported for Copilot API."
                        );
                    }
                },
                3 => {
                    println!("[*] Enter a GitHub fine-grained Personal Access Token.");
                    println!("    Classic tokens (ghp_) are NOT supported for Copilot API.");
                    println!("    Create one at: https://github.com/settings/tokens?type=beta\n");

                    let pat = Password::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter your GitHub PAT")
                        .interact()?;

                    if pat.trim().is_empty() {
                        return Err(anyhow::anyhow!("Token cannot be empty."));
                    }

                    if pat.trim().starts_with("ghp_") {
                        println!("[!] Classic PATs (ghp_) are not supported for Copilot API.");
                        println!("    Please use a fine-grained token instead.");
                        return Ok(());
                    }

                    match auth::copilot::validate_github_token(pat.trim()).await {
                        Ok(login) => {
                            auth::persistence::save_tokens(pat.trim(), None)?;
                            let _ = auth::persistence::save_auth_method("copilot_oauth");
                            let _ = auth::persistence::save_endpoint_mode("copilot");
                            println!("[+] GitHub PAT saved! User: {}", login);

                            crate::runtime::copilot_model_selection().await;
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
            let base_url = dialoguer::Input::<String>::with_theme(&ColorfulTheme::default())
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

            crate::runtime::interactive_model_selection(
                "custom",
                base_url.trim(),
                api_key.trim(),
                "apikey",
            )
            .await;
        }
    }
    Ok(())
}
