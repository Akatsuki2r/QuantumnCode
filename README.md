# Quantumn Code

An advanced AI-powered coding assistant CLI built in Rust.

## Features

- **Multi-Provider AI Support**: Works with Anthropic Claude, OpenAI, and Ollama (local LLMs)
- **Beautiful Themes**: Tokyo Night, Hacker (Matrix-style), Deep Black, and custom themes
- **Git Integration**: AI-generated commit messages, branch management, PR creation
- **Code Tools**: Read, write, edit files with AI assistance
- **Test Runner**: Run tests with AI failure analysis
- **Project Scaffolding**: Create new projects from templates
- **Interactive TUI**: Comfortable terminal UI for long coding sessions

## Installation

### Prerequisites

- Rust 1.70+ (install with `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Cargo (comes with Rust)

### Build from Source

```bash
git clone https://github.com/Akatsuki2r/Quantumn-code.git
cd Quantumn-code
cargo build --release
```

The binary will be at `target/release/quantumn`.

## Quick Start

```bash
# Start interactive session
quantumn

# One-shot query
quantumn "explain this error"

# Edit a file with AI
quantumn edit src/main.rs --prompt "add error handling"

# Generate commit message
quantumn commit

# Run tests with AI analysis
quantumn test

# Create new project
quantumn scaffold rust my-project

# List available themes
quantumn theme list

# Set theme
quantumn theme set tokyo_night
```

## Commands

### Interactive Mode

```bash
quantumn                    # Start interactive session
quantumn --model opus       # Start with specific model
quantumn --theme hacker     # Start with specific theme
```

### Chat

```bash
quantumn chat               # Interactive chat
quantumn chat "prompt"       # One-shot query
```

### Code Editing

```bash
quantumn edit <file>                    # Edit file interactively
quantumn edit <file> --prompt "..."     # Edit with instructions
```

### Git Integration

```bash
quantumn commit              # Generate commit message
quantumn commit --message "..."  # Use custom prompt
```

### Code Review

```bash
quantumn review              # Review staged changes
quantumn review <files>...    # Review specific files
```

### Testing

```bash
quantumn test                # Run all tests
quantumn test <path>          # Run tests at path
```

### Project Scaffolding

```bash
quantumn scaffold rust my-project    # Rust project
quantumn scaffold python my-project   # Python project
quantumn scaffold node my-project     # Node.js project
quantumn scaffold web my-project      # Web project
quantumn scaffold go my-project       # Go project
```

### Session Management

```bash
quantumn session list        # List saved sessions
quantumn session resume       # Resume last session
quantumn session save <name>  # Save current session
```

### Configuration

```bash
quantumn config show          # Show configuration
quantumn config set <key> <value>  # Set value
quantumn config edit          # Open config in editor
```

### Themes

```bash
quantumn theme list          # List themes
quantumn theme set <name>     # Set theme
quantumn theme current        # Show current theme
quantumn theme preview <name> # Preview theme
```

### Models

```bash
quantumn model                # Show current model
quantumn model --list         # List available models
quantumn model anthropic      # Switch to Claude
quantumn model openai         # Switch to OpenAI
quantumn model ollama         # Switch to local LLM
```

## Interactive Commands

Inside the TUI, use these slash commands:

- `/help` - Show help
- `/clear` - Clear conversation
- `/model <name>` - Change model
- `/theme <name>` - Change theme
- `/commit` - Generate commit message
- `/review` - Review code
- `/test` - Run tests
- `/status` - Show status
- `/quit` - Exit

### Keyboard Shortcuts

- `Enter` - Send message
- `Ctrl+C` - Quit
- `Esc` - Exit current mode
- `Ctrl+L` - Clear screen
- `F1` - Toggle help
- `F2` - Toggle file tree
- `F3` - Toggle token count
- `F4` - Change theme

## Configuration

Configuration is stored at:

- Linux/macOS: `~/.config/quantumn/config.toml`
- Windows: `%APPDATA%\quantumn\config.toml`

### Example Configuration

```toml
[model]
provider = "anthropic"
default_model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"

[git]
commit_format = "{type}: {description}"
include_coauthors = true
conventional_commits = true

[editor]
tab_width = 4
use_spaces = true
line_numbers = true
auto_save = 30

[ui]
theme = "tokyo_night"
show_file_tree = true
show_token_count = true
show_cost = true
animation_speed = 5
```

## API Keys

Set your API keys as environment variables:

```bash
# Anthropic Claude
export ANTHROPIC_API_KEY="your-key-here"

# OpenAI
export OPENAI_API_KEY="your-key-here"

# Ollama (no key needed, just run locally)
# Install: https://ollama.ai
ollama pull llama3.2
```

## Themes

### Built-in Themes

| Theme | Description |
|-------|-------------|
| `default` | Claude Code inspired purple theme |
| `tokyo_night` | Tokyo Night with purple/blue accents |
| `hacker` | Matrix-style green on black |
| `deep_black` | Minimal high contrast |

### Custom Themes

Create custom themes in `~/.config/quantumn/themes/<name>.toml`:

```toml
name = "my_theme"
description = "My custom theme"
author = "Me"

[colors]
background = "#1a1b26"
foreground = "#c0caf5"
accent = "#7aa2f7"
# ... etc

[syntax]
keyword = "#bb9af7"
string = "#9ece6a"
# ... etc
```

## Project Structure

```
QuantumCode/
├── src/
│   ├── main.rs           # Entry point
│   ├── cli.rs            # CLI argument parsing
│   ├── app.rs            # Application state
│   ├── config/           # Configuration system
│   │   ├── settings.rs   # User settings
│   │   └── themes.rs     # Theme definitions
│   ├── providers/        # AI providers
│   │   ├── anthropic.rs   # Claude API
│   │   ├── openai.rs     # OpenAI API
│   │   └── ollama.rs     # Local LLM
│   ├── commands/         # CLI commands
│   │   ├── chat.rs
│   │   ├── edit.rs
│   │   ├── git.rs
│   │   └── ...
│   ├── tui/              # Terminal UI
│   │   ├── render.rs     # Rendering
│   │   └── event.rs      # Event handling
│   ├── tools/            # File tools
│   │   ├── read_file.rs
│   │   ├── write_file.rs
│   │   └── ...
│   └── utils/            # Utilities
│       └── syntax.rs     # Syntax highlighting
├── themes/               # Theme files
│   ├── default.toml
│   ├── tokyo_night.toml
│   ├── hacker.toml
│   └── deep_black.toml
└── Cargo.toml
```

## Development

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Build release
cargo build --release

# Check code
cargo clippy

# Format code
cargo fmt
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

## Credits

Built with:
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client

## Comparison to Claude Code

| Feature | Quantumn Code | Claude Code |
|---------|---------------|-------------|
| Language | Rust | Node.js |
| Providers | Multi | Anthropic only |
| Themes | 4+ custom | 1 |
| Local LLM | Yes | No |
| Open Source | Yes | No |
| Speed | Native | Node.js |

---

**Quantumn Code** - *Code faster, think deeper.*