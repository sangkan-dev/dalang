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

        /// Maximum iterations for auto-pilot mode (0 = unlimited, default: 15)
        #[arg(short = 'n', long = "max-iter", default_value_t = 15)]
        max_iter: u32,

        /// Command execution timeout in seconds (0 = unlimited, default: 300)
        #[arg(long = "cmd-timeout", default_value_t = 300)]
        cmd_timeout: u64,
    },

    /// Start an interactive / copilot mode session
    Interact {
        /// Target URL or IP for the interactive session
        #[arg(short, long)]
        target: String,

        /// Command execution timeout in seconds (0 = unlimited, default: 300)
        #[arg(long = "cmd-timeout", default_value_t = 300)]
        cmd_timeout: u64,
    },

    /// Switch the active AI model (no re-login required)
    Model {
        /// Set model directly (skip interactive picker)
        #[arg(short, long)]
        set: Option<String>,
    },
}
