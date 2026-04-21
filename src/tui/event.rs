//! Event handling for the TUI

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::Duration;

use crate::app::{App, Mode};
use color_eyre::eyre::Result;

/// Handle all events (async for AI responses)
pub async fn handle_events(app: &mut App) -> Result<bool> {
    // Background maintenance
    app.update_git_status();

    if crossterm::event::poll(Duration::from_millis(100))? {
        match crossterm::event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    return handle_key_event(app, key).await;
                }
            }
            Event::Resize(_, _) => {
                // Terminal resized, will be handled on next render
            }
            _ => {}
        }
    }
    Ok(false)
}

/// Handle keyboard events (async for AI responses)
async fn handle_key_event(app: &mut App, key: crossterm::event::KeyEvent) -> Result<bool> {
    // Global shortcuts — always fire regardless of mode
    match (key.modifiers, key.code) {
        // Quit
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            app.dropdown.close();
            app.quit();
            return Ok(true);
        }

        // Clear screen
        (KeyModifiers::CONTROL, KeyCode::Char('l')) if matches!(app.mode, Mode::Chat) => {
            app.clear_conversation(); // Only clear chat if in chat mode
            return Ok(false);
        }

        // Toggle help
        (KeyModifiers::NONE, KeyCode::F(1)) => {
            app.mode = match app.mode {
                Mode::Chat => Mode::Focus, // Temporarily use Focus for help overlay
                Mode::Focus => Mode::Chat,
                _ => Mode::Chat, // Fallback
            };
            return Ok(false);
        }

        // Command Palette (Ctrl+K)
        (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
            app.toggle_command_palette();
            return Ok(false);
        }

        // Escape — close command palette or return to chat
        (KeyModifiers::NONE, KeyCode::Esc) => {
            if app.command_palette_active {
                app.toggle_command_palette();
                app.mode = Mode::Chat;
            } else if app.mode != Mode::Chat {
                app.mode = Mode::Chat; // Exit any focus mode
            }
            return Ok(false);
        }

        _ => {}
    }

    // Mode-specific handling
    match app.mode {
        Mode::Chat => handle_chat_mode(app, key).await,
        Mode::Command => handle_command_palette_mode(app, key).await,
        Mode::Focus => handle_focus_mode(app, key), // For help overlay, editing, etc.
    }
}

