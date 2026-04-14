//! Local LLM discovery module
//!
//! Auto-detects installed models from Ollama, LM Studio, and llama.cpp.
//! Runs on startup and stores discovered models in config.

use color_eyre::eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// Discovered local model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModel {
    /// Provider (ollama, lm_studio, llama_cpp)
    pub provider: String,
    /// Model name
    pub name: String,
    /// Full path or identifier
    pub path: String,
    /// Size in bytes (if available)
    pub size_bytes: Option<u64>,
}

/// Discovery configuration stored in config
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalModelConfig {
    /// Ollama models
    pub ollama: HashMap<String, OllamaModelInfo>,
    /// LM Studio models
    pub lm_studio: HashMap<String, LmStudioModelInfo>,
    /// llama.cpp models
    pub llama_cpp: HashMap<String, LlamaCppModelInfo>,
    /// Last discovery timestamp
    pub last_discovery: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelInfo {
    pub name: String,
    pub size: String,
    pub modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LmStudioModelInfo {
    pub path: PathBuf,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaCppModelInfo {
    pub path: PathBuf,
    pub size_bytes: u64,
}

/// Discover all local models
pub fn discover_all_models() -> LocalModelConfig {
    let mut config = LocalModelConfig::default();

    // Discover from each provider
    discover_ollama_models(&mut config);
    discover_lm_studio_models(&mut config);
    discover_llama_cpp_models(&mut config);

    // Set timestamp
    config.last_discovery = Some(chrono::Utc::now().to_rfc3339());

    config
}

/// Discover Ollama models using `ollama list`
fn discover_ollama_models(config: &mut LocalModelConfig) {
    let output = Command::new("ollama").args(["list"]).output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse ollama list output
        // Format: NAME                    ID          SIZE      MODIFIED
        for line in stdout.lines().skip(1) {
            // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                let size = parts[2].to_string();
                let modified = parts[3..].join(" ");

                config.ollama.insert(
                    name.clone(),
                    OllamaModelInfo {
                        name,
                        size,
                        modified,
                    },
                );
            }
        }
    }
}

/// Discover LM Studio models from ~/.lmstudio/models/
fn discover_lm_studio_models(config: &mut LocalModelConfig) {
    let home = std::env::var("HOME").unwrap_or_default();
    let models_dir = PathBuf::from(format!("{}/.lmstudio/models", home));

    if !models_dir.exists() {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(&models_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Check for GGUF files in subdirectories
            if path.is_dir() {
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.extension().map(|e| e == "gguf").unwrap_or(false) {
                            let size = std::fs::metadata(&sub_path).map(|m| m.len()).unwrap_or(0);

                            let name = sub_path
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "unknown".to_string());

                            config.lm_studio.insert(
                                name.clone(),
                                LmStudioModelInfo {
                                    path: sub_path,
                                    size_bytes: size,
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Discover llama.cpp models from common locations
fn discover_llama_cpp_models(config: &mut LocalModelConfig) {
    let possible_paths = vec![
        PathBuf::from("~/.llama.cpp/models"),
        PathBuf::from("~/models"),
        PathBuf::from("/usr/local/share/llama.cpp/models"),
    ];

    for base_path in possible_paths {
        let expanded = expand_tilde(&base_path);
        if !expanded.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&expanded) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "gguf").unwrap_or(false) {
                    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                    let name = path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    config.llama_cpp.insert(
                        name.clone(),
                        LlamaCppModelInfo {
                            path,
                            size_bytes: size,
                        },
                    );
                }
            }
        }
    }
}

/// Expand tilde in path
fn expand_tilde(path: &PathBuf) -> PathBuf {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(path.to_string_lossy().replacen("~", &home, 1));
        }
    }
    path.clone()
}

/// Get all discovered models as a flat list
pub fn get_all_models(config: &LocalModelConfig) -> Vec<LocalModel> {
    let mut models = Vec::new();

    for (name, info) in &config.ollama {
        models.push(LocalModel {
            provider: "ollama".to_string(),
            name: name.clone(),
            path: format!("ollama://{}", name),
            size_bytes: parse_size(&info.size),
        });
    }

    for (name, info) in &config.lm_studio {
        models.push(LocalModel {
            provider: "lm_studio".to_string(),
            name: name.clone(),
            path: info.path.to_string_lossy().to_string(),
            size_bytes: Some(info.size_bytes),
        });
    }

    for (name, info) in &config.llama_cpp {
        models.push(LocalModel {
            provider: "llama_cpp".to_string(),
            name: name.clone(),
            path: info.path.to_string_lossy().to_string(),
            size_bytes: Some(info.size_bytes),
        });
    }

    models
}

/// Parse size string like "4.7GB" or "1.2MB" to bytes
fn parse_size(size_str: &str) -> Option<u64> {
    let size_str = size_str.trim();
    let multiplier: u64;
    let numeric: f64;

    if size_str.ends_with("GB") || size_str.ends_with("G") {
        multiplier = 1024 * 1024 * 1024;
        numeric = size_str[..size_str.len() - 2].parse().ok()?;
    } else if size_str.ends_with("MB") || size_str.ends_with("M") {
        multiplier = 1024 * 1024;
        numeric = size_str[..size_str.len() - 2].parse().ok()?;
    } else if size_str.ends_with("KB") || size_str.ends_with("K") {
        multiplier = 1024;
        numeric = size_str[..size_str.len() - 2].parse().ok()?;
    } else if size_str.ends_with("B") {
        multiplier = 1;
        numeric = size_str[..size_str.len() - 1].parse().ok()?;
    } else {
        return None;
    }

    Some((numeric * multiplier as f64) as u64)
}

/// Format bytes to human readable string
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("4.7GB"), Some(4_700_000_000));
        assert_eq!(parse_size("1.2MB"), Some(1_200_000));
        assert_eq!(parse_size("500KB"), Some(500_000));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(4_700_000_000), "4.4GB");
        assert_eq!(format_size(1_200_000), "1.1MB");
        assert_eq!(format_size(500_000), "488.3KB");
    }
}
