//! Terminal User Interface module
//!
//! Provides the interactive TUI for Quantumn Code

pub mod app;
pub mod event;
pub mod render;
pub mod widgets;

pub use render::TuiApp;

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

    // Index project files for RAG
    app.index_project_files();

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
    // Setup terminal - use stdout for the backend
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Setup panic hook to restore terminal
    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic_hook(info);
    }));

    // Enter alternate screen
    crossterm::terminal::enable_raw_mode().map_err(|e| {
        color_eyre::eyre::eyre!(
            "Failed to enable raw mode. This usually means you're not running in a terminal.\n\
            \nFor non-interactive usage, try:\n\
              • quantumn chat \"your question\"  (one-shot query)\n\
              • quantumn model --list          (list models)\n\
              • quantumn provider              (show providers)\n\
            \nOr run this command in a real terminal (not VS Code integrated terminal)."
        )
    })?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    // Main loop
    let res = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async { run_app(&mut terminal, &mut app).await })
    });

    // Restore terminal
    restore_terminal()?;

    res
}

/// Run the main application loop
async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut crate::app::App) -> Result<()> {
    loop {
        // Draw frame
        terminal
            .draw(|frame| render::render(frame, app))
            .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;

        // Handle events (now async for AI responses)
        if event::handle_events(app).await? {
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
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    Ok(())
}
