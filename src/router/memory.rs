//! Memory policy selection
//!
//! Handles memory loading strategy based on intent and mode.

use crate::router::types::{AgentMode, Complexity, Intent, MemoryPolicy};

/// Pick the memory loading policy
pub fn pick_memory_policy(intent: Intent, complexity: Complexity, mode: AgentMode) -> MemoryPolicy {
    // Trivial tasks don't need memory
    if complexity == Complexity::Trivial {
        return MemoryPolicy::None;
    }

    // Simple chat doesn't need memory
    if mode == AgentMode::Chat && complexity <= Complexity::Simple {
        return MemoryPolicy::None;
    }

    // Planning benefits from recent context
    if mode == AgentMode::Plan {
        return MemoryPolicy::Recent;
    }

    // Review/debug need relevant context
    if mode == AgentMode::Review || mode == AgentMode::Debug {
        return MemoryPolicy::Relevant;
    }

    // Build mode needs full context for complex tasks
    if mode == AgentMode::Build && complexity >= Complexity::Complex {
        return MemoryPolicy::Full;
    }

    // Default to recent
    MemoryPolicy::Recent
}

/// Get memory loading hint for a policy
pub fn get_memory_hint(policy: MemoryPolicy) -> &'static str {
    match policy {
        MemoryPolicy::None => "No memory loading needed",
        MemoryPolicy::Recent => "Load recently modified files",
        MemoryPolicy::Relevant => "Load files relevant to current task",
        MemoryPolicy::Full => "Load all recent project context",
    }
}
