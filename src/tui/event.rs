//! Event handling for the TUI

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::Duration;

use crate::app::{App, Mode};
use color_eyre::eyre::Result;

/// Handle all events
pub fn handle_events(app: &mut App) -> Result<bool> {
    if crossterm::event::poll(Duration::from_millis(100))? {
        match crossterm::event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    return handle_key_event(app, key);
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

/// Handle keyboard events
fn handle_key_event(app: &mut App, key: crossterm::event::KeyEvent) -> Result<bool> {
    // Global shortcuts — always fire regardless of mode
    match (key.modifiers, key.code) {
        // Quit
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            app.dropdown.close();
            app.quit();
            return Ok(true);
        }

        // Clear screen
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
            app.clear_conversation();
            return Ok(false);
        }

        // Toggle help
        (KeyModifiers::NONE, KeyCode::F(1)) => {
            app.mode = match app.mode {
                Mode::Help => Mode::Normal,
                _ => Mode::Help,
            };
            return Ok(false);
        }

        // Open provider selector — immediately show provider list
        (KeyModifiers::NONE, KeyCode::Char('p')) if !matches!(app.mode, Mode::ProviderSelect) => {
            app.dropdown.open();
            app.mode = Mode::ProviderSelect;
            return Ok(false);
        }

        // Escape — close dropdown or return to normal
        (KeyModifiers::NONE, KeyCode::Esc) => {
            if matches!(app.mode, Mode::ProviderSelect) {
                app.dropdown.close();
                app.mode = Mode::Normal;
            } else if app.mode != Mode::Normal {
                app.mode = Mode::Normal;
            }
            return Ok(false);
        }

        // Tab switching (only when not in provider select)
        (KeyModifiers::NONE, KeyCode::Left) | (KeyModifiers::NONE, KeyCode::Right)
            if !matches!(app.mode, Mode::ProviderSelect) =>
        {
            if key.code == KeyCode::Left {
                app.tab_bar.previous();
            } else {
                app.tab_bar.next();
            }
            return Ok(false);
        }

        // Number keys for direct tab access (only outside dropdown)
        (KeyModifiers::NONE, KeyCode::Char('1')) if !matches!(app.mode, Mode::ProviderSelect) => {
            app.tab_bar.select(0);
            return Ok(false);
        }
        (KeyModifiers::NONE, KeyCode::Char('2')) if !matches!(app.mode, Mode::ProviderSelect) => {
            app.tab_bar.select(1);
            return Ok(false);
        }
        (KeyModifiers::NONE, KeyCode::Char('3')) if !matches!(app.mode, Mode::ProviderSelect) => {
            app.tab_bar.select(2);
            return Ok(false);
        }
        (KeyModifiers::NONE, KeyCode::Char('4')) if !matches!(app.mode, Mode::ProviderSelect) => {
            app.tab_bar.select(3);
            return Ok(false);
        }

        _ => {}
    }

    // Mode-specific handling
    match app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        Mode::Help => handle_help_mode(app, key),
        Mode::Editing => handle_editing_mode(app, key),
        Mode::Review => handle_review_mode(app, key),
        Mode::Command => handle_command_mode(app, key),
        // Dropdown consumes ALL remaining keys when open
        Mode::ProviderSelect => handle_provider_select_mode(app, key),
    }
}

