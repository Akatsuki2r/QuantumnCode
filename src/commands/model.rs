//! Model/provider commands

use color_eyre::eyre::Result;
use std::path::PathBuf;

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

/// Scan LM Studio's model directory for downloaded GGUF models
fn scan_lm_studio_models() -> Vec<(String, PathBuf)> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let models_dir = PathBuf::from(format!("{}/.lmstudio/models", home));
    let mut models = Vec::new();

    if models_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&models_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if let Ok(sub) = std::fs::read_dir(entry.path()) {
                        for sub_entry in sub.flatten() {
                            if sub_entry
                                .path()
                                .extension()
                                .map(|e| e == "gguf")
                                .unwrap_or(false)
                            {
                                let name = sub_entry.file_name().to_string_lossy().to_string();
                                models.push((name, sub_entry.path()));
                            }
                        }
                    }
                }
            }
        }
    }
    models
}

/// List available models
fn list_models(provider: Option<String>) -> Result<()> {
    match provider.as_deref() {
        Some("anthropic") => list_anthropic_models(),
        Some("openai") => list_openai_models(),
        Some("ollama") => list_ollama_models(),
        Some("llama_cpp") => list_llama_cpp_models(),
        Some("lm_studio") => list_lm_studio_models(),
        None => {
            println!("╔════════════════════════════════════════════════════════════════╗");
            println!("║ CLOUD MODELS                                                   ║");
            println!("╠════════════════════════════════════════════════════════════════╣");
            println!("║ ANTHROPIC (Claude)                                            ║");
            list_anthropic_models();
            println!("\n║ OPENAI                                                        ║");
            list_openai_models();

            println!("\n╔════════════════════════════════════════════════════════════════╗");
            println!("║ DOWNLOADED LOCAL MODELS                                       ║");
            println!("╠════════════════════════════════════════════════════════════════╣");
            println!("║ LM STUDIO (~/.lmstudio/models/)                              ║");
            let local_models = scan_lm_studio_models();
            if !local_models.is_empty() {
                for (name, path) in &local_models {
                    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                    let size_mb = size as f64 / (1024.0 * 1024.0);
                    let display_name = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| name.clone());
                    println!("║   {} ({:.1} MB)", display_name, size_mb);
                }
            } else {
                println!("║   No GGUF models found");
            }

            println!("\n║ OLLAMA                                                        ║");
            list_ollama_models();

            println!("\n║ LLAMA.CPP                                                    ║");
            list_llama_cpp_models();

            println!();
            println!("To set a provider: quantumn model <provider_name>");
        }
        Some(p) => {
            println!("Unknown provider: {}", p);
            println!("Available: anthropic, openai, ollama, llama_cpp, lm_studio");
        }
    }

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
    // Note: Auto-detection of Ollama models requires async runtime
    // Falling back to common models list
    // To see installed models, run: ollama list
    println!("  llama3.2       - Meta Llama 3.2");
    println!("  llama3.1       - Meta Llama 3.1");
    println!("  mistral        - Mistral");
    println!("  codellama      - Code Llama");
    println!("  deepseek-coder - DeepSeek Coder");
    println!("  qwen2.5-coder  - Qwen 2.5 Coder");
    println!();
    println!("  Note: Run 'ollama list' to see installed models.");
    println!("  Or use 'quantumn agent --auto-detect' to auto-detect local LLMs");
}

fn list_llama_cpp_models() {
    println!("  llama3.2      - Meta Llama 3.2 (GGUF)");
    println!("  llama3.1       - Meta Llama 3.1 (GGUF)");
    println!("  mistral        - Mistral (GGUF)");
    println!("  qwen2.5        - Qwen 2.5 (GGUF)");
    println!("  deepseek-coder - DeepSeek Coder (GGUF)");
    println!();
    println!("  Requires llama-server binary and GGUF model files.");
    println!("  Configure model paths in config.toml under [llama_cpp.model_paths]");
}

fn list_lm_studio_models() {
    println!("  llama3.2       - Meta Llama 3.2 (GGUF)");
    println!("  llama3.1        - Meta Llama 3.1 (GGUF)");
    println!("  mistral         - Mistral (GGUF)");
    println!("  qwen2.5         - Qwen 2.5 (GGUF)");
    println!("  granite-3.0-2b  - IBM Granite 3.0 2B (GGUF)");
    println!();
    println!("  LM Studio manages models directly.");
    println!("  Start LM Studio server: lms server start");
    println!("  Models are auto-discovered from LM Studio library.");
}

