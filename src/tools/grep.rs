//! Pattern search tool (grep-like)

use std::path::Path;
use regex::Regex;
use color_eyre::eyre::Result;

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file: String,
    pub line: usize,
    pub content: String,
}

/// Search for pattern in file
pub fn search_file(path: &Path, pattern: &str) -> Result<Vec<SearchResult>> {
    let content = super::read_file::read_file(path)?;
    let regex = Regex::new(pattern)
        .map_err(|e| color_eyre::eyre::eyre!("Invalid regex pattern: {}", e))?;

    let results: Vec<SearchResult> = content
        .lines()
        .enumerate()
        .filter(|(_, line)| regex.is_match(line))
        .map(|(i, line)| SearchResult {
            file: path.display().to_string(),
            line: i + 1,
            content: line.to_string(),
        })
        .collect();

    Ok(results)
}

/// Search for pattern in multiple files
pub fn search_pattern(files: &[&Path], pattern: &str) -> Result<Vec<SearchResult>> {
    let mut all_results = Vec::new();

    for file in files {
        if file.is_file() {
            let results = search_file(file, pattern)?;
            all_results.extend(results);
        }
    }

    Ok(all_results)
}

/// Search with context
pub fn search_with_context(path: &Path, pattern: &str, context: usize) -> Result<Vec<(SearchResult, Vec<String>)>> {
    let content = super::read_file::read_file(path)?;
    let regex = Regex::new(pattern)
        .map_err(|e| color_eyre::eyre::eyre!("Invalid regex pattern: {}", e))?;

    let lines: Vec<&str> = content.lines().collect();
    let mut results = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if regex.is_match(line) {
            let start = i.saturating_sub(context);
            let end = (i + context + 1).min(lines.len());
            let context_lines: Vec<String> = lines[start..end]
                .iter()
                .map(|l| l.to_string())
                .collect();

            results.push((
                SearchResult {
                    file: path.display().to_string(),
                    line: i + 1,
                    content: line.to_string(),
                },
                context_lines,
            ));
        }
    }

    Ok(results)
}