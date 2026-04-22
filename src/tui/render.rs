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

    // Create main layout with tabs
    let chunks = Layout::vertical([
        Constraint::Length(3), // Tab bar
        Constraint::Length(1), // Status bar
        Constraint::Min(1),    // Main content
        Constraint::Length(3), // Input
        Constraint::Length(1), // Suggestion bar
    ])
    .split(frame.area());

    // Render tab bar
    app.tab_bar.render_sleek(frame, chunks[0]);

    // Render based on active tab
    match app.tab_bar.active_index {
        1 => render_files_tab(frame, chunks[2], app, &colors),
        2 => render_builder_tab(frame, chunks[2], app, &colors),
        3 => render_debug_tab(frame, chunks[2], app, &colors),
        4 => render_settings_tab(frame, chunks[2], app, &colors),
        _ => {
            // Always render chat underneath for context
            render_status_bar(frame, chunks[1], app, &colors);
            match app.mode {
                Mode::ProviderSelect => render_chat(frame, chunks[2], app, &colors),
                Mode::Normal => render_chat(frame, chunks[2], app, &colors),
                Mode::Help => render_help(frame, chunks[2], app, &colors),
                Mode::Editing => render_editor(frame, chunks[2], app, &colors),
                Mode::Review => render_review(frame, chunks[2], app, &colors),
                Mode::Command => render_command_palette(frame, chunks[2], app, &colors),
            }
        }
    }

    // Render input (not on settings tab)
    if app.tab_bar.active_index != 4 {
        render_input(frame, chunks[3], app, &colors);
        render_suggestions(frame, chunks[4], app, &colors);
    }

    // Render dropdown as a SINGLE centered modal overlay — only when active
    if matches!(app.mode, Mode::ProviderSelect) {
        render_dropdown_overlay(frame, app, &colors);
    }
}

/// Render the dropdown overlay for provider/model selection — single render only
fn render_dropdown_overlay(
    frame: &mut Frame,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    // Dim / darken the background by drawing a translucent block
    let full = frame.area();
    let dim_block = Block::default().style(Style::default().bg(colors.background));
    frame.render_widget(dim_block, full);

    // Centered modal — width 58 cols, height adapts to content
    let area = center_rect(58, 18, frame.area());

    // Outer modal shell — themed
    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.accent))
        .style(Style::default().bg(colors.background).fg(colors.foreground));
    frame.render_widget(modal_block, area);

    // Inner area with 1-cell padding
    let inner = Rect::new(
        area.x + 1,
        area.y + 1,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );
    app.dropdown.render(frame, inner, colors);
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
    let router_indicator = if app.router_enabled {
        Span::styled(
            " [AUTO] ",
            Style::default().fg(colors.success).bg(colors.background),
        )
    } else {
        Span::styled(
            " [MANUAL] ",
            Style::default().fg(colors.muted).bg(colors.background),
        )
    };

    let rag_indicator = Span::styled(
        format!(" [RAG: {}] ", app.rag_index.document_count()),
        Style::default().fg(colors.info).bg(colors.background),
    );

    // Get the current activity status from the app
    // This shows if the model is [THINKING], [ROUTING], etc.
    let activity_status = if let Some(status) = app.get_status() {
        Span::styled(
            format!(" ◉ {} ", status),
            Style::default().fg(colors.accent).bold(),
        )
    } else {
        Span::raw("")
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(
            " ⚡ QUANTUMN ",
            Style::default()
                .fg(colors.accent)
                .bg(colors.background)
                .bold(),
        ),
        Span::styled(
            format!(" 󰘦 {} ", app.session.model),
            Style::default().fg(colors.foreground),
        ),
        Span::styled(
            format!(" ({}) ", app.session.provider),
            Style::default().fg(colors.muted),
        ),
        router_indicator,
        rag_indicator,
        activity_status,
        Span::raw(" | "),
        Span::styled(
            format!(" 󰌓 {} tokens ", app.total_tokens()),
            Style::default().fg(colors.info),
        ),
        Span::styled(
            match app.mode {
                Mode::Normal => " NORMAL ",
                Mode::Editing => " EDIT ",
                Mode::Review => " REVIEW ",
                Mode::Help => " HELP ",
                Mode::Command => " COMMAND ",
                Mode::ProviderSelect => " SELECT ",
            },
            Style::default()
                .fg(colors.background)
                .bg(colors.accent)
                .bold(),
        ),
    ]))
    .style(Style::default().bg(colors.background));

    frame.render_widget(status, area);
}