/// Set current provider
fn set_provider(provider: &str) -> Result<()> {
    let mut settings = crate::config::Settings::load()?;

    let (provider_name, default_model) = match provider {
        "anthropic" => ("anthropic", "claude-sonnet-4-20250514"),
        "openai" => ("openai", "gpt-4o"),
        "ollama" => ("ollama", "llama3.2"),
        "llama_cpp" => ("llama_cpp", "llama3.2"),
        "lm_studio" => ("lm_studio", "llama3.2"),
        _ => {
            println!("Unknown provider: {}", provider);
            println!("Available providers: anthropic, openai, ollama, llama_cpp, lm_studio");
            return Ok(());
        }
    };

    settings.model.provider = provider_name.to_string();
    settings.model.default_model = default_model.to_string();
    settings.save()?;

    println!("✓ Provider set to: {}", provider);
    println!("  Default model: {}", default_model);

    if provider == "llama_cpp" {
        println!();
        println!("Note: llama.cpp requires:");
        println!("  1. llama-server binary in PATH");
        println!("  2. GGUF model files configured in config.toml");
        println!("  3. Or use Ollama as fallback (enabled by default)");
    } else if provider == "lm_studio" {
        println!();
        println!("Note: LM Studio requires:");
        println!("  1. LM Studio application running");
        println!("  2. lms server start (or enable server in LM Studio GUI)");
        println!("  3. Models downloaded in LM Studio library");
    }

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

    // Show llama.cpp config if relevant
    if settings.model.provider == "llama_cpp" {
        println!();
        println!("llama.cpp configuration:");
        println!("  Enabled: {}", settings.llama_cpp.enabled);
        println!("  Port: {}", settings.llama_cpp.default_port);
        println!(
            "  Fallback to Ollama: {}",
            settings.llama_cpp.fallback_to_ollama
        );
        println!("  Auto-start: {}", settings.llama_cpp.auto_start);
        if !settings.llama_cpp.model_paths.is_empty() {
            println!("  Model paths:");
            for (name, path) in &settings.llama_cpp.model_paths {
                println!("    {}: {}", name, path);
            }
        }
    }

    // Show LM Studio config if relevant
    if settings.model.provider == "lm_studio" {
        println!();
        println!("LM Studio configuration:");
        println!("  Enabled: {}", settings.lm_studio.enabled);
        println!("  Base URL: {}", settings.lm_studio.base_url);
        if settings.lm_studio.api_token.is_some() {
            println!("  API Token: ✓ Set");
        } else {
            println!("  API Token: Not set (optional)");
        }
        if !settings.lm_studio.model_paths.is_empty() {
            println!("  Model paths:");
            for (name, path) in &settings.lm_studio.model_paths {
                println!("    {}: {}", name, path);
            }
        }
    }

    Ok(())
}

/// Run provider command to show all providers
pub async fn run_provider(_list: bool) -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║ AVAILABLE AI PROVIDERS                                          ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ ANTHROPIC (Cloud)                                             ║");
    println!("║   Provider: anthropic                                          ║");
    println!("║   Default: claude-sonnet-4-20250514                            ║");
    println!("║   Models: claude-opus-4, claude-sonnet-4, claude-haiku-4      ║");
    println!("║   Setup: export ANTHROPIC_API_KEY=your_key                      ║");
    println!();
    println!("║ OPENAI (Cloud)                                                 ║");
    println!("║   Provider: openai                                             ║");
    println!("║   Default: gpt-4o                                              ║");
    println!("║   Models: gpt-4o, gpt-4o-mini, gpt-4-turbo, o1, o1-mini        ║");
    println!("║   Setup: export OPENAI_API_KEY=your_key                        ║");
    println!();
    println!("║ OLLAMA (Local)                                                 ║");
    println!("║   Provider: ollama                                             ║");
    println!("║   Default: llama3.2                                            ║");
    println!("║   Setup: ollama serve && ollama pull llama3.2                  ║");
    println!();
    println!("║ LM STUDIO (Local)                                              ║");
    println!("║   Provider: lm_studio                                           ║");
    println!("║   Default: llama3.2                                             ║");
    println!("║   Setup: lms server start OR LM Studio GUI                    ║");
    println!();
    println!("║ LLAMA.CPP (Local)                                              ║");
    println!("║   Provider: llama_cpp                                          ║");
    println!("║   Default: llama3.2                                             ║");
    println!("║   Setup: llama-server binary + GGUF model files                ║");
    println!();
    println!("To switch: quantumn model <provider_name>");

    Ok(())
}
