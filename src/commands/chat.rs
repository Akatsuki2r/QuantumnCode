//! Chat command implementation

use color_eyre::eyre::Result;

/// Run chat mode (one-shot or interactive)
pub async fn run(prompt: Option<String>, model: Option<String>) -> Result<()> {
    match prompt {
        Some(p) => {
            // One-shot mode
            run_one_shot(&p, model).await
        }
        None => {
            // Interactive mode
            crate::tui::run_interactive(model, None).await
        }
    }
}

/// Run one-shot query
async fn run_one_shot(prompt: &str, model: Option<String>) -> Result<()> {
    // Load settings
    let settings = crate::config::Settings::load()?;
    let model_name = model.unwrap_or(settings.model.default_model);

    println!("Quantumn Code - Using model: {}", model_name);
    println!("Prompt: {}", prompt);
    println!();

    // Determine provider
    let provider_name = if model_name.starts_with("gpt") || model_name.starts_with("o1") {
        "openai"
    } else if model_name.starts_with("llama") || model_name.starts_with("mistral") || model_name.starts_with("deepseek") {
        "ollama"
    } else {
        "anthropic"
    };

    println!("Provider: {}", provider_name);
    println!();

    // TODO: Actually send to AI
    println!("AI response will be implemented in Phase 2.");
    println!("For now, here's your prompt echoed: {}", prompt);

    Ok(())
}