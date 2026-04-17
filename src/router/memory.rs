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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_memory_policy_trivial() {
        assert_eq!(pick_memory_policy(Intent::Read, Complexity::Trivial, AgentMode::Chat), MemoryPolicy::None);
    }

    #[test]
    fn test_pick_memory_policy_chat_simple() {
        assert_eq!(pick_memory_policy(Intent::Read, Complexity::Simple, AgentMode::Chat), MemoryPolicy::None);
    }

    #[test]
    fn test_pick_memory_policy_plan_mode() {
        assert_eq!(pick_memory_policy(Intent::Plan, Complexity::Simple, AgentMode::Plan), MemoryPolicy::Recent);
    }

    #[test]
    fn test_pick_memory_policy_review_mode() {
        assert_eq!(pick_memory_policy(Intent::Review, Complexity::Simple, AgentMode::Review), MemoryPolicy::Relevant);
        assert_eq!(pick_memory_policy(Intent::Debug, Complexity::Complex, AgentMode::Debug), MemoryPolicy::Relevant);
    }

    #[test]
    fn test_pick_memory_policy_build_complex() {
        assert_eq!(pick_memory_policy(Intent::Write, Complexity::Complex, AgentMode::Build), MemoryPolicy::Full);
    }

    #[test]
    fn test_get_memory_hint() {
        assert_eq!(get_memory_hint(MemoryPolicy::None), "No memory loading needed");
        assert_eq!(get_memory_hint(MemoryPolicy::Recent), "Load recently modified files");
        assert_eq!(get_memory_hint(MemoryPolicy::Relevant), "Load files relevant to current task");
        assert_eq!(get_memory_hint(MemoryPolicy::Full), "Load all recent project context");
    }
}
