//! Test runner command implementation

use std::path::PathBuf;
use color_eyre::eyre::Result;

/// Run tests with AI analysis
pub async fn run(path: Option<PathBuf>, model: Option<String>) -> Result<()> {
    // Load settings
    let settings = crate::config::Settings::load()?;
    let model_name = model.unwrap_or(settings.model.default_model);

    println!("Quantumn Code - Test Runner");
    println!("Model: {}", model_name);
    println!();

    // Detect project type and test command
    let test_command = detect_test_command()?;

    println!("Detected test command: {} {}", test_command.0, test_command.1.join(" "));
    println!();

    // Run tests
    println!("Running tests...");
    let output = std::process::Command::new(&test_command.0)
        .args(&test_command.1)
        .current_dir(path.unwrap_or_else(|| std::env::current_dir().unwrap()))
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            println!("{}", stdout);
            if !stderr.is_empty() {
                println!("Errors:\n{}", stderr);
            }

            if output.status.success() {
                println!("\n✓ All tests passed!");
            } else {
                println!("\n✗ Some tests failed.");
                println!("\nAnalyzing failures with AI...");
                // TODO: Send failures to AI
                println!("AI failure analysis will be implemented in Phase 5.");
            }
        }
        Err(e) => {
            println!("Failed to run tests: {}", e);
            println!("Make sure you have the appropriate test runner installed.");
        }
    }

    Ok(())
}

/// Detect the project type and appropriate test command
fn detect_test_command() -> Result<(String, Vec<String>)> {
    // Check for various project types
    if std::path::Path::new("Cargo.toml").exists() {
        return Ok(("cargo".to_string(), vec!["test".to_string()]));
    }

    if std::path::Path::new("package.json").exists() {
        return Ok(("npm".to_string(), vec!["test".to_string()]));
    }

    if std::path::Path::new("go.mod").exists() {
        return Ok(("go".to_string(), vec!["test".to_string(), "./...".to_string()]));
    }

    if std::path::Path::new("pytest.ini").exists() || std::path::Path::new("setup.py").exists() {
        return Ok(("pytest".to_string(), vec![]));
    }

    // Default to cargo test if nothing else found
    Ok(("cargo".to_string(), vec!["test".to_string()]))
}