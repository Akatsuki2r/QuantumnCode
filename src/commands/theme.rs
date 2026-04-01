//! Theme commands

use color_eyre::eyre::Result;

use crate::cli::ThemeCommands;

/// Run theme command
pub async fn run(command: ThemeCommands) -> Result<()> {
    match command {
        ThemeCommands::List => list_themes(),
        ThemeCommands::Set { name } => set_theme(&name),
        ThemeCommands::Current => current_theme(),
        ThemeCommands::Preview { name } => preview_theme(&name),
    }
}

/// List available themes
fn list_themes() -> Result<()> {
    println!("Quantumn Code - Available Themes");
    println!();

    let themes = crate::config::Theme::list()?;

    println!("Built-in themes:");
    println!("  default      - Claude Code inspired purple theme");
    println!("  tokyo_night  - Tokyo Night (purple/blue accents)");
    println!("  hacker       - Matrix-style green on black");
    println!("  deep_black   - Minimal high contrast dark");
    println!();

    // Check for custom themes
    let custom: Vec<String> = themes.iter()
        .filter(|t| !["default", "tokyo_night", "hacker", "deep_black"].contains(&t.as_str()))
        .cloned()
        .collect();

    if !custom.is_empty() {
        println!("Custom themes:");
        for theme in custom {
            println!("  {}", theme);
        }
        println!();
    }

    println!("To set a theme:");
    println!("  quantumn theme set <name>");
    println!();
    println!("To preview a theme:");
    println!("  quantumn theme preview <name>");

    Ok(())
}

/// Set current theme
fn set_theme(name: &str) -> Result<()> {
    // Verify theme exists
    let theme = crate::config::Theme::load(name)?;

    // Update settings
    let mut settings = crate::config::Settings::load()?;
    settings.set("ui.theme", name)?;
    settings.save()?;

    println!("✓ Theme set to: {} ({})", theme.name, theme.description);

    Ok(())
}

/// Show current theme
fn current_theme() -> Result<()> {
    let settings = crate::config::Settings::load()?;
    let theme = crate::config::Theme::load(&settings.ui.theme)?;

    println!("Current theme: {} ({})", theme.name, theme.description);
    println!();
    println!("Colors:");
    println!("  Background: {}", theme.colors.background);
    println!("  Foreground: {}", theme.colors.foreground);
    println!("  Accent: {}", theme.colors.accent);
    println!("  Secondary: {}", theme.colors.secondary);
    println!("  Success: {}", theme.colors.success);
    println!("  Error: {}", theme.colors.error);
    println!("  Warning: {}", theme.colors.warning);
    println!("  Info: {}", theme.colors.info);

    Ok(())
}

/// Preview a theme
fn preview_theme(name: &str) -> Result<()> {
    let theme = crate::config::Theme::load(name)?;

    println!("Theme: {} ({})", theme.name, theme.description);
    println!("Author: {}", theme.author);
    println!();

    // Print color preview using ANSI escape codes
    println!("Color preview:");
    println!();

    // Background
    print!("\x1b[48;2;");
    let bg = crate::config::themes::RgbColor::from_hex(&theme.colors.background)?;
    print!("{};{};{}", bg.r, bg.g, bg.b);

    // Foreground
    print!("\x1b[38;2;");
    let fg = crate::config::themes::RgbColor::from_hex(&theme.colors.foreground)?;
    print!("{};{};{}m", fg.r, fg.g, fg.b);
    println!("  Background & Foreground  \x1b[0m");

    // Accent
    print!("\x1b[38;2;");
    let accent = crate::config::themes::RgbColor::from_hex(&theme.colors.accent)?;
    print!("{};{};{}m", accent.r, accent.g, accent.b);
    println!("  ■ Accent Color\x1b[0m");

    // Secondary
    print!("\x1b[38;2;");
    let secondary = crate::config::themes::RgbColor::from_hex(&theme.colors.secondary)?;
    print!("{};{};{}m", secondary.r, secondary.g, secondary.b);
    println!("  ■ Secondary Color\x1b[0m");

    // Success
    print!("\x1b[38;2;");
    let success = crate::config::themes::RgbColor::from_hex(&theme.colors.success)?;
    print!("{};{};{}m", success.r, success.g, success.b);
    println!("  ✓ Success Color\x1b[0m");

    // Error
    print!("\x1b[38;2;");
    let error = crate::config::themes::RgbColor::from_hex(&theme.colors.error)?;
    print!("{};{};{}m", error.r, error.g, error.b);
    println!("  ✗ Error Color\x1b[0m");

    // Warning
    print!("\x1b[38;2;");
    let warning = crate::config::themes::RgbColor::from_hex(&theme.colors.warning)?;
    print!("{};{};{}m", warning.r, warning.g, warning.b);
    println!("  ⚠ Warning Color\x1b[0m");

    // Info
    print!("\x1b[38;2;");
    let info = crate::config::themes::RgbColor::from_hex(&theme.colors.info)?;
    print!("{};{};{}m", info.r, info.g, info.b);
    println!("  ℹ Info Color\x1b[0m");

    println!();
    println!("To use this theme:");
    println!("  quantumn theme set {}", name);

    Ok(())
}