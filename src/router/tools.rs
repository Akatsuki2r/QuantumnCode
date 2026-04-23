//! Tool policy selection
//!
//! Handles per-intent tool permissions and activation levels.

use crate::router::types::{AgentMode, Intent, ToolPolicy};

/// Get the tool policy based on intent and mode
pub fn pick_tools(intent: Intent, mode: AgentMode) -> ToolPolicy {
    match mode {
        // Build mode has full tools
        AgentMode::Build => build_mode_tools(intent),

        // Review mode - read only
        AgentMode::Review => ToolPolicy::read_only(),

        // Debug mode - read + limited execution
        AgentMode::Debug => debug_mode_tools(),

        // Plan mode - analysis only
        AgentMode::Plan => plan_mode_tools(),

        // Chat mode - minimal tools
        AgentMode::Chat => chat_mode_tools(),
    }
}

/// Tools for build mode based on intent
fn build_mode_tools(intent: Intent) -> ToolPolicy {
    match intent {
        // Destructive operations require confirmation
        Intent::Delete | Intent::Bash | Intent::Write => {
            ToolPolicy::with_confirmation(ToolPolicy::default_policy())
        }
        // Git operations also need confirmation
        Intent::Git => ToolPolicy::with_confirmation(ToolPolicy::default_policy()),
        // Everything else is fine
        _ => ToolPolicy::default_policy(),
    }
}

/// Tools for debug mode
fn debug_mode_tools() -> ToolPolicy {
    ToolPolicy {
        allowed_tools: vec![
            "Read".to_string(),
            "Grep".to_string(),
            "Glob".to_string(),
            "Bash".to_string(),
        ],
        disallowed_tools: vec!["Write".to_string()],
        require_confirmation: true,
    }
}

/// Tools for plan mode
fn plan_mode_tools() -> ToolPolicy {
    ToolPolicy {
        allowed_tools: vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string()],
        disallowed_tools: vec!["Write".to_string(), "Bash".to_string()],
        require_confirmation: false,
    }
}

/// Tools for chat mode
fn chat_mode_tools() -> ToolPolicy {
    ToolPolicy {
        allowed_tools: vec!["Read".to_string()],
        disallowed_tools: vec![
            "Write".to_string(),
            "Bash".to_string(),
            "Grep".to_string(),
            "Glob".to_string(),
        ],
        require_confirmation: false,
    }
}

/// Filter a list of tool names against a policy
pub fn filter_tools_by_policy<'a>(tool_names: &'a [&str], policy: &ToolPolicy) -> Vec<&'a str> {
    tool_names
        .iter()
        .filter(|name| policy.is_tool_allowed(name))
        .copied()
        .collect()
}
