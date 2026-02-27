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
    // Parse command line arguments
    let args = DalangArgs::parse();

    match args.command {
        Commands::Init => {
            println!("Initializing Dalang environment...");
            // TODO: Implement init logic
        }
        Commands::Scan { target, skills } => {
            println!("Starting automated scan...");
            println!("Target: {}", target);
            println!("Skills: {}", skills);

            // Dummy initialization for now: in reality we'd load this from config
            let base_url = std::env::var("LLM_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434/v1".to_string());
            let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "llama3.2".to_string());
            let auth = llm::AuthToken::None; // Or load from env

            let provider = llm::openai::OpenAiCompatibleProvider::new(base_url, model, auth)?;
            let engine = core::engine::DalangEngine::new(Box::new(provider));

            engine.run_scan_loop(&target, &skills).await?;
        }
        Commands::Interact { target } => {
            println!("Starting interactive session...");
            println!("Target: {}", target);
            // TODO: Implement interactive logic
        }
    }

    Ok(())
}
