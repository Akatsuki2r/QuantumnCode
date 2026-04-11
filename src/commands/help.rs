//! Help command - Comprehensive command documentation and usage guide
//! With colored output for better readability

use color_eyre::eyre::Result;

// ANSI color codes
const CYAN: &str = "\x1b[96m";
const GREEN: &str = "\x1b[92m";
const YELLOW: &str = "\x1b[93m";
const MAGENTA: &str = "\x1b[95m";
const BLUE: &str = "\x1b[94m";
const RED: &str = "\x1b[91m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// Command information structure
pub struct CommandInfo {
    pub name: &'static str,
    pub short_desc: &'static str,
    pub long_desc: &'static str,
    pub usage: &'static str,
    pub examples: &'static [&'static str],
    pub aliases: &'static [&'static str],
}

/// Get all available commands with descriptions
pub fn get_commands() -> Vec<CommandInfo> {
    vec![
        CommandInfo {
            name: "chat",
            short_desc: "Start interactive chat or one-shot query",
            long_desc: "Chat with the AI assistant. Can be used interactively (no arguments) or for one-shot queries with a prompt.",
            usage: "quantumn chat [PROMPT] [--model MODEL]",
            examples: &[
                "quantumn chat                              # Start interactive session",
                "quantumn chat \"Explain this function\"      # One-shot query",
                "quantumn chat --model claude-opus           # Use specific model",
            ],
            aliases: &["c", "ask"],
        },
        CommandInfo {
            name: "edit",
            short_desc: "Edit a file with AI assistance",
            long_desc: "Open a file for AI-assisted editing. Provide instructions and the AI will modify the file accordingly.",
            usage: "quantumn edit <FILE> [--prompt PROMPT] [--model MODEL]",
            examples: &[
                "quantumn edit src/main.rs                  # Interactive edit",
                "quantumn edit src/main.rs --prompt \"Add error handling\"",
                "quantumn edit config.toml --model gpt-4o",
            ],
            aliases: &["e", "modify"],
        },
        CommandInfo {
            name: "review",
            short_desc: "AI-powered code review",
            long_desc: "Perform comprehensive code review on specified files or staged changes. Analyzes code quality, security, and best practices.",
            usage: "quantumn review [FILES...] [--model MODEL]",
            examples: &[
                "quantumn review                            # Review staged changes",
                "quantumn review src/lib.rs                 # Review specific file",
                "quantumn review src/**/*.rs                # Review multiple files",
            ],
            aliases: &["r", "code-review"],
        },
        CommandInfo {
            name: "test",
            short_desc: "Run tests with AI analysis",
            long_desc: "Execute tests and get AI-powered analysis of results, failures, and suggestions for improvement.",
            usage: "quantumn test [PATH] [--model MODEL]",
            examples: &[
                "quantumn test                              # Run all tests",
                "quantumn test tests/unit.rs                # Run specific test file",
                "quantumn test --model claude-opus          # Use specific model",
            ],
            aliases: &["t"],
        },
        CommandInfo {
            name: "agent",
            short_desc: "Run agentic tasks (Bear mode)",
            long_desc: "Execute autonomous agentic workflows with tool calling. The AI agent can read/write files, run commands, and accomplish complex tasks.",
            usage: "quantumn agent <TASK> [--model MODEL]",
            examples: &[
                "quantumn agent \"Refactor the error handling\"",
                "quantumn agent \"Add unit tests for auth module\"",
                "quantumn agent \"Fix all clippy warnings\" --model claude-opus",
            ],
            aliases: &["bear"],
        },
        CommandInfo {
            name: "scaffold",
            short_desc: "Create new project from templates",
            long_desc: "Generate new project scaffolding from templates. Supports multiple languages and frameworks.",
            usage: "quantumn scaffold [TEMPLATE] [--name NAME]",
            examples: &[
                "quantumn scaffold                          # List templates",
                "quantumn scaffold rust-cli --name my-app   # Create Rust CLI",
                "quantumn scaffold react --name my-frontend # Create React app",
            ],
            aliases: &["s", "new", "create"],
        },
        CommandInfo {
            name: "session",
            short_desc: "Manage conversation sessions",
            long_desc: "Save, load, list, and manage conversation sessions for resuming work later.",
            usage: "quantumn session <COMMAND>",
            examples: &[
                "quantumn session list                      # List saved sessions",
                "quantumn session save feature-x            # Save current session",
                "quantumn session resume feature-x          # Resume session",
                "quantumn session delete feature-x          # Delete session",
            ],
            aliases: &["sess"],
        },
        CommandInfo {
            name: "config",
            short_desc: "Manage configuration settings",
            long_desc: "View and modify Quantumn Code configuration. Settings are stored in ~/.config/quantumn-code/config.toml",
            usage: "quantumn config <COMMAND>",
            examples: &[
                "quantumn config show                       # Show all settings",
                "quantumn config get model.provider         # Get specific value",
                "quantumn config set ui.theme oxidized      # Set theme",
                "quantumn config reset                      # Reset to defaults",
                "quantumn config edit                       # Open in editor",
            ],
            aliases: &["cfg"],
        },
        CommandInfo {
            name: "theme",
            short_desc: "Manage terminal themes",
            long_desc: "List, set, and preview terminal UI themes. Supports built-in and custom themes.",
            usage: "quantumn theme <COMMAND>",
            examples: &[
                "quantumn theme list                        # List available themes",
                "quantumn theme set oxidized                # Set theme",
                "quantumn theme current                     # Show current theme",
                "quantumn theme preview tokyo_night         # Preview theme",
            ],
            aliases: &["th"],
        },
        CommandInfo {
            name: "model",
            short_desc: "Switch AI models and providers",
            long_desc: "List available models, switch between providers (Anthropic, OpenAI, Ollama, llama.cpp, LM Studio), and configure API keys.",
            usage: "quantumn model [PROVIDER] [--list]",
            examples: &[
                "quantumn model list                        # List all models",
                "quantumn model                             # Show current model",
                "quantumn model anthropic                   # Switch to Claude",
                "quantumn model openai                      # Switch to OpenAI",
                "quantumn model ollama                      # Switch to Ollama (local)",
            ],
            aliases: &["m"],
        },
        CommandInfo {
            name: "provider",
            short_desc: "Show all AI providers",
            long_desc: "Display all available AI providers with their setup instructions.",
            usage: "quantumn provider",
            examples: &[
                "quantumn provider                          # Show all providers",
            ],
            aliases: &["p"],
        },
        CommandInfo {
            name: "status",
            short_desc: "Show system status",
            long_desc: "Display current configuration, model, provider, and git status.",
            usage: "quantumn status",
            examples: &[
                "quantumn status                            # Show status",
            ],
            aliases: &["st"],
        },
        CommandInfo {
            name: "version",
            short_desc: "Show version information",
            long_desc: "Display Quantumn Code version and build information.",
            usage: "quantumn version",
            examples: &[
                "quantumn version                           # Show version",
            ],
            aliases: &["v", "-v", "--version"],
        },
        CommandInfo {
            name: "completions",
            short_desc: "Generate shell completions",
            long_desc: "Generate shell completion scripts for bash, zsh, fish, powershell, and elvish.",
            usage: "quantumn completions [SHELL]",
            examples: &[
                "quantumn completions bash                  # Bash completions",
                "quantumn completions zsh                   # Zsh completions",
                "quantumn completions fish                  # Fish completions",
            ],
            aliases: &[],
        },
    ]
}

