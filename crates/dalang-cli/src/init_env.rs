//! `dalang init` — skills directory + bundled skill files.

use anyhow::Result;
use dalang_adapters::adapters::outbound::skills_parser;
use std::path::Path;

pub fn run() -> Result<()> {
    println!("Initializing Dalang environment...");
    let skills_dir = Path::new("skills");
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
    Ok(())
}
