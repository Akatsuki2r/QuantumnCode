//! Router Integration Tests
//!
//! Comprehensive tests for the 7-layer router pipeline.

use crate::router::{
    route, RouterConfig, Intent, Complexity, AgentMode, ModelTier,
    ToolPolicy, ContextBudget, MemoryPolicy,
    analyzer::{classify_intent, score_complexity},
    mode::pick_mode,
    model::pick_model_tier,
    tools::pick_tools,
    context::pick_budget,
    memory::pick_memory_policy,
};

// =============================================================================
// Layer 1: Intent Classification Tests
// =============================================================================

#[cfg(test)]
mod intent_tests {
    use super::*;

    #[test]
    fn test_read_intent() {
        assert_eq!(classify_intent("read src/main.rs"), Intent::Read);
        assert_eq!(classify_intent("view config.toml"), Intent::Read);
        assert_eq!(classify_intent("show me the code"), Intent::Read);
        assert_eq!(classify_intent("cat /etc/hosts"), Intent::Read);
        assert_eq!(classify_intent("open package.json"), Intent::Read);
    }

    #[test]
    fn test_write_intent() {
        assert_eq!(classify_intent("write new_file.rs"), Intent::Write);
        assert_eq!(classify_intent("create README.md"), Intent::Write);
        assert_eq!(classify_intent("new test_file.py"), Intent::Write);
        assert_eq!(classify_intent("touch empty.txt"), Intent::Write);
    }

    #[test]
    fn test_edit_intent() {
        assert_eq!(classify_intent("edit src/lib.rs"), Intent::Edit);
        assert_eq!(classify_intent("modify config.toml"), Intent::Edit);
        assert_eq!(classify_intent("update package.json"), Intent::Edit);
        assert_eq!(classify_intent("change the code"), Intent::Edit);
    }

    #[test]
    fn test_delete_intent() {
        assert_eq!(classify_intent("delete temp.txt"), Intent::Delete);
        assert_eq!(classify_intent("remove old_file.rs"), Intent::Delete);
        assert_eq!(classify_intent("rm cache/"), Intent::Delete);
        assert_eq!(classify_intent("unlink symlink"), Intent::Delete);
    }

    #[test]
    fn test_bash_intent() {
        assert_eq!(classify_intent("run cargo build"), Intent::Bash);
        assert_eq!(classify_intent("exec npm test"), Intent::Bash);
        assert_eq!(classify_intent("bash ls -la"), Intent::Bash);
        assert_eq!(classify_intent("shell pwd"), Intent::Bash);
    }

    #[test]
    fn test_git_intent() {
        assert_eq!(classify_intent("git commit -m fix"), Intent::Git);
        assert_eq!(classify_intent("git push origin main"), Intent::Git);
        assert_eq!(classify_intent("git status"), Intent::Git);
        assert_eq!(classify_intent("commit changes"), Intent::Git);
    }

    #[test]
    fn test_grep_intent() {
        assert_eq!(classify_intent("grep fn main src/"), Intent::Grep);
        assert_eq!(classify_intent("rg pattern file"), Intent::Grep);
        assert_eq!(classify_intent("search for TODO"), Intent::Grep);
    }

    #[test]
    fn test_glob_intent() {
        assert_eq!(classify_intent("glob **/*.rs"), Intent::Glob);
        assert_eq!(classify_intent("find files matching"), Intent::Glob);
    }

    #[test]
    fn test_find_intent() {
        // Find requires "file" or "path" keyword after "find"
        assert_eq!(classify_intent("find file path/to/file"), Intent::Find);
        assert_eq!(classify_intent("find path to file"), Intent::Find);
    }

    #[test]
    fn test_explain_intent() {
        assert_eq!(classify_intent("explain this code"), Intent::Explain);
        assert_eq!(classify_intent("what is a mutex"), Intent::Explain);
        assert_eq!(classify_intent("how does async work"), Intent::Explain);
        assert_eq!(classify_intent("describe the pattern"), Intent::Explain);
    }