/// Get provider information
pub fn get_providers() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            name: "anthropic",
            display_name: "Anthropic Claude",
            description: "Advanced AI assistant with Claude models",
            models: &[
                "claude-opus-4-20250514   - Most capable (Opus 4)",
                "claude-sonnet-4-20250514 - Balanced (Sonnet 4) [default]",
                "claude-haiku-4-20250514  - Fast (Haiku 4)",
                "claude-3-5-sonnet-20241022 - Legacy (Sonnet 3.5)",
                "claude-3-5-haiku-20241022  - Legacy (Haiku 3.5)",
            ],
            env_key: "ANTHROPIC_API_KEY",
            setup: "export ANTHROPIC_API_KEY=your_key_here",
        },
        ProviderInfo {
            name: "openai",
            display_name: "OpenAI",
            description: "GPT models from OpenAI",
            models: &[
                "gpt-4o       - GPT-4 Omni (recommended)",
                "gpt-4o-mini  - GPT-4 Omni Mini (fast, cheap)",
                "gpt-4-turbo  - GPT-4 Turbo",
                "o1           - O1 (advanced reasoning)",
                "o1-mini      - O1 Mini",
            ],
            env_key: "OPENAI_API_KEY",
            setup: "export OPENAI_API_KEY=your_key_here",
        },
        ProviderInfo {
            name: "ollama",
            display_name: "Ollama (Local)",
            description: "Run models locally with Ollama - No API key required",
            models: &[
                "llama3.2       - Meta Llama 3.2",
                "llama3.1       - Meta Llama 3.1",
                "mistral        - Mistral",
                "codellama      - Code Llama",
                "deepseek-coder - DeepSeek Coder",
                "qwen2.5-coder  - Qwen 2.5 Coder",
            ],
            env_key: "N/A (local)",
            setup: "1. Install: curl https://ollama.ai/install.sh | sh\n2. Run: ollama serve\n3. Pull model: ollama pull llama3.2",
        },
        ProviderInfo {
            name: "lm_studio",
            display_name: "LM Studio (Local)",
            description: "High-performance local inference with LM Studio",
            models: &[
                "llama3.2       - Meta Llama 3.2 (GGUF)",
                "mistral        - Mistral (GGUF)",
                "qwen2.5        - Qwen 2.5 (GGUF)",
            ],
            env_key: "N/A (local)",
            setup: "1. Download LM Studio\n2. lms server start OR enable server in GUI\n3. Download models to ~/.lmstudio/models/",
        },
        ProviderInfo {
            name: "llama_cpp",
            display_name: "llama.cpp (Local, High-Performance)",
            description: "High-performance local inference with llama.cpp",
            models: &[
                "llama3.2       - Meta Llama 3.2 (GGUF)",
                "llama3.1       - Meta Llama 3.1 (GGUF)",
                "mistral        - Mistral (GGUF)",
                "qwen2.5        - Qwen 2.5 (GGUF)",
                "deepseek-coder - DeepSeek Coder (GGUF)",
            ],
            env_key: "N/A (local)",
            setup: "1. Install llama-server binary\n2. Download GGUF model files\n3. Configure paths in ~/.config/quantumn-code/config.toml",
        },
    ]
}

