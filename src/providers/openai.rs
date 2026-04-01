//! OpenAI provider

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::env;

use super::provider_trait::{Provider, ProviderError, Message, Role, StreamChunk};
use color_eyre::eyre::Result;

/// OpenAI API client
pub struct OpenAIProvider {
    api_key: Option<String>,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

/// OpenAI API request
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<usize>,
    stream: bool,
}

/// OpenAI message format
#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI API response
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageResponse,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessageResponse {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
}

use serde::{Deserialize, Serialize};

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new() -> Self {
        Self {
            api_key: env::var("OPENAI_API_KEY").ok(),
            model: "gpt-4o".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create with specific model
    pub fn with_model(model: String) -> Self {
        let mut provider = Self::new();
        provider.model = model;
        provider
    }

    /// Create with custom base URL (for compatible APIs)
    pub fn with_base_url(base_url: String) -> Self {
        let mut provider = Self::new();
        provider.base_url = base_url;
        provider
    }

    /// Convert messages to OpenAI format
    fn convert_messages(&self, messages: Vec<Message>) -> Vec<OpenAIMessage> {
        messages
            .into_iter()
            .map(|m| OpenAIMessage {
                role: match m.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: m.content,
            })
            .collect()
    }
}

impl Default for OpenAIProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    fn name(&self) -> &str {
        "OpenAI"
    }

    fn models(&self) -> Vec<String> {
        vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
            "o1".to_string(),
            "o1-mini".to_string(),
            "o1-preview".to_string(),
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
            .ok_or(ProviderError::AuthError("OPENAI_API_KEY not set".to_string()))?;

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            max_tokens: Some(4096),
            stream: false,
        };

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError(error_text));
        }

        let result: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        let text = result.choices
            .first()
            .and_then(|c| c.message.content.clone())
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
        // Rough approximation: ~4 characters per token
        text.len() / 4
    }

    fn cost_per_million(&self) -> (f64, f64) {
        match self.model.as_str() {
            "gpt-4o" => (5.0, 15.0),
            "gpt-4o-mini" => (0.15, 0.60),
            "gpt-4-turbo" => (10.0, 30.0),
            "gpt-4" => (30.0, 60.0),
            "gpt-3.5-turbo" => (0.50, 1.50),
            "o1" => (15.0, 60.0),
            "o1-mini" => (3.0, 12.0),
            "o1-preview" => (15.0, 60.0),
            _ => (5.0, 15.0),
        }
    }
}