//! Chat command implementation

use crate::config::Settings;
use crate::prompts::{get_full_prompt, Mode};
use crate::providers::anthropic::AnthropicProvider;
use crate::providers::llama_cpp::LlamaCppProvider;
use crate::providers::lm_studio::LmStudioProvider;
use crate::providers::ollama::OllamaProvider;
use crate::providers::openai::OpenAIProvider;
use crate::providers::{Message, Provider, Role};
use crate::supervisor::ModelSupervisor;
use color_eyre::eyre::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Run chat mode (one-shot or interactive)
pub async fn run(prompt: Option<String>, model: Option<String>, mode: Mode) -> Result<()> {
    match prompt {
        Some(p) => {
            // One-shot mode
            run_one_shot(&p, model, mode).await
        }
        None => {
            // Interactive mode
            crate::tui::run_interactive(model, None).await
        }
    }
}

/// Get the appropriate provider based on settings and model name
pub fn get_provider(model_name: &str, settings: &Settings) -> Box<dyn Provider> {
    // Check if explicitly set to llama.cpp
    if settings.model.provider == "llama_cpp" {
        let supervisor = Arc::new(Mutex::new(ModelSupervisor::new()));
        let mut provider = LlamaCppProvider::new(supervisor.clone());
        provider.set_model(model_name.to_string());
        // Load model paths from settings
        for (name, path) in &settings.llama_cpp.model_paths {
            provider.add_model_path(name.clone(), PathBuf::from(path));
        }
        return Box::new(provider);
    }

    // Auto-detect based on model name
    let provider_name = if model_name.starts_with("gpt") || model_name.starts_with("o1") {
        "openai"
    } else if model_name.starts_with("claude") {
        "anthropic"
    } else {
        // Default to Ollama for local models
        "ollama"
    };

    match provider_name {
        "anthropic" => {
            let mut provider = AnthropicProvider::new();
            provider.set_model(model_name.to_string());
            Box::new(provider)
        }
        "openai" => {
            let mut provider = OpenAIProvider::new();
            provider.set_model(model_name.to_string());
            Box::new(provider)
        }
        "llama_cpp" => {
            let supervisor = Arc::new(Mutex::new(ModelSupervisor::new()));
            let mut provider = LlamaCppProvider::new(supervisor);
            provider.set_model(model_name.to_string());
            // Load model paths from settings
            for (name, path) in &settings.llama_cpp.model_paths {
                provider.add_model_path(name.clone(), PathBuf::from(path));
            }
            Box::new(provider)
        }
        "lm_studio" => {
            let mut provider = LmStudioProvider::new();
            provider.set_model(model_name.to_string());
            Box::new(provider)
        }
        _ => {
            let mut provider = OllamaProvider::new();
            provider.set_model(model_name.to_string());
            Box::new(provider)
        }
    }
}

/// Check if local providers are running and prompt user if needed
pub fn check_local_provider(provider_name: &str) -> Result<bool> {
    match provider_name {
        "ollama" => {
            let ollama = OllamaProvider::new();
            let is_running = futures::executor::block_on(ollama.is_running());
            if !is_running {
                println!("⚠ Ollama is not running!");
                println!("  To fix: ollama serve");
                println!("  Then: ollama pull <model_name>");
                return Ok(false);
            }
            println!("✓ Ollama is running");
        }
        "lm_studio" => {
            let lm_studio = LmStudioProvider::new();
            let is_available = futures::executor::block_on(lm_studio.is_available());
            if !is_available {
                println!("⚠ LM Studio is not available!");
                println!("  Options:");
                println!("  1. Start LM Studio application");
                println!("  2. Run: lms server start");
                println!("  3. Or download GGUF models to ~/.lmstudio/models/");
                return Ok(false);
            }
            println!("✓ LM Studio is available");
        }
        "llama_cpp" => {
            println!("⚠ llama.cpp requires manual setup:");
            println!("  1. llama-server binary in PATH");
            println!("  2. GGUF model files in config");
        }
        _ => {}
    }
    Ok(true)
}

/// Run one-shot query
async fn run_one_shot(prompt: &str, model: Option<String>, mode: Mode) -> Result<()> {
    // Load settings
    let settings = Settings::load()?;
    let model_name = model.unwrap_or_else(|| settings.model.default_model.clone());
    let provider_name = &settings.model.provider;

    // Check local providers before attempting chat
    if let Err(_) = check_local_provider(provider_name) {
        println!("\nPlease start the local provider and try again.");
        return Ok(());
    }

    println!("Quantumn Code - Using model: {}", model_name);
    println!("Provider: {}", provider_name);
    println!("Mode: {}", mode);
    println!("Prompt: {}", prompt);
    println!();

    // Get provider
    let provider = get_provider(&model_name, &settings);

    // Check if configured
    if !provider.is_configured() {
        println!("Provider not configured. Please set up your API key.");
        return Ok(());
    }

    // Get system prompt for mode
    let system_prompt = get_full_prompt(mode);

    // Create user message
    let user_message = Message {
        role: Role::User,
        content: prompt.to_string(),
        name: None,
    };

    // Send request with system prompt
    match provider
        .send_with_system(vec![user_message.clone()], Some(&system_prompt))
        .await
    {
        Ok(response) => {
            println!("AI Response:");
            println!("{}", response);
        }
        Err(e) => {
            eprintln!("Error: {}", e);

            // Try fallback to Ollama if llama.cpp failed
            if settings.model.provider == "llama_cpp" && settings.llama_cpp.fallback_to_ollama {
                println!("\nFalling back to Ollama...");
                let ollama_provider = OllamaProvider::with_model(model_name);
                match ollama_provider.send(vec![user_message]).await {
                    Ok(response) => {
                        println!("Ollama Response:");
                        println!("{}", response);
                    }
                    Err(e) => {
                        eprintln!("Ollama fallback error: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
