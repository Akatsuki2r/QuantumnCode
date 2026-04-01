//! Syntax highlighting utilities

use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::easy::HighlightLines;
use std::sync::OnceLock;

/// Global syntax set
static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();

/// Global theme set
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

/// Get the syntax set
fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(|| {
        SyntaxSet::load_defaults_newlines()
    })
}

/// Get the theme set
fn get_theme_set() -> &'static ThemeSet {
    THEME_SET.get_or_init(|| {
        ThemeSet::load_defaults()
    })
}

/// Highlight code with syntax highlighting
pub fn highlight(code: &str, language: &str) -> String {
    let syntax_set = get_syntax_set();
    let theme_set = get_theme_set();

    // Find syntax
    let syntax = syntax_set
        .find_syntax_by_token(language)
        .or_else(|| syntax_set.find_syntax_by_extension(language))
        .or_else(|| syntax_set.find_syntax_by_name(language))
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    // Use default theme
    let theme = &theme_set.themes["base16-eighties"];

    // Highlight
    let mut highlighter = HighlightLines::new(syntax, theme);

    let mut highlighted = String::new();
    for line in code.lines() {
        match syntect::util::as_24_bit_terminal_escaped(&highlighter.highlight_line(line, syntax_set), false) {
            Ok(colored) => highlighted.push_str(&colored),
            Err(_) => highlighted.push_str(line),
        }
        highlighted.push('\n');
    }

    // Reset terminal colors
    highlighted.push_str("\x1b[0m");

    highlighted
}

/// Highlight with custom theme
pub fn highlight_with_theme(code: &str, language: &str, theme_name: &str) -> String {
    let syntax_set = get_syntax_set();
    let theme_set = get_theme_set();

    // Find syntax
    let syntax = syntax_set
        .find_syntax_by_token(language)
        .or_else(|| syntax_set.find_syntax_by_extension(language))
        .or_else(|| syntax_set.find_syntax_by_name(language))
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    // Find theme
    let theme = theme_set.themes.get(theme_name)
        .or_else(|| theme_set.themes.get("base16-eighties"))
        .unwrap();

    // Highlight
    let mut highlighter = HighlightLines::new(syntax, theme);

    let mut highlighted = String::new();
    for line in code.lines() {
        match syntect::util::as_24_bit_terminal_escaped(&highlighter.highlight_line(line, syntax_set), false) {
            Ok(colored) => highlighted.push_str(&colored),
            Err(_) => highlighted.push_str(line),
        }
        highlighted.push('\n');
    }

    highlighted.push_str("\x1b[0m");

    highlighted
}

/// List available themes
pub fn list_themes() -> Vec<String> {
    let theme_set = get_theme_set();
    theme_set.themes.keys().cloned().collect()
}

/// Detect language from file extension
pub fn detect_language(path: &std::path::Path) -> Option<String> {
    let ext = path.extension()?.to_str()?;

    let language = match ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "tsx" => "typescriptreact",
        "jsx" => "jsx",
        "go" => "go",
        "java" => "java",
        "kt" => "kotlin",
        "c" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "h" | "hpp" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "scala" => "scala",
        "html" => "html",
        "css" => "css",
        "scss" => "scss",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "md" => "markdown",
        "sh" | "bash" => "bash",
        "sql" => "sql",
        "lua" => "lua",
        "r" => "r",
        _ => ext,
    };

    Some(language.to_string())
}