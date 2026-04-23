//! Model tier selection
//!
//! Handles selection of appropriate model tier based on complexity and intent.

use crate::providers::local_discover::LocalModelConfig;
use crate::router::types::{AgentMode, Complexity, Intent, ModelTier, RouterConfig};
use std::sync::{Arc, RwLock};

/// Global state for discovered local models
/// Lazily initialized on first access
static LOCAL_MODELS: once_cell::sync::Lazy<Arc<RwLock<Option<LocalModelConfig>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

/// Initialize local model discovery
pub fn init_local_model_discovery() {
    let config = crate::providers::local_discover::discover_all_models();
    if let Ok(mut models) = LOCAL_MODELS.write() {
        *models = Some(config);
    }
}

/// Get available local models
pub fn get_available_local_models() -> Vec<String> {
    if let Ok(models) = LOCAL_MODELS.read() {
        if let Some(config) = models.as_ref() {
            return config.ollama.keys().cloned().collect();
        }
    }
    Vec::new()
}

/// Check if any local models are available
pub fn has_local_models() -> bool {
    if let Ok(models) = LOCAL_MODELS.read() {
        if let Some(config) = models.as_ref() {
            return !config.ollama.is_empty();
        }
    }
    false
}

/// Get the best available local model for a given tier
/// Returns the first available model, or None if no models exist
pub fn get_best_local_model() -> Option<String> {
    get_available_local_models().into_iter().next()
}

/// Check if Ollama server is running
pub fn is_ollama_running() -> bool {
    use crate::providers::ollama::OllamaProvider;
    let provider = OllamaProvider::new();
    let rt = tokio::runtime::Handle::current();
    rt.block_on(async { provider.is_running().await })
}

/// Get detailed information about local models
pub fn get_local_models_info() -> Vec<(String, String, String)> {
    // Returns (name, size, modified_date)
    if let Ok(models) = LOCAL_MODELS.read() {
        if let Some(config) = models.as_ref() {
            return config
                .ollama
                .iter()
                .map(|(name, info)| (name.clone(), info.size.clone(), info.modified.clone()))
                .collect();
        }
    }
    Vec::new()
}

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

/// Get the default model name for a tier, preferring available local models
pub fn get_model_for_tier_with_local(tier: ModelTier) -> String {
    match tier {
        ModelTier::Local => {
            // Try to get an available local model
            let models = get_available_local_models();
            if let Some(first_model) = models.first() {
                first_model.clone()
            } else {
                tier.default_model().to_string()
            }
        }
        _ => tier.default_model().to_string(),
    }
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
        ModelTier::OpenCode => 0.0, // Free
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
    fn test_pick_model_tier_planning() {
        let config = RouterConfig::default();

        // Plan/Design intent gets Standard
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
    fn test_pick_model_tier_review_debug() {
        let config = RouterConfig::default();

        // Review/Debug get Standard
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
    fn test_pick_model_tier_simple() {
        let config = RouterConfig::default();

        // Simple tasks get Fast (or Local if prefer_local)
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
    fn test_pick_model_tier_prefer_local() {
        let mut config = RouterConfig::default();
        config.prefer_local = true;

        assert_eq!(
            pick_model_tier(Complexity::Simple, Intent::Read, AgentMode::Chat, &config),
            ModelTier::Local
        );
    }

    #[test]
    fn test_get_model_for_tier() {
        assert_eq!(get_model_for_tier(ModelTier::Local), "llama3.2:latest");
        assert_eq!(get_model_for_tier(ModelTier::OpenCode), "qwen-2.5-coder-7b");
        assert_eq!(
            get_model_for_tier(ModelTier::Fast),
            "claude-haiku-4-20250514"
        );
        assert_eq!(
            get_model_for_tier(ModelTier::Standard),
            "claude-sonnet-4-20250514"
        );
        assert_eq!(
            get_model_for_tier(ModelTier::Capable),
            "claude-opus-4-20250514"
        );
    }

    #[test]
    fn test_estimate_cost_per_1k() {
        assert_eq!(estimate_cost_per_1k(ModelTier::Local), 0.0);
        assert_eq!(estimate_cost_per_1k(ModelTier::OpenCode), 0.0);
        assert_eq!(estimate_cost_per_1k(ModelTier::Fast), 0.25);
        assert_eq!(estimate_cost_per_1k(ModelTier::Standard), 1.0);
        assert_eq!(estimate_cost_per_1k(ModelTier::Capable), 3.0);
    }

    #[test]
    fn test_tier_supports_streaming() {
        // All tiers support streaming
        assert!(tier_supports_streaming(ModelTier::Local));
        assert!(tier_supports_streaming(ModelTier::OpenCode));
        assert!(tier_supports_streaming(ModelTier::Fast));
        assert!(tier_supports_streaming(ModelTier::Standard));
        assert!(tier_supports_streaming(ModelTier::Capable));
    }
}