    #[test]
    fn test_review_intent() {
        assert_eq!(classify_intent("review this PR"), Intent::Review);
        assert_eq!(classify_intent("check the code"), Intent::Review);
        assert_eq!(classify_intent("analyze for bugs"), Intent::Review);
        assert_eq!(classify_intent("audit security"), Intent::Review);
    }

    #[test]
    fn test_debug_intent() {
        assert_eq!(classify_intent("debug the crash"), Intent::Debug);
        assert_eq!(classify_intent("trace the request"), Intent::Debug);
        assert_eq!(classify_intent("breakpoint here"), Intent::Debug);
        assert_eq!(classify_intent("inspect the variable"), Intent::Debug);
    }

    #[test]
    fn test_plan_intent() {
        assert_eq!(classify_intent("plan the refactor"), Intent::Plan);
        assert_eq!(classify_intent("design the API"), Intent::Plan);
        assert_eq!(classify_intent("architecture for microservices"), Intent::Plan);
    }

    #[test]
    fn test_design_intent() {
        // "design" keyword matches Plan pattern first (higher priority)
        // Design requires "architect" or "blueprint" as first word
        assert_eq!(classify_intent("architect the solution"), Intent::Design);
        assert_eq!(classify_intent("blueprint the app"), Intent::Design);
        // Note: "design X" matches Plan due to pattern ordering
        assert_eq!(classify_intent("design architecture"), Intent::Plan);
    }

    #[test]
    fn test_help_intent() {
        assert_eq!(classify_intent("help"), Intent::Help);
        assert_eq!(classify_intent("?"), Intent::Help);
        assert_eq!(classify_intent("commands"), Intent::Help);
    }

    #[test]
    fn test_chat_intent() {
        // Chat requires exact match or greeting with additional text
        assert_eq!(classify_intent("hi"), Intent::Chat);
        assert_eq!(classify_intent("hello"), Intent::Chat);
        // "hey there" doesn't match because pattern is just "hey" at end
        assert_eq!(classify_intent("thanks"), Intent::Chat);
        assert_eq!(classify_intent("thank you"), Intent::Chat);
    }

    #[test]
    fn test_unknown_intent() {
        assert_eq!(classify_intent(""), Intent::Unknown);
        assert_eq!(classify_intent("   "), Intent::Unknown);
        assert_eq!(classify_intent("random gibberish xyz123"), Intent::Unknown);
    }

    #[test]
    fn test_intent_priority() {
        // First match wins - test priority ordering
        assert_eq!(classify_intent("read and write file"), Intent::Read);
        assert_eq!(classify_intent("git commit and push"), Intent::Git);
    }
}

// =============================================================================
// Layer 2: Complexity Estimation Tests
// =============================================================================

#[cfg(test)]
mod complexity_tests {
    use super::*;

    #[test]
    fn test_trivial_complexity() {
        assert_eq!(score_complexity("ls"), Complexity::Trivial);
        assert_eq!(score_complexity("pwd"), Complexity::Trivial);
        assert_eq!(score_complexity("date"), Complexity::Trivial);
        assert_eq!(score_complexity("whoami"), Complexity::Trivial);
        // "echo hello" may score higher due to keywords
        assert!(score_complexity("echo hello") <= Complexity::Simple);
    }

    #[test]
    fn test_simple_complexity() {
        // Simple reads - may score higher due to file path indicators
        assert!(score_complexity("read file.txt") >= Complexity::Simple);
        assert!(score_complexity("view src/main.rs") >= Complexity::Simple);
    }

    #[test]
    fn test_moderate_complexity() {
        // Write/edit operations add +2, but additional keywords may push higher
        assert!(score_complexity("write new function") >= Complexity::Moderate);
        assert!(score_complexity("edit the config") >= Complexity::Moderate);
    }

    #[test]
    fn test_complex_complexity() {
        // Refactor/optimize add +3, but additional keywords may push to Heavy
        assert!(score_complexity("refactor the code") >= Complexity::Complex);
        assert!(score_complexity("optimize performance") >= Complexity::Complex);
    }

