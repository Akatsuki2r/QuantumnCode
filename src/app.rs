//! Application state management

use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::config::settings::Settings;
use crate::config::themes::Theme;
use crate::providers::Provider;
use crate::rag::{RagConfig, RagIndex};
use crate::router::RouterConfig;
use crate::tui::widgets::{DropdownSelector, KanbanBoard, TabBar};
use ratatui::widgets::ListState;

/// Current mode of the application
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    /// Normal input mode
    Normal,
    /// Editing a file
    Editing,
    /// Reviewing changes
    Review,
    /// Help screen
    Help,
    /// Command palette
    Command,
    /// Provider/model selection
    ProviderSelect,
}

/// A single message in the conversation
#[derive(Debug, Clone)]
pub struct Message {
    /// Role (user, assistant, system)
    pub role: String,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Token count (if available)
    pub tokens: Option<usize>,
}

/// A file in the context
#[derive(Debug, Clone)]
pub struct FileContext {
    /// File path
    pub path: String,
    /// File content
    pub content: String,
    /// Whether file is staged for context
    pub staged: bool,
}

/// Session information
#[derive(Debug, Clone)]
pub struct Session {
    /// Session ID
    pub id: String,
    /// Session name
    pub name: Option<String>,
    /// Creation time
    pub created: DateTime<Utc>,
    /// Last updated
    pub updated: DateTime<Utc>,
    /// Conversation messages
    pub messages: Vec<Message>,
    /// Files in context
    pub files: HashMap<String, FileContext>,
    /// Current provider
    pub provider: String,
    /// Current model
    pub model: String,
}

/// Main application state
pub struct App {
    /// Application settings
    pub settings: Settings,
    /// Current theme
    pub theme: Theme,
    /// Current session
    pub session: Session,
    /// Current mode
    pub mode: Mode,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Current input buffer
    pub input: String,
    /// Cursor position in input
    pub cursor_position: usize,
    /// Scroll position in output
    pub scroll_offset: usize,
    /// Status message
    pub status: Option<String>,
    /// Available providers
    pub providers: Vec<Box<dyn Provider>>,
    /// API key status
    pub api_keys: HashMap<String, bool>,
    /// Dropdown selector for providers/models
    pub dropdown: DropdownSelector,
    /// Tab bar
    pub tab_bar: TabBar,
    /// Kanban board
    pub kanban: KanbanBoard,
    /// Router configuration for automatic model selection
    pub router_config: RouterConfig,
    /// Whether automatic model switching via router is enabled
    pub router_enabled: bool,
    /// Debug log messages (circular buffer, latest 100)
    pub debug_logs: Vec<(std::time::Instant, String)>,
    /// Whether debug console is visible
    pub debug_visible: bool,
    /// State for the debug log list (for auto-scrolling)
    pub debug_state: ListState,
    /// RAG index for context-aware retrieval
    pub rag_index: RagIndex,
}

