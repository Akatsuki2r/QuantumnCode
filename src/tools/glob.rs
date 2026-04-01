//! File globbing tool

use std::path::{Path, PathBuf};
use color_eyre::eyre::Result;

/// Find files matching pattern
pub fn find_files(pattern: &str, base: &Path) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();

    if !base.exists() {
        return Ok(results);
    }

    // Simple glob implementation
    // For more complex patterns, consider using the `glob` crate
    if pattern.contains('*') {
        // Handle glob patterns
        let parts: Vec<&str> = pattern.split('*').collect();
        search_with_glob(base, &parts, &mut results);
    } else {
        // Exact match
        let path = base.join(pattern);
        if path.exists() {
            results.push(path);
        }
    }

    Ok(results)
}

fn search_with_glob(dir: &Path, pattern_parts: &[&str], results: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                search_with_glob(&path, pattern_parts, results);
            } else if matches_pattern(&path, pattern_parts) {
                results.push(path);
            }
        }
    }
}

fn matches_pattern(path: &Path, pattern_parts: &[&str]) -> bool {
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if pattern_parts.is_empty() {
        return true;
    }

    if pattern_parts.len() == 1 {
        return file_name.contains(pattern_parts[0]);
    }

    let first = pattern_parts[0];
    let last = pattern_parts[pattern_parts.len() - 1];

    if !pattern_parts[0].is_empty() && !file_name.starts_with(first) {
        return false;
    }

    if !pattern_parts[pattern_parts.len() - 1].is_empty() && !file_name.ends_with(last) {
        return false;
    }

    true
}

/// Find files by extension
pub fn find_by_extension(ext: &str, base: &Path) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    find_by_extension_recursive(base, ext, &mut results);
    Ok(results)
}

fn find_by_extension_recursive(dir: &Path, ext: &str, results: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                find_by_extension_recursive(&path, ext, results);
            } else if path.extension().map(|e| e == ext).unwrap_or(false) {
                results.push(path);
            }
        }
    }
}

/// Find all files recursively
pub fn find_all_files(base: &Path) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    find_all_recursive(base, &mut results);
    Ok(results)
}

fn find_all_recursive(dir: &Path, results: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Skip hidden directories and common non-source directories
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') || ["node_modules", "target", "build", "dist", "vendor"].contains(&name) {
                        continue;
                    }
                }
                find_all_recursive(&path, results);
            } else {
                results.push(path);
            }
        }
    }
}