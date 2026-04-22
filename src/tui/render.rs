//! TUI Application state and rendering

use ratatui::layout::Alignment;
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

/// Main render function
pub fn render(frame: &mut Frame, app: &App) {
    // Create theme colors
    let colors = match app.theme.colors.to_ratatui() {
        Ok(c) => c,
        Err(_) => {
            let default_theme = Theme::default_theme();
            default_theme.colors.to_ratatui().unwrap()
        }
    };

    // Clear the entire area to prevent ghosting/artifacts from previous frames
    frame.render_widget(Clear, frame.area());

    // Intent-driven layout: focus on the conversation
    // Only allocate space for suggestion bar if there's content to show
    // Don't show empty state hint - it wastes space and can cause layout issues
    let has_suggestions = !app.input.is_empty();
    let suggestion_height = if has_suggestions { 2 } else { 0 };

    let chunks = Layout::vertical([
        Constraint::Min(1),                    // Chat Area
        Constraint::Length(3),                 // Input
        Constraint::Length(suggestion_height), // Suggestion bar (conditional)
        Constraint::Length(1),                 // Status bar
    ])
    .split(frame.area());

    render_chat(frame, chunks[0], app, &colors);
    render_input(frame, chunks[1], app, &colors);
    if has_suggestions {
        render_suggestions(frame, chunks[2], app, &colors);
    }
    render_status_bar(frame, chunks[3], app, &colors);

    // Conditionally render overlays
    if matches!(app.mode, Mode::Focus) {
        let help_area = center_rect(80, 25, frame.area());
        frame.render_widget(Clear, help_area);
        render_help(frame, help_area, app, &colors);
    }

    if app.command_palette_active {
        render_command_palette_overlay(frame, app, &colors);
    }

    // Render dropdown overlay when open (not collapsed)
    if !matches!(
        app.dropdown.state,
        crate::tui::widgets::DropdownState::Closed
    ) {
        let dropdown_area = center_rect(60, 15, frame.area());
        frame.render_widget(Clear, dropdown_area);
        app.dropdown.render(frame, dropdown_area, &colors);
    }
}

/// Center a rect within another rect
fn center_rect(width: u16, height: u16, outer: Rect) -> Rect {
    let x = outer.x + (outer.width.saturating_sub(width)) / 2;
    let y = outer.y + (outer.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(outer.width), height.min(outer.height))
}

/// Render the status bar
fn render_status_bar(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    // Compressed Status Bar: [Mode] (branch) [R] [Tier] [Tokens] [Model]
    let mut spans = vec![Span::styled(
        format!(
            " {} ",
            match app.mode {
                Mode::Chat => "CHAT",
                Mode::Command => "CMD",
                Mode::Focus => "FOCUS",
            }
        ),
        Style::default().fg(colors.background).bg(colors.accent),
    )];

    if let Some(ref branch) = app.git_branch {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!("({})", branch),
            Style::default().fg(colors.secondary),
        ));
    }

    spans.push(Span::raw(" "));
    spans.push(if app.router_enabled {
        Span::styled("[Auto]", Style::default().fg(colors.success))
    } else {
        Span::styled("[Man]", Style::default().fg(colors.muted))
    });

    spans.extend(vec![
        Span::raw(" "),
        Span::styled(
            if app.session.provider == "ollama" {
                " [Local] "
            } else {
                " [Cloud] "
            },
            Style::default().fg(colors.info),
        ),
        Span::styled(
            format!(" ~{}k ", app.total_tokens() / 1000),
            Style::default().fg(colors.muted),
        ),
        Span::styled(
            format!(" {} ", app.session.model),
            Style::default().fg(colors.muted).italic(),
        ),
    ]);

    let status = Paragraph::new(Line::from(spans)).style(Style::default().bg(colors.background));
    frame.render_widget(status, area);
}