/// Handle command palette mode
async fn handle_command_palette_mode(app: &mut App, key: crossterm::event::KeyEvent) -> Result<bool> {
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Enter) => {
            if !app.command_palette_input.is_empty() {
                let command_input = app.command_palette_input.clone();
                app.toggle_command_palette(); // Close palette before executing
                app.mode = Mode::Chat; // Return to chat mode
                app.input = format!("/{}", command_input); // Prepare for slash command execution
                handle_slash_command(app)?; // Execute the command
            }
            Ok(false)
        }
        (KeyModifiers::NONE, KeyCode::Char(c)) => {
            app.command_palette_input.insert(app.command_palette_cursor_position, c);
            app.command_palette_cursor_position += c.len_utf8();
            Ok(false)
        }
        (KeyModifiers::NONE, KeyCode::Backspace) => {
            if app.command_palette_cursor_position > 0 {
                app.command_palette_cursor_position -= 1;
                app.command_palette_input.remove(app.command_palette_cursor_position);
            }
            Ok(false)
        }
        (KeyModifiers::NONE, KeyCode::Delete) => {
            if app.command_palette_cursor_position < app.command_palette_input.len() {
                app.command_palette_input.remove(app.command_palette_cursor_position);
            }
            Ok(false)
        }
        (KeyModifiers::NONE, KeyCode::Left) => {
            if app.command_palette_cursor_position > 0 {
                app.command_palette_cursor_position -= 1;
            }
            Ok(false)
        }
        (KeyModifiers::NONE, KeyCode::Right) => {
            if app.command_palette_cursor_position < app.command_palette_input.len() {
                app.command_palette_cursor_position += 1;
            }
            Ok(false)
        }
        (KeyModifiers::NONE, KeyCode::Home) => {
            app.command_palette_cursor_position = 0;
            Ok(false)
        }
        (KeyModifiers::NONE, KeyCode::End) => {
            app.command_palette_cursor_position = app.command_palette_input.len();
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle focus mode (e.g., help overlay, editing, etc.)
fn handle_focus_mode(app: &mut App, key: crossterm::event::KeyEvent) -> Result<bool> {
    // For now, any key in focus mode returns to chat, or specific actions
    match (key.modifiers, key.code) {
        _ => {
            app.mode = Mode::Chat; // Exit focus mode
            Ok(false)
        }
    }
}

/// Send a message to the AI provider and get a response
async fn send_to_ai(app: &mut App, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    use crate::providers::{Message, Provider, Role};

    let provider_name = app.session.provider.clone();
    let model = app.session.model.clone();

    tracing::info!(
        target: "chat_flow",
        "Sending to AI: provider={}, model={}, message_count={}, prompt_length={}",
        provider_name,
        model,
        app.session.messages.len(),
        prompt.len()
    );

    let start_time = std::time::Instant::now();

    // Convert app messages to provider format
    let messages: Vec<Message> = app
        .session
        .messages
        .iter()
        .map(|m| Message {
            role: match m.role.as_str() {
                "user" => Role::User,
                "assistant" => Role::Assistant,
                "system" => Role::System,
                _ => Role::User,
            },
            content: m.content.clone(),
            name: None,
        })
        .collect();

    tracing::debug!(
        target: "chat_flow",
        "Converted {} messages to provider format",
        messages.len()
    );

    // Create appropriate provider and send
    let response = match provider_name.as_str() {
        "ollama" => {
            tracing::debug!(target: "chat_flow", "Creating Ollama provider with model: {}", model);
            let provider = crate::providers::OllamaProvider::with_model(model);
            tracing::trace!(target: "chat_flow", "Sending request to Ollama API");
            let response = provider.send(messages).await?;
            tracing::debug!(target: "chat_flow", "Ollama response received, length: {}", response.len());
            response
        }
        "anthropic" => {
            tracing::debug!(target: "chat_flow", "Creating Anthropic provider with model: {}", model);
            let mut provider = crate::providers::AnthropicProvider::new();
            provider.set_model(model);
            tracing::trace!(target: "chat_flow", "Sending request to Anthropic API");
            let response = provider.send(messages).await?;
            tracing::debug!(target: "chat_flow", "Anthropic response received, length: {}", response.len());
            response
        }
        _ => return Err(format!("Unknown provider: {}", provider_name).into()),
    };

    let elapsed = start_time.elapsed();
    tracing::info!(
        target: "chat_flow",
        "AI response received in {:.2?} ({} bytes)",
        elapsed,
        response.len()
    );

    Ok(response)
}

/// Handle normal mode input (async for AI responses)
async fn handle_chat_mode(app: &mut App, key: crossterm::event::KeyEvent) -> Result<bool> {
    match (key.modifiers, key.code) {
        // Enter - send message
        (KeyModifiers::NONE, KeyCode::Enter) => {
            if !app.input.is_empty() {
                // Record history if it's different from the last entry
                if app.input_history.last() != Some(&app.input) {
                    app.input_history.push(app.input.clone());
                }
                app.history_index = None;

                // Check if it's a command
                if app.input.starts_with('/') {
                    handle_slash_command(app)?;
                } else {
                    let prompt = app.input.clone();

                    // Route the prompt through the router for automatic model selection
                    let (selected_provider, selected_model) = app.route_prompt(&prompt);

                    // Update session with selected provider/model
                    app.session.provider = selected_provider.clone();
                    app.session.model = selected_model.clone();

                    // Notify user if router selected a different model
                    if app.router_enabled && (app.session.model != selected_model || app.session.provider != selected_provider) {
                        let status = format!(
                            "[ROUTING] {} → {} ({})",
                            app.session.model, selected_model, selected_provider
                        );
                        // Only update status if model/provider actually changed
                        app.debug_log(&status);
                        app.set_status(Some(status.clone()));
                        app.debug_log(&status);
                    }

                    // Add user message
                    app.add_message("user", &prompt);
                    app.input.clear();
                    app.cursor_position = 0;
                    app.debug_log(&format!("Message sent to AI: {}", prompt));

                    // Send to AI and get response with detailed status updates
                    let ai_start_time = std::time::Instant::now();
                    app.set_status(Some("[CONNECTING] Contacting AI model...".to_string()));

                    match send_to_ai(app, &prompt).await {
                        Ok(response) => {
                            app.last_routing_duration = Some(ai_start_time.elapsed());
                            app.set_status(Some("[RECEIVED] Processing response...".to_string()));
                            app.debug_log("AI response received successfully");
                            app.add_message("assistant", &response);
                            app.set_status(Some(format!(
                                "[COMPLETE] Response from {} ({})",
                                selected_provider, selected_model
                            )));
                        }
                        Err(e) => {
                            app.last_routing_duration = Some(ai_start_time.elapsed());
                            app.debug_log(&format!("AI Error: {}", e));
                            tracing::error!(target: "chat_flow", "AI request failed: {}", e);
                            app.add_message("system", &format!("[ERROR] AI request failed: {}", e));
                            app.set_status(Some("[ERROR] Failed to get response".to_string()));
                        }
                    }
                }
            }
            Ok(false)
        }

        // Tab - slash command autocomplete
        (KeyModifiers::NONE, KeyCode::Tab) => {
            if app.input.starts_with('/') && app.input.len() > 1 {
                let partial = &app.input[1..].to_lowercase();
                let commands = [
                    "help", "clear", "quit", "exit", "provider", "model", "theme", "session",
                    "config", "status", "version", "mode", "commit", "review", "test", "router",
                    "ollama",
                ];
                // Find the first command that starts with the partial
                if let Some(matched) = commands.iter().find(|c| c.starts_with(partial.as_str())) {
                    app.input = format!("/{}", matched);
                    app.cursor_position = app.input.len();
                }
            }
            Ok(false)
        }

        // Text input
        (KeyModifiers::NONE, KeyCode::Char(c)) => {
            app.input.insert(app.cursor_position, c);
            app.cursor_position += c.len_utf8();
            Ok(false)
        }

        // Backspace
        (KeyModifiers::NONE, KeyCode::Backspace) => {
            if app.cursor_position > 0 {
                app.cursor_position -= 1;
                app.input.remove(app.cursor_position);
            }
            Ok(false)
        }

        // Delete
        (KeyModifiers::NONE, KeyCode::Delete) => {
            if app.cursor_position < app.input.len() {
                app.input.remove(app.cursor_position);
            }
            Ok(false)
        }

        // Left arrow
        (KeyModifiers::NONE, KeyCode::Left) => {
            if app.cursor_position > 0 {
                app.cursor_position -= 1;
            }
            Ok(false)
        }

        // Right arrow
        (KeyModifiers::NONE, KeyCode::Right) => {
            if app.cursor_position < app.input.len() {
                app.cursor_position += 1;
            }
            Ok(false)
        }

        // Home
        (KeyModifiers::NONE, KeyCode::Home) => {
            app.cursor_position = 0;
            Ok(false)
        }

        // End
        (KeyModifiers::NONE, KeyCode::End) => {
            app.cursor_position = app.input.len();
            Ok(false)
        }

        // Up arrow - history navigation
        (KeyModifiers::NONE, KeyCode::Up) => {
            if !app.input_history.is_empty() {
                let current_idx = app.history_index.unwrap_or(app.input_history.len());
                if current_idx > 0 {
                    let new_idx = current_idx - 1;
                    app.history_index = Some(new_idx);
                    app.input = app.input_history[new_idx].clone();
                    app.cursor_position = app.input.len();
                }
            }
            Ok(false)
        }

        // Down arrow - history navigation
        (KeyModifiers::NONE, KeyCode::Down) => {
            if let Some(idx) = app.history_index {
                let new_idx = idx + 1;
                if new_idx < app.input_history.len() {
                    app.history_index = Some(new_idx);
                    app.input = app.input_history[new_idx].clone();
                    app.cursor_position = app.input.len();
                } else {
                    app.history_index = None;
                    app.input.clear();
                    app.cursor_position = 0;
                }
            }
            Ok(false)
        }

        // Page Up - scroll messages up
        (KeyModifiers::NONE, KeyCode::PageUp) => {
            app.auto_scroll = false;
            if app.scroll_offset > 0 {
                app.scroll_offset = app.scroll_offset.saturating_sub(5);
            }
            Ok(false)
        }

        // Page Down - scroll messages down
        (KeyModifiers::NONE, KeyCode::PageDown) => {
            app.auto_scroll = false;
            app.scroll_offset += 5;
            Ok(false)
        }

        // End - snap to bottom and enable auto-scroll
        (KeyModifiers::NONE, KeyCode::End) => {
            app.auto_scroll = true;
            app.scroll_offset = 0; // Will be recalculated in render
            Ok(false)
        }

        _ => Ok(false),
    }
}

/// Handle slash commands
fn handle_slash_command(app: &mut App) -> Result<bool> {
    let input = app.input.clone();
    app.input.clear();
    app.cursor_position = 0;

    let parts: Vec<&str> = input[1..].split_whitespace().collect();
    let command = parts.first().unwrap_or(&"");
    let arg = parts.get(1).map(|s| *s);
    let start = std::time::Instant::now();

    tracing::info!(target: "command_exec", "Executing command: /{}", command);
    app.debug_log(&format!("Command Fired: /{command}"));

    let result = match *command {
        "help" | "h" | "?" => {
            app.mode = Mode::Focus;
            Ok(false)
        }
        "clear" | "c" => {
            app.clear_conversation();
            Ok(false)
        }
        "quit" | "q" | "exit" => {
            app.quit();
            Ok(true)
        }
        "provider" | "p" => {
            if let Some(provider_name) = arg {
                app.session.provider = provider_name.to_string();
                app.set_status(Some(format!("Provider changed to: {}", provider_name)));
                app.add_message("system", &format!("✓ Provider set to: {}", provider_name));
            } else {
                // Show all providers
                let msg = "╔════════════════════════════════════════════════════════════════╗\n\
║ AVAILABLE AI PROVIDERS                                          ║\n\
╠════════════════════════════════════════════════════════════════╣\n\
║ ANTHROPIC (Cloud)                                             ║\n\
║   Provider: anthropic                                          ║\n\
║   Default: claude-sonnet-4-20250514                            ║\n\
║   Models: claude-opus-4, claude-sonnet-4, claude-haiku-4      ║\n\
║   Setup: export ANTHROPIC_API_KEY=your_key                      ║\n\
\n\
║ OPENAI (Cloud)                                                 ║\n\
║   Provider: openai                                             ║\n\
║   Default: gpt-4o                                              ║\n\
║   Models: gpt-4o, gpt-4o-mini, gpt-4-turbo, o1, o1-mini        ║\n\
║   Setup: export OPENAI_API_KEY=your_key                        ║\n\
\n\
║ OLLAMA (Local)                                                 ║\n\
║   Provider: ollama                                             ║\n\
║   Default: llama3.2                                            ║\n\
║   Setup: ollama serve && ollama pull llama3.2                  ║\n\
\n\
║ LM STUDIO (Local)                                              ║\n\
║   Provider: lm_studio                                           ║\n\
║   Default: llama3.2                                             ║\n\
║   Setup: lms server start OR LM Studio GUI                    ║\n\
\n\
║ LLAMA.CPP (Local)                                              ║\n\
║   Provider: llama_cpp                                          ║\n\
║   Default: llama3.2                                             ║\n\
║   Setup: llama-server binary + GGUF model files                ║\n\
\n\
To switch: /provider <provider_name>";
                app.add_message("system", msg);
            }
            Ok(false)
        }
        "model" | "m" => {
            if let Some(model) = arg {
                app.session.model = model.to_string();
                app.set_status(Some(format!("Model changed to: {}", model)));
                app.add_message("system", &format!("✓ Model set to: {}", model));
            } else {
                // Show all models
                let msg = "╔════════════════════════════════════════════════════════════════╗\n\
║ CLOUD MODELS                                                   ║\n\
╠════════════════════════════════════════════════════════════════╣\n\
║ ANTHROPIC (Claude)                                            ║\n\
  claude-opus-4-20250514   - Most capable (Opus 4)\n\
  claude-sonnet-4-20250514 - Balanced (Sonnet 4) [default]\n\
  claude-haiku-4-20250514  - Fast (Haiku 4)\n\
  claude-3-5-sonnet-20241022 - Legacy (Sonnet 3.5)\n\
  claude-3-5-haiku-20241022  - Legacy (Haiku 3.5)\n\
\n\
║ OPENAI                                                        ║\n\
  gpt-4o       - GPT-4 Omni (recommended)\n\
  gpt-4o-mini  - GPT-4 Omni Mini (fast, cheap)\n\
  gpt-4-turbo  - GPT-4 Turbo\n\
  o1           - O1 (advanced reasoning)\n\
  o1-mini      - O1 Mini\n\
\n\
╔════════════════════════════════════════════════════════════════╗\n\
║ LOCAL MODELS (Ollama / LM Studio / llama.cpp)                 ║\n\
╠════════════════════════════════════════════════════════════════╣\n\
  llama3.2       - Meta Llama 3.2\n\
  llama3.1       - Meta Llama 3.1\n\
  mistral        - Mistral\n\
  codellama      - Code Llama\n\
  deepseek-coder - DeepSeek Coder\n\
  qwen2.5-coder  - Qwen 2.5 Coder\n\
\n\
To switch: /model <model_name>";
                app.add_message("system", msg);
            }
            Ok(false)
        }
        "theme" | "t" => {
            if let Some(theme_name) = arg {
                match crate::config::Theme::load(theme_name) {
                    Ok(theme) => {
                        app.theme = theme;
                        app.set_status(Some(format!("Theme changed to: {}", theme_name)));
                        app.add_message("system", &format!("✓ Theme set to: {}", theme_name));
                    }
                    Err(e) => {
                        app.set_status(Some(format!("Error loading theme: {}", e)));
                        app.add_message("system", &format!("✗ Error: {}", e));
                    }
                }
            } else {
                // List themes
                let msg = "Available Themes:\n\
  • oxidized    - Rusty brown on deep black [default]\n\
  • default     - Classic Claude-inspired purple\n\
  • tokyo_night - Purple and blue accents\n\
  • hacker      - Matrix-style green on black\n\
  • deep_black  - Minimal high-contrast dark\n\
\n\
To switch: /theme <theme_name>";
                app.add_message("system", msg);
            }
            Ok(false)
        }
        "session" | "sess" => {
            match arg {
                Some("list") | Some("ls") | Some("l") => {
                    // List sessions (TODO: implement proper session listing)
                    app.add_message(
                        "system",
                        "Sessions: 0 saved\n\nUse /session save <name> to save current session",
                    );
                }
                Some("save") | Some("s") => {
                    let name = parts.get(2).unwrap_or(&"unnamed");
                    // TODO: Implement session saving
                    app.add_message("system", &format!("Session saved as: {}", name));
                }
                Some("load") | Some("resume") | Some("r") => {
                    // TODO: Implement session loading
                    app.add_message("system", "Session loading coming soon");
                }
                Some("delete") | Some("del") | Some("d") => {
                    // TODO: Implement session deletion
                    app.add_message("system", "Session deletion coming soon");
                }
                None => {
                    app.add_message("system", "Session commands:\n  /session list   - List saved sessions\n  /session save <name> - Save current session\n  /session load <name> - Load session\n  /session delete <name> - Delete session");
                }
                _ => {
                    app.add_message(
                        "system",
                        "Unknown session command. Use: list, save, load, delete",
                    );
                }
            }
            Ok(false)
        }
        "config" | "cfg" => {
            match arg {
                Some("show") | Some("s") => {
                    let msg = format!(
                        "Current Configuration:\n\
  Provider: {}\n\
  Model: {}\n\
  Theme: {}\n\
\n\
Config file: ~/.config/quantumn-code/config.toml",
                        app.session.provider,
                        app.session.model,
                        "oxidized" // TODO: get from theme
                    );
                    app.add_message("system", &msg);
                }
                Some("edit") | Some("e") => {
                    app.add_message("system", "Opening config editor... (coming soon)");
                }
                None => {
                    app.add_message("system", "Config commands:\n  /config show  - Show current config\n  /config edit  - Open config in editor");
                }
                _ => {
                    app.add_message("system", "Unknown config command. Use: show, edit");
                }
            }
            Ok(false)
        }
        "ollama" | "o" => {
            // List detected Ollama models with details
            let models_info = crate::router::model::get_local_models_info();
            let is_running = crate::router::model::is_ollama_running();

            if models_info.is_empty() {
                let msg = if is_running {
                    "No Ollama models found.\n\nInstall models with:\n  ollama pull llama3.2\n  ollama pull mistral\n  ollama pull codellama"
                } else {
                    "Ollama server is not running.\n\nStart it with:\n  ollama serve\n\nThen install models:\n  ollama pull llama3.2"
                };
                app.add_message("system", msg);
            } else {
                let mut lines = vec![
                    format!("╔════════════════════════════════════════════════════════════════╗"),
                    format!(
                        "║ OLLAMA MODELS {:<16}                                ║",
                        if is_running {
                            "(Server Running)"
                        } else {
                            "(Server Offline)"
                        }
                    ),
                    format!("╠════════════════════════════════════════════════════════════════╣"),
                ];

                lines.push(format!(
                    "║ {:<25} {:>10}  {:>12} ║",
                    "Model", "Size", "Modified"
                ));
                lines.push(format!(
                    "╠════════════════════════════════════════════════════════════════╣"
                ));

                for (name, size, modified) in &models_info {
                    // Truncate long names
                    let display_name = if name.len() > 24 {
                        format!("{}...", &name[..21])
                    } else {
                        name.clone()
                    };
                    lines.push(format!(
                        "║ {:<25} {:>10}  {:>12} ║",
                        display_name, size, modified
                    ));
                }

                lines.push(format!(
                    "╚════════════════════════════════════════════════════════════════╝"
                ));
                lines.push(String::new());
                lines.push(format!("Total: {} model(s)", models_info.len()));
                lines.push(String::new());
                lines.push(format!("Use: /model <model_name> to switch"));

                app.add_message("system", &lines.join("\n"));
            }
            Ok(false)
        }
        "status" | "s" => {
            let status = format!(
                "╔════════════════════════════════════════════════════════════════╗\n\
║ QUANTUMN CODE STATUS                                          ║\n\
╠════════════════════════════════════════════════════════════════╣\n\
║ Model: {}\n\
║ Provider: {}\n\
║ Theme: oxidized\n\
║ Tokens used: {}\n\
╚════════════════════════════════════════════════════════════════╝",
                app.session.model,
                app.session.provider,
                app.total_tokens()
            );
            app.add_message("system", &status);
            Ok(false)
        }
        "version" | "v" => {
            let version = env!("CARGO_PKG_VERSION");
            app.add_message("system", &format!("Quantumn Code v{}", version));
            Ok(false)
        }
        "mode" => {
            if let Some(mode_name) = arg {
                match mode_name {
                    "plan" => {
                        app.add_message(
                            "system",
                            "Switched to PLAN mode - AI will plan before implementing",
                        );
                    }
                    "build" => {
                        app.add_message(
                            "system",
                            "Switched to BUILD mode - AI will implement directly",
                        );
                    }
                    "chat" => {
                        app.add_message("system", "Switched to CHAT mode - Casual conversation");
                    }
                    _ => {
                        app.add_message("system", "Unknown mode. Use: plan, build, chat");
                    }
                }
            } else {
                app.add_message("system", "Available modes:\n  /mode plan  - AI plans before implementing\n  /mode build - AI implements directly\n  /mode chat  - Casual conversation");
            }
            Ok(false)
        }
        "router" | "r" => match arg {
            Some("on") | Some("enable") => {
                app.router_enabled = true;
                app.add_message(
                    "system",
                    "✓ Router enabled - automatic model switching active",
                );
                Ok(false)
            }
            Some("off") | Some("disable") => {
                app.router_enabled = false;
                app.add_message(
                    "system",
                    "✓ Router disabled - using manually selected model",
                );
                Ok(false)
            }
            Some("status") | Some("s") => {
                let status = if app.router_enabled {
                    "enabled"
                } else {
                    "disabled"
                };
                let msg = format!(
                    "Router Status: {}\n\
                         Prefer Local: {}\n\
                         Cost Limit: ${}/1M tokens\n\
                         RAG: {}\n\
                         Prompt Compaction: {}",
                    status,
                    app.router_config.prefer_local,
                    app.router_config.cost_limit,
                    if app.router_config.rag.enabled {
                        "on"
                    } else {
                        "off"
                    },
                    if app.router_config.prompt_compaction.enabled {
                        "on"
                    } else {
                        "off"
                    }
                );
                app.add_message("system", &msg);
                Ok(false)
            }
            Some("prefer-local") | Some("pl") => {
                app.router_config.prefer_local = !app.router_config.prefer_local;
                let status = if app.router_config.prefer_local {
                    "enabled"
                } else {
                    "disabled"
                };
                app.add_message("system", &format!("✓ Prefer local models: {}", status));
                Ok(false)
            }
            None => {
                app.add_message("system", "Router commands:\n  /router on      - Enable automatic model switching\n  /router off     - Disable router, use manual selection\n  /router status  - Show router configuration\n  /router prefer-local - Toggle preference for local models");
                Ok(false)
            }
            _ => {
                app.add_message(
                    "system",
                    "Unknown router command. Use: on, off, status, prefer-local",
                );
                Ok(false)
            }
        },
        "commit" => {
            app.add_message("system", "Commit generation coming soon.");
            Ok(false)
        }
        "review" => {
            app.add_message("system", "Code review coming soon.");
            Ok(false)
        }
        "test" => {
            app.add_message("system", "Test runner coming soon.");
            Ok(false)
        }
        _ => {
            app.add_message(
                "system",
                &format!(
                    "Unknown command: {}. Type /help for available commands.",
                    command
                ),
            );
            Ok(false)
        }
    };

    let elapsed = start.elapsed();
    tracing::info!(target: "command_exec", "Command /{} finished in {:?}", command, elapsed);
    app.debug_log(&format!("Command /{} finished in {:?}", command, elapsed));
    result
}
