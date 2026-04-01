//! Git commands implementation

use color_eyre::eyre::Result;

/// Generate commit message with AI
pub async fn commit(message: Option<String>, model: Option<String>) -> Result<()> {
    // Load settings
    let settings = crate::config::Settings::load()?;
    let model_name = model.unwrap_or(settings.model.default_model);

    println!("Quantumn Code - Git Commit");
    println!("Model: {}", model_name);
    println!();

    // Check if we're in a git repo
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            // Get git status
            let status = get_git_status()?;
            println!("Git Status:");
            println!("{}", status);
            println!();

            // Get diff
            let diff = get_git_diff()?;
            if diff.is_empty() {
                println!("No changes to commit.");
                return Ok(());
            }

            println!("Changes:");
            println!("{}", "-".repeat(40));
            println!("{}", diff.lines().take(50).collect::<Vec<_>>().join("\n"));
            if diff.lines().count() > 50 {
                println!("... ({} more lines)", diff.lines().count() - 50);
            }
            println!("{}", "-".repeat(40));
            println!();

            match message {
                Some(m) => {
                    println!("Using custom message: {}", m);
                    // TODO: Generate commit with AI
                    println!("\nCommit message: {}", m);
                    println!("\nTo commit, run:");
                    println!("  git commit -m \"{}\"", m);
                }
                None => {
                    println!("Generating commit message with AI...");
                    // TODO: Send diff to AI
                    println!("AI commit generation will be implemented in Phase 4.");
                    println!("\nSuggested format:");
                    println!("  type(scope): description");
                    println!("\nTypes: feat, fix, docs, style, refactor, test, chore");
                }
            }
        }
        _ => {
            println!("Not in a git repository.");
        }
    }

    Ok(())
}

/// Get git status
fn get_git_status() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["status", "--short"])
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get git diff
fn get_git_diff() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--cached", "--stat"])
        .output()?;

    let cached = String::from_utf8_lossy(&output.stdout).to_string();

    let output2 = std::process::Command::new("git")
        .args(["diff", "--stat"])
        .output()?;

    let unstaged = String::from_utf8_lossy(&output2.stdout).to_string();

    Ok(format!("Staged:\n{}\n\nUnstaged:\n{}", cached, unstaged))
}

/// Get git log
pub fn get_git_log(count: usize) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["log", &format!("-{}", count), "--oneline"])
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}