//! Context budget allocation
//!
//! Handles token budget allocation for conversation context.

use crate::router::types::{AgentMode, Complexity, ContextBudget};

/// Pick the context budget based on complexity and mode
pub fn pick_budget(complexity: Complexity, mode: AgentMode) -> ContextBudget {
    let base = ContextBudget::from_complexity(complexity);

    // Reduce budget for certain modes that need less context
    match mode {
        // Chat needs minimal context
        AgentMode::Chat => ContextBudget::Minimal,

        // Plan needs moderate context for analysis
        AgentMode::Plan => ContextBudget::Relevant.max(base),

        // Review needs good context for understanding code
        AgentMode::Review => ContextBudget::Relevant.max(base),

        // Debug needs moderate context
        AgentMode::Debug => ContextBudget::Relevant.max(base),

        // Build needs full context for implementation
        AgentMode::Build => base,
    }
}

/// Calculate available tokens for agent after system prompt
pub fn agent_token_budget(budget: ContextBudget, system_prompt_tokens: usize) -> usize {
    budget.tokens().saturating_sub(system_prompt_tokens)
}

/// Estimate tokens in a prompt
pub fn estimate_prompt_tokens(prompt: &str) -> usize {
    // Rough estimate: ~4 characters per token for English
    prompt.len() / 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_budget_trivial_tasks() {
        // Trivial tasks get minimal budget regardless of mode
        assert_eq!(pick_budget(Complexity::Trivial, AgentMode::Chat), ContextBudget::Minimal);
        assert_eq!(pick_budget(Complexity::Trivial, AgentMode::Build), ContextBudget::Minimal);
    }

    #[test]
    fn test_pick_budget_chat_mode() {
        // Chat always gets minimal
        assert_eq!(pick_budget(Complexity::Heavy, AgentMode::Chat), ContextBudget::Minimal);
        assert_eq!(pick_budget(Complexity::Simple, AgentMode::Chat), ContextBudget::Minimal);
    }

    #[test]
    fn test_pick_budget_plan_mode() {
        // Plan mode gets at least Relevant
        assert_eq!(pick_budget(Complexity::Trivial, AgentMode::Plan), ContextBudget::Relevant);
        assert_eq!(pick_budget(Complexity::Complex, AgentMode::Plan), ContextBudget::Standard);
    }

    #[test]
    fn test_pick_budget_review_mode() {
        // Review mode gets at least Relevant
        assert_eq!(pick_budget(Complexity::Trivial, AgentMode::Review), ContextBudget::Relevant);
        assert_eq!(pick_budget(Complexity::Heavy, AgentMode::Review), ContextBudget::Comprehensive);
    }

    #[test]
    fn test_pick_budget_build_mode() {
        // Build mode uses base budget
        assert_eq!(pick_budget(Complexity::Simple, AgentMode::Build), ContextBudget::Minimal);
        assert_eq!(pick_budget(Complexity::Complex, AgentMode::Build), ContextBudget::Standard);
        assert_eq!(pick_budget(Complexity::Heavy, AgentMode::Build), ContextBudget::Comprehensive);
    }

    #[test]
    fn test_agent_token_budget() {
        // Test budget calculation after system prompt
        let budget = ContextBudget::Minimal; // 4000 tokens
        assert_eq!(agent_token_budget(budget, 1000), 3000);
        assert_eq!(agent_token_budget(budget, 5000), 0); // saturates at 0
    }

    #[test]
    fn test_estimate_prompt_tokens() {
        // ~4 chars per token
        assert_eq!(estimate_prompt_tokens("hello"), 1);
        // "Hello, world! This is a test." = 29 chars / 4 = 7
        assert_eq!(estimate_prompt_tokens("Hello, world! This is a test."), 7);
        assert_eq!(estimate_prompt_tokens(""), 0);
    }

    #[test]
    fn test_context_budget_from_complexity() {
        assert_eq!(ContextBudget::from_complexity(Complexity::Trivial), ContextBudget::Minimal);
        assert_eq!(ContextBudget::from_complexity(Complexity::Simple), ContextBudget::Minimal);
        assert_eq!(ContextBudget::from_complexity(Complexity::Moderate), ContextBudget::Relevant);
        assert_eq!(ContextBudget::from_complexity(Complexity::Complex), ContextBudget::Standard);
        assert_eq!(ContextBudget::from_complexity(Complexity::Heavy), ContextBudget::Comprehensive);
    }
}
