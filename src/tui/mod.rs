//! Terminal User Interface module
//!
//! Provides the interactive TUI for Quantumn Code

pub mod app;
pub mod render;
pub mod event;

pub use app::TuiApp;

use color_eyre::eyre::Result;
use ratatui::prelude::*;

use crate::config::Theme;

/// Run the interactive TUI
pub async fn run_interactive(model: Option<String>, theme: Option<String>) -> Result<()> {
    // Load configuration
    let settings = crate::config::Settings::load()?;

    // Load theme
    let theme_name = theme.unwrap_or(settings.ui.theme.clone());
    let theme = Theme::load(&theme_name)?;

    // Create app
    let mut app = crate::app::App::new(settings, theme);

    // Set model if specified
    if let Some(m) = model {
        app.session.model = m;
    }

    // Run TUI
    run_tui(app)?;

    Ok(())
}

/// Initialize and run the TUI
fn run_tui(mut app: crate::app::App) -> Result<()> {
    // Setup terminal
    let backend = CrosstermBackend::new(std::io::stderr());
    let mut terminal = Terminal::new(backend)?;

    // Setup panic hook to restore terminal
    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic_hook(info);
    }));

    // Enter alternate screen
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        std::io::stderr(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    // Main loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    restore_terminal()?;

    res
}

/// Run the main application loop
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut crate::app::App,
) -> Result<()> {
    loop {
        // Draw frame
        terminal.draw(|frame| render::render(frame, app))?;

        // Handle events
        if event::handle_events(app)? {
            break;
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Restore terminal state
fn restore_terminal() -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        std::io::stderr(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    Ok(())
}