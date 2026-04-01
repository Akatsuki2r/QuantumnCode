//! TUI Application state and rendering

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, Mode};
use crate::config::themes::Theme;

/// TUI Application
pub struct TuiApp {
    /// The main app state
    pub app: App,
    /// Theme colors converted for ratatui
    pub colors: crate::config::themes::RatatuiColors,
}

impl TuiApp {
    pub fn new(app: App) -> Self {
        let colors = app.theme.colors.to_ratatui().unwrap_or_else(|_| {
            crate::config::Theme::default_theme()
                .colors
                .to_ratatui()
                .unwrap()
        });
        Self { app, colors }
    }
}

/// Create the main layout
pub fn create_layout(frame: &Frame) -> Rect {
    frame.area()
}

/// Render the application
pub fn render(frame: &mut Frame, app: &App) {
    // Create theme colors
    let colors = match app.theme.colors.to_ratatui() {
        Ok(c) => c,
        Err(_) => {
            let default_theme = Theme::default_theme();
            default_theme.colors.to_ratatui().unwrap()
        }
    };

    // Create main layout
    let chunks = Layout::vertical([
        Constraint::Length(1),  // Status bar
        Constraint::Min(1),      // Main content
        Constraint::Length(3),   // Input
    ])
    .split(frame.area());

    // Render status bar
    render_status_bar(frame, chunks[0], app, &colors);

    // Render main content based on mode
    match app.mode {
        Mode::Normal => render_chat(frame, chunks[1], app, &colors),
        Mode::Help => render_help(frame, chunks[1], app, &colors),
        Mode::Editing => render_editor(frame, chunks[1], app, &colors),
        Mode::Review => render_review(frame, chunks[1], app, &colors),
        Mode::Command => render_command_palette(frame, chunks[1], app, &colors),
    }

    // Render input
    render_input(frame, chunks[2], app, &colors);
}

/// Render the status bar
fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, colors: &crate::config::themes::RatatuiColors) {
    let status = Paragraph::new(
        Line::from(vec![
            Span::styled(" Quantumn ", Style::default().fg(colors.accent).bg(colors.background).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!(" {} ", app.session.model),
                Style::default().fg(colors.foreground).bg(colors.background),
            ),
            Span::styled(
                format!(" {} ", app.session.provider),
                Style::default().fg(colors.muted).bg(colors.background),
            ),
            Span::styled(
                format!(" {} tokens ", app.total_tokens()),
                Style::default().fg(colors.info).bg(colors.background),
            ),
            Span::styled(
                match app.mode {
                    Mode::Normal => " NORMAL ",
                    Mode::Editing => " EDIT ",
                    Mode::Review => " REVIEW ",
                    Mode::Help => " HELP ",
                    Mode::Command => " COMMAND ",
                },
                Style::default().fg(colors.accent).bg(colors.background).add_modifier(Modifier::BOLD),
            ),
        ])
    )
    .style(Style::default().bg(colors.background));

    frame.render_widget(status, area);
}

/// Render the chat area
fn render_chat(frame: &mut Frame, area: Rect, app: &App, colors: &crate::config::themes::RatatuiColors) {
    // Build list of messages
    let messages: Vec<Line> = app.session.messages
        .iter()
        .flat_map(|msg| {
            let role_style = match msg.role.as_str() {
                "user" => Style::default().fg(colors.accent).add_modifier(Modifier::BOLD),
                "assistant" => Style::default().fg(colors.success),
                _ => Style::default().fg(colors.muted),
            };

            let role_prefix = Span::styled(
                match msg.role.as_str() {
                    "user" => "You: ",
                    "assistant" => "AI: ",
                    _ => "System: ",
                },
                role_style,
            );

            // Wrap content into lines
            let lines: Vec<Line> = textwrap::wrap(&msg.content, area.width as usize)
                .into_iter()
                .map(|line| {
                    Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(colors.foreground),
                    ))
                })
                .collect();

            let mut result = vec![Line::from(role_prefix)];
            result.extend(lines);
            result.push(Line::default()); // Empty line between messages

            result
        })
        .collect();

    let paragraph = Paragraph::new(messages)
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .wrap(Wrap { trim: false })
        .scroll(app.scroll_offset as u16);

    frame.render_widget(paragraph, area);
}

/// Render the input area
fn render_input(frame: &mut Frame, area: Rect, app: &App, colors: &crate::config::themes::RatatuiColors) {
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Input ")
                .title_style(Style::default().fg(colors.accent)),
        );

    frame.render_widget(input, area);

    // Show cursor
    let cursor_x = (area.x + 1 + app.cursor_position as u16).min(area.x + area.width - 2);
    let cursor_y = area.y + 1;
    frame.set_cursor_position(Position { x: cursor_x, y: cursor_y });
}

/// Render help screen
fn render_help(frame: &mut Frame, area: Rect, app: &App, colors: &crate::config::themes::RatatuiColors) {
    let help_text = vec![
        Line::from(Span::styled("Quantumn Code - Help", Style::default().fg(colors.accent).add_modifier(Modifier::BOLD))),
        Line::default(),
        Line::from(Span::styled("Keyboard Shortcuts:", Style::default().fg(colors.secondary).add_modifier(Modifier::BOLD))),
        Line::from("  Enter       - Send message"),
        Line::from("  Ctrl+C      - Quit"),
        Line::from("  Esc         - Cancel/Exit"),
        Line::from("  Tab         - Autocomplete"),
        Line::from("  Ctrl+L      - Clear screen"),
        Line::from("  Ctrl+S      - Save session"),
        Line::from("  F1          - Toggle help"),
        Line::from("  F2          - Toggle file tree"),
        Line::from("  F3          - Toggle token count"),
        Line::from("  F4          - Change theme"),
        Line::from("  /           - Command palette"),
        Line::default(),
        Line::from(Span::styled("Commands:", Style::default().fg(colors.secondary).add_modifier(Modifier::BOLD))),
        Line::from("  /help       - Show help"),
        Line::from("  /clear      - Clear conversation"),
        Line::from("  /model      - Change model"),
        Line::from("  /theme      - Change theme"),
        Line::from("  /commit     - Generate commit"),
        Line::from("  /review     - Review code"),
        Line::from("  /test       - Run tests"),
        Line::from("  /quit       - Exit"),
        Line::default(),
        Line::from(Span::styled("Press any key to close", Style::default().fg(colors.muted))),
    ];

    let paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Help ")
                .title_style(Style::default().fg(colors.accent)),
        );

    frame.render_widget(paragraph, area);
}

/// Render editor mode
fn render_editor(frame: &mut Frame, area: Rect, app: &App, colors: &crate::config::themes::RatatuiColors) {
    // TODO: Implement file editor view
    let paragraph = Paragraph::new("Editor mode - Coming soon")
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Editor ")
                .title_style(Style::default().fg(colors.accent)),
        );

    frame.render_widget(paragraph, area);
}

/// Render review mode
fn render_review(frame: &mut Frame, area: Rect, app: &App, colors: &crate::config::themes::RatatuiColors) {
    // TODO: Implement code review view
    let paragraph = Paragraph::new("Review mode - Coming soon")
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Review ")
                .title_style(Style::default().fg(colors.accent)),
        );

    frame.render_widget(paragraph, area);
}

/// Render command palette
fn render_command_palette(frame: &mut Frame, area: Rect, app: &App, colors: &crate::config::themes::RatatuiColors) {
    // TODO: Implement command palette
    let paragraph = Paragraph::new("Command palette - Coming soon")
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Commands ")
                .title_style(Style::default().fg(colors.accent)),
        );

    frame.render_widget(paragraph, area);
}