//! AI Provider trait definition

use async_trait::async_trait;
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_window: usize,
    pub supports_vision: bool,
    pub supports_tools: bool,
}

/// Cost per 1M tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCost {
    pub input_per_million: f64,
    pub output_per_million: f64,
}

/// Response from AI provider
#[derive(Debug, Clone)]
pub struct Response {
    pub content: String,
    pub model: String,
    pub usage: TokenUsage,
    pub finish_reason: String,
}

/// Provider trait for AI backends
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Check if the provider is configured (has API key)
    fn is_configured(&self) -> bool;

    /// List available models
    fn list_models(&self) -> Vec<Model>;

    /// Get the default model
    fn default_model(&self) -> &str;

    /// Send a chat message
    async fn chat(&self, messages: &[Message], model: Option<&str>) -> Result<Response>;

    /// Send a chat message with system prompt
    async fn chat_with_system(
        &self,
        messages: &[Message],
        system: &str,
        model: Option<&str>,
    ) -> Result<Response>;

    /// Stream a chat message (returns async iterator)
    async fn stream_chat(
        &self,
        messages: &[Message],
        model: Option<&str>,
        on_token: impl Fn(&str) + Send + Sync + 'static,
    ) -> Result<Response>;

    /// Count tokens for a message
    fn count_tokens(&self, messages: &[Message], model: Option<&str>) -> usize;

    /// Estimate cost for a request
    fn estimate_cost(&self, usage: &TokenUsage, model: Option<&str>) -> f64;

    /// Get the API key environment variable name
    fn api_key_env(&self) -> &str;

    /// Set the API key (for runtime configuration)
    fn set_api_key(&mut self, key: String);
}