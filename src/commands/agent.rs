//! Agent command - Bear mode for agentic workflow

use color_eyre::eyre::Result;

use crate::config::Settings;
use crate::commands::chat::get_provider;

/// Run an agentic task
pub async fn run(task: String, model: Option<String>) -> Result<()> {
    let settings = Settings::load()?;
    let model_name = model.unwrap_or_else(|| settings.model.default_model.clone());

    println!("Quantumn Code - Agent Mode (Bear)");
    println!("Task: {}", task);
    println!("Model: {}", model_name);
    println!();
    println!("Running agentic workflow...");
    println!("(Use Read, Write, Bash, Grep, Glob tools as needed)");
    println!("{}", "─".repeat(50));
    println!();

    let provider = get_provider(&model_name, &settings);

    if !provider.is_configured() {
        println!("Provider not configured. Please set up your API key.");
        return Ok(());
    }

    let result = crate::agent::run_agentic(&task, provider.as_ref()).await?;

    println!("{}", "─".repeat(50));
    println!();
    println!("Agent finished.");

    Ok(())
}
