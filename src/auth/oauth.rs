use anyhow::{Result, anyhow};
use tiny_http::{Response, Server};
use url::Url;

pub struct OauthConfig {
    pub client_id: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl OauthConfig {
    pub fn gemini_default() -> Self {
        Self {
            client_id: "681255809395-oo8ft2oprdrnp9e3aqf6av3hmdib135j.apps.googleusercontent.com"
                .to_string(), // Public client ID from logs
            auth_uri: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_uri: "https://oauth2.googleapis.com/token".to_string(),
            redirect_uri: "http://127.0.0.1:38343/oauth2callback".to_string(),
            scopes: vec![
                "https://www.googleapis.com/auth/cloud-platform".to_string(),
                "https://www.googleapis.com/auth/userinfo.email".to_string(),
                "https://www.googleapis.com/auth/userinfo.profile".to_string(),
            ],
        }
    }
}

pub fn build_auth_url(config: &OauthConfig) -> String {
    let mut url = Url::parse(&config.auth_uri).unwrap();
    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", &config.redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", &config.scopes.join(" "))
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent");
    url.to_string()
}

pub fn run_callback_server(port: u16) -> Result<String> {
    let server = Server::http(format!("127.0.0.1:{}", port))
        .map_err(|e| anyhow!("Failed to start server: {}", e))?;

    println!(
        "[*] Waiting for OAuth callback on http://127.0.0.1:{}/oauth2callback",
        port
    );

    for request in server.incoming_requests() {
        let url = format!("http://localhost{}", request.url());
        let parsed_url = Url::parse(&url)?;

        if let Some((_, code)) = parsed_url.query_pairs().find(|(key, _)| key == "code") {
            let response = Response::from_string(
                "Authentication complete! You can close this tab and return to Dalang.",
            );
            request.respond(response)?;
            return Ok(code.to_string());
        }

        let response = Response::from_string("Waiting for code...");
        request.respond(response)?;
    }

    Err(anyhow!("Server stopped without receiving code"))
}

pub async fn perform_token_exchange(config: &OauthConfig, code: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let params: [(&str, &str); 4] = [
        ("client_id", config.client_id.as_str()),
        ("code", code),
        ("redirect_uri", config.redirect_uri.as_str()),
        ("grant_type", "authorization_code"),
    ];

    let res = client.post(&config.token_uri).form(&params).send().await?;

    if res.status().is_success() {
        Ok(res.json().await?)
    } else {
        let err = res.text().await?;
        Err(anyhow!("Failed to exchange token: {}", err))
    }
}
