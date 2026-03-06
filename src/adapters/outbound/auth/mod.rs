pub mod cli_extractor;
pub mod copilot;
pub mod gemini_codeassist;

pub mod persistence;

use anyhow::Result;

pub enum AuthProvider {
    Gemini,
    Anthropic,
    OpenAi,
    Copilot,
}

impl AuthProvider {
    pub fn from_name(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "gemini" | "google" => Ok(AuthProvider::Gemini),
            "anthropic" | "claude" => Ok(AuthProvider::Anthropic),
            "openai" => Ok(AuthProvider::OpenAi),
            "copilot" | "github" | "github-copilot" => Ok(AuthProvider::Copilot),
            _ => Err(anyhow::anyhow!("Unsupported provider: {}", s)),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            AuthProvider::Gemini => "gemini",
            AuthProvider::Anthropic => "anthropic",
            AuthProvider::OpenAi => "openai",
            AuthProvider::Copilot => "copilot",
        }
    }
}
