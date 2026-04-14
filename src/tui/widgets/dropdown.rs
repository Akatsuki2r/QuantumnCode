//! Dropdown selector widget for providers and models

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Provider info for display
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub display_name: String,
    pub requires_api_key: bool,
    pub default_model: String,
    pub models: Vec<String>,
    pub is_local: bool,
}

impl ProviderInfo {
    pub fn new(name: &str, display_name: &str, requires_api_key: bool, default_model: &str, models: Vec<String>, is_local: bool) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
            requires_api_key,
            default_model: default_model.to_string(),
            models,
            is_local,
        }
    }
}

/// Dropdown state
#[derive(Debug, Clone)]
pub enum DropdownState {
    Closed,
    ProviderSelected,
    ModelSelected,
}

/// A dropdown selector for providers and models
pub struct DropdownSelector {
    pub providers: Vec<ProviderInfo>,
    pub state: DropdownState,
    pub provider_index: usize,
    pub model_index: usize,
    pub selected_provider: Option<String>,
    pub selected_model: Option<String>,
    pub show_api_key_prompt: bool,
    pub api_key_input: String,
}

impl DropdownSelector {
    pub fn new() -> Self {
        Self {
            providers: Self::default_providers(),
            state: DropdownState::Closed,
            provider_index: 0,
            model_index: 0,
            selected_provider: None,
            selected_model: None,
            show_api_key_prompt: false,
            api_key_input: String::new(),
        }
    }

    fn default_providers() -> Vec<ProviderInfo> {
        vec![
            ProviderInfo::new(
                "anthropic",
                "Anthropic (Cloud)",
                true,
                "claude-sonnet-4-20250514",
                vec![
                    "claude-opus-4-20250514".to_string(),
                    "claude-sonnet-4-20250514".to_string(),
                    "claude-haiku-4-20250514".to_string(),
                    "claude-3-5-sonnet-20241022".to_string(),
                    "claude-3-5-haiku-20241022".to_string(),
                ],
                false,
            ),
            ProviderInfo::new(
                "openai",
                "OpenAI (Cloud)",
                true,
                "gpt-4o",
                vec![
                    "gpt-4o".to_string(),
                    "gpt-4o-mini".to_string(),
                    "gpt-4-turbo".to_string(),
                    "o1".to_string(),
                    "o1-mini".to_string(),
                ],
                false,
            ),
            ProviderInfo::new(
                "ollama",
                "Ollama (Local)",
                false,
                "llama3.2",
                vec![
                    "llama3.2".to_string(),
                    "llama3.1".to_string(),
                    "llama3".to_string(),
                    "mistral".to_string(),
                    "codellama".to_string(),
                    "deepseek-coder".to_string(),
                    "qwen2.5-coder".to_string(),
                    "phi3".to_string(),
                    "gemma2".to_string(),
                ],
                true,
            ),
            ProviderInfo::new(
                "lm_studio",
                "LM Studio (Local)",
                false,
                "llama3.2",
                vec![
                    "llama3.2".to_string(),
                    "mistral".to_string(),
                    "codellama".to_string(),
                ],
                true,
            ),
            ProviderInfo::new(
                "llama_cpp",
                "llama.cpp (Local)",
                false,
                "llama3.2",
                vec![
                    "llama3.2".to_string(),
                    "mistral".to_string(),
                ],
                true,
            ),
        ]
    }

    pub fn get_current_provider(&self) -> Option<&ProviderInfo> {
        self.providers.get(self.provider_index)
    }

    pub fn select_provider(&mut self, index: usize) {
        if index < self.providers.len() {
            let default_model = self.providers[index].default_model.clone();
            self.provider_index = index;
            self.selected_provider = Some(self.providers[index].name.clone());
            self.model_index = 0;
            self.selected_model = Some(default_model);
            self.state = DropdownState::ModelSelected;
        }
    }

    pub fn select_model(&mut self, index: usize) {
        // Get the current provider and models outside of the borrow
        let models: Vec<String>;
        let provider_opt;

        {
            provider_opt = self.providers.get(self.provider_index);
            if let Some(p) = provider_opt {
                models = p.models.clone();
            } else {
                return;
            }
        }

        if index < models.len() {
            self.model_index = index;
            self.selected_model = Some(models[index].clone());
        }
    }

    pub fn confirm_selection(&mut self) -> (String, String) {
        let provider = self.selected_provider.clone().unwrap_or_else(|| "anthropic".to_string());
        let model = self.selected_model.clone().unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
        self.state = DropdownState::Closed;
        (provider, model)
    }

