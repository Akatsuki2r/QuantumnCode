//! Anthropic Claude provider

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::env;

use super::provider_trait::{Provider, ProviderError, Message, Role, StreamChunk};
use color_eyre::eyre::Result;

/// Anthropic API client
pub struct AnthropicProvider {
    api_key: Option<String>,
    model: String,
    client: reqwest::Client,
}

/// Anthropic API request
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: usize,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    stream: bool,
}

/// Anthropic message format
#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new() -> Self {
        Self {
            api_key: env::var("ANTHROPIC_API_KEY").ok(),
            model: "claude-sonnet-4-20250514".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create with specific model
    pub fn with_model(model: String) -> Self {
        let mut provider = Self::new();
        provider.model = model;
        provider
    }

    /// Convert messages to Anthropic format
    fn convert_messages(&self, messages: Vec<Message>) -> Vec<AnthropicMessage> {
        messages
            .into_iter()
            .filter(|m| m.role != Role::System)
            .map(|m| AnthropicMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::System => "user".to_string(),
                },
                content: m.content,
            })
            .collect()
    }

    /// Get API endpoint
    fn endpoint(&self) -> &str {
        "https://api.anthropic.com/v1/messages"
    }
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "Anthropic"
    }

    fn models(&self) -> Vec<String> {
        vec![
            "claude-opus-4-20250514".to_string(),
            "claude-sonnet-4-20250514".to_string(),
            "claude-haiku-4-20250514".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
        ]
    }

    fn current_model(&self) -> &str {
        &self.model
    }

    fn set_model(&mut self, model: String) {
        self.model = model;
    }

    fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    async fn send(&self, messages: Vec<Message>) -> Result<String, ProviderError> {
        let api_key = self.api_key.as_ref()
            .ok_or(ProviderError::AuthError("ANTHROPIC_API_KEY not set".to_string()))?;

        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages: self.convert_messages(messages),
            system: None,
            stream: false,
        };

        let response = self.client
            .post(self.endpoint())
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError(error_text));
        }

        let result: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        let text = result.content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        Ok(text)
    }

    async fn send_stream(
        &self,
        messages: Vec<Message>,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>> {
        // For now, fall back to non-streaming
        // TODO: Implement proper SSE streaming
        let result = self.send(messages).await;

        Box::pin(futures::stream::once(async move {
            match result {
                Ok(text) => Ok(StreamChunk {
                    content: text,
                    done: true,
                    tokens: None,
                }),
                Err(e) => Err(e),
            }
        }))
    }

    fn count_tokens(&self, text: &str) -> usize {
        // Rough approximation: ~4 characters per token for Claude
        text.len() / 4
    }

    fn cost_per_million(&self) -> (f64, f64) {
        match self.model.as_str() {
            "claude-opus-4-20250514" => (15.0, 75.0),
            "claude-sonnet-4-20250514" => (3.0, 15.0),
            "claude-haiku-4-20250514" => (0.80, 4.0),
            "claude-3-5-sonnet-20241022" => (3.0, 15.0),
            "claude-3-5-haiku-20241022" => (0.80, 4.0),
            _ => (3.0, 15.0),
        }
    }
}