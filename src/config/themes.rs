//! Theme system for terminal styling

use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};
use ratatui::style::{Color, Modifier};
use color_eyre::eyre::{Result, WrapErr};

/// RGB color representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(color_eyre::eyre::eyre!("Invalid hex color: {}", hex));
        }
        let r = u8::from_str_radix(&hex[0..2], 16)
            .wrap_err("Invalid hex color")?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .wrap_err("Invalid hex color")?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .wrap_err("Invalid hex color")?;
        Ok(Self { r, g, b })
    }

    pub fn to_ratatui(&self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }
}

/// Theme color palette
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    /// Background color
    pub background: String,
    /// Foreground (text) color
    pub foreground: String,
    /// Accent color (primary actions)
    pub accent: String,
    /// Secondary accent
    pub secondary: String,
    /// Success color
    pub success: String,
    /// Error color
    pub error: String,
    /// Warning color
    pub warning: String,
    /// Info color
    pub info: String,
    /// Selection background
    pub selection_bg: String,
    /// Selection foreground
    pub selection_fg: String,
    /// Border color
    pub border: String,
    /// Muted text color
    pub muted: String,
}

impl ThemeColors {
    pub fn to_ratatui(&self) -> Result<RatatuiColors> {
        Ok(RatatuiColors {
            background: RgbColor::from_hex(&self.background)?.to_ratatui(),
            foreground: RgbColor::from_hex(&self.foreground)?.to_ratatui(),
            accent: RgbColor::from_hex(&self.accent)?.to_ratatui(),
            secondary: RgbColor::from_hex(&self.secondary)?.to_ratatui(),
            success: RgbColor::from_hex(&self.success)?.to_ratatui(),
            error: RgbColor::from_hex(&self.error)?.to_ratatui(),
            warning: RgbColor::from_hex(&self.warning)?.to_ratatui(),
            info: RgbColor::from_hex(&self.info)?.to_ratatui(),
            selection_bg: RgbColor::from_hex(&self.selection_bg)?.to_ratatui(),
            selection_fg: RgbColor::from_hex(&self.selection_fg)?.to_ratatui(),
            border: RgbColor::from_hex(&self.border)?.to_ratatui(),
            muted: RgbColor::from_hex(&self.muted)?.to_ratatui(),
        })
    }
}

/// Converted colors for ratatui
#[derive(Debug, Clone)]
pub struct RatatuiColors {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub secondary: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub border: Color,
    pub muted: Color,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Theme description
    pub description: String,
    /// Author
    pub author: String,
    /// Color palette
    pub colors: ThemeColors,
    /// Code syntax highlighting colors
    pub syntax: SyntaxColors,
}

/// Syntax highlighting colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxColors {
    /// Keywords
    pub keyword: String,
    /// Strings
    pub string: String,
    /// Numbers
    pub number: String,
    /// Comments
    pub comment: String,
    /// Functions
    pub function: String,
    /// Types
    pub r#type: String,
    /// Variables
    pub variable: String,
    /// Operators
    pub operator: String,
    /// Punctuation
    pub punctuation: String,
}

