use clap::{Parser, Subcommand};

/// Dalang - AI Agent Cybersecurity Framework
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct DalangArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the Dalang environment
    Init,

    /// Login to an LLM provider (Google/Gemini, Anthropic, OpenAI)
    Login {
        /// Provider name (gemini, anthropic, openai)
        #[arg(short, long)]
        provider: String,

        /// Use OAuth flow instead of API Key (Gemini/Google only)
        #[arg(long, default_value_t = false)]
        oauth: bool,
    },

    /// Run an automated scan against a target
    Scan {
        /// Target URL or IP to scan
        #[arg(short, long)]
        target: String,

        /// Comma-separated list of skills to execute (e.g., web-basic,nmap-port)
        #[arg(short, long)]
        skills: Option<String>,

        /// Enable Autonomous Auto-Pilot mode (ignores --skills)
        #[arg(short, long, default_value_t = false)]
        auto: bool,
    },

    /// Start an interactive / copilot mode session
    Interact {
        /// Target URL or IP for the interactive session
        #[arg(short, long)]
        target: String,
    },
}