    pub fn needs_api_key(&self) -> bool {
        self.get_current_provider()
            .map(|p| p.requires_api_key)
            .unwrap_or(false)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        match self.state {
            DropdownState::Closed => {
                // Render collapsed dropdown showing current selection
                let display = match (&self.selected_provider, &self.selected_model) {
                    (Some(p), Some(m)) => format!("{}: {}", p, m),
                    _ => "Select provider...".to_string(),
                };

                let style = if focused {
                    Style::default()
                        .fg(Color::Cyan)
                        .bold()
                } else {
                    Style::default().fg(Color::White)
                };

                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(if focused { Color::Cyan } else { Color::Gray }))
                    .title(" Provider ");

                let paragraph = Paragraph::new(display.as_str())
                    .style(style)
                    .block(block);

                frame.render_widget(paragraph, area);
            }
            DropdownState::ProviderSelected => {
                // Render provider list
                self.render_provider_list(frame, area);
            }
            DropdownState::ModelSelected => {
                // Render model list
                self.render_model_list(frame, area);
            }
        }
    }

    fn render_provider_list(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.providers.iter().enumerate().map(|(i, p)| {
            let icon = if p.is_local { "[L]" } else { "[C]" };
            let api_note = if p.requires_api_key { " *" } else { "" };
            let selected = if i == self.provider_index { " > " } else { "   " };
            let content = format!("{}{} {}{}", selected, icon, p.display_name, api_note);

            let style = if i == self.provider_index {
                Style::default()
                    .fg(Color::Cyan)
                    .bold()
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(Span::styled(content, style)))
        }).collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Select Provider (↑↓ to navigate, Enter to select, Esc to close) ")
                    .title_style(Style::default().fg(Color::Cyan).bold()),
            )
            .style(Style::default().bg(Color::Reset));

        frame.render_widget(list, area);
    }

    fn render_model_list(&self, frame: &mut Frame, area: Rect) {
        if let Some(provider) = self.get_current_provider() {
            let items: Vec<ListItem> = provider.models.iter().enumerate().map(|(i, m)| {
                let selected = if i == self.model_index { " > " } else { "   " };
                let style = if i == self.model_index {
                    Style::default()
                        .fg(Color::Cyan)
                        .bold()
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Line::from(Span::styled(format!("{}{}", selected, m), style)))
            }).collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan))
                        .title(format!(" Models for {} (↑↓ to navigate, Enter to confirm, ← Back) ", provider.display_name))
                        .title_style(Style::default().fg(Color::Cyan).bold()),
                )
                .style(Style::default().bg(Color::Reset));

            frame.render_widget(list, area);

            // Show API key note if needed
            if provider.requires_api_key {
                let note = "⚠ API key required. Set ANTHROPIC_API_KEY or OPENAI_API_KEY";
                let note_para = Paragraph::new(note)
                    .style(Style::default().fg(Color::Yellow));
                let note_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
                frame.render_widget(note_para, note_area);
            }
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<DropdownAction> {
        use crossterm::event::{KeyCode, KeyModifiers};

        match self.state {
            DropdownState::Closed => {
                match (key.modifiers, key.code) {
                    (KeyModifiers::NONE, KeyCode::Enter) | (KeyModifiers::NONE, KeyCode::Char('p')) => {
                        self.state = DropdownState::ProviderSelected;
                        Some(DropdownAction::OpenProviders)
                    }
                    _ => None,
                }
            }
            DropdownState::ProviderSelected => {
                match (key.modifiers, key.code) {
                    (KeyModifiers::NONE, KeyCode::Up) => {
                        if self.provider_index > 0 {
                            self.provider_index -= 1;
                        }
                        Some(DropdownAction::Navigate)
                    }
                    (KeyModifiers::NONE, KeyCode::Down) => {
                        if self.provider_index < self.providers.len() - 1 {
                            self.provider_index += 1;
                        }
                        Some(DropdownAction::Navigate)
                    }
                    (KeyModifiers::NONE, KeyCode::Enter) => {
                        self.select_provider(self.provider_index);
                        if self.needs_api_key() {
                            self.show_api_key_prompt = true;
                            Some(DropdownAction::NeedsApiKey)
                        } else {
                            Some(DropdownAction::ProviderSelected)
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Esc) => {
                        self.state = DropdownState::Closed;
                        Some(DropdownAction::Close)
                    }
                    (KeyModifiers::NONE, KeyCode::Left) => {
                        self.state = DropdownState::Closed;
                        Some(DropdownAction::Close)
                    }
                    _ => None,
                }
            }
            DropdownState::ModelSelected => {
                match (key.modifiers, key.code) {
                    (KeyModifiers::NONE, KeyCode::Up) => {
                        if self.model_index > 0 {
                            self.model_index -= 1;
                        }
                        Some(DropdownAction::Navigate)
                    }
                    (KeyModifiers::NONE, KeyCode::Down) => {
                        if let Some(provider) = self.get_current_provider() {
                            if self.model_index < provider.models.len() - 1 {
                                self.model_index += 1;
                            }
                        }
                        Some(DropdownAction::Navigate)
                    }
                    (KeyModifiers::NONE, KeyCode::Enter) => {
                        self.select_model(self.model_index);
                        let (p, m) = self.confirm_selection();
                        Some(DropdownAction::Confirmed(p, m))
                    }
                    (KeyModifiers::NONE, KeyCode::Esc) | (KeyModifiers::NONE, KeyCode::Left) => {
                        self.state = DropdownState::ProviderSelected;
                        Some(DropdownAction::BackToProviders)
                    }
                    _ => None,
                }
            }
        }
    }
}

impl Default for DropdownSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Actions from dropdown interaction
#[derive(Debug, Clone)]
pub enum DropdownAction {
    OpenProviders,
    ProviderSelected,
    NeedsApiKey,
    Navigate,
    Confirmed(String, String),
    Close,
    BackToProviders,
}
