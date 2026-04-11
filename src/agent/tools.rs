//! Minimalistic tools for agentic workflow

use std::path::Path;
use std::process::Output;
use serde::{Deserialize, Serialize};

/// A tool that the AI can call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
}

/// A tool call from the AI
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arg: String,
    pub content: Option<String>,
}

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

impl ToolResult {
    pub fn success(stdout: String) -> Self {
        Self {
            stdout,
            stderr: String::new(),
            success: true,
        }
    }

    pub fn error(msg: String) -> Self {
        Self {
            stdout: String::new(),
            stderr: msg,
            success: false,
        }
    }
}

/// Execute a Read tool
pub fn tool_read(arg: &str) -> ToolResult {
    let path = Path::new(arg);
    if !path.exists() {
        return ToolResult::error(format!("File not found: {}", arg));
    }

    match std::fs::read_to_string(path) {
        Ok(content) => ToolResult::success(content),
        Err(e) => ToolResult::error(format!("Failed to read {}: {}", arg, e)),
    }
}

/// Execute a Write tool
pub fn tool_write(arg: &str, content: &str) -> ToolResult {
    let path = Path::new(arg);

    // Create parent directories
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return ToolResult::error(format!("Failed to create directory: {}", e));
            }
        }
    }

    match std::fs::write(path, content) {
        Ok(_) => ToolResult::success(format!("Written {} bytes to {}", content.len(), arg)),
        Err(e) => ToolResult::error(format!("Failed to write {}: {}", arg, e)),
    }
}

/// Execute a Bash tool
pub fn tool_bash(arg: &str) -> ToolResult {
    // Parse command - use sh -c for proper shell behavior
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(arg)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if out.status.success() {
                ToolResult {
                    stdout: if stdout.is_empty() { "(no output)".to_string() } else { stdout },
                    stderr,
                    success: true,
                }
            } else {
                ToolResult {
                    stdout,
                    stderr,
                    success: false,
                }
            }
        }
        Err(e) => ToolResult::error(format!("Failed to execute: {}", e)),
    }
}

/// Execute a Grep tool
pub fn tool_grep(arg: &str, path: &str) -> ToolResult {
    let path_arg = if path.is_empty() { "." } else { path };

    let output = std::process::Command::new("grep")
        .args(["-n", "-r", arg, path_arg])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            ToolResult {
                stdout: if stdout.is_empty() { "(no matches)".to_string() } else { stdout },
                stderr,
                success: out.status.success(),
            }
        }
        Err(e) => ToolResult::error(format!("Grep failed: {}", e)),
    }
}

/// Execute a Glob tool
pub fn tool_glob(arg: &str) -> ToolResult {
    let pattern = if arg.is_empty() { "*" } else { arg };

    // Use find with glob pattern
    let output = std::process::Command::new("find")
        .args(["." , "-name", pattern, "-type", "f"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            ToolResult::success(if stdout.is_empty() { "(no matches)".to_string() } else { stdout })
        }
        Err(e) => ToolResult::error(format!("Glob failed: {}", e)),
    }
}

/// Execute a tool by name
pub fn execute_tool(call: &ToolCall) -> ToolResult {
    match call.name.to_lowercase().as_str() {
        "read" => tool_read(&call.arg),
        "write" => tool_write(&call.arg, call.content.as_deref().unwrap_or("")),
        "bash" | "shell" | "cmd" => tool_bash(&call.arg),
        "grep" | "search" => tool_grep(&call.arg, ""),
        "glob" | "find" | "files" => tool_glob(&call.arg),
        _ => ToolResult::error(format!("Unknown tool: {}", call.name)),
    }
}

/// Get all available tools
pub fn get_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "Read".to_string(),
            description: "Read file contents. Arg: file path".to_string(),
        },
        Tool {
            name: "Write".to_string(),
            description: "Write content to file. Arg: file path, Content: content to write".to_string(),
        },
        Tool {
            name: "Bash".to_string(),
            description: "Execute shell command. Arg: command string".to_string(),
        },
        Tool {
            name: "Grep".to_string(),
            description: "Search file contents. Arg: pattern, Path: directory to search".to_string(),
        },
        Tool {
            name: "Glob".to_string(),
            description: "Find files by pattern. Arg: glob pattern (e.g., *.rs)".to_string(),
        },
    ]
}
