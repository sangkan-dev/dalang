//! CLI runtime configuration: LLM auth, base URL, model, and post-login model picker.

use dalang_adapters::adapters::outbound::{auth, llm};

/// Fully resolved settings needed to construct an LLM provider and orchestrator.
#[derive(Clone)]
pub struct ResolvedLlmRuntime {
    pub auth: llm::AuthToken,
    pub base_url: String,
    pub model: String,
    pub endpoint_mode: String,
    pub codeassist_endpoint: Option<String>,
    pub gcp_project: Option<String>,
}

pub fn resolve_runtime_config() -> ResolvedLlmRuntime {
    let active_provider =
        auth::persistence::get_active_provider().unwrap_or_else(|_| "gemini".to_string());
    let auth_method = auth::persistence::get_auth_method().unwrap_or_else(|_| "apikey".to_string());
    let endpoint_mode =
        auth::persistence::get_endpoint_mode().unwrap_or_else(|_| "openai_compat".to_string());

    let auth = resolve_auth_token(&auth_method);

    let env_base_url = std::env::var("LLM_BASE_URL")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    let base_url = if let Some(url) = env_base_url {
        url
    } else if endpoint_mode == "openai_compat" {
        auth::persistence::get_custom_base_url()
            .unwrap_or_else(|_| llm::get_default_base_url(&active_provider))
    } else {
        llm::get_default_base_url(&active_provider)
    };

    let (codeassist_endpoint, gcp_project) = if endpoint_mode == "cloudcode" {
        (
            auth::persistence::get_codeassist_endpoint().ok(),
            auth::persistence::get_gcp_project().ok(),
        )
    } else {
        (None, None)
    };

    let model = std::env::var("LLM_MODEL")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| {
            auth::persistence::get_model_preference()
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        })
        .unwrap_or_else(|| llm::get_default_model(&active_provider));

    println!(
        "[*] Using Provider: {} | Model: {} | Auth: {} | Mode: {}",
        active_provider, model, auth_method, endpoint_mode
    );

    ResolvedLlmRuntime {
        auth,
        base_url,
        model,
        endpoint_mode,
        codeassist_endpoint,
        gcp_project,
    }
}

fn resolve_auth_token(auth_method: &str) -> llm::AuthToken {
    if let Ok(token) = auth::persistence::get_access_token() {
        println!("[+] Using stored session from keyring");
        return match auth_method {
            "bearer" | "copilot_oauth" => llm::AuthToken::Bearer(token),
            _ => llm::AuthToken::ApiKey(token),
        };
    }

    if let Ok(key) = std::env::var("LLM_API_KEY") {
        let key = key.trim();
        if !key.is_empty() {
            return llm::AuthToken::ApiKey(key.to_string());
        }
    }

    if let Some(token) = auth::cli_extractor::try_all_cli_extractors() {
        return llm::AuthToken::Bearer(token);
    }

    println!("[!] No active session found. Please run 'dalang login' or set LLM_API_KEY");
    llm::AuthToken::None
}

pub async fn interactive_model_selection(
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
        use dalang_adapters::adapters::outbound::llm::LlmProvider;
        if let Ok(fetched) = llm_provider.get_available_models().await {
            models = fetched;
        }
    }

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

pub async fn copilot_model_selection() {
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

pub fn get_fallback_models(provider: &str) -> Vec<String> {
    match provider {
        "gemini" | "google" => vec![
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
