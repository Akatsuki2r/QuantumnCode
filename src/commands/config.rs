//! Configuration commands

use color_eyre::eyre::Result;

use crate::cli::ConfigCommands;

/// Run config command
pub async fn run(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Show => show_config(),
        ConfigCommands::Set { key, value } => set_config(&key, &value),
        ConfigCommands::Get { key } => get_config(&key),
        ConfigCommands::Reset => reset_config(),
        ConfigCommands::Edit => edit_config(),
    }
}

/// Show current configuration
fn show_config() -> Result<()> {
    println!("Quantumn Code - Configuration");
    println!();

    let settings = crate::config::Settings::load()?;

    println!("Model:");
    println!("  provider: {}", settings.model.provider);
    println!("  default_model: {}", settings.model.default_model);
    println!("  api_key_env: {}", settings.model.api_key_env);
    println!();

    println!("Git:");
    println!("  commit_format: {}", settings.git.commit_format);
    println!("  include_coauthors: {}", settings.git.include_coauthors);
    println!("  conventional_commits: {}", settings.git.conventional_commits);
    println!();

    println!("Editor:");
    println!("  tab_width: {}", settings.editor.tab_width);
    println!("  use_spaces: {}", settings.editor.use_spaces);
    println!("  line_numbers: {}", settings.editor.line_numbers);
    println!("  auto_save: {:?}", settings.editor.auto_save);
    println!();

    println!("UI:");
    println!("  theme: {}", settings.ui.theme);
    println!("  show_file_tree: {}", settings.ui.show_file_tree);
    println!("  show_token_count: {}", settings.ui.show_token_count);
    println!("  show_cost: {}", settings.ui.show_cost);
    println!("  animation_speed: {}", settings.ui.animation_speed);
    println!();

    println!("Config file: {:?}", crate::config::Settings::config_path()?);

    Ok(())
}

/// Set a configuration value
fn set_config(key: &str, value: &str) -> Result<()> {
    let mut settings = crate::config::Settings::load()?;
    settings.set(key, value)?;
    settings.save()?;

    println!("✓ Set {} = {}", key, value);

    Ok(())
}

/// Get a configuration value
fn get_config(key: &str) -> Result<()> {
    let settings = crate::config::Settings::load()?;

    match settings.get(key) {
        Some(value) => println!("{}", value),
        None => println!("Key not found: {}", key),
    }

    Ok(())
}

/// Reset configuration to defaults
fn reset_config() -> Result<()> {
    let settings = crate::config::Settings::default();
    settings.save()?;

    println!("✓ Configuration reset to defaults.");

    Ok(())
}

/// Open configuration file in editor
fn edit_config() -> Result<()> {
    let config_path = crate::config::Settings::config_path()?;

    // Create the file if it doesn't exist
    if !config_path.exists() {
        crate::config::Settings::default().save()?;
    }

    println!("Config file location: {:?}", config_path);
    println!("Open this file in your preferred editor.");

    // Try to open in editor
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "nano".to_string());

    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("✓ Configuration saved.");
        }
        _ => {
            println!("Could not open editor. Please edit the file manually.");
            println!("  {:?} {:?}", config_path);
        }
    }

    Ok(())
}