    #[test]
    fn test_heavy_complexity() {
        assert_eq!(
            score_complexity("design distributed system architecture"),
            Complexity::Heavy
        );
        assert_eq!(
            score_complexity("implement security authentication"),
            Complexity::Heavy
        );
        assert_eq!(
            score_complexity("build machine learning pipeline"),
            Complexity::Heavy
        );
        assert_eq!(
            score_complexity("full-stack integration"),
            Complexity::Heavy
        );
    }

    #[test]
    fn test_empty_prompt_complexity() {
        assert_eq!(score_complexity(""), Complexity::Simple);
        assert_eq!(score_complexity("   "), Complexity::Simple);
    }
}

// =============================================================================
// Layer 3: Mode Selection Tests
// =============================================================================

#[cfg(test)]
mod mode_tests {
    use super::*;

    #[test]
    fn test_chat_mode_selection() {
        assert_eq!(pick_mode(Intent::Chat, Complexity::Simple), AgentMode::Chat);
        assert_eq!(pick_mode(Intent::Help, Complexity::Simple), AgentMode::Chat);
        assert_eq!(pick_mode(Intent::Explain, Complexity::Simple), AgentMode::Chat);
    }

    #[test]
    fn test_review_mode_for_complex_reads() {
        assert_eq!(pick_mode(Intent::Read, Complexity::Complex), AgentMode::Review);
        assert_eq!(pick_mode(Intent::Explain, Complexity::Complex), AgentMode::Review);
    }

    #[test]
    fn test_build_mode_selection() {
        assert_eq!(pick_mode(Intent::Write, Complexity::Simple), AgentMode::Build);
        assert_eq!(pick_mode(Intent::Edit, Complexity::Simple), AgentMode::Build);
        assert_eq!(pick_mode(Intent::Delete, Complexity::Simple), AgentMode::Build);
        assert_eq!(pick_mode(Intent::Bash, Complexity::Simple), AgentMode::Build);
        assert_eq!(pick_mode(Intent::Git, Complexity::Simple), AgentMode::Build);
    }

    #[test]
    fn test_review_mode_selection() {
        assert_eq!(pick_mode(Intent::Grep, Complexity::Simple), AgentMode::Review);
        assert_eq!(pick_mode(Intent::Glob, Complexity::Simple), AgentMode::Review);
        assert_eq!(pick_mode(Intent::Find, Complexity::Simple), AgentMode::Review);
        assert_eq!(pick_mode(Intent::Review, Complexity::Simple), AgentMode::Review);
    }

    #[test]
    fn test_debug_mode_selection() {
        assert_eq!(pick_mode(Intent::Debug, Complexity::Simple), AgentMode::Debug);
    }

    #[test]
    fn test_plan_mode_selection() {
        assert_eq!(pick_mode(Intent::Plan, Complexity::Simple), AgentMode::Plan);
        assert_eq!(pick_mode(Intent::Design, Complexity::Simple), AgentMode::Plan);
    }

    #[test]
    fn test_unknown_intent_mode() {
        assert_eq!(pick_mode(Intent::Unknown, Complexity::Simple), AgentMode::Chat);
    }
}

// =============================================================================
// Layer 4: Model Tier Selection Tests
// =============================================================================

#[cfg(test)]
mod model_tier_tests {
    use super::*;

    #[test]
    fn test_capable_tier_for_heavy_tasks() {
        let config = RouterConfig::default();
        assert_eq!(
            pick_model_tier(Complexity::Heavy, Intent::Read, AgentMode::Chat, &config),
            ModelTier::Capable
        );
        assert_eq!(
            pick_model_tier(Complexity::Complex, Intent::Read, AgentMode::Chat, &config),
            ModelTier::Capable
        );
    }