/// Provider information structure
pub struct ProviderInfo {
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub models: &'static [&'static str],
    pub env_key: &'static str,
    pub setup: &'static str,
}

/// Get theme information
pub fn get_themes() -> Vec<ThemeInfo> {
    vec![
        ThemeInfo {
            name: "oxidized",
            description: "Rusty brown on deep black - Elegant, unique (default)",
            author: "Quantumn",
        },
        ThemeInfo {
            name: "default",
            description: "Classic Claude-inspired purple theme",
            author: "Quantumn",
        },
        ThemeInfo {
            name: "tokyo_night",
            description: "Purple and blue accents, popular dark theme",
            author: "Quantumn",
        },
        ThemeInfo {
            name: "hacker",
            description: "Matrix-style green on black",
            author: "Quantumn",
        },
        ThemeInfo {
            name: "deep_black",
            description: "Minimal high-contrast dark theme",
            author: "Quantumn",
        },
    ]
}

/// Theme information structure
pub struct ThemeInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub author: &'static str,
}

/// Get keyboard shortcuts
pub fn get_shortcuts() -> Vec<ShortcutInfo> {
    vec![
        ShortcutInfo { key: "Ctrl+S", action: "Save current session" },
        ShortcutInfo { key: "Ctrl+L", action: "Load saved session" },
        ShortcutInfo { key: "Ctrl+T", action: "Cycle through themes" },
        ShortcutInfo { key: "Ctrl+P", action: "Switch AI provider" },
        ShortcutInfo { key: "Ctrl+M", action: "Switch AI model" },
        ShortcutInfo { key: "Ctrl+Q", action: "Quit application" },
        ShortcutInfo { key: "Tab", action: "Switch between panes" },
        ShortcutInfo { key: "Shift+Tab", action: "Switch panes (reverse)" },
        ShortcutInfo { key: "Enter", action: "Send message" },
        ShortcutInfo { key: "Esc", action: "Cancel/exit" },
        ShortcutInfo { key: "Up/Down", action: "Navigate history" },
        ShortcutInfo { key: "Ctrl+C", action: "Cancel current operation" },
        ShortcutInfo { key: "Ctrl+R", action: "Clear screen" },
        ShortcutInfo { key: "Ctrl+H", action: "Show help" },
        ShortcutInfo { key: "/", action: "Enter command mode (prefix commands with /)" },
    ]
}

