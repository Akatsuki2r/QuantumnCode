# Quantumn Code

<div align="center">

![Quantumn Code](https://img.shields.io/badge/Quantumn-Code-purple?style=for-the-badge)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange?style=for-the-badge&logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)

**An advanced AI-powered coding assistant CLI built in Rust**

*Local-first • Multi-provider • High-performance • Mode-aware*

[Features](#features) • [Installation](#installation) • [Quick Start](#quick-start) • [Providers](#ai-providers)

</div>

---

## What is Quantumn Code?

Quantumn Code is a local-first, high-performance coding assistant that runs in your terminal. Inspired by Claude Code, it provides intelligent code assistance through multiple AI backends while prioritizing speed, privacy, and developer experience.

**Key Principles:**
- **Local-First**: Works offline with Ollama or llama.cpp - no cloud required
- **Performance**: Built in Rust for fast startup and low memory usage
- **Multi-Provider**: Switch seamlessly between Claude, OpenAI, Ollama, and llama.cpp
- **Mode-Aware**: Plan mode, build mode, chat mode - each optimized for different workflows
- **Privacy-Focused**: Your code stays on your machine

---

## Features

### Core Capabilities
- **Multi-Provider AI Support**: Works with Anthropic Claude, OpenAI, Ollama (local LLMs), and llama.cpp
- **20+ CLI Commands**: chat, edit, commit, review, test, scaffold, session management, and more
- **Interactive TUI**: Beautiful terminal UI with multiple themes for long coding sessions
- **Git Integration**: AI-generated commit messages, PR descriptions, and code reviews
- **Project Scaffolding**: Create new projects from templates (Rust, Python, Node, Web, etc.)
- **Syntax Highlighting**: Beautiful code display powered by syntect
- **Session Management**: Save and resume conversations across work sessions
- **Cross-Platform**: Works on Linux, macOS, and Windows

### AI Providers
| Provider | Type | Models | API Key Required |
|----------|------|--------|-----------------|
| **Anthropic Claude** | Cloud | claude-opus-4, claude-sonnet-4, claude-haiku-4 | Yes |
| **OpenAI** | Cloud | gpt-4o, gpt-4-turbo, o1 | Yes |
| **Ollama** | Local | llama3.2, mistral, deepseek-coder, qwen2.5 | No |
| **llama.cpp** | Local | llama3.2, mistral (GGUF) | No |

### Themes
- **Oxidized** (default): Elegant rusty brown on deep black - inspired by Rust
- **Default**: Classic purple theme inspired by Claude Code
- **Tokyo Night**: Purple and blue accents, popular dark theme
- **Hacker**: Matrix-style green on black
- **Deep Black**: Minimal high-contrast dark theme

---

## Installation

### Option 1: Cargo (Rust) - Recommended

```bash
# Install from crates.io
cargo install quantumn

# Or build from source
git clone https://github.com/Akatsuki2r/QuantumCode.git
cd QuantumCode
cargo install --path .
```

### Option 2: npm

```bash
npm install -g @quantumn/code
```

### Option 3: curl (Linux/macOS)

```bash
# Quick install
curl -sSL https://get.quantumn.dev | bash

# Or manual install
curl -sL https://github.com/Akatsuki2r/QuantumCode/releases/latest/download/quantumn-$(uname -m)-unknown-linux-gnu.tar.gz | tar xz
sudo mv quantumn /usr/local/bin/
```

### Option 4: apt (Debian/Ubuntu)

```bash
# Add repository
curl -sSL https://get.quantumn.dev/apt.gpg | sudo apt-key add -
echo "deb https://get.quantumn.dev/apt stable main" | sudo tee /etc/apt/sources.list.d/quantumn.list

# Install
sudo apt update
sudo apt install quantumn-code
```

### Option 5: Build from Source

```bash
git clone https://github.com/Akatsuki2r/QuantumCode.git
cd QuantumCode
cargo build --release
sudo cp target/release/quantumn /usr/local/bin/
```

---

## Quick Start

### 1. Choose Your Provider

#### Option A: Anthropic Claude (Recommended for quality)
```bash
# Set your API key
export ANTHROPIC_API_KEY=your_api_key_here

# Set as default provider
quantumn model anthropic
```

**Get an API key:** [console.anthropic.com](https://console.anthropic.com)

#### Option B: OpenAI
```bash
# Set your API key
export OPENAI_API_KEY=your_api_key_here

# Set as default provider
quantumn model openai
```

**Get an API key:** [platform.openai.com](https://platform.openai.com)

#### Option C: Ollama (Local, Free)
```bash
# Install Ollama
curl https://ollama.ai/install.sh | sh

# Start server and download a model
ollama serve
ollama pull llama3.2

# Use with Quantumn
quantumn model ollama
```

#### Option D: llama.cpp (Local, High-Performance)
```bash
# Install llama.cpp
# See: https://github.com/ggerganov/llama.cpp

# Download a GGUF model
# Configure model paths in ~/.config/quantumn-code/config.toml

# Use with Quantumn
quantumn model llama_cpp
```

### 2. Start Using

**For Cloud Providers (Claude, OpenAI):**

```bash
# Set your API key
export ANTHROPIC_API_KEY=sk-ant-...   # for Claude
export OPENAI_API_KEY=sk-...         # for GPT

# Interactive mode
quantumn

# Quick query
quantumn chat "Explain this code"
```

**For Local Models (Ollama, LM Studio, llama.cpp):**

```bash
# Option A: Ollama (easiest)
curl https://ollama.ai/install.sh | sh
ollama serve
ollama pull llama3.2
quantumn model ollama

# Option B: LM Studio (GUI-based)
# 1. Download from https://lmstudio.ai
# 2. Download a model (e.g., llama3.2)
# 3. Click "Start Server" button
# 4. Run: quantumn model lm_studio

# Option C: llama.cpp (manual)
# 1. Build llama.cpp: git clone https://github.com/ggerganov/llama.cpp && cd llama.cpp && make
# 2. Download GGUF model to ~/.config/quantumn-code/models/
# 3. Configure in config.toml:
quantumn model llama_cpp
```

### First-Time Setup

1. **Choose your provider:**
   ```bash
   quantumn model list  # See available models
   quantumn model <provider_name>  # Set provider
   ```

2. **Configure local models:**
   ```bash
   # Edit config file
   quantumn config edit
   ```

3. **Enable shell completions (optional but recommended):**
   ```bash
   quantumn completions bash >> ~/.bashrc
   source ~/.bashrc
   # Now press Tab for auto-complete!
   ```

---

## Usage

### Interactive Mode

```bash
quantumn
```

This launches the interactive TUI where you can:
- Chat with AI about your code
- Edit files with AI assistance
- Run git commands with AI-generated messages
- Switch between themes and providers
- Manage sessions

### Command Reference

```bash
# Chat
quantumn chat                              # Start interactive session
quantumn chat "Explain this function"      # One-shot query
quantumn chat --model claude-opus          # Use specific model

# Edit files
quantumn edit src/main.rs                  # Interactive edit
quantumn edit src/main.rs --prompt "Add error handling"
quantumn edit config.toml --model gpt-4o

# Git integration
quantumn commit                            # Generate from staged changes
quantumn commit --message "Fix login bug"  # Use custom message

# Code review
quantumn review                            # Review staged changes
quantumn review src/lib.rs                 # Review specific file
quantumn review src/**/*.rs                # Review multiple files

# Testing
quantumn test                              # Run all tests with analysis
quantumn test src/auth_tests.rs           # Run specific tests

# Project scaffolding
quantumn scaffold rust my-app              # New Rust project
quantumn scaffold python my-script         # New Python project
quantumn scaffold node my-api              # New Node.js project
quantumn scaffold web my-website           # New web project

# Session management
quantumn session list                      # List saved sessions
quantumn session save feature-x            # Save current session
quantumn session resume feature-x          # Resume session
quantumn session delete feature-x         # Delete session

# Configuration
quantumn config show                        # Show all settings
quantumn config get model.provider         # Get specific value
quantumn config set ui.theme oxidized      # Set theme
quantumn config set model.default_model claude-sonnet-4-20250514

# Local Model Discovery
quantumn model list                        # List all models (auto-detects installed)
quantumn model ollama                       # Show/check Ollama models
quantumn model lm_studio                    # Show/check LM Studio models
quantumn model llama_cpp                   # Show llama.cpp models

# Shell Completions (recommended!)
quantumn completions bash >> ~/.bashrc     # Bash
quantumn completions zsh > ~/.zsh/completions/_quantumn  # Zsh
source ~/.bashrc                           # Reload shell

# Other
quantumn status                            # Show system status
quantumn version                           # Show version
quantumn help                              # Show comprehensive help
quantumn help providers                    # Provider setup guide
quantumn help commands                     # Command reference
```

### Local Model Setup

**Ollama:**
```bash
ollama serve                    # Start Ollama server
ollama pull llama3.2           # Download a model
ollama list                     # List installed models
quantumn model ollama           # Switch to Ollama provider
```

**LM Studio:**
```bash
# GUI method:
# 1. Download from https://lmstudio.ai
# 2. Download a model through the app
# 3. Click "Start Server" (Local Inference -> Server)
# CLI method:
lms server start               # Start LM Studio server
quantumn model lm_studio       # Switch to LM Studio provider
```

**llama.cpp:**
```bash
# Build llama.cpp server
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp && mkdir build && cd build && cmake .. && make -j4
# Download a GGUF model and configure
quantumn model llama_cpp       # Switch to llama.cpp provider
```

### Keyboard Shortcuts (TUI)

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save session |
| `Ctrl+L` | Load session |
| `Ctrl+T` | Cycle themes |
| `Ctrl+P` | Switch provider |
| `Ctrl+M` | Switch model |
| `Ctrl+Q` | Quit |
| `Tab` | Switch panes |
| `Enter` | Send message |
| `Ctrl+H` | Show help |

### Slash Commands (Interactive Mode)

Type `/` in interactive mode to access commands:

| Command | Description |
|---------|-------------|
| `/help` | Show help |
| `/clear` | Clear chat history |
| `/mode plan` | Plan mode (no execution) |
| `/mode build` | Build mode (execution enabled) |
| `/mode chat` | Chat mode (conversational) |
| `/model <name>` | Switch model |
| `/provider <name>` | Switch provider |
| `/theme <name>` | Switch theme |
| `/session save/load/list` | Manage sessions |
| `/config` | View/edit configuration |
| `/status` | Show current status |
| `/exit` | Exit Quantumn Code |

---

## AI Providers

### Anthropic Claude

The most capable AI for coding tasks.

```bash
# Setup
export ANTHROPIC_API_KEY=sk-ant-...

# Available models
claude-opus-4-20250514    # Most capable
claude-sonnet-4-20250514  # Balanced (default)
claude-haiku-4-20250514   # Fast

# Pricing (per million tokens)
# Opus:   $15 input / $75 output
# Sonnet: $3 input / $15 output
# Haiku:  $0.25 input / $1.25 output
```

### OpenAI

GPT models with strong code capabilities.

```bash
# Setup
export OPENAI_API_KEY=sk-...

# Available models
gpt-4o       # GPT-4 Omni (recommended)
gpt-4o-mini  # Fast, cheap
gpt-4-turbo  # GPT-4 Turbo
o1           # Advanced reasoning
o1-mini      # Reasoning, cheap

# Pricing (per million tokens)
# GPT-4o:      $5 input / $15 output
# GPT-4o-mini: $0.15 input / $0.60 output
```

### Ollama (Local)

Run models locally - no API key required.

```bash
# Install
curl https://ollama.ai/install.sh | sh

# Start server
ollama serve

# Download models
ollama pull llama3.2        # Meta Llama 3.2
ollama pull mistral         # Mistral
ollama pull deepseek-coder  # DeepSeek Coder
ollama pull qwen2.5-coder   # Qwen 2.5 Coder

# Use with Quantumn
quantumn model ollama
```

**Recommended models:**
- `llama3.2` - General purpose, good balance
- `deepseek-coder` - Excellent for coding
- `qwen2.5-coder` - Strong code generation
- `mistral` - Fast and capable

### llama.cpp (Local, High-Performance)

Maximum performance for local inference.

```bash
# Install llama.cpp
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp && make

# Download GGUF models from HuggingFace
# Configure paths in ~/.config/quantumn-code/config.toml:

[llama_cpp]
enabled = true
default_port = 8080
fallback_to_ollama = true
auto_start = false

[llama_cpp.model_paths]
llama3.2 = "/path/to/llama-3.2.gguf"
mistral = "/path/to/mistral.gguf"

# Use with Quantumn
quantumn model llama_cpp
```

---

## Configuration

Configuration is stored in `~/.config/quantumn-code/config.toml`:

```toml
[model]
provider = "anthropic"
default_model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"

[ui]
theme = "oxidized"
show_file_tree = true
show_token_count = true
show_cost = true
animation_speed = 5

[git]
commit_format = "{type}: {description}"
include_coauthors = true
conventional_commits = true

[editor]
tab_width = 4
use_spaces = true
line_numbers = true
auto_save = 30

[llama_cpp]
enabled = true
default_port = 8080
fallback_to_ollama = true
auto_start = false

[llama_cpp.model_paths]
llama3.2 = "/path/to/model.gguf"
```

---

## Modes

Quantumn Code operates in different modes for different workflows:

### Plan Mode
- Analyze and plan without execution
- No writes or modifications
- Uses smaller, faster models
- Best for: Understanding code, architecture decisions

### Build Mode
- Full execution capabilities
- Can modify files, run commands
- Uses most capable models
- Best for: Implementing changes, fixing bugs

### Chat Mode
- Conversational assistance
- Minimal tool usage
- Fast responses
- Best for: Questions, explanations, brainstorming

Switch modes with `/mode plan`, `/mode build`, or `/mode chat`.

---

## Project Structure

```
QuantumCode/
├── Cargo.toml              # Rust dependencies
├── src/
│   ├── main.rs             # Entry point
│   ├── cli.rs              # CLI argument parsing
│   ├── app.rs              # Application state
│   ├── commands/           # All CLI commands
│   │   ├── chat.rs
│   │   ├── edit.rs
│   │   ├── commit.rs
│   │   ├── review.rs
│   │   ├── scaffold.rs
│   │   ├── session.rs
│   │   ├── config.rs
│   │   ├── theme.rs
│   │   ├── model.rs
│   │   ├── status.rs
│   │   └── help.rs
│   ├── providers/          # AI provider implementations
│   │   ├── anthropic.rs    # Claude
│   │   ├── openai.rs       # GPT
│   │   ├── ollama.rs       # Local models
│   │   └── llama_cpp.rs    # llama.cpp integration
│   ├── supervisor/         # Model supervision
│   ├── config/             # Configuration management
│   ├── tui/                # Terminal UI
│   ├── tools/              # File/shell tools
│   └── utils/              # Utilities
└── themes/                  # Theme files
```

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- Inspired by [Claude Code](https://code.claude.com)
- Built with [Ratatui](https://ratatui.rs) for the TUI
- Syntax highlighting by [syntect](https://github.com/trishume/syntect)
                                    
---

<div align="center">

**Quantumn Code** - *Your AI coding companion, running locally*

Made with Rust by [NahanoMark](https://github.com/Akatsuki2r)

[GitHub](https://github.com/Akatsuki2r/QuantumCode) • [Issues](https://github.com/Akatsuki2r/QuantumCode/issues) • [Discussions](https://github.com/Akatsuki2r/QuantumCode/discussions)

</div>