impl App {
    /// Create a new application instance
    pub fn new(settings: Settings, theme: Theme) -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        Self {
            settings,
            theme,
            session: Session {
                id: session_id,
                name: None,
                created: now,
                updated: now,
                messages: Vec::new(),
                files: HashMap::new(),
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
            },
            mode: Mode::Normal,
            should_quit: false,
            input: String::new(),
            cursor_position: 0,
            scroll_offset: 0,
            status: None,
            providers: Vec::new(),
            api_keys: HashMap::new(),
            dropdown: DropdownSelector::new(),
            tab_bar: TabBar::new(),
            kanban: KanbanBoard::new(),
            router_config: RouterConfig::default(),
            router_enabled: true,
            debug_logs: Vec::new(),
            debug_visible: false,
            debug_state: ListState::default(),
            rag_index: {
                let mut index = RagIndex::new(RagConfig::default());
                // Automatic indexing on startup
                index
            },
        }
        .initialize()
    }

    /// Initialize application state (e.g., startup indexing)
    fn initialize(mut self) -> Self {
        self.debug_log("System: Initializing RAG index...");
        self.index_project_files();
        self
    }

    /// Add a debug log entry
    pub fn debug_log(&mut self, message: &str) {
        use std::time::Instant;
        self.debug_logs.push((Instant::now(), message.to_string()));
        // Keep only last 100 entries
        if self.debug_logs.len() > 100 {
            self.debug_logs.remove(0);
        }

        // Auto-scroll to the bottom
        let last_idx = self.debug_logs.len().saturating_sub(1);
        self.debug_state.select(Some(last_idx));
        tracing::debug!(target: "debug_console", "{}", message);
    }

    /// Toggle debug console visibility
    pub fn toggle_debug(&mut self) {
        self.debug_visible = !self.debug_visible;
    }

    /// Search the RAG index for relevant context
    pub fn search_context(&self, query: &str) -> crate::rag::RagResult {
        self.rag_index.search(query)
    }

    /// Add a file to the RAG index
    pub fn index_file(&mut self, path: String, content: String) {
        self.rag_index.add_document(path, content);
    }

    /// Index all Rust source files in the current project
    pub fn index_project_files(&mut self) {
        use crate::tools::glob::find_files;
        use std::path::Path;

        let base = Path::new(".");
        // Find all Rust source files
        if let Ok(matches) = find_files("src/**/*.rs", base) {
            for path in matches {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let path_str = path.to_string_lossy().to_string();
                    self.index_file(path_str, content);
                }
            }
            tracing::debug!(
                target: "rag",
                "Indexed {} files into RAG",
                self.rag_index.document_count()
            );
        }
    }

    /// Route a prompt through the router and automatically select model
    /// Returns (provider, model) pair based on router decision
    pub fn route_prompt(&self, prompt: &str) -> (String, String) {
        use crate::router::{model::get_model_for_tier_with_local, route};

        if !self.router_enabled {
            // Router disabled, use current selection
            tracing::debug!(
                target: "router",
                "Router disabled, using current selection: provider={}, model={}",
                self.session.provider,
                self.session.model
            );
            return (self.session.provider.clone(), self.session.model.clone());
        }

        // Get routing decision
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/".to_string());

        tracing::debug!(
            target: "router",
            "Routing prompt: length={}, cwd={}",
            prompt.len(),
            cwd
        );

        let decision = route(prompt, &cwd, &self.router_config);

        tracing::info!(
            target: "router",
            "Routing decision: intent={}, complexity={}, mode={}, tier={}, confidence={:.2}",
            decision.intent.as_str(),
            decision.complexity.as_str(),
            decision.mode.as_str(),
            decision.model_tier.as_str(),
            decision.confidence
        );

        // Map model tier to actual provider/model
        // Local tier uses discovered Ollama models, others use cloud providers
        let model = get_model_for_tier_with_local(decision.model_tier);

        // Determine provider based on tier
        let provider = match decision.model_tier {
            crate::router::ModelTier::Local => {
                // Check if we have local models available
                if crate::router::model::has_local_models() {
                    "ollama".to_string()
                } else {
                    // Fall back to fast tier if no local models
                    "anthropic".to_string()
                }
            }
            crate::router::ModelTier::Fast => "anthropic".to_string(),
            crate::router::ModelTier::Standard => "anthropic".to_string(),
            crate::router::ModelTier::Capable => "anthropic".to_string(),
        };

        tracing::info!(
            target: "router",
            "Selected: provider={}, model={}",
            provider,
            model
        );

        (provider, model)
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, role: &str, content: &str) {
        self.session.messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            tokens: None,
        });
        self.session.updated = Utc::now();
    }

    /// Get the last message
    pub fn last_message(&self) -> Option<&Message> {
        self.session.messages.last()
    }

    /// Clear the conversation
    pub fn clear_conversation(&mut self) {
        self.session.messages.clear();
        self.session.files.clear();
        self.scroll_offset = 0;
    }

    /// Add a file to context
    pub fn add_file(&mut self, path: &str, content: &str) {
        self.session.files.insert(
            path.to_string(),
            FileContext {
                path: path.to_string(),
                content: content.to_string(),
                staged: true,
            },
        );
    }

    /// Remove a file from context
    pub fn remove_file(&mut self, path: &str) {
        self.session.files.remove(path);
    }

    /// Toggle file staged status
    pub fn toggle_file(&mut self, path: &str) {
        if let Some(file) = self.session.files.get_mut(path) {
            file.staged = !file.staged;
        }
    }

    /// Set the current mode
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Set status message
    pub fn set_status(&mut self, status: Option<String>) {
        self.status = status;
    }

    /// Get status message
    pub fn get_status(&self) -> Option<&String> {
        self.status.as_ref()
    }

    /// Get total tokens used in session
    pub fn total_tokens(&self) -> usize {
        self.session.messages.iter().filter_map(|m| m.tokens).sum()
    }

    /// Estimate tokens for a string (rough approximation)
    pub fn estimate_tokens(text: &str) -> usize {
        // Rough approximation: ~4 characters per token
        text.len() / 4
    }
}

impl Session {
    /// Create a new session
    pub fn new() -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        Self {
            id: session_id,
            name: None,
            created: now,
            updated: now,
            messages: Vec::new(),
            files: HashMap::new(),
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
        }
    }

    /// Create a named session
    pub fn with_name(name: String) -> Self {
        let mut session = Self::new();
        session.name = Some(name);
        session
    }
}