/// Shortcut information
pub struct ShortcutInfo {
    pub key: &'static str,
    pub action: &'static str,
}

/// Command mode commands (prefixed with /)
pub fn get_slash_commands() -> Vec<SlashCommandInfo> {
    vec![
        SlashCommandInfo { command: "/help", description: "Show this help message", autocomplete: true },
        SlashCommandInfo { command: "/clear", description: "Clear the chat history", autocomplete: true },
        SlashCommandInfo { command: "/mode", description: "Switch mode: plan | build | chat", autocomplete: true },
        SlashCommandInfo { command: "/model", description: "Switch AI model (show all if no arg)", autocomplete: true },
        SlashCommandInfo { command: "/provider", description: "Switch AI provider", autocomplete: true },
        SlashCommandInfo { command: "/theme", description: "Switch theme", autocomplete: true },
        SlashCommandInfo { command: "/session", description: "Manage sessions: save | load | list", autocomplete: true },
        SlashCommandInfo { command: "/config", description: "View or edit configuration", autocomplete: true },
        SlashCommandInfo { command: "/status", description: "Show current status", autocomplete: true },
        SlashCommandInfo { command: "/version", description: "Show version", autocomplete: true },
        SlashCommandInfo { command: "/exit", description: "Exit Quantumn Code", autocomplete: true },
        SlashCommandInfo { command: "/quit", description: "Exit Quantumn Code (alias for /exit)", autocomplete: true },
    ]
}

/// Slash command information
pub struct SlashCommandInfo {
    pub command: &'static str,
    pub description: &'static str,
    pub autocomplete: bool,
}

/// Run the help command
pub async fn run(section: Option<String>) -> Result<()> {
    match section.as_deref() {
        Some("commands") => print_commands(),
        Some("providers") => print_providers(),
        Some("themes") => print_themes(),
        Some("shortcuts") => print_shortcuts(),
        Some("slash") => print_slash_commands(),
        Some("quick") => print_quick_start(),
        None => print_full_help(),
        _ => {
            eprintln!("{}Unknown help section: {}{}", RED, section.unwrap(), RESET);
            println!("Available sections: {}, {}, {}, {}, {}, {}",
                format!("{}commands{}", CYAN, RESET),
                format!("{}providers{}", CYAN, RESET),
                format!("{}themes{}", CYAN, RESET),
                format!("{}shortcuts{}", CYAN, RESET),
                format!("{}slash{}", CYAN, RESET),
                format!("{}quick{}", CYAN, RESET));
        }
    }
    Ok(())
}

/// Print full help documentation
fn print_full_help() {
    print_banner();
    print_quick_start();
    print_commands();
    print_providers();
    print_themes();
    print_shortcuts();
    print_slash_commands();
    print_footer();
}

