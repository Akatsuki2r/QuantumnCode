//! Model/provider commands

use color_eyre::eyre::Result;

/// Run model command
pub async fn run(provider: Option<String>, list: bool) -> Result<()> {
    if list {
        list_models(provider)?;
    } else if let Some(p) = provider {
        set_provider(&p)?;
    } else {
        show_current_provider()?;
    }

    Ok(())
}

/// List available models
fn list_models(provider: Option<String>) -> Result<()> {
    println!("Quantumn Code - Available Models");
    println!();

    match provider.as_deref() {
        Some("anthropic") => list_anthropic_models(),
        Some("openai") => list_openai_models(),
        Some("ollama") => list_ollama_models(),
        None => {
            println!("Anthropic (Claude):");
            list_anthropic_models();
            println!();

            println!("OpenAI:");
            list_openai_models();
            println!();

            println!("Ollama (local):");
            list_ollama_models();
        }
        Some(p) => {
            println!("Unknown provider: {}", p);
            println!("Available providers: anthropic, openai, ollama");
        }
    }

    println!();
    println!("To set a model:");
    println!("  quantumn --model <model_name>");

    Ok(())
}

fn list_anthropic_models() {
    println!("  claude-opus-4-20250514   - Most capable (Opus 4)");
    println!("  claude-sonnet-4-20250514 - Balanced (Sonnet 4) [default]");
    println!("  claude-haiku-4-20250514  - Fast (Haiku 4)");
    println!("  claude-3-5-sonnet-20241022 - Legacy (Sonnet 3.5)");
    println!("  claude-3-5-haiku-20241022  - Legacy (Haiku 3.5)");
}

fn list_openai_models() {
    println!("  gpt-4o       - GPT-4 Omni (recommended)");
    println!("  gpt-4o-mini  - GPT-4 Omni Mini (fast, cheap)");
    println!("  gpt-4-turbo  - GPT-4 Turbo");
    println!("  o1           - O1 (advanced reasoning)");
    println!("  o1-mini      - O1 Mini");
}

fn list_ollama_models() {
    println!("  llama3.2       - Meta Llama 3.2");
    println!("  llama3.1       - Meta Llama 3.1");
    println!("  mistral        - Mistral");
    println!("  codellama      - Code Llama");
    println!("  deepseek-coder - DeepSeek Coder");
    println!("  qwen2.5-coder  - Qwen 2.5 Coder");
    println!();
    println!("  Run 'ollama list' to see installed models.");
}

/// Set current provider
fn set_provider(provider: &str) -> Result<()> {
    let mut settings = crate::config::Settings::load()?;

    let (provider_name, default_model) = match provider {
        "anthropic" => ("anthropic", "claude-sonnet-4-20250514"),
        "openai" => ("openai", "gpt-4o"),
        "ollama" => ("ollama", "llama3.2"),
        _ => {
            println!("Unknown provider: {}", provider);
            println!("Available providers: anthropic, openai, ollama");
            return Ok(());
        }
    };

    settings.model.provider = provider_name.to_string();
    settings.model.default_model = default_model.to_string();
    settings.save()?;

    println!("✓ Provider set to: {}", provider);
    println!("  Default model: {}", default_model);

    Ok(())
}

/// Show current provider and model
fn show_current_provider() -> Result<()> {
    let settings = crate::config::Settings::load()?;

    println!("Current provider: {}", settings.model.provider);
    println!("Current model: {}", settings.model.default_model);
    println!();

    // Check if API key is set
    let api_key_env = &settings.model.api_key_env;
    let has_key = std::env::var(api_key_env).is_ok();

    if has_key {
        println!("API key: ✓ Set ({})", api_key_env);
    } else {
        println!("API key: ✗ Not set");
        println!();
        println!("To set your API key:");
        println!("  export {}=your-api-key", api_key_env);
    }

    Ok(())
}