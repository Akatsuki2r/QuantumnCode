//! Model tier selection
//!
//! Handles selection of appropriate model tier based on complexity and intent.

use crate::router::types::{AgentMode, Complexity, Intent, ModelTier, RouterConfig};

/// Pick the appropriate model tier based on complexity, intent, and config
pub fn pick_model_tier(
    complexity: Complexity,
    intent: Intent,
    mode: AgentMode,
    config: &RouterConfig,
) -> ModelTier {
    // Heavy/complex tasks need capable models
    if complexity >= Complexity::Complex {
        return ModelTier::Capable;
    }

    // Planning benefits from standard tier
    if intent == Intent::Plan || intent == Intent::Design {
        return ModelTier::Standard;
    }

    // Review/debug can use standard tier
    if intent == Intent::Review || intent == Intent::Debug {
        return ModelTier::Standard;
    }

    // Build mode for complex work needs standard
    if mode == AgentMode::Build && complexity >= Complexity::Moderate {
        return ModelTier::Standard;
    }

    // Trivial/simple tasks can use local or fast
    if complexity <= Complexity::Simple {
        if config.prefer_local {
            return ModelTier::Local;
        }
        return ModelTier::Fast;
    }

    // Default to standard
    ModelTier::Standard
}

/// Get the default model name for a tier
pub fn get_model_for_tier(tier: ModelTier) -> &'static str {
    tier.default_model()
}

/// Check if a model tier supports a given feature
pub fn tier_supports_streaming(_tier: ModelTier) -> bool {
    // All tiers support streaming
    true
}

/// Estimate cost per 1K tokens for a tier
pub fn estimate_cost_per_1k(tier: ModelTier) -> f64 {
    match tier {
        ModelTier::Local => 0.0,    // No API cost
        ModelTier::Fast => 0.25,    // Haiku pricing
        ModelTier::Standard => 1.0, // Sonnet pricing
        ModelTier::Capable => 3.0,  // Opus pricing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_model_tier_complexity() {
        let config = RouterConfig::default();

        // Heavy tasks need Capable
        assert_eq!(pick_model_tier(Complexity::Heavy, Intent::Read, AgentMode::Chat, &config), ModelTier::Capable);
        assert_eq!(pick_model_tier(Complexity::Complex, Intent::Read, AgentMode::Chat, &config), ModelTier::Capable);
    }

    #[test]
    fn test_pick_model_tier_planning() {
        let config = RouterConfig::default();

        // Plan/Design intent gets Standard
        assert_eq!(pick_model_tier(Complexity::Simple, Intent::Plan, AgentMode::Chat, &config), ModelTier::Standard);
        assert_eq!(pick_model_tier(Complexity::Simple, Intent::Design, AgentMode::Chat, &config), ModelTier::Standard);
    }

    #[test]
    fn test_pick_model_tier_review_debug() {
        let config = RouterConfig::default();

        // Review/Debug get Standard
        assert_eq!(pick_model_tier(Complexity::Simple, Intent::Review, AgentMode::Chat, &config), ModelTier::Standard);
        assert_eq!(pick_model_tier(Complexity::Simple, Intent::Debug, AgentMode::Chat, &config), ModelTier::Standard);
    }

    #[test]
    fn test_pick_model_tier_simple() {
        let config = RouterConfig::default();

        // Simple tasks get Fast (or Local if prefer_local)
        assert_eq!(pick_model_tier(Complexity::Simple, Intent::Read, AgentMode::Chat, &config), ModelTier::Fast);
        assert_eq!(pick_model_tier(Complexity::Trivial, Intent::Read, AgentMode::Chat, &config), ModelTier::Fast);
    }

    #[test]
    fn test_pick_model_tier_prefer_local() {
        let mut config = RouterConfig::default();
        config.prefer_local = true;

        assert_eq!(pick_model_tier(Complexity::Simple, Intent::Read, AgentMode::Chat, &config), ModelTier::Local);
    }

    #[test]
    fn test_get_model_for_tier() {
        assert_eq!(get_model_for_tier(ModelTier::Local), "llama3.2:latest");
        assert_eq!(get_model_for_tier(ModelTier::Fast), "claude-haiku-4-20250514");
        assert_eq!(get_model_for_tier(ModelTier::Standard), "claude-sonnet-4-20250514");
        assert_eq!(get_model_for_tier(ModelTier::Capable), "claude-opus-4-20250514");
    }

    #[test]
    fn test_estimate_cost_per_1k() {
        assert_eq!(estimate_cost_per_1k(ModelTier::Local), 0.0);
        assert_eq!(estimate_cost_per_1k(ModelTier::Fast), 0.25);
        assert_eq!(estimate_cost_per_1k(ModelTier::Standard), 1.0);
        assert_eq!(estimate_cost_per_1k(ModelTier::Capable), 3.0);
    }

    #[test]
    fn test_tier_supports_streaming() {
        // All tiers support streaming
        assert!(tier_supports_streaming(ModelTier::Local));
        assert!(tier_supports_streaming(ModelTier::Fast));
        assert!(tier_supports_streaming(ModelTier::Standard));
        assert!(tier_supports_streaming(ModelTier::Capable));
    }
}