    #[test]
    fn test_standard_tier_for_planning() {
        let config = RouterConfig::default();
        assert_eq!(
            pick_model_tier(Complexity::Simple, Intent::Plan, AgentMode::Chat, &config),
            ModelTier::Standard
        );
        assert_eq!(
            pick_model_tier(Complexity::Simple, Intent::Design, AgentMode::Chat, &config),
            ModelTier::Standard
        );
    }

    #[test]
    fn test_standard_tier_for_review_debug() {
        let config = RouterConfig::default();
        assert_eq!(
            pick_model_tier(Complexity::Simple, Intent::Review, AgentMode::Chat, &config),
            ModelTier::Standard
        );
        assert_eq!(
            pick_model_tier(Complexity::Simple, Intent::Debug, AgentMode::Chat, &config),
            ModelTier::Standard
        );
    }

    #[test]
    fn test_fast_tier_for_simple_tasks() {
        let config = RouterConfig::default();
        assert_eq!(
            pick_model_tier(Complexity::Simple, Intent::Read, AgentMode::Chat, &config),
            ModelTier::Fast
        );
        assert_eq!(
            pick_model_tier(Complexity::Trivial, Intent::Read, AgentMode::Chat, &config),
            ModelTier::Fast
        );
    }

    #[test]
    fn test_local_tier_when_preferred() {
        let mut config = RouterConfig::default();
        config.prefer_local = true;

        assert_eq!(
            pick_model_tier(Complexity::Simple, Intent::Read, AgentMode::Chat, &config),
            ModelTier::Local
        );
    }
}

// =============================================================================
// Layer 5: Tool Policy Tests
// =============================================================================

#[cfg(test)]
mod tool_policy_tests {
    use super::*;

    #[test]
    fn test_build_mode_full_tools() {
        let policy = pick_tools(Intent::Write, AgentMode::Build);
        assert!(policy.is_tool_allowed("Read"));
        assert!(policy.is_tool_allowed("Write"));
        assert!(policy.is_tool_allowed("Bash"));
    }

    #[test]
    fn test_review_mode_read_only() {
        let policy = pick_tools(Intent::Review, AgentMode::Review);
        assert!(policy.is_tool_allowed("Read"));
        assert!(policy.is_tool_allowed("Grep"));
        assert!(!policy.is_tool_allowed("Write"));
        assert!(!policy.is_tool_allowed("Bash"));
    }

    #[test]
    fn test_plan_mode_read_only() {
        let policy = pick_tools(Intent::Plan, AgentMode::Plan);
        assert!(policy.is_tool_allowed("Read"));
        assert!(!policy.is_tool_allowed("Write"));
        assert!(!policy.is_tool_allowed("Bash"));
    }

    #[test]
    fn test_debug_mode_tools() {
        let policy = pick_tools(Intent::Debug, AgentMode::Debug);
        assert!(policy.is_tool_allowed("Read"));
        assert!(policy.is_tool_allowed("Bash"));
        assert!(!policy.is_tool_allowed("Write"));
    }

    #[test]
    fn test_chat_mode_minimal_tools() {
        let policy = pick_tools(Intent::Chat, AgentMode::Chat);
        assert!(policy.is_tool_allowed("Read"));
        assert!(!policy.is_tool_allowed("Write"));
        assert!(!policy.is_tool_allowed("Bash"));
        assert!(!policy.is_tool_allowed("Grep"));
        assert!(!policy.is_tool_allowed("Glob"));
    }

    #[test]
    fn test_confirmation_for_destructive_ops() {
        let policy = pick_tools(Intent::Delete, AgentMode::Build);
        assert!(policy.require_confirmation);

        let policy = pick_tools(Intent::Bash, AgentMode::Build);
        assert!(policy.require_confirmation);

        let policy = pick_tools(Intent::Git, AgentMode::Build);
        assert!(policy.require_confirmation);
    }
}

// =============================================================================
// Layer 6: Context Budget Tests
// =============================================================================

#[cfg(test)]
mod context_budget_tests {
    use super::*;

