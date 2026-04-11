//! Event handling for the TUI

use std::time::Duration;
use crossterm::event::{Event, KeyCode, KeyModifiers, KeyEventKind};

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
    // Global shortcuts
    match (key.modifiers, key.code) {
        // Quit
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
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

        // Escape - return to normal mode
        (KeyModifiers::NONE, KeyCode::Esc) => {
            if app.mode != Mode::Normal {
                app.mode = Mode::Normal;
            }
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
                    // Add user message
                    app.add_message("user", &app.input.clone());
                    app.input.clear();
                    app.cursor_position = 0;
                    // TODO: Send to AI and get response
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

        // Tab - autocomplete (TODO)
        (KeyModifiers::NONE, KeyCode::Tab) => {
            // TODO: Implement autocomplete
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
                    app.add_message("system", "Sessions: 0 saved\n\nUse /session save <name> to save current session");
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
                    app.add_message("system", "Unknown session command. Use: list, save, load, delete");
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
                        app.add_message("system", "Switched to PLAN mode - AI will plan before implementing");
                    }
                    "build" => {
                        app.add_message("system", "Switched to BUILD mode - AI will implement directly");
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
        _ => {
            app.add_message("system", &format!("Unknown command: {}. Type /help for available commands.", command));
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