impl Theme {
    /// Get the themes directory path
    pub fn themes_dir() -> Result<PathBuf> {
        let config_dir = directories::ProjectDirs::from("com", "quantumn", "code")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".quantumn"));

        Ok(config_dir.join("themes"))
    }

    /// Load a theme by name
    pub fn load(name: &str) -> Result<Self> {
        // First check built-in themes
        if let Some(theme) = Self::builtin(name) {
            return Ok(theme);
        }

        // Then check custom themes
        let themes_dir = Self::themes_dir()?;
        let theme_path = themes_dir.join(format!("{}.toml", name));

        if theme_path.exists() {
            let content = fs::read_to_string(&theme_path)
                .wrap_err("Failed to read theme file")?;
            let theme: Theme = toml::from_str(&content)
                .wrap_err("Failed to parse theme file")?;
            Ok(theme)
        } else {
            Err(color_eyre::eyre::eyre!("Theme not found: {}", name))
        }
    }

    /// List available themes
    pub fn list() -> Result<Vec<String>> {
        let mut themes = vec![
            "default".to_string(),
            "tokyo_night".to_string(),
            "hacker".to_string(),
            "deep_black".to_string(),
        ];

        // Add custom themes
        let themes_dir = Self::themes_dir()?;
        if themes_dir.exists() {
            for entry in fs::read_dir(&themes_dir)
                .wrap_err("Failed to read themes directory")? {
                let entry = entry.wrap_err("Failed to read theme entry")?;
                if let Some(name) = entry.path().file_stem() {
                    if let Some(name_str) = name.to_str() {
                        if !themes.contains(&name_str.to_string()) {
                            themes.push(name_str.to_string());
                        }
                    }
                }
            }
        }

        Ok(themes)
    }

    /// Get a built-in theme
    fn builtin(name: &str) -> Option<Self> {
        match name {
            "default" => Some(Self::default_theme()),
            "tokyo_night" => Some(Self::tokyo_night()),
            "hacker" => Some(Self::hacker()),
            "deep_black" => Some(Self::deep_black()),
            _ => None,
        }
    }

    /// Default Claude-style theme
    pub fn default_theme() -> Self {
        Self {
            name: "default".to_string(),
            description: "Classic Claude Code inspired theme".to_string(),
            author: "Quantumn".to_string(),
            colors: ThemeColors {
                background: "#1a1a2e".to_string(),
                foreground: "#eaeaea".to_string(),
                accent: "#7c3aed".to_string(),
                secondary: "#a855f7".to_string(),
                success: "#22c55e".to_string(),
                error: "#ef4444".to_string(),
                warning: "#f59e0b".to_string(),
                info: "#3b82f6".to_string(),
                selection_bg: "#7c3aed".to_string(),
                selection_fg: "#ffffff".to_string(),
                border: "#374151".to_string(),
                muted: "#6b7280".to_string(),
            },
            syntax: SyntaxColors {
                keyword: "#a855f7".to_string(),
                string: "#22c55e".to_string(),
                number: "#f59e0b".to_string(),
                comment: "#6b7280".to_string(),
                function: "#3b82f6".to_string(),
                r#type: "#7c3aed".to_string(),
                variable: "#eaeaea".to_string(),
                operator: "#f59e0b".to_string(),
                punctuation: "#9ca3af".to_string(),
            },
        }
    }

    /// Tokyo Night theme
    pub fn tokyo_night() -> Self {
        Self {
            name: "tokyo_night".to_string(),
            description: "Tokyo Night - A dark theme with purple and blue accents".to_string(),
            author: "Quantumn".to_string(),
            colors: ThemeColors {
                background: "#1a1b26".to_string(),
                foreground: "#c0caf5".to_string(),
                accent: "#7aa2f7".to_string(),
                secondary: "#bb9af7".to_string(),
                success: "#9ece6a".to_string(),
                error: "#f7768e".to_string(),
                warning: "#e0af68".to_string(),
                info: "#7dcfff".to_string(),
                selection_bg: "#364a82".to_string(),
                selection_fg: "#c0caf5".to_string(),
                border: "#3b4261".to_string(),
                muted: "#565f89".to_string(),
            },
            syntax: SyntaxColors {
                keyword: "#bb9af7".to_string(),
                string: "#9ece6a".to_string(),
                number: "#ff9e64".to_string(),
                comment: "#565f89".to_string(),
                function: "#7aa2f7".to_string(),
                r#type: "#2ac3de".to_string(),
                variable: "#c0caf5".to_string(),
                operator: "#89ddff".to_string(),
                punctuation: "#89ddff".to_string(),
            },
        }
    }

    /// Hacker theme (Matrix-style green on black)
    pub fn hacker() -> Self {
        Self {
            name: "hacker".to_string(),
            description: "Hacker - Matrix-style green on black theme".to_string(),
            author: "Quantumn".to_string(),
            colors: ThemeColors {
                background: "#000000".to_string(),
                foreground: "#00ff00".to_string(),
                accent: "#00ff00".to_string(),
                secondary: "#00aa00".to_string(),
                success: "#00ff00".to_string(),
                error: "#ff0000".to_string(),
                warning: "#ffff00".to_string(),
                info: "#00ffff".to_string(),
                selection_bg: "#003300".to_string(),
                selection_fg: "#00ff00".to_string(),
                border: "#004400".to_string(),
                muted: "#006600".to_string(),
            },
            syntax: SyntaxColors {
                keyword: "#00ff00".to_string(),
                string: "#00aa00".to_string(),
                number: "#00ff00".to_string(),
                comment: "#006600".to_string(),
                function: "#00ff00".to_string(),
                r#type: "#00ff00".to_string(),
                variable: "#00cc00".to_string(),
                operator: "#00ff00".to_string(),
                punctuation: "#00aa00".to_string(),
            },
        }
    }

    /// Deep Black theme (minimal, high contrast)
    pub fn deep_black() -> Self {
        Self {
            name: "deep_black".to_string(),
            description: "Deep Black - Minimal high contrast dark theme".to_string(),
            author: "Quantumn".to_string(),
            colors: ThemeColors {
                background: "#0d0d0d".to_string(),
                foreground: "#ffffff".to_string(),
                accent: "#e0e0e0".to_string(),
                secondary: "#808080".to_string(),
                success: "#00ff00".to_string(),
                error: "#ff3333".to_string(),
                warning: "#ffaa00".to_string(),
                info: "#5599ff".to_string(),
                selection_bg: "#1a1a1a".to_string(),
                selection_fg: "#ffffff".to_string(),
                border: "#333333".to_string(),
                muted: "#666666".to_string(),
            },
            syntax: SyntaxColors {
                keyword: "#ffaa00".to_string(),
                string: "#00ff00".to_string(),
                number: "#ff6666".to_string(),
                comment: "#555555".to_string(),
                function: "#5599ff".to_string(),
                r#type: "#ff66ff".to_string(),
                variable: "#ffffff".to_string(),
                operator: "#ffaa00".to_string(),
                punctuation: "#888888".to_string(),
            },
        }
    }

    /// Save theme to file
    pub fn save(&self) -> Result<()> {
        let themes_dir = Self::themes_dir()?;
        fs::create_dir_all(&themes_dir)
            .wrap_err("Failed to create themes directory")?;

        let theme_path = themes_dir.join(format!("{}.toml", self.name));
        let content = toml::to_string_pretty(self)
            .wrap_err("Failed to serialize theme")?;

        fs::write(&theme_path, content)
            .wrap_err("Failed to write theme file")?;

        Ok(())
    }
}