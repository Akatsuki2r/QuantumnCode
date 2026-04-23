//! OpenCode Zen provider

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::env;
use std::pin::Pin;

use super::provider_trait::{Message, Provider, ProviderError, Role, StreamChunk};
use color_eyre::eyre::Result;

pub struct OpenCodeProvider {
    api_key: Option<String>,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct OpenCodeRequest {
    model: String,
    messages: Vec<OpenCodeMessage>,
    max_tokens: Option<usize>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OpenCodeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenCodeResponse {
    choices: Vec<OpenCodeChoice>,
    usage: Option<OpenCodeUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenCodeChoice {
    message: OpenCodeMessageResponse,
}

#[derive(Debug, Deserialize)]
struct OpenCodeMessageResponse {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenCodeUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
}

use serde::{Deserialize, Serialize};

impl OpenCodeProvider {
    pub fn new() -> Self {
        Self {
            api_key: env::var("OPENCODE_API_KEY").ok(),
            model: "qwen-2.5-coder-7b".to_string(),
            base_url: "https://opencode.ai/zen/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_model(model: String) -> Self {
        let mut provider = Self::new();
        provider.model = model;
        provider
    }

    pub fn with_base_url(base_url: String) -> Self {
        let mut provider = Self::new();
        provider.base_url = base_url;
        provider
    }

    fn convert_messages(&self, messages: Vec<Message>) -> Vec<OpenCodeMessage> {
        messages
            .into_iter()
            .map(|m| OpenCodeMessage {
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

impl Default for OpenCodeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for OpenCodeProvider {
    fn name(&self) -> &str {
        "OpenCode"
    }

    fn models(&self) -> Vec<String> {
        vec![
            "qwen-2.5-coder-7b".to_string(),
            "qwen-2.5-coder-14b".to_string(),
            "qwen-2.5-coder-32b".to_string(),
            "qwen-2.5-coder-72b".to_string(),
            "deepseek-coder-v2".to_string(),
            "deepseek-coder-v2.5".to_string(),
            "llama-3.1-sonar-small".to_string(),
            "llama-3.1-sonar-large".to_string(),
            "gemma-2-27b".to_string(),
        ]
    }

    fn current_model(&self) -> &str {
        &self.model
    }

    fn set_model(&mut self, model: String) {
        self.model = model;
    }

    fn is_configured(&self) -> bool {
        true
    }

    async fn send(&self, messages: Vec<Message>) -> Result<String, ProviderError> {
        let request = OpenCodeRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            max_tokens: Some(4096),
            stream: false,
        };

        let mut req = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Content-Type", "application/json");

        if let Some(key) = self.api_key.as_ref() {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let response = req
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError(error_text));
        }

        let result: OpenCodeResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        let text = result
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(text)
    }

    async fn send_with_system(
        &self,
        messages: Vec<Message>,
        system: Option<&str>,
    ) -> Result<String, ProviderError> {
        let mut all_messages = Vec::new();
        if let Some(sys) = system {
            all_messages.push(OpenCodeMessage {
                role: "system".to_string(),
                content: sys.to_string(),
            });
        }
        all_messages.extend(self.convert_messages(messages));

        let request = OpenCodeRequest {
            model: self.model.clone(),
            messages: all_messages,
            max_tokens: Some(4096),
            stream: false,
        };

        let mut req = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Content-Type", "application/json");

        if let Some(key) = self.api_key.as_ref() {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let response = req
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError(error_text));
        }

        let result: OpenCodeResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        let text = result
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(text)
    }

    async fn send_stream(
        &self,
        messages: Vec<Message>,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>> {
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
        text.len() / 4
    }

    fn cost_per_million(&self) -> (f64, f64) {
        (0.0, 0.0)
    }
}
