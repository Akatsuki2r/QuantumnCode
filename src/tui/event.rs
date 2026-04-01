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
        "model" | "m" => {
            if let Some(model) = parts.get(1) {
                app.session.model = model.to_string();
                app.set_status(Some(format!("Model changed to: {}", model)));
            }
        }
        "theme" | "t" => {
            if let Some(theme_name) = parts.get(1) {
                match crate::config::Theme::load(theme_name) {
                    Ok(theme) => {
                        app.theme = theme;
                        app.set_status(Some(format!("Theme changed to: {}", theme_name)));
                    }
                    Err(e) => {
                        app.set_status(Some(format!("Error loading theme: {}", e)));
                    }
                }
            }
        }
        "commit" => {
            // TODO: Generate commit message
            app.add_message("system", "/commit - Coming soon");
        }
        "review" => {
            app.mode = Mode::Review;
        }
        "test" => {
            // TODO: Run tests
            app.add_message("system", "/test - Coming soon");
        }
        "status" | "s" => {
            let status = format!(
                "Model: {} | Provider: {} | Tokens: {}",
                app.session.model,
                app.session.provider,
                app.total_tokens()
            );
            app.add_message("system", &status);
        }
        _ => {
            app.add_message("system", &format!("Unknown command: {}", command));
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