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

    /// Run an automated scan against a target
    Scan {
        /// Target URL or IP to scan
        #[arg(short, long)]
        target: String,

        /// Comma-separated list of skills to execute (e.g., web-basic,nmap-port)
        #[arg(short, long)]
        skills: String,
    },

    /// Start an interactive / copilot mode session
    Interact {
        /// Target URL or IP for the interactive session
        #[arg(short, long)]
        target: String,
    },
}