/// Render the chat area
fn render_chat(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let chat_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border))
        .title(Span::styled(
            " Chat ",
            Style::default().fg(colors.accent).bold(),
        ));
    let inner_area = chat_block.inner(area);

    // Dynamic padding based on window width
    let padding_width = match inner_area.width {
        w if w < 60 => 2,
        w if w < 100 => 4,
        _ => 6,
    } as usize;
    let padding = " ".repeat(padding_width);

    // Build list of messages
    let messages: Vec<Line> = app
        .session
        .messages
        .iter()
        .flat_map(|msg| {
            let role_style = match msg.role.as_str() {
                "user" => Style::default().fg(colors.accent),
                "assistant" => Style::default().fg(colors.success),
                _ => Style::default().fg(colors.muted),
            };

            let role_icon = match msg.role.as_str() {
                "user" => "󰭹 ",
                "assistant" => "󰚩 ",
                _ => "󱐋 ",
            };

            let mut lines = Vec::new();
            let mut in_code_block = false;

            for line in msg.content.lines() {
                // Detect code block delimiters
                if line.trim().starts_with("```") {
                    in_code_block = !in_code_block;
                    let border = if in_code_block {
                        format!("┌── Code {}", line.trim().trim_start_matches("```"))
                    } else {
                        "└───────".to_string()
                    };
                    lines.push(Line::from(Span::styled(
                        format!("{}{}", padding, border),
                        Style::default().fg(colors.muted).italic(),
                    )));
                    continue;
                }

                let style = if in_code_block {
                    // Highlight code blocks with a different background color
                    Style::default().fg(colors.foreground).bg(colors.muted)
                } else {
                    Style::default().fg(colors.foreground)
                };

                // Add slight indentation for code or specific style
                let display_line = if in_code_block {
                    format!("│ {}", line)
                } else {
                    line.to_string()
                };
                let wrap_width = inner_area.width.saturating_sub(padding_width as u16) as usize;

                for wrapped in textwrap::wrap(&display_line, wrap_width) {
                    lines.push(Line::from(Span::styled(
                        format!("{}{}", padding, wrapped),
                        style,
                    )));
                }
            }

            let mut result = vec![Line::from(vec![
                Span::raw(padding.clone()),
                Span::styled(role_icon, role_style),
            ])];
            result.extend(lines);
            result.push(Line::default()); // Empty line between messages

            result
        })
        .collect();

    let total_lines = messages.len();
    let visible_height = inner_area.height as usize;

    let max_scroll = total_lines.saturating_sub(visible_height);

    // Ensure the if-expression is properly grouped before the type cast
    let scroll_offset = (if app.auto_scroll {
        max_scroll
    } else {
        app.scroll_offset.min(max_scroll)
    }) as u16;

    let paragraph = Paragraph::new(messages)
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(chat_block)
        .scroll((scroll_offset, 0));

    frame.render_widget(paragraph, area);
}

/// Render the input area
fn render_input(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    // This is the main chat input bar
    // Show provider/model in the input title
    let provider_text = format!("[{}:{}]", app.session.provider, app.session.model);

    let title_line = Line::from(vec![Span::styled(
        format!(" {} ", provider_text),
        Style::default().fg(colors.accent),
    )]);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(title_line),
        );

    frame.render_widget(input, area);

    // Show cursor
    let cursor_x =
        (area.x + 1 + app.cursor_position as u16).min(area.x + area.width.saturating_sub(2));
    let cursor_y = area.y + 1;
    frame.set_cursor_position(Position {
        x: cursor_x,
        y: cursor_y,
    });
}

