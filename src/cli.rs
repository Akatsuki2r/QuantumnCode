//! CLI argument parsing using clap

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Quantumn Code - An advanced AI-powered coding assistant CLI
#[derive(Parser, Debug)]
#[command(
    name = "quantumn",
    author,
    version,
    about = "Quantumn Code - AI-powered coding assistant",
    long_about = "
Quantumn Code is an advanced AI coding assistant that helps you write, edit,
and understand code. It supports multiple AI providers (Claude, OpenAI, Ollama)
and provides a beautiful terminal UI for long development sessions.

Examples:
  quantumn                      Start interactive session
  quantumn \"explain this code\"  One-shot query
  quantumn edit src/main.rs     Edit file with AI
  quantumn commit               Generate commit message
  quantumn --theme tokyo_night  Use Tokyo Night theme
"
)]
pub struct Cli {
    /// AI model to use (haiku, sonnet, opus, or specific model name)
    #[arg(short, long, global = true)]
    pub model: Option<String>,

    /// Theme to use (default, tokyo_night, hacker, deep_black, or path to custom theme)
    #[arg(short, long, global = true)]
    pub theme: Option<String>,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start an interactive chat session
    Chat {
        /// Initial prompt to start the conversation
        prompt: Option<String>,

        /// Model to use for this session
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Edit a file with AI assistance
    Edit {
        /// File to edit
        file: PathBuf,

        /// Instructions for the edit
        #[arg(short, long)]
        prompt: Option<String>,

        /// Model to use
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Generate a git commit message with AI
    Commit {
        /// Custom commit message prompt
        #[arg(short, long)]
        message: Option<String>,

        /// Model to use
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Perform AI-powered code review
    Review {
        /// Files to review (defaults to staged changes)
        files: Vec<PathBuf>,

        /// Model to use
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Run tests with AI analysis
    Test {
        /// Path to test (defaults to all tests)
        path: Option<PathBuf>,

        /// Model to use
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Scaffold a new project
    Scaffold {
        /// Project type (rust, python, node, web, etc.)
        project_type: String,

        /// Project name
        name: String,
    },

    /// Manage sessions
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },

    /// Configure settings
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Manage themes
    Theme {
        #[command(subcommand)]
        command: ThemeCommands,
    },

    /// Switch between AI models/providers
    Model {
        /// Provider to use (anthropic, openai, ollama)
        provider: Option<String>,

        /// List available models
        #[arg(short, long)]
        list: bool,
    },

    /// Show current status (model, session, git)
    Status,

    /// Show version information
    Version,
}

/// Session management commands
#[derive(Subcommand, Debug)]
pub enum SessionCommands {
    /// List saved sessions
    List,

    /// Resume a session
    Resume {
        /// Session ID to resume
        id: Option<String>,
    },

    /// Save current session
    Save {
        /// Session name
        name: Option<String>,
    },

    /// Delete a session
    Delete {
        /// Session ID to delete
        id: String,
    },
}

/// Configuration commands
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },

    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// Reset to default configuration
    Reset,

    /// Open configuration file in editor
    Edit,
}

/// Theme commands
#[derive(Subcommand, Debug)]
pub enum ThemeCommands {
    /// List available themes
    List,

    /// Set current theme
    Set {
        /// Theme name
        name: String,
    },

    /// Show current theme
    Current,

    /// Preview a theme
    Preview {
        /// Theme name
        name: String,
    },
}