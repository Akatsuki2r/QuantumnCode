//! Ollama (local LLM) provider

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::pin::Pin;

use super::provider_trait::{Message, Provider, ProviderError, Role, StreamChunk};
use color_eyre::eyre::Result;

/// Ollama API client
pub struct OllamaProvider {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

/// Ollama API request
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

/// Ollama message format
#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

/// Ollama API response
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessageResponse,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaMessageResponse {
    role: String,
    content: String,
}

use serde::{Deserialize, Serialize};

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model: "llama3.2".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create with specific model
    pub fn with_model(model: String) -> Self {
        let mut provider = Self::new();
        provider.model = model;
        provider
    }

    /// Create with custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        let mut provider = Self::new();
        provider.base_url = base_url;
        provider
    }

    /// Convert messages to Ollama format
    fn convert_messages(&self, messages: Vec<Message>) -> Vec<OllamaMessage> {
        messages
            .into_iter()
            .map(|m| OllamaMessage {
                role: match m.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: m.content,
            })
            .collect()
    }

    /// Check if Ollama is running
    pub async fn is_running(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
        #[derive(Debug, Deserialize)]
        struct TagsResponse {
            models: Vec<ModelInfo>,
        }

        #[derive(Debug, Deserialize)]
        struct ModelInfo {
            name: String,
        }

        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError("Failed to list models".to_string()));
        }

        let tags: TagsResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        Ok(tags.models.into_iter().map(|m| m.name).collect())
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "Ollama"
    }

    fn models(&self) -> Vec<String> {
        // Note: This returns a fallback list. For actual model detection,
        // use list_models() async method which queries the Ollama API.
        // The hardcoded list is only for cases where Ollama is not running.
        vec![
            "llama3.2".to_string(),
            "llama3.1".to_string(),
            "llama3".to_string(),
            "mistral".to_string(),
            "mistral-nemo".to_string(),
            "codellama".to_string(),
            "deepseek-coder".to_string(),
            "deepseek-coder-v2".to_string(),
            "qwen2.5-coder".to_string(),
            "qwen2.5".to_string(),
            "phi3".to_string(),
            "phi3-mini".to_string(),
            "gemma2".to_string(),
            "gemma2-9b".to_string(),
            "starcoder2".to_string(),
            "codestral".to_string(),
            "wizardcoder".to_string(),
            "wizardlm2".to_string(),
            "llava".to_string(),
            "mixtral".to_string(),
            "command-r-plus".to_string(),
        ]
    }

    fn current_model(&self) -> &str {
        &self.model
    }

    fn set_model(&mut self, model: String) {
        self.model = model;
    }

    fn is_configured(&self) -> bool {
        // Ollama doesn't need API key, just needs to be running
        // We check this lazily when sending
        true
    }

    async fn send(&self, messages: Vec<Message>) -> Result<String, ProviderError> {
        let request = OllamaRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError(error_text));
        }

        let result: OllamaResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        Ok(result.message.content)
    }

    async fn send_with_system(
        &self,
        messages: Vec<Message>,
        system: Option<&str>,
    ) -> Result<String, ProviderError> {
        // Prepend system message if provided
        let mut all_messages = Vec::new();
        if let Some(sys) = system {
            all_messages.push(OllamaMessage {
                role: "system".to_string(),
                content: sys.to_string(),
            });
        }
        all_messages.extend(self.convert_messages(messages));

        let request = OllamaRequest {
            model: self.model.clone(),
            messages: all_messages,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError(error_text));
        }

        let result: OllamaResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        Ok(result.message.content)
    }

    async fn send_stream(
        &self,
        messages: Vec<Message>,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>> {
        // For now, fall back to non-streaming
        // TODO: Implement proper SSE streaming for Ollama
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
        // Rough approximation
        text.len() / 4
    }

    fn cost_per_million(&self) -> (f64, f64) {
        // Ollama is free (local)
        (0.0, 0.0)
    }
}