/// Print the Quantumn Code banner with colors
fn print_banner() {
    println!();
    println!("{}   ____  _   _ _   _ _   _ ___  ____  {}", MAGENTA, RESET);
    println!("{}  |  _ \\| | | | \\ | | \\ | |   \\|_  / {}", CYAN, RESET);
    println!("{}  | |_) | |_| |  \\| |  \\| | |) |/ /  {}", YELLOW, RESET);
    println!("{}  |  _ <|  _  | |\\  | |\\  |   // /   {}", BLUE, RESET);
    println!("{}  |_| \\_\\_| |_|_| \\_|_| \\_|__/___|   {}", GREEN, RESET);
    println!();
    println!("{}  {}An advanced AI-powered coding assistant CLI{}", BOLD, YELLOW, RESET);
    println!("{}  Built in Rust for performance and reliability{}", CYAN, RESET);
    println!();
}

/// Print footer
fn print_footer() {
    println!();
    println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", CYAN, RESET);
    println!();
    println!("{}  Documentation:{} https://github.com/Akatsuki2r/QuantumCode", GREEN, RESET);
    println!("{}  Issues:{}        https://github.com/Akatsuki2r/QuantumCode/issues", RED, RESET);
    println!();
    println!("{}  Made with Rust by NahanoMark{}", YELLOW, RESET);
    println!();
}

/// Print quick start guide
fn print_quick_start() {
    println!("{}┌─────────────────────────────────────────────────────────────────┐{}", CYAN, RESET);
    println!("{}│  {}QUICK START{}                                                   │{}", CYAN, YELLOW, CYAN, RESET);
    println!("{}└─────────────────────────────────────────────────────────────────┘{}", CYAN, RESET);
    println!();
    println!("  {}1. Install:{} {}", CYAN, RESET, RESET);
    println!("     cargo install quantumn");
    println!("     {}# or{}", CYAN, RESET);
    println!("     npm install -g @quantumn/code");
    println!();
    println!("  {}2. Set up your AI provider:{} {}", CYAN, RESET, RESET);
    println!("     export ANTHROPIC_API_KEY=your_key_here   {}# For Claude{}", CYAN, RESET);
    println!("     {}# or{}", CYAN, RESET);
    println!("     export OPENAI_API_KEY=your_key_here        {}# For OpenAI{}", CYAN, RESET);
    println!("     {}# or use local models with Ollama:{} {}", CYAN, RESET, RESET);
    println!("     ollama serve && ollama pull llama3.2");
    println!();
    println!("  {}3. Start chatting:{} {}", CYAN, RESET, RESET);
    println!("     quantumn                                 {}# Interactive mode{}", CYAN, RESET);
    println!("     quantumn chat \"Explain this code\"        {}# One-shot query{}", CYAN, RESET);
    println!();
    println!("  {}4. Useful commands:{} {}", CYAN, RESET, RESET);
    println!("     quantumn edit src/main.rs                 {}# Edit file with AI{}", CYAN, RESET);
    println!("     quantumn agent \"Refactor this\"          {}# Agentic mode{}", CYAN, RESET);
    println!("     quantumn review src/lib.rs                {}# Code review{}", CYAN, RESET);
    println!();
}

/// Print commands section
fn print_commands() {
    println!("{}┌─────────────────────────────────────────────────────────────────┐{}", CYAN, RESET);
    println!("{}│  {}COMMANDS{}                                                      │{}", CYAN, GREEN, CYAN, RESET);
    println!("{}└─────────────────────────────────────────────────────────────────┘{}", CYAN, RESET);
    println!();

    let commands = get_commands();
    let max_name_len = commands.iter().map(|c| c.name.len()).max().unwrap_or(12);

    for cmd in &commands {
        println!("  {}{:width$}{}  {}", CYAN, cmd.name, RESET, cmd.short_desc, width = max_name_len);
        if !cmd.aliases.is_empty() {
            println!("  {:width$}  {}Aliases: {}{}", "", CYAN, cmd.aliases.join(", "), RESET, width = max_name_len);
        }
    }

    println!();
    println!("  {}Use 'quantumn <command> --help' for detailed usage{}", YELLOW, RESET);
}