/// Render the suggestion bar under input
fn render_suggestions(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    // Clear area to prevent artifacts from stderr or previous frames
    frame.render_widget(Clear, area);

    let mut spans = Vec::new();

    if app.input.starts_with('/') {
        let partial = if app.input.len() > 1 {
            app.input[1..].to_lowercase()
        } else {
            String::new()
        };

        let commands = [
            ("help", "show help"),
            ("clear", "clear chat"),
            ("quit", "exit app"),
            ("exit", "exit app"),
            ("provider", "select AI provider"),
            ("model", "select AI model"),
            ("theme", "change TUI theme"),
            ("session", "manage sessions"),
            ("config", "app settings"),
            ("router", "model switching"),
            ("ollama", "list local models"),
            ("status", "app diagnostics"),
            ("version", "show version"),
            ("mode", "switch behavior"),
            ("commit", "git commit help"),
            ("review", "code review"),
            ("test", "run tests"),
        ];

        let mut matches: Vec<_> = commands
            .iter()
            .filter(|(c, _)| c.starts_with(&partial))
            .collect();

        if !matches.is_empty() {
            spans.push(Span::styled(" 󱊖 ", Style::default().fg(colors.accent)));
            for (i, (cmd, desc)) in matches.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::styled(" • ", Style::default().fg(colors.muted)));
                }
                spans.push(Span::styled(
                    format!("/{}", cmd),
                    Style::default().fg(colors.foreground).bold(),
                ));
                spans.push(Span::styled(
                    format!(" {}", desc),
                    Style::default().fg(colors.muted),
                ));
            }
        }
    } else if app.input.is_empty() {
        spans.push(Span::styled(
            " 󰟶 Type / for commands, p for providers, or just chat... ",
            Style::default().fg(colors.muted).italic(),
        ));
    }

    if !spans.is_empty() {
        let suggestion_bar = Paragraph::new(Line::from(spans))
            .style(Style::default().bg(colors.background))
            .alignment(Alignment::Left);
        frame.render_widget(suggestion_bar, area);
    }
}

/// Render help screen
fn render_help(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let help_text = vec![
        Line::from(Span::styled(
            "Quantumn Code - Help",
            Style::default().fg(colors.accent).bold().underlined(),
        )),
        Line::default(),
        Line::from(Span::styled(
            "Keyboard Shortcuts:",
            Style::default().fg(colors.secondary).bold(),
        )),
        Line::from("  Enter       - Send message"),
        Line::from("  Ctrl+C      - Quit"),
        Line::from("  Esc         - Cancel/Exit"),
        Line::from("  Tab         - Autocomplete"),
        Line::from("  Ctrl+L      - Clear screen"),
        Line::from("  Ctrl+S      - Save session"),
        Line::from("  F1          - Toggle help"),
        Line::from("  Ctrl+K      - Open Command Palette"),
        Line::from("  /           - Slash commands in chat"),
        Line::from("  P           - Open provider selector"),
        Line::from("  ←→         - Switch tabs"),
        Line::from("  Ctrl+P      - Toggle provider quick menu"),
        Line::default(),
        Line::from(Span::styled(
            "Commands:",
            Style::default().fg(colors.secondary).bold(),
        )),
        Line::from("  /help       - Show this help screen"),
        Line::from("  /clear      - Clear current conversation"),
        Line::from("  /model      - List or change the active AI model"),
        Line::from("  /ollama     - List locally available Ollama models"),
        Line::from("  /router     - Manage automatic model switching settings"),
        Line::from("  /theme      - Change the application's color theme"),
        Line::from("  /provider   - List or change the active AI provider"),
        Line::from("  /commit     - Generate a git commit message for staged changes"),
        Line::from("  /review     - Initiate a code review of specified files"),
        Line::from("  /test       - Run tests and analyze their output"),
        Line::from("  /status     - Display current application diagnostics"),
        Line::from("  /debug      - Toggle visibility of the debug log panel"),
        Line::from("  /quit       - Exit the application"),
        Line::default(),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(colors.muted),
        )),
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

/// Render the command palette overlay
fn render_command_palette_overlay(
    frame: &mut Frame,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let area = center_rect(60, 3, frame.area());

    // Clear the background of the palette area
    frame.render_widget(Clear, area);

    let cursor_pos = app.command_palette_cursor_position as u16;

    let input = Paragraph::new(app.command_palette_input.as_str())
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.accent))
                .title(" Command Palette (type command...) "),
        );

    frame.render_widget(input, area);

    frame.set_cursor_position(Position {
        x: (area.x + 1 + cursor_pos).min(area.x + area.width.saturating_sub(2)),
        y: area.y + 1,
    });
}
