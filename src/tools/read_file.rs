//! File reading tool

use std::path::Path;
use color_eyre::eyre::Result;

/// Read file contents
pub fn read_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to read file {:?}: {}", path, e))
}

/// Read file with line numbers
pub fn read_file_with_lines(path: &Path) -> Result<String> {
    let content = read_file(path)?;
    let lines: Vec<String> = content
        .lines()
        .enumerate()
        .map(|(i, line)| format!("{:6} | {}", i + 1, line))
        .collect();
    Ok(lines.join("\n"))
}

/// Read file with limit
pub fn read_file_limit(path: &Path, start: usize, limit: usize) -> Result<String> {
    let content = read_file(path)?;
    let lines: Vec<&str> = content.lines().skip(start).take(limit).collect();
    Ok(lines.join("\n"))
}