/// Print providers section
fn print_providers() {
    println!("{}┌─────────────────────────────────────────────────────────────────┐{}", CYAN, RESET);
    println!("{}│  {}AI PROVIDERS{}                                                  │{}", CYAN, MAGENTA, CYAN, RESET);
    println!("{}└─────────────────────────────────────────────────────────────────┘{}", CYAN, RESET);
    println!();

    for provider in get_providers() {
        println!("  {}{} ({}){}", BOLD, provider.display_name, provider.name, RESET);
        println!("  {}", provider.description);
        println!("  {}Models:{}", GREEN, RESET);
        for model in provider.models {
            println!("    {}• {}{}", CYAN, model, RESET);
        }
        println!("  {}API Key:{} {}", YELLOW, RESET, provider.env_key);
        println!("  {}Setup:{} {}", YELLOW, RESET, provider.setup);
        println!();
    }
}

/// Print themes section
fn print_themes() {
    println!("{}┌─────────────────────────────────────────────────────────────────┐{}", CYAN, RESET);
    println!("{}│  {}THEMES{}                                                        │{}", CYAN, BLUE, CYAN, RESET);
    println!("{}└─────────────────────────────────────────────────────────────────┘{}", CYAN, RESET);
    println!();

    for theme in get_themes() {
        let default_marker = if theme.name == "oxidized" { format!(" {}[default]{}", GREEN, RESET) } else { String::new() };
        println!("  {}{}{}{} - {}", CYAN, theme.name, default_marker, RESET, theme.description);
    }

    println!();
    println!("  {}Set theme:{} quantumn theme set <name>", YELLOW, RESET);
    println!("  {}List themes:{} quantumn theme list", YELLOW, RESET);
}

/// Print shortcuts section
fn print_shortcuts() {
    println!("{}┌─────────────────────────────────────────────────────────────────┐{}", CYAN, RESET);
    println!("{}│  {}KEYBOARD SHORTCUTS{}                                            │{}", CYAN, YELLOW, CYAN, RESET);
    println!("{}└─────────────────────────────────────────────────────────────────┘{}", CYAN, RESET);
    println!();

    let shortcuts = get_shortcuts();
    let max_key_len = shortcuts.iter().map(|s| s.key.len()).max().unwrap_or(10);

    for shortcut in &shortcuts {
        println!("  {}{:width$}{}  {}", CYAN, shortcut.key, RESET, shortcut.action, width = max_key_len);
    }
}

/// Print slash commands section
fn print_slash_commands() {
    println!("{}┌─────────────────────────────────────────────────────────────────┐{}", CYAN, RESET);
    println!("{}│  {}SLASH COMMANDS (in interactive mode){}                         │{}", CYAN, MAGENTA, CYAN, RESET);
    println!("{}└─────────────────────────────────────────────────────────────────┘{}", CYAN, RESET);
    println!();

    let commands = get_slash_commands();
    let max_cmd_len = commands.iter().map(|c| c.command.len()).max().unwrap_or(10);

    for cmd in &commands {
        println!("  {}{:width$}{}  {}", CYAN, cmd.command, RESET, cmd.description, width = max_cmd_len);
    }

    println!();
    println!("  {}Type / followed by command in interactive mode{}", YELLOW, RESET);
    println!("  {}Tab completion is available for all slash commands{}", CYAN, RESET);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_commands() {
        let commands = get_commands();
        assert!(!commands.is_empty());
        assert!(commands.iter().any(|c| c.name == "chat"));
        assert!(commands.iter().any(|c| c.name == "agent"));
        assert!(commands.iter().any(|c| c.name == "completions"));
    }

    #[test]
    fn test_get_providers() {
        let providers = get_providers();
        assert!(!providers.is_empty());
        assert!(providers.iter().any(|p| p.name == "anthropic"));
        assert!(providers.iter().any(|p| p.name == "llama_cpp"));
    }

    #[test]
    fn test_get_themes() {
        let themes = get_themes();
        assert!(!themes.is_empty());
        assert!(themes.iter().any(|t| t.name == "oxidized"));
    }

    #[test]
    fn test_get_slash_commands() {
        let commands = get_slash_commands();
        assert!(!commands.is_empty());
        assert!(commands.iter().any(|c| c.command == "/model"));
        assert!(commands.iter().any(|c| c.command == "/provider"));
        assert!(commands.iter().any(|c| c.command == "/theme"));
    }
}
