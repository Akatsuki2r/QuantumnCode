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
