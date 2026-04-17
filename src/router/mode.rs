//! Mode selection and state machine
//!
//! Handles selection of execution mode based on intent and complexity.

use crate::router::types::{AgentMode, Complexity, Intent};

/// Pick the appropriate mode based on intent and complexity
pub fn pick_mode(intent: Intent, complexity: Complexity) -> AgentMode {
    match intent {
        // Read operations - typically chat or review
        Intent::Read | Intent::Explain | Intent::Chat | Intent::Help => {
            if complexity >= Complexity::Complex {
                AgentMode::Review
            } else {
                AgentMode::Chat
            }
        }

        // Write operations - build mode
        Intent::Write | Intent::Edit => AgentMode::Build,

        // Delete operations - build with caution
        Intent::Delete => AgentMode::Build,

        // Shell operations - build mode
        Intent::Bash | Intent::Git => AgentMode::Build,

        // Search operations - review mode
        Intent::Grep | Intent::Glob | Intent::Find => AgentMode::Review,

        // Review operations - review mode
        Intent::Review => AgentMode::Review,

        // Debug operations - debug mode
        Intent::Debug => AgentMode::Debug,

        // Planning operations - plan mode
        Intent::Plan | Intent::Design => AgentMode::Plan,

        // Fallback
        Intent::Unknown => AgentMode::Chat,
    }
}

/// Check if a mode transition is valid
pub fn can_transition(current: AgentMode, target: AgentMode) -> bool {
    current.can_transition_to(target)
}

/// Transition to a new mode if valid, returns Some(new_mode) or None
pub fn transition(current: AgentMode, target: AgentMode) -> Option<AgentMode> {
    if can_transition(current, target) {
        Some(target)
    } else {
        None
    }
}

/// Get the mode instruction for system prompt injection
pub fn get_mode_instruction(mode: AgentMode) -> &'static str {
    mode.instruction()
}

/// Get the mode display name
pub fn get_mode_display(mode: AgentMode) -> &'static str {
    mode.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_mode_read_operations() {
        // Simple reads go to Chat
        assert_eq!(pick_mode(Intent::Read, Complexity::Simple), AgentMode::Chat);
        assert_eq!(pick_mode(Intent::Explain, Complexity::Simple), AgentMode::Chat);
        assert_eq!(pick_mode(Intent::Chat, Complexity::Simple), AgentMode::Chat);
        assert_eq!(pick_mode(Intent::Help, Complexity::Simple), AgentMode::Chat);

        // Complex reads go to Review
        assert_eq!(pick_mode(Intent::Read, Complexity::Complex), AgentMode::Review);
        assert_eq!(pick_mode(Intent::Explain, Complexity::Complex), AgentMode::Review);
    }

    #[test]
    fn test_pick_mode_write_operations() {
        // All write/edit/delete operations go to Build
        assert_eq!(pick_mode(Intent::Write, Complexity::Simple), AgentMode::Build);
        assert_eq!(pick_mode(Intent::Edit, Complexity::Simple), AgentMode::Build);
        assert_eq!(pick_mode(Intent::Delete, Complexity::Simple), AgentMode::Build);
    }

    #[test]
    fn test_pick_mode_shell_operations() {
        // Bash/Git operations go to Build
        assert_eq!(pick_mode(Intent::Bash, Complexity::Simple), AgentMode::Build);
        assert_eq!(pick_mode(Intent::Git, Complexity::Simple), AgentMode::Build);
    }

    #[test]
    fn test_pick_mode_search_operations() {
        // Search operations go to Review
        assert_eq!(pick_mode(Intent::Grep, Complexity::Simple), AgentMode::Review);
        assert_eq!(pick_mode(Intent::Glob, Complexity::Simple), AgentMode::Review);
        assert_eq!(pick_mode(Intent::Find, Complexity::Simple), AgentMode::Review);
    }

    #[test]
    fn test_pick_mode_planning() {
        // Planning operations go to Plan
        assert_eq!(pick_mode(Intent::Plan, Complexity::Simple), AgentMode::Plan);
        assert_eq!(pick_mode(Intent::Design, Complexity::Simple), AgentMode::Plan);
    }

    #[test]
    fn test_pick_mode_debug() {
        assert_eq!(pick_mode(Intent::Debug, Complexity::Simple), AgentMode::Debug);
    }

    #[test]
    fn test_mode_transitions() {
        // Valid transitions
        assert!(can_transition(AgentMode::Chat, AgentMode::Plan));
        assert!(can_transition(AgentMode::Chat, AgentMode::Build));
        assert!(can_transition(AgentMode::Plan, AgentMode::Build));
        assert!(can_transition(AgentMode::Build, AgentMode::Plan)); // replanning
        assert!(can_transition(AgentMode::Build, AgentMode::Debug));

        // Invalid transitions
        assert!(!can_transition(AgentMode::Plan, AgentMode::Debug));
        assert!(!can_transition(AgentMode::Review, AgentMode::Build));
    }

    #[test]
    fn test_transition_function() {
        assert_eq!(transition(AgentMode::Chat, AgentMode::Plan), Some(AgentMode::Plan));
        assert_eq!(transition(AgentMode::Plan, AgentMode::Debug), None);
    }

    #[test]
    fn test_mode_instructions() {
        assert!(!get_mode_instruction(AgentMode::Chat).is_empty());
        assert!(!get_mode_instruction(AgentMode::Plan).is_empty());
        assert!(!get_mode_instruction(AgentMode::Build).is_empty());
        assert!(!get_mode_instruction(AgentMode::Review).is_empty());
        assert!(!get_mode_instruction(AgentMode::Debug).is_empty());
    }
}
