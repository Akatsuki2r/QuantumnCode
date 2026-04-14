//! Intelligent Router Module
//!
//! A 7-layer policy engine for routing user prompts to appropriate
//! execution contexts. Pure functions, no state, no singletons.
//!
//! ## Architecture
//!
//! 1. **Intent Classification** - regex-based, < 1ms, 16 intent types
//! 2. **Complexity Estimation** - keyword-weighted scoring, 5 levels
//! 3. **Mode Selection** - 5 execution modes (chat/plan/build/review/debug)
//! 4. **Model Tier Selection** - 4 capability tiers
//! 5. **Tool Policy** - per-intent allowed/disallowed tools
//! 6. **Context Strategy** - token budget allocation
//! 7. **Memory Policy** - relevance-based memory loading
//!
//! ## Usage
//!
//! ```rust
//! use quantumn::router::{route, RouterConfig, RoutingDecision};
//!
//! let config = RouterConfig::default();
//! let decision = route("read src/main.rs", "/project", &config);
//! println!("Intent: {:?}", decision.intent);
//! println!("Mode: {:?}", decision.mode);
//! println!("Tools: {:?}", decision.tools.allowed_tools);
//! ```

pub mod analyzer;
pub mod context;
pub mod memory;
pub mod mode;
pub mod model;
pub mod tools;
pub mod types;

// Re-export main types
pub use types::{
    AgentMode, Complexity, ContextBudget, Intent, MemoryPolicy, ModelTier, RouterConfig,
    RoutingDecision, ToolPolicy,
};

// Re-export functions
pub use analyzer::{classify_intent, score_complexity};
pub use context::pick_budget;
pub use memory::pick_memory_policy;
pub use mode::{can_transition, get_mode_display, get_mode_instruction, pick_mode, transition};
pub use model::{
    estimate_cost_per_1k, get_model_for_tier, pick_model_tier, tier_supports_streaming,
};
pub use tools::{filter_tools_by_policy, pick_tools};

/// Main routing function - pure, no side effects
///
/// # Arguments
/// * `prompt` - User's input prompt
/// * `cwd` - Current working directory (for context)
/// * `config` - Router configuration
///
/// # Returns
/// Complete routing decision with all 7 layers
///
/// # Example
///
/// ```rust
/// let decision = route("read src/main.rs", "/project", &RouterConfig::default());
/// assert!(matches!(decision.intent, Intent::Read));
/// assert!(matches!(decision.mode, AgentMode::Chat | AgentMode::Review));
/// ```
pub fn route(prompt: &str, cwd: &str, config: &RouterConfig) -> RoutingDecision {
    // Layer 1: Intent Classification
    let intent = classify_intent(prompt);

    // Layer 2: Complexity Estimation
    let complexity = score_complexity(prompt);

    // Layer 3: Mode Selection
    let mode = pick_mode(intent, complexity);

    // Layer 4: Model Tier Selection
    let model_tier = pick_model_tier(complexity, intent, mode, config);

    // Layer 5: Tool Policy
    let tools = pick_tools(intent, mode);

    // Layer 6: Context Budget
    let context_budget = pick_budget(complexity, mode);

    // Layer 7: Memory Policy
    let memory_policy = pick_memory_policy(intent, complexity, mode);

    // Calculate confidence
    let confidence = calculate_confidence(intent, complexity);

    // Generate reasoning
    let reasoning = format!(
        "intent={}, complexity={}, mode={}, model={}, tools={}, budget={}, memory={}",
        intent.as_str(),
        complexity.as_str(),
        mode.as_str(),
        model_tier.as_str(),
        tools.allowed_tools.len(),
        context_budget.tokens(),
        memory_policy.as_str()
    );

    RoutingDecision::new(
        intent,
        complexity,
        mode,
        model_tier,
        tools,
        context_budget,
        memory_policy,
        confidence,
        reasoning,
    )
}

/// Calculate routing confidence (0.0 - 1.0)
fn calculate_confidence(intent: Intent, complexity: Complexity) -> f32 {
    let mut confidence: f32 = 0.7; // Base confidence

    // Higher confidence for clear intents
    if intent != Intent::Unknown {
        confidence += 0.15;
    }

    // Higher confidence for clear complexity
    if complexity != Complexity::Moderate {
        confidence += 0.1;
    }

    confidence.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_read() {
        let config = RouterConfig::default();
        let decision = route("read src/main.rs", "/project", &config);

        assert!(matches!(decision.intent, Intent::Read));
        assert!(decision.complexity <= Complexity::Simple);
        assert!(decision.tools.is_tool_allowed("Read"));
    }

    #[test]
    fn test_route_write() {
        let config = RouterConfig::default();
        let decision = route("write new_feature.rs", "/project", &config);

        assert!(matches!(decision.intent, Intent::Write));
        assert!(matches!(decision.mode, AgentMode::Build));
        assert!(decision.tools.is_tool_allowed("Write"));
    }

    #[test]
    fn test_route_plan() {
        let config = RouterConfig::default();
        let decision = route("plan a microservices architecture", "/project", &config);

        assert!(matches!(decision.intent, Intent::Plan));
        assert!(matches!(decision.mode, AgentMode::Plan));
        assert!(decision.tools.is_tool_allowed("Read"));
        assert!(!decision.tools.is_tool_allowed("Write"));
    }

    #[test]
    fn test_route_git() {
        let config = RouterConfig::default();
        let decision = route("git commit -m 'fix bug'", "/project", &config);

        assert!(matches!(decision.intent, Intent::Git));
        assert!(matches!(decision.mode, AgentMode::Build));
        assert!(decision.tools.require_confirmation);
    }

    #[test]
    fn test_route_explain() {
        let config = RouterConfig::default();
        let decision = route("what is a mutex?", "/project", &config);

        assert!(matches!(decision.intent, Intent::Explain));
        assert!(matches!(decision.mode, AgentMode::Chat));
    }

    #[test]
    fn test_route_complex() {
        let config = RouterConfig::default();
        let decision = route(
            "refactor the entire authentication system with OAuth2",
            "/project",
            &config,
        );

        assert!(decision.complexity >= Complexity::Complex);
        assert!(matches!(decision.model_tier, ModelTier::Capable));
    }

    #[test]
    fn test_route_prefer_local() {
        let mut config = RouterConfig::default();
        config.prefer_local = true;

        let decision = route("read src/main.rs", "/project", &config);

        assert!(matches!(decision.model_tier, ModelTier::Local));
    }
}
