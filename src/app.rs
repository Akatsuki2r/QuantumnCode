//! Application state management

use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::config::settings::Settings;
use crate::config::themes::Theme;
use crate::providers::Provider;

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
        }
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

    /// Get total tokens used in session
    pub fn total_tokens(&self) -> usize {
        self.session
            .messages
            .iter()
            .filter_map(|m| m.tokens)
            .sum()
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