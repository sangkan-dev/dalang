use anyhow::{Result, anyhow};
use std::process::Command;

/// Mencoba mengekstrak token dari Google Cloud SDK jika terinstal dan sudah login.
pub fn extract_gcloud_token() -> Result<String> {
    let output = Command::new("gcloud")
        .arg("auth")
        .arg("print-access-token")
        .output()?;

    if output.status.success() {
        let token = String::from_utf8(output.stdout)?.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    Err(anyhow!("Gcloud token not found or not logged in"))
}

/// Extract token from Gemini CLI credentials (checks multiple known locations).
pub fn extract_gemini_cli_token() -> Result<String> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;

    // Known credential file locations for gemini / gemini-cli
    let candidates = [
        home.join(".gemini/oauth_creds.json"), // Gemini CLI (google-gemini/gemini-cli)
        home.join(".gemini/credentials.json"), // Alternative location
    ];

    for path in &candidates {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            if let Some(token) = json.get("access_token").and_then(|v| v.as_str())
                && !token.is_empty()
            {
                println!("[+] Found credentials at: {}", path.display());
                return Ok(token.to_string());
            }
        }
    }

    let checked: Vec<String> = candidates.iter().map(|p| p.display().to_string()).collect();
    Err(anyhow!(
        "Gemini CLI token not found. Checked:\n    - {}",
        checked.join("\n    - ")
    ))
}

/// Fungsi wrapper untuk mencoba semua kemungkinan CLI extraction.
pub fn try_all_cli_extractors() -> Option<String> {
    if let Ok(token) = extract_gcloud_token() {
        println!("[+] Found active session from gcloud");
        return Some(token);
    }

    if let Ok(token) = extract_gemini_cli_token() {
        println!("[+] Found active session from gemini-cli");
        return Some(token);
    }

    None
}

// Note: Kita butuh crate 'dirs' untuk home_dir yang portable,
// tapi untuk linux hasanh47 kita bisa asumsikan home dir.
// Mari tambahkan 'dirs' ke Cargo.toml nanti jika diperlukan.
// Untuk sekarang saya akan koreksi Cargo.toml jika lupa.
mod dirs {
    use std::path::PathBuf;
    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}
