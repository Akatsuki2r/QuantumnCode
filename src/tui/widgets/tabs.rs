//! Tab widget with modern styling

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Tab definition
#[derive(Debug, Clone)]
pub struct TabItem {
    pub title: String,
    pub content: String,
    pub icon: String,
}

/// Tab bar state
#[derive(Debug, Clone)]
pub struct TabBar {
    pub tabs: Vec<TabItem>,
    pub active_index: usize,
    pub show_icons: bool,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            tabs: Self::default_tabs(),
            active_index: 0,
            show_icons: true,
        }
    }

    fn default_tabs() -> Vec<TabItem> {
        vec![
            TabItem {
                title: "Chat".to_string(),
                content: String::new(),
                icon: "💬".to_string(),
            },
            TabItem {
                title: "Files".to_string(),
                content: String::new(),
                icon: "📁".to_string(),
            },
            TabItem {
                title: "Kanban".to_string(),
                content: String::new(),
                icon: "📋".to_string(),
            },
            TabItem {
                title: "Settings".to_string(),
                content: String::new(),
                icon: "⚙️".to_string(),
            },
        ]
    }

    pub fn get_active_tab(&self) -> Option<&TabItem> {
        self.tabs.get(self.active_index)
    }

    pub fn get_active_mut(&mut self) -> Option<&mut TabItem> {
        self.tabs.get_mut(self.active_index)
    }

    pub fn select(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_index = index;
        }
    }

    pub fn next(&mut self) {
        self.active_index = (self.active_index + 1) % self.tabs.len();
    }

    pub fn previous(&mut self) {
        if self.active_index > 0 {
            self.active_index -= 1;
        } else {
            self.active_index = self.tabs.len() - 1;
        }
    }

    /// Render the tab bar with sleek custom styling
    pub fn render_sleek(&self, frame: &mut Frame, area: Rect) {
        // Create custom tab rendering for a more modern look
        let width = area.width as usize;
        let total_tabs = self.tabs.len();
        let tab_width = width / total_tabs;

        let items: Vec<Line> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let is_active = i == self.active_index;
                let style = if is_active {
                    Style::default().fg(Color::Cyan).bold()
                } else {
                    Style::default().fg(Color::Gray)
                };

                let label = format!(" {} {} ", t.icon, t.title);
                let padded: String = if label.len() < tab_width {
                    let padding = tab_width - label.len();
                    let left_pad = padding / 2;
                    let right_pad = padding - left_pad;
                    format!("{}{}{}", " ".repeat(left_pad), label, " ".repeat(right_pad))
                } else {
                    label
                };

                Line::from(Span::styled(padded, style))
            })
            .collect();

        // Draw tabs
        let tabs_para = Paragraph::new(items).style(Style::default().bg(Color::Black));

        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(area);

        // Draw top accent line
        let top_border = Paragraph::new("").style(Style::default().bg(Color::Cyan));
        frame.render_widget(top_border, Rect::new(area.x, area.y, area.width, 1));

        // Draw tabs
        frame.render_widget(tabs_para, chunks[1]);

        // Draw bottom indicator for active tab
        let indicator_width = tab_width as u16;
        let indicator_x = area.x + (self.active_index as u16 * indicator_width);
        let indicator = Paragraph::new("").style(Style::default().fg(Color::Cyan));
        frame.render_widget(
            indicator,
            Rect::new(indicator_x, area.y, indicator_width, 1),
        );
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<TabAction> {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Left) => {
                self.previous();
                Some(TabAction::Changed(self.active_index))
            }
            (KeyModifiers::NONE, KeyCode::Right) => {
                self.next();
                Some(TabAction::Changed(self.active_index))
            }
            (KeyModifiers::NONE, KeyCode::Char('1')) => {
                self.select(0);
                Some(TabAction::Changed(self.active_index))
            }
            (KeyModifiers::NONE, KeyCode::Char('2')) => {
                self.select(1);
                Some(TabAction::Changed(self.active_index))
            }
            (KeyModifiers::NONE, KeyCode::Char('3')) => {
                self.select(2);
                Some(TabAction::Changed(self.active_index))
            }
            (KeyModifiers::NONE, KeyCode::Char('4')) => {
                self.select(3);
                Some(TabAction::Changed(self.active_index))
            }
            _ => None,
        }
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Tab actions
#[derive(Debug, Clone)]
pub enum TabAction {
    Changed(usize),
}

/// Kanban board widget
#[derive(Debug, Clone)]
pub struct KanbanColumn {
    pub title: String,
    pub color: Color,
    pub cards: Vec<KanbanCard>,
}

