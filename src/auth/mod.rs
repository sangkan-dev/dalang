pub mod cli_extractor;

pub mod persistence;

use anyhow::Result;

pub enum AuthProvider {
    Gemini,
    Anthropic,
    OpenAi,
}

impl AuthProvider {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "gemini" | "google" => Ok(AuthProvider::Gemini),
            "anthropic" | "claude" => Ok(AuthProvider::Anthropic),
            "openai" => Ok(AuthProvider::OpenAi),
            _ => Err(anyhow::anyhow!("Unsupported provider: {}", s)),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            AuthProvider::Gemini => "gemini",
            AuthProvider::Anthropic => "anthropic",
            AuthProvider::OpenAi => "openai",
        }
    }
}
