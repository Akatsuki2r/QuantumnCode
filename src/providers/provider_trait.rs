//! Provider trait and common types

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use color_eyre::eyre::Result;

/// Message role
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Streaming chunk from AI
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub done: bool,
    pub tokens: Option<usize>,
}

/// Provider error type
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// AI provider trait
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Get available models
    fn models(&self) -> Vec<String>;

    /// Get current model
    fn current_model(&self) -> &str;

    /// Set current model
    fn set_model(&mut self, model: String);

    /// Check if provider is configured (has API key, etc.)
    fn is_configured(&self) -> bool;

    /// Send a message and get a response
    async fn send(&self, messages: Vec<Message>) -> Result<String, ProviderError>;

    /// Send a message and get a streaming response
    async fn send_stream(
        &self,
        messages: Vec<Message>,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>>;

    /// Count tokens in a message
    fn count_tokens(&self, text: &str) -> usize;

    /// Get cost per million tokens (input, output)
    fn cost_per_million(&self) -> (f64, f64);
}

use serde::{Deserialize, Serialize};