#[derive(Debug, Clone)]
pub struct KanbanCard {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    pub fn color(&self) -> Color {
        match self {
            Priority::Low => Color::Green,
            Priority::Medium => Color::Yellow,
            Priority::High => Color::Red,
            Priority::Critical => Color::Magenta,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Priority::Low => "○",
            Priority::Medium => "◐",
            Priority::High => "●",
            Priority::Critical => "✕",
        }
    }
}

#[derive(Debug, Clone)]
pub struct KanbanBoard {
    pub columns: Vec<KanbanColumn>,
    pub selected_column: usize,
    pub selected_card: usize,
}

impl KanbanBoard {
    pub fn new() -> Self {
        Self {
            columns: vec![
                KanbanColumn {
                    title: "To Do".to_string(),
                    color: Color::Gray,
                    cards: vec![
                        KanbanCard {
                            id: "1".to_string(),
                            title: "Implement dropdown".to_string(),
                            description: "Add provider/model selector".to_string(),
                            priority: Priority::High,
                        },
                        KanbanCard {
                            id: "2".to_string(),
                            title: "Add RAG support".to_string(),
                            description: "Implement vector store".to_string(),
                            priority: Priority::Medium,
                        },
                    ],
                },
                KanbanColumn {
                    title: "In Progress".to_string(),
                    color: Color::Blue,
                    cards: vec![KanbanCard {
                        id: "3".to_string(),
                        title: "Tabs UI".to_string(),
                        description: "Modern tab interface".to_string(),
                        priority: Priority::High,
                    }],
                },
                KanbanColumn {
                    title: "Done".to_string(),
                    color: Color::Green,
                    cards: vec![KanbanCard {
                        id: "4".to_string(),
                        title: "Router".to_string(),
                        description: "7-layer policy engine".to_string(),
                        priority: Priority::Low,
                    }],
                },
            ],
            selected_column: 0,
            selected_card: 0,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let column_width = area.width as usize / self.columns.len();
        let column_width = column_width as u16;

        for (i, column) in self.columns.iter().enumerate() {
            let x = area.x + (i as u16 * column_width);
            let col_area = Rect::new(x, area.y, column_width - 1, area.height);

            // Column header background
            let header_bg = Block::default()
                .style(Style::default().bg(Color::DarkGray))
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT);

            let header_area = Rect::new(col_area.x, col_area.y, col_area.width, 1);
            frame.render_widget(header_bg, header_area);

            // Column header text
            let header = Paragraph::new(Line::from(vec![
                Span::styled(" ", Style::default().fg(Color::Reset)),
                Span::styled(&column.title, Style::default().fg(Color::White).bold()),
                Span::styled(
                    format!(" ({})", column.cards.len()),
                    Style::default().fg(Color::Gray),
                ),
            ]))
            .style(Style::default().bg(Color::DarkGray));
            frame.render_widget(header, header_area);

            // Cards
            let card_area = Rect::new(
                col_area.x,
                col_area.y + 1,
                col_area.width,
                col_area.height - 1,
            );
            let cards_text: Vec<Line> = column
                .cards
                .iter()
                .enumerate()
                .map(|(j, card)| {
                    let is_selected = i == self.selected_column && j == self.selected_card;
                    let style = if is_selected {
                        Style::default().fg(Color::Cyan).bold()
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let priority_style = Style::default().fg(card.priority.color()).bold();

                    Line::from(vec![
                        Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                        Span::styled(card.priority.icon(), priority_style),
                        Span::styled(" ", Style::default()),
                        Span::styled(&card.title, style),
                    ])
                })
                .collect();

            let cards_para = Paragraph::new(cards_text)
                .wrap(Wrap { trim: true })
                .scroll((self.selected_card as u16, 0));

            frame.render_widget(cards_para, card_area);
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<KanbanAction> {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Left) => {
                if self.selected_column > 0 {
                    self.selected_column -= 1;
                    self.selected_card = 0;
                }
                Some(KanbanAction::Moved)
            }
            (KeyModifiers::NONE, KeyCode::Right) => {
                if self.selected_column < self.columns.len() - 1 {
                    self.selected_column += 1;
                    self.selected_card = 0;
                }
                Some(KanbanAction::Moved)
            }
            (KeyModifiers::NONE, KeyCode::Up) => {
                if self.selected_card > 0 {
                    self.selected_card -= 1;
                }
                Some(KanbanAction::Moved)
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                if let Some(col) = self.columns.get(self.selected_column) {
                    if self.selected_card < col.cards.len() - 1 {
                        self.selected_card += 1;
                    }
                }
                Some(KanbanAction::Moved)
            }
            (KeyModifiers::NONE, KeyCode::Enter) => Some(KanbanAction::SelectedCard),
            _ => None,
        }
    }
}

impl Default for KanbanBoard {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum KanbanAction {
    Moved,
    SelectedCard,
}