/// Handle normal mode input
fn handle_normal_mode(app: &mut App, key: crossterm::event::KeyEvent) -> Result<bool> {
    match (key.modifiers, key.code) {
        // Enter - send message
        (KeyModifiers::NONE, KeyCode::Enter) => {
            if !app.input.is_empty() {
                // Check if it's a command
                if app.input.starts_with('/') {
                    handle_slash_command(app)?;
                } else {
                    let prompt = app.input.clone();

                    // Route the prompt through the router for automatic model selection
                    let (selected_provider, selected_model) = app.route_prompt(&prompt);

                    // Notify user if router selected a different model
                    if app.router_enabled && selected_provider != app.session.provider
                        || selected_model != app.session.model
                    {
                        app.set_status(Some(format!(
                            "Router: {} → {} ({})",
                            app.session.model, selected_model, selected_provider
                        )));
                    }

                    // Add user message
                    app.add_message("user", &prompt);
                    app.input.clear();
                    app.cursor_position = 0;
                    // TODO: Send to AI and get response using selected_provider/selected_model
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
                    "config", "status", "version", "mode", "commit", "review", "test",
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

        // Up arrow - scroll messages up
        (KeyModifiers::NONE, KeyCode::Up) => {
            if app.scroll_offset > 0 {
                app.scroll_offset -= 1;
            }
            Ok(false)
        }

        // Down arrow - scroll messages down
        (KeyModifiers::NONE, KeyCode::Down) => {
            app.scroll_offset += 1;
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

    match *command {
        "help" | "h" | "?" => {
            app.mode = Mode::Help;
        }
        "clear" | "c" => {
            app.clear_conversation();
        }
        "quit" | "q" | "exit" => {
            app.quit();
            return Ok(true);
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
        }
        "version" | "v" => {
            let version = env!("CARGO_PKG_VERSION");
            app.add_message("system", &format!("Quantumn Code v{}", version));
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
        }
        "router" | "r" => match arg {
            Some("on") | Some("enable") => {
                app.router_enabled = true;
                app.add_message(
                    "system",
                    "✓ Router enabled - automatic model switching active",
                );
            }
            Some("off") | Some("disable") => {
                app.router_enabled = false;
                app.add_message(
                    "system",
                    "✓ Router disabled - using manually selected model",
                );
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
            }
            Some("prefer-local") | Some("pl") => {
                app.router_config.prefer_local = !app.router_config.prefer_local;
                let status = if app.router_config.prefer_local {
                    "enabled"
                } else {
                    "disabled"
                };
                app.add_message("system", &format!("✓ Prefer local models: {}", status));
            }
            None => {
                app.add_message("system", "Router commands:\n  /router on      - Enable automatic model switching\n  /router off     - Disable router, use manual selection\n  /router status  - Show router configuration\n  /router prefer-local - Toggle preference for local models");
            }
            _ => {
                app.add_message(
                    "system",
                    "Unknown router command. Use: on, off, status, prefer-local",
                );
            }
        },
        _ => {
            app.add_message(
                "system",
                &format!(
                    "Unknown command: {}. Type /help for available commands.",
                    command
                ),
            );
        }
    }

    Ok(false)
}

/// Handle help mode
fn handle_help_mode(app: &mut App, key: crossterm::event::KeyEvent) -> Result<bool> {
    // Any key exits help
    app.mode = Mode::Normal;
    Ok(false)
}

/// Handle editing mode
fn handle_editing_mode(app: &mut App, key: crossterm::event::KeyEvent) -> Result<bool> {
    // TODO: Implement editor mode
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Esc) => {
            app.mode = Mode::Normal;
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle review mode
fn handle_review_mode(app: &mut App, key: crossterm::event::KeyEvent) -> Result<bool> {
    // TODO: Implement review mode
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Esc) => {
            app.mode = Mode::Normal;
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle command mode
fn handle_command_mode(app: &mut App, key: crossterm::event::KeyEvent) -> Result<bool> {
    // TODO: Implement command palette
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Esc) => {
            app.mode = Mode::Normal;
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle provider/model selection mode
fn handle_provider_select_mode(app: &mut App, key: crossterm::event::KeyEvent) -> Result<bool> {
    use crate::tui::widgets::DropdownAction;

    match app.dropdown.handle_key(key) {
        Some(DropdownAction::Confirmed(provider, model)) => {
            app.session.provider = provider.clone();
            app.session.model = model.clone();
            app.dropdown.close();
            app.mode = Mode::Normal;
            app.add_message("system", &format!("✓ Switched to {} — {}", provider, model));
            Ok(false)
        }
        Some(DropdownAction::Close) => {
            app.dropdown.close();
            app.mode = Mode::Normal;
            Ok(false)
        }
        Some(DropdownAction::NeedsApiKey) => {
            // Dropdown transitions itself to ApiKeyInput state; stay in ProviderSelect
            Ok(false)
        }
        Some(DropdownAction::ProviderSelected) => Ok(false),
        Some(DropdownAction::BackToProviders) => Ok(false),
        Some(DropdownAction::Navigate) | Some(DropdownAction::OpenProviders) => Ok(false),
        None => Ok(false),
    }
}
