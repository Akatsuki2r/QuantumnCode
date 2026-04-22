//! Dropdown selector widget for providers and models

use crate::config::themes::RatatuiColors;
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
    pub fn new(
        name: &str,
        display_name: &str,
        requires_api_key: bool,
        default_model: &str,
        models: Vec<String>,
        is_local: bool,
    ) -> Self {
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
    ApiKeyInput,
}

/// A dropdown selector for providers and models
pub struct DropdownSelector {
    pub providers: Vec<ProviderInfo>,
    pub state: DropdownState,
    pub provider_index: usize,
    pub model_index: usize,
    pub selected_provider: Option<String>,
    pub selected_model: Option<String>,
    pub api_key_input: String,
    pub api_key_env_var: String,
    pub pending_provider: Option<String>,
    pub pending_model: Option<String>,
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
            api_key_input: String::new(),
            api_key_env_var: String::new(),
            pending_provider: None,
            pending_model: None,
        }
    }

    /// Open the dropdown — immediately show the provider list
    pub fn open(&mut self) {
        self.state = DropdownState::ProviderSelected;
    }

    /// Close the dropdown
    pub fn close(&mut self) {
        self.state = DropdownState::Closed;
    }

    /// Check if API key is set in environment for current provider
    pub fn check_api_key_set(&self) -> bool {
        if let Some(provider) = self.get_current_provider() {
            if !provider.requires_api_key {
                return true; // Local providers don't need API keys
            }
            let env_var = match provider.name.as_str() {
                "anthropic" => std::env::var("ANTHROPIC_API_KEY").is_ok(),
                "openai" => std::env::var("OPENAI_API_KEY").is_ok(),
                _ => true,
            };
            return env_var;
        }
        false
    }

    /// Get the API key environment variable name for current provider
    pub fn get_api_key_env_name(&self) -> Option<&str> {
        let provider = self.get_current_provider()?;
        if !provider.requires_api_key {
            return None;
        }
        match provider.name.as_str() {
            "anthropic" => Some("ANTHROPIC_API_KEY"),
            "openai" => Some("OPENAI_API_KEY"),
            _ => None,
        }
    }

    fn default_providers() -> Vec<ProviderInfo> {
        // Get detected Ollama models if available
        let ollama_models = Self::get_detected_ollama_models();

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
                ollama_models
                    .first()
                    .map(|s| s.as_str())
                    .unwrap_or("llama3.2"),
                ollama_models.clone(),
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
                vec!["llama3.2".to_string(), "mistral".to_string()],
                true,
            ),
        ]
    }

    /// Get detected Ollama models from local discovery
    fn get_detected_ollama_models() -> Vec<String> {
        // Try to get from router's local model cache first (most comprehensive)
        let models = crate::router::model::get_available_local_models();
        if !models.is_empty() {
            return models;
        }

        // Fallback: Try comprehensive detection directly
        use crate::providers::ollama::OllamaProvider;
        let (names, _, _) = OllamaProvider::detect_models_comprehensive();
        if !names.is_empty() {
            return names;
        }

        // Default hardcoded list if nothing detected
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
        ]
    }

    /// Refresh the list of Ollama models (call after model download/deletion)
    pub fn refresh_ollama_models(&mut self) {
        let ollama_models = Self::get_detected_ollama_models();

        // Update Ollama provider models
        if let Some(ollama_provider) = self.providers.iter_mut().find(|p| p.name == "ollama") {
            ollama_provider.models = ollama_models.clone();
            if ollama_provider.models.is_empty() {
                ollama_provider.default_model = "llama3.2".to_string();
            } else {
                ollama_provider.default_model = ollama_models
                    .first()
                    .map(|s| s.clone())
                    .unwrap_or("llama3.2".to_string());
            }
        }
    }

    /// Select a provider and model by name
    pub fn select(&mut self, provider_name: &str, model_name: &str) {
        if let Some(p_idx) = self.providers.iter().position(|p| p.name == provider_name) {
            self.provider_index = p_idx;
            self.selected_provider = Some(provider_name.to_string());

            if let Some(m_idx) = self.providers[p_idx]
                .models
                .iter()
                .position(|m| m == model_name)
            {
                self.model_index = m_idx;
                self.selected_model = Some(model_name.to_string());
            }
        }
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

    pub fn confirm_selection(&mut self) -> Option<(String, String)> {
        // Check if API key is needed and not set
        if let Some(provider) = self.get_current_provider() {
            if provider.requires_api_key && !self.check_api_key_set() {
                // Store pending selection and move to API key input state
                self.pending_provider = Some(provider.name.clone());
                self.pending_model = self.selected_model.clone();
                self.api_key_env_var = self.get_api_key_env_name().unwrap_or("API_KEY").to_string();
                self.state = DropdownState::ApiKeyInput;
                return None;
            }
        }

        let provider = self
            .selected_provider
            .clone()
            .unwrap_or_else(|| "anthropic".to_string());
        let model = self
            .selected_model
            .clone()
            .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
        self.state = DropdownState::Closed;
        Some((provider, model))
    }

    /// Confirm API key was set (user will set it externally)
    pub fn confirm_api_key_set(&mut self) -> Option<(String, String)> {
        self.state = DropdownState::Closed;
        let provider = self
            .pending_provider
            .take()
            .or_else(|| self.selected_provider.clone())?;
        let model = self
            .pending_model
            .take()
            .or_else(|| self.selected_model.clone())?;
        Some((provider, model))
    }

    pub fn needs_api_key(&self) -> bool {
        self.get_current_provider()
            .map(|p| p.requires_api_key)
            .unwrap_or(false)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, colors: &RatatuiColors) {
        match self.state {
            DropdownState::Closed => {
                // Render collapsed dropdown showing current selection
                let display = match (&self.selected_provider, &self.selected_model) {
                    (Some(p), Some(m)) => format!("{}: {}", p, m),
                    _ => "Select provider... (press P)".to_string(),
                };

                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.accent))
                    .title(" Provider ")
                    .title_style(Style::default().fg(colors.accent).bold());

                let paragraph = Paragraph::new(display.as_str())
                    .style(Style::default().fg(colors.foreground).bg(colors.background))
                    .block(block);

                frame.render_widget(paragraph, area);
            }
            DropdownState::ProviderSelected => {
                self.render_provider_list(frame, area, colors);
            }
            DropdownState::ModelSelected => {
                self.render_model_list(frame, area, colors);
            }
            DropdownState::ApiKeyInput => {
                self.render_api_key_prompt(frame, area, colors);
            }
        }
    }

    fn render_api_key_prompt(&self, frame: &mut Frame, area: Rect, colors: &RatatuiColors) {
        let env_var = &self.api_key_env_var;
        let provider = self.pending_provider.as_deref().unwrap_or("provider");

        let prompt_text = vec![
            Line::from(Span::styled(
                "API Key Required",
                Style::default().fg(Color::Yellow).bold(),
            )),
            Line::default(),
            Line::from(Span::styled(
                format!("{} requires an API key.", provider),
                Style::default().fg(colors.foreground),
            )),
            Line::default(),
            Line::from(Span::styled(
                "Set the environment variable, then press Enter:",
                Style::default().fg(colors.info),
            )),
            Line::default(),
            Line::from(Span::styled(
                format!("  export {}=<your-key>", env_var),
                Style::default().fg(Color::Green).bold(),
            )),
            Line::default(),
            Line::from(Span::styled(
                "Enter → confirm  │  Esc → cancel",
                Style::default().fg(colors.muted),
            )),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" ⚠ API Key Required ")
            .title_style(Style::default().fg(Color::Yellow).bold());

        let paragraph = Paragraph::new(prompt_text)
            .style(Style::default().bg(colors.background).fg(colors.foreground))
            .block(block);

        frame.render_widget(paragraph, area);
    }

    fn render_provider_list(&self, frame: &mut Frame, area: Rect, colors: &RatatuiColors) {
        let items: Vec<ListItem> = self
            .providers
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let icon = if p.is_local { "⬇ Local" } else { "☁ Cloud" };
                let api_badge = if p.requires_api_key {
                    " [API key]"
                } else {
                    "         "
                };
                let cursor = if i == self.provider_index {
                    "▶ "
                } else {
                    "  "
                };
                let content = format!("{}{:<24}{}{}", cursor, p.display_name, api_badge, icon);

                let style = if i == self.provider_index {
                    Style::default().fg(colors.accent).bold()
                } else {
                    Style::default().fg(colors.foreground)
                };

                ListItem::new(Line::from(Span::styled(content, style)))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.accent))
                    .title(" 📡 Select Provider — ↑↓ navigate  Enter select  Esc close ")
                    .title_style(Style::default().fg(colors.accent).bold()),
            )
            .style(Style::default().bg(colors.background).fg(colors.foreground));

        frame.render_widget(list, area);
    }

    fn render_model_list(&self, frame: &mut Frame, area: Rect, colors: &RatatuiColors) {
        if let Some(provider) = self.get_current_provider() {
            let items: Vec<ListItem> = provider
                .models
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let cursor = if i == self.model_index { "▶ " } else { "  " };
                    let style = if i == self.model_index {
                        Style::default().fg(colors.accent).bold()
                    } else {
                        Style::default().fg(colors.foreground)
                    };

                    ListItem::new(Line::from(Span::styled(format!("{}{}", cursor, m), style)))
                })
                .collect();

            let api_suffix = if provider.requires_api_key {
                "  ⚠ API key required"
            } else {
                ""
            };

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(colors.accent))
                        .title(format!(
                            " Models — {}{}  ↩ Enter confirm  ← back ",
                            provider.display_name, api_suffix
                        ))
                        .title_style(Style::default().fg(colors.accent).bold()),
                )
                .style(Style::default().bg(colors.background).fg(colors.foreground));

            frame.render_widget(list, area);
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<DropdownAction> {
        use crossterm::event::{KeyCode, KeyModifiers};

        match self.state {
            DropdownState::Closed => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Enter) | (KeyModifiers::NONE, KeyCode::Char('p')) => {
                    self.state = DropdownState::ProviderSelected;
                    Some(DropdownAction::OpenProviders)
                }
                _ => None,
            },
            DropdownState::ProviderSelected => match (key.modifiers, key.code) {
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
                    Some(DropdownAction::ProviderSelected)
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
            },
            DropdownState::ModelSelected => match (key.modifiers, key.code) {
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
                    match self.confirm_selection() {
                        Some((p, m)) => Some(DropdownAction::Confirmed(p, m)),
                        None => Some(DropdownAction::NeedsApiKey),
                    }
                }
                (KeyModifiers::NONE, KeyCode::Esc) | (KeyModifiers::NONE, KeyCode::Left) => {
                    self.state = DropdownState::ProviderSelected;
                    Some(DropdownAction::BackToProviders)
                }
                _ => None,
            },
            DropdownState::ApiKeyInput => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    // User confirms they've set the API key
                    match self.confirm_api_key_set() {
                        Some((p, m)) => Some(DropdownAction::Confirmed(p, m)),
                        None => Some(DropdownAction::Close),
                    }
                }
                (KeyModifiers::NONE, KeyCode::Esc) => {
                    self.state = DropdownState::Closed;
                    self.pending_provider = None;
                    self.pending_model = None;
                    Some(DropdownAction::Close)
                }
                _ => None,
            },
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