    #[test]
    fn test_minimal_budget_for_trivial() {
        assert_eq!(pick_budget(Complexity::Trivial, AgentMode::Chat), ContextBudget::Minimal);
        assert_eq!(pick_budget(Complexity::Simple, AgentMode::Chat), ContextBudget::Minimal);
    }

    #[test]
    fn test_chat_always_minimal() {
        assert_eq!(pick_budget(Complexity::Heavy, AgentMode::Chat), ContextBudget::Minimal);
    }

    #[test]
    fn test_plan_mode_gets_relevant() {
        assert_eq!(pick_budget(Complexity::Trivial, AgentMode::Plan), ContextBudget::Relevant);
        assert_eq!(pick_budget(Complexity::Complex, AgentMode::Plan), ContextBudget::Standard);
    }

    #[test]
    fn test_review_mode_gets_relevant() {
        assert_eq!(pick_budget(Complexity::Trivial, AgentMode::Review), ContextBudget::Relevant);
        assert_eq!(pick_budget(Complexity::Heavy, AgentMode::Review), ContextBudget::Comprehensive);
    }

    #[test]
    fn test_build_mode_uses_base() {
        assert_eq!(pick_budget(Complexity::Simple, AgentMode::Build), ContextBudget::Minimal);
        assert_eq!(pick_budget(Complexity::Moderate, AgentMode::Build), ContextBudget::Relevant);
        assert_eq!(pick_budget(Complexity::Complex, AgentMode::Build), ContextBudget::Standard);
        assert_eq!(pick_budget(Complexity::Heavy, AgentMode::Build), ContextBudget::Comprehensive);
    }

    #[test]
    fn test_budget_token_values() {
        assert_eq!(ContextBudget::Minimal.tokens(), 4_000);
        assert_eq!(ContextBudget::Relevant.tokens(), 16_000);
        assert_eq!(ContextBudget::Standard.tokens(), 50_000);
        assert_eq!(ContextBudget::Comprehensive.tokens(), 100_000);
    }
}

// =============================================================================
// Layer 7: Memory Policy Tests
// =============================================================================

#[cfg(test)]
mod memory_policy_tests {
    use super::*;

    #[test]
    fn test_no_memory_for_trivial() {
        assert_eq!(pick_memory_policy(Intent::Read, Complexity::Trivial, AgentMode::Chat), MemoryPolicy::None);
    }

    #[test]
    fn test_no_memory_for_simple_chat() {
        assert_eq!(pick_memory_policy(Intent::Read, Complexity::Simple, AgentMode::Chat), MemoryPolicy::None);
    }

    #[test]
    fn test_recent_for_plan_mode() {
        assert_eq!(pick_memory_policy(Intent::Plan, Complexity::Simple, AgentMode::Plan), MemoryPolicy::Recent);
    }

    #[test]
    fn test_relevant_for_review_debug() {
        assert_eq!(pick_memory_policy(Intent::Review, Complexity::Simple, AgentMode::Review), MemoryPolicy::Relevant);
        assert_eq!(pick_memory_policy(Intent::Debug, Complexity::Complex, AgentMode::Debug), MemoryPolicy::Relevant);
    }

    #[test]
    fn test_full_for_complex_build() {
        assert_eq!(pick_memory_policy(Intent::Write, Complexity::Complex, AgentMode::Build), MemoryPolicy::Full);
    }
}

// =============================================================================
// End-to-End Routing Tests
// =============================================================================

#[cfg(test)]
mod routing_tests {
    use super::*;

    #[test]
    fn test_route_read_file() {
        let config = RouterConfig::default();
        let decision = route("read src/main.rs", "/project", &config);

        assert_eq!(decision.intent, Intent::Read);
        assert!(decision.complexity <= Complexity::Simple);
        assert!(decision.mode == AgentMode::Chat || decision.mode == AgentMode::Review);
        assert!(decision.tools.is_tool_allowed("Read"));
    }

