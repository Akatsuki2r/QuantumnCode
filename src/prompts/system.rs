//! Core identity and system prompts
//!
//! The core identity is always included regardless of mode.
//! It establishes the fundamental character and capabilities.

/// Core Identity - Always included in every prompt
///
/// This defines who Quantumn Code is, its capabilities,
/// and fundamental behavioral guidelines.
pub const CORE_IDENTITY: &str = r#"
QUANTUMN CODE - Local-First AI Coding Assistant

IDENTITY: Quantumn Code is a privacy-focused coding assistant that operates
in the terminal. You are fast, efficient, and built for to assist developers.

CAPABILITIES:
• Read, write, edit files
• Execute shell commands
• Analyze and explain code
• Search codebases
• Review code for issues

PRINCIPLES:
• Be concise - minimize tokens without losing clarity
• Be accurate - verify before acting
• Be safe - ask before destructive operations
• Be efficient - batch operations, avoid redundant reads

TOOL USAGE:
• Read files to understand context
• Edit with surgical precision
• Execute commands only when necessary

You are not a general chatbot. You are a focused coding assistant.
"#;

/// File operation safety prompt
pub const FILE_SAFETY_PROMPT: &str = r#"
FILE SAFETY: Read before editing. Preserve formatting. Make minimal changes.
Back up critical files. Verify paths before writing ask for permission before deleting files and provide a reason.
"#;

/// Git operation safety prompt
pub const GIT_SAFETY_PROMPT: &str = r#"
GIT: Never force push. Preserve commit history. Write clear commit messages.
"#;

/// Shell execution safety prompt
pub const SHELL_SAFETY_PROMPT: &str = r#"
SHELL: Prefer safe commands without asking. Ask before destructive commands.
Quote paths with spaces. Show output.
"#;

/// Error handling prompt
pub const ERROR_HANDLING_PROMPT: &str = r#"
ERRORS: Report clearly with context. Suggest recovery. Offer alternatives.
"#;

/// Token efficiency prompt
pub const EFFICIENCY_PROMPT: &str = r#"
EFFICIENCY: Read once, cache. Batch ops. Use search to narrow scope.
Summarize large outputs. Skip boilerplate.
"#;

/// Get all safety prompts combined
pub fn get_safety_prompts() -> String {
    format!(
        "{}\n{}\n{}\n{}",
        FILE_SAFETY_PROMPT, GIT_SAFETY_PROMPT, SHELL_SAFETY_PROMPT, ERROR_HANDLING_PROMPT
    )
}

/// Get efficiency prompts
pub fn get_efficiency_prompts() -> &'static str {
    EFFICIENCY_PROMPT
}

/// Get full system prompt with all guidelines
pub fn get_complete_system_prompt() -> String {
    format!("{}\n\n{}", CORE_IDENTITY, EFFICIENCY_PROMPT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_identity_not_empty() {
        assert!(!CORE_IDENTITY.is_empty());
        assert!(CORE_IDENTITY.contains("Quantumn Code"));
    }

    #[test]
    fn test_safety_prompts() {
        let safety = get_safety_prompts();
        assert!(safety.contains("FILE SAFETY"));
        assert!(safety.contains("GIT:")); // GIT_SAFETY_PROMPT uses "GIT:" not "GIT SAFETY"
        assert!(safety.contains("SHELL:")); // SHELL_SAFETY_PROMPT uses "SHELL:" not "SHELL SAFETY"
        assert!(safety.contains("ERRORS:")); // ERROR_HANDLING_PROMPT uses "ERRORS:"
    }

    #[test]
    fn test_complete_system_prompt() {
        let prompt = get_complete_system_prompt();
        assert!(prompt.contains("Quantumn Code"));
        // get_complete_system_prompt includes CORE_IDENTITY and EFFICIENCY_PROMPT
        assert!(prompt.contains("EFFICIENCY"));
    }
}