/// Render the chat area
fn render_chat(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    // Build list of messages
    let messages: Vec<Line> = app
        .session
        .messages
        .iter()
        .flat_map(|msg| {
            let role_style = match msg.role.as_str() {
                "user" => Style::default().fg(colors.accent).bold(),
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
        .scroll((app.scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);
}

/// Render the input area
fn render_input(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
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
    let cursor_x = (area.x + 1 + app.cursor_position as u16).min(area.x + area.width - 2);
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
            Style::default().fg(colors.accent).bold(),
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
        Line::from("  F2          - Toggle file tree"),
        Line::from("  F3          - Toggle token count"),
        Line::from("  F4          - Change theme"),
        Line::from("  /           - Command palette"),
        Line::from("  ←→         - Switch tabs"),
        Line::from("  P           - Open provider selector"),
        Line::default(),
        Line::from(Span::styled(
            "Commands:",
            Style::default().fg(colors.secondary).bold(),
        )),
        Line::from("  /help       - Show help"),
        Line::from("  /clear      - Clear conversation"),
        Line::from("  /model      - List/change models"),
        Line::from("  /ollama     - List local Ollama models"),
        Line::from("  /router     - Toggle automatic model switching"),
        Line::from("  /theme      - Change theme"),
        Line::from("  /provider   - List/change providers"),
        Line::from("  /commit     - Generate commit"),
        Line::from("  /review     - Review code"),
        Line::from("  /test       - Run tests"),
        Line::from("  /status     - Show status"),
        Line::from("  /quit       - Exit"),
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

/// Render editor mode
fn render_editor(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
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
fn render_review(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
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
fn render_command_palette(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
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

// render_provider_select removed — dropdown is now rendered exclusively
// through render_dropdown_overlay as a single centered modal.

/// Render files tab with improved styling
fn render_files_tab(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let mut items = Vec::new();
    for (path, file) in &app.session.files {
        let status = if file.staged { "●" } else { "○" };
        let style = if file.staged {
            Style::default().fg(colors.success)
        } else {
            Style::default().fg(colors.muted)
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!(" {} ", status), style),
            Span::styled(path.clone(), Style::default().fg(colors.foreground)),
        ])));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Context Files ")
                .title_style(Style::default().fg(colors.accent)),
        )
        .highlight_style(Style::default().bg(colors.secondary).fg(colors.background));

    if app.session.files.is_empty() {
        let empty = Paragraph::new(
            "\n  No files in context.\n  Type '/add <file>' to include code for AI analysis.",
        )
        .style(Style::default().fg(colors.muted))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border)),
        );
        frame.render_widget(empty, area);
    } else {
        frame.render_widget(list, area);
    }
}

/// Render Debug tab to show what's happening under the hood
fn render_debug_tab(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let logs: Vec<ListItem> = app
        .debug_logs
        .iter()
        .map(|(time, msg)| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {:<10} ", format!("{:?}", time.elapsed().as_secs())),
                    Style::default().fg(colors.muted),
                ),
                Span::styled(" » ", Style::default().fg(colors.accent)),
                Span::raw(msg),
            ]))
        })
        .collect();

    let list = List::new(logs)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(Span::styled(
                    "  DEBUG CONSOLE ",
                    Style::default().fg(colors.accent).bold(),
                ))
                .padding(Padding::new(1, 1, 0, 0)),
        )
        .style(Style::default().fg(colors.foreground).bg(colors.background))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    // Render with state to enable auto-scrolling
    frame.render_stateful_widget(list, area, &mut app.debug_state.clone());
}

/// Render Builder tab (Enhanced Kanban IDE view)
fn render_builder_tab(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)]).split(area);

    // Kanban Board in main area
    app.kanban.render(frame, chunks[0]);

    // Sidebar for build info
    let info_text = vec![
        Line::from(Span::styled(
            " BUILD STATUS ",
            Style::default()
                .bg(colors.accent)
                .fg(colors.background)
                .bold(),
        )),
        Line::default(),
        Line::from(vec![
            Span::styled(" Mode: ", Style::default().fg(colors.muted)),
            Span::raw("Build (Agentive)"),
        ]),
        Line::from(vec![
            Span::styled(" Task: ", Style::default().fg(colors.muted)),
            Span::raw(app.status.as_deref().unwrap_or("Idle")),
        ]),
        Line::default(),
        Line::from(Span::styled(
            " ACTIVE TOOLS ",
            Style::default().fg(colors.secondary).bold(),
        )),
        Line::from(" • file_read"),
        Line::from(" • file_edit"),
        Line::from(" • shell_exec"),
    ];

    let sidebar = Paragraph::new(info_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .padding(Padding::new(1, 1, 1, 1)),
        )
        .style(Style::default().bg(colors.background));

    frame.render_widget(sidebar, chunks[1]);
}

/// Render settings tab with improved table layout
fn render_settings_tab(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    colors: &crate::config::themes::RatatuiColors,
) {
    let total_tokens_label = app.total_tokens().to_string();
    let rows = vec![
        Row::new(vec!["Provider", app.session.provider.as_str()]),
        Row::new(vec!["Model", app.session.model.as_str()]),
        Row::new(vec!["Theme", app.settings.ui.theme.as_str()]),
        Row::new(vec![
            "Router",
            if app.router_enabled {
                "Enabled"
            } else {
                "Disabled"
            },
        ]),
        Row::new(vec!["Total Tokens", total_tokens_label.as_str()]),
    ];

    let table = Table::new(rows, [Constraint::Length(15), Constraint::Min(20)])
        .header(Row::new(vec!["Setting", "Value"]).style(Style::default().fg(colors.accent).bold()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .title(" Configuration ")
                .title_style(Style::default().fg(colors.accent))
                .padding(Padding::uniform(1)),
        )
        .style(Style::default().fg(colors.foreground).bg(colors.background));

    frame.render_widget(table, area);
}
