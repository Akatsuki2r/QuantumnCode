//! Code review command implementation

use std::path::PathBuf;
use color_eyre::eyre::Result;

/// Run code review with AI
pub async fn run(files: Vec<PathBuf>, model: Option<String>) -> Result<()> {
    // Load settings
    let settings = crate::config::Settings::load()?;
    let model_name = model.unwrap_or(settings.model.default_model);

    println!("Quantumn Code - Code Review");
    println!("Model: {}", model_name);
    println!();

    if files.is_empty() {
        // Review staged changes
        println!("No files specified. Reviewing staged changes...");
        let output = std::process::Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let staged_files = String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(PathBuf::from)
                    .collect::<Vec<_>>();

                if staged_files.is_empty() {
                    println!("No staged changes to review.");
                    return Ok(());
                }

                println!("Files to review:");
                for f in &staged_files {
                    println!("  - {:?}", f);
                }
                println!();

                for file in staged_files {
                    review_file(&file)?;
                }
            }
            _ => {
                println!("Not in a git repository or no staged changes.");
            }
        }
    } else {
        println!("Files to review:");
        for f in &files {
            println!("  - {:?}", f);
        }
        println!();

        for file in files {
            review_file(&file)?;
        }
    }

    // TODO: Send to AI for review
    println!("\nAI code review will be implemented in Phase 5.");
    println!("Review complete.");

    Ok(())
}

/// Review a single file
fn review_file(file: &PathBuf) -> Result<()> {
    if file.exists() {
        let content = std::fs::read_to_string(file)?;
        println!("Reviewing: {:?}", file);
        println!("  Size: {} bytes", content.len());
        println!("  Lines: {}", content.lines().count());
    } else {
        println!("File not found: {:?}", file);
    }
    Ok(())
}