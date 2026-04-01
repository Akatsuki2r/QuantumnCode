//! Shell command execution tool

use std::process::{Command, Output};
use color_eyre::eyre::Result;

/// Run a shell command
pub fn run_command(cmd: &str, args: &[&str]) -> Result<Output> {
    Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to run command {}: {}", cmd, e))
}

/// Run command in directory
pub fn run_command_in_dir(cmd: &str, args: &[&str], dir: &std::path::Path) -> Result<Output> {
    Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to run command {}: {}", cmd, e))
}

/// Run command and get output as string
pub fn run_command_string(cmd: &str, args: &[&str]) -> Result<String> {
    let output = run_command(cmd, args)?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Check if command exists
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}