//! Status command implementation

use color_eyre::eyre::Result;

/// Show current status
pub async fn run() -> Result<()> {
    println!("Quantumn Code - Status");
    println!("{}", "=".repeat(40));
    println!();

    // Version
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();

    // Configuration
    let settings = crate::config::Settings::load()?;
    println!("Configuration:");
    println!("  Provider: {}", settings.model.provider);
    println!("  Model: {}", settings.model.default_model);
    println!("  Theme: {}", settings.ui.theme);
    println!();

    // API keys
    println!("API Keys:");
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY").ok();
    let openai_key = std::env::var("OPENAI_API_KEY").ok();

    if let Some(_) = anthropic_key {
        println!("  ANTHROPIC_API_KEY: ✓ Set");
    } else {
        println!("  ANTHROPIC_API_KEY: ✗ Not set");
    }

    if let Some(_) = openai_key {
        println!("  OPENAI_API_KEY: ✓ Set");
    } else {
        println!("  OPENAI_API_KEY: ✗ Not set");
    }
    println!();

    // Git status
    println!("Git Status:");
    let in_git = std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if in_git {
        let branch = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let status = std::process::Command::new("git")
            .args(["status", "--short"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.lines().count())
            .unwrap_or(0);

        println!("  Branch: {}", branch);
        println!("  Changes: {} files", status);
    } else {
        println!("  Not in a git repository");
    }
    println!();

    // Sessions
    let sessions_dir = get_sessions_dir()?;
    let session_count = if sessions_dir.exists() {
        std::fs::read_dir(&sessions_dir)
            .map(|d| d.filter(|e| e.as_ref().ok().map(|e| e.path().extension().map(|e| e == "json").unwrap_or(false)).unwrap_or(false)).count())
            .unwrap_or(0)
    } else {
        0
    };

    println!("Sessions: {} saved", session_count);
    println!();

    // Config file location
    println!("Config file: {:?}", crate::config::Settings::config_path()?);

    Ok(())
}

fn get_sessions_dir() -> Result<std::path::PathBuf> {
    let config_dir = directories::ProjectDirs::from("com", "quantumn", "code")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from(".quantumn"));

    Ok(config_dir.join("sessions"))
}