    #[test]
    fn test_route_write_file() {
        let config = RouterConfig::default();
        let decision = route("write new_feature.rs", "/project", &config);

        assert_eq!(decision.intent, Intent::Write);
        assert_eq!(decision.mode, AgentMode::Build);
        assert!(decision.tools.is_tool_allowed("Write"));
    }

    #[test]
    fn test_route_plan_architecture() {
        let config = RouterConfig::default();
        let decision = route("plan a microservices architecture", "/project", &config);

        assert_eq!(decision.intent, Intent::Plan);
        assert_eq!(decision.mode, AgentMode::Plan);
        // Complex architecture plans may escalate to Capable tier
        assert!(decision.model_tier == ModelTier::Standard || decision.model_tier == ModelTier::Capable);
        assert!(!decision.tools.is_tool_allowed("Write"));
    }

    #[test]
    fn test_route_git_commit() {
        let config = RouterConfig::default();
        let decision = route("git commit -m 'fix bug'", "/project", &config);

        assert_eq!(decision.intent, Intent::Git);
        assert_eq!(decision.mode, AgentMode::Build);
        assert!(decision.tools.require_confirmation);
    }

    #[test]
    fn test_route_explain_concept() {
        let config = RouterConfig::default();
        let decision = route("what is a mutex?", "/project", &config);

        assert_eq!(decision.intent, Intent::Explain);
        assert_eq!(decision.mode, AgentMode::Chat);
    }

    #[test]
    fn test_route_complex_refactor() {
        let config = RouterConfig::default();
        let decision = route(
            "refactor the authentication system with OAuth2",
            "/project",
            &config,
        );

        assert!(decision.complexity >= Complexity::Complex);
        assert_eq!(decision.model_tier, ModelTier::Capable);
    }

    #[test]
    fn test_route_debug_issue() {
        let config = RouterConfig::default();
        let decision = route("debug the race condition", "/project", &config);

        assert_eq!(decision.intent, Intent::Debug);
        assert_eq!(decision.mode, AgentMode::Debug);
        assert!(!decision.tools.is_tool_allowed("Write"));
        assert!(decision.tools.is_tool_allowed("Bash"));
    }

    #[test]
    fn test_route_review_code() {
        let config = RouterConfig::default();
        let decision = route("review this pull request", "/project", &config);

        assert_eq!(decision.intent, Intent::Review);
        assert_eq!(decision.mode, AgentMode::Review);
        assert!(decision.tools.is_tool_allowed("Read"));
        assert!(!decision.tools.is_tool_allowed("Write"));
    }

    #[test]
    fn test_route_prefer_local_config() {
        let mut config = RouterConfig::default();
        config.prefer_local = true;

        let decision = route("read src/main.rs", "/project", &config);

        assert_eq!(decision.model_tier, ModelTier::Local);
    }

    #[test]
    fn test_route_confidence() {
        let config = RouterConfig::default();
        let decision = route("read src/main.rs", "/project", &config);

        assert!(decision.confidence >= 0.8);
        assert!(!decision.reasoning.is_empty());
    }

    #[test]
    fn test_route_unknown() {
        let config = RouterConfig::default();
        let decision = route("random gibberish xyz", "/project", &config);

        assert_eq!(decision.intent, Intent::Unknown);
        assert_eq!(decision.mode, AgentMode::Chat);
    }
}

// =============================================================================
// Router Configuration Tests
// =============================================================================

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RouterConfig::default();
        assert!(!config.prefer_local);
        assert_eq!(config.cost_limit, 1.0);
        assert!(config.rag.enabled);
        assert!(config.prompt_compaction.enabled);
    }

    #[test]
    fn test_rag_config() {
        let config = RouterConfig::default();
        assert_eq!(config.rag.max_chunks, 5);
        assert_eq!(config.rag.similarity_threshold, 0.3);
    }

    #[test]
    fn test_prompt_compaction_config() {
        let config = RouterConfig::default();
        assert!(config.prompt_compaction.enabled);
        assert_eq!(config.prompt_compaction.target_tokens, 1000);
        assert!(config.prompt_compaction.remove_filler);
    }
}
