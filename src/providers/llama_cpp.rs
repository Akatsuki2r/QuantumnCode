//! llama.cpp server provider
//!
//! High-performance inference using llama.cpp HTTP server.
//! Supports real SSE streaming and model switching via supervisor.

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::provider_trait::{Message, Provider, ProviderError, Role, StreamChunk};
use crate::supervisor::ModelSupervisor;

/// llama.cpp server provider
///
/// Connects to a running llama.cpp HTTP server for inference.
/// Uses ModelSupervisor to manage the server lifecycle.
pub struct LlamaCppProvider {
    /// Base URL for the llama.cpp server
    base_url: String,
    /// Current model name
    model: String,
    /// HTTP client
    client: reqwest::Client,
    /// Model supervisor for process management
    supervisor: Arc<Mutex<ModelSupervisor>>,
    /// Whether to use the supervisor for auto-starting
    auto_start: bool,
    /// Prompt format type
    prompt_format: PromptFormat,
    /// Available models (name -> path mapping loaded from config)
    models: Vec<String>,
}

/// Prompt format types for different model architectures
#[derive(Debug, Clone, Copy)]
pub enum PromptFormat {
    /// LLaMA-style [INST] format
    Llama,
    /// ChatML format (Qwen, etc.)
    ChatML,
    /// Alpaca format
    Alpaca,
    /// Vicuna format
    Vicuna,
    /// Plain text (no special formatting)
    Plain,
}

impl Default for PromptFormat {
    fn default() -> Self {
        PromptFormat::Llama
    }
}

/// llama.cpp completion request
#[derive(Debug, serde::Serialize)]
struct CompletionRequest {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    n_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<i32>,
    stream: bool,
}

/// llama.cpp completion response
#[derive(Debug, serde::Deserialize)]
struct CompletionResponse {
    content: String,
    #[serde(default)]
    tokens_eval: i32,
    #[serde(default)]
    tokens_cached: i32,
    #[serde(default)]
    truncated: bool,
    #[serde(default)]
    stopped_eos: bool,
    #[serde(default)]
    stopped_word: bool,
    #[serde(default)]
    stopped_limit: bool,
    #[serde(default)]
    stopping_word: String,
    #[serde(default)]
    tokens_evaluated: i32,
    #[serde(default)]
    tokens_predicted: i32,
}

impl LlamaCppProvider {
    /// Create a new llama.cpp provider with supervisor
    pub fn new(supervisor: Arc<Mutex<ModelSupervisor>>) -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            model: "llama3.2".to_string(),
            client: reqwest::Client::new(),
            supervisor,
            auto_start: true,
            prompt_format: PromptFormat::Llama,
            models: vec![
                "llama3.2".to_string(),
                "llama3.1".to_string(),
                "llama3".to_string(),
                "mistral".to_string(),
                "qwen2.5".to_string(),
                "deepseek-coder".to_string(),
            ],
        }
    }

    /// Create a standalone provider without supervisor (connects to existing server)
    pub fn standalone() -> Self {
        let supervisor = Arc::new(Mutex::new(ModelSupervisor::new()));
        Self {
            auto_start: false,
            ..Self::new(supervisor)
        }
    }

    /// Create with specific model
    pub fn with_model(model: String) -> Self {
        let mut provider = Self::standalone();
        provider.model = model;
        provider
    }

    /// Create with custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        let mut provider = Self::standalone();
        provider.base_url = base_url;
        provider
    }

    /// Create with supervisor and model paths
    pub fn with_model_paths(model_paths: HashMap<String, PathBuf>) -> Self {
        let models: Vec<String> = model_paths.keys().cloned().collect();
        let supervisor = Arc::new(Mutex::new(ModelSupervisor::new()));

        // Add model paths to supervisor
        {
            let mut sup = supervisor.blocking_lock();
            for (name, path) in model_paths {
                sup.add_model_path(name, path);
            }
        }

        Self {
            models,
            supervisor,
            ..Self::new(Arc::new(Mutex::new(ModelSupervisor::new())))
        }
    }

    /// Set the base URL
    pub fn set_base_url(&mut self, url: String) {
        self.base_url = url;
    }

    /// Set auto-start behavior
    pub fn set_auto_start(&mut self, auto_start: bool) {
        self.auto_start = auto_start;
    }

    /// Set prompt format
    pub fn set_prompt_format(&mut self, format: PromptFormat) {
        self.prompt_format = format;
    }

    /// Add a model path mapping (for models from config)
    pub fn add_model_path(&mut self, name: String, path: PathBuf) {
        // Also add to supervisor's model paths if using supervisor
        if let Ok(mut sup) = self.supervisor.try_lock() {
            sup.add_model_path(name.clone(), path);
        }
        // Add to local models list if not already present
        if !self.models.contains(&name) {
            self.models.push(name);
        }
    }

    /// Detect prompt format from model name
    fn detect_format(&self, model: &str) -> PromptFormat {
        let model_lower = model.to_lowercase();

        if model_lower.contains("qwen") {
            PromptFormat::ChatML
        } else if model_lower.contains("vicuna") {
            PromptFormat::Vicuna
        } else if model_lower.contains("alpaca") {
            PromptFormat::Alpaca
        } else if model_lower.contains("llama") || model_lower.contains("mistral") {
            PromptFormat::Llama
        } else {
            PromptFormat::Llama // Default to LLaMA format
        }
    }

    /// Format messages into a prompt string
    fn format_prompt(&self, messages: &[Message]) -> String {
        let format = self.detect_format(&self.model);

        match format {
            PromptFormat::Llama => self.format_llama(messages),
            PromptFormat::ChatML => self.format_chatml(messages),
            PromptFormat::Alpaca => self.format_alpaca(messages),
            PromptFormat::Vicuna => self.format_vicuna(messages),
            PromptFormat::Plain => self.format_plain(messages),
        }
    }

    /// LLaMA [INST] format
    fn format_llama(&self, messages: &[Message]) -> String {
        let mut prompt = String::new();

        for message in messages {
            match message.role {
                Role::System => {
                    prompt.push_str(&format!("<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n\n{}<|eot_id|>", message.content));
                }
                Role::User => {
                    prompt.push_str(&format!("<|start_header_id|>user<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n", message.content));
                }
                Role::Assistant => {
                    prompt.push_str(&format!("{}<|eot_id|>", message.content));
                }
            }
        }

        prompt
    }

    /// ChatML format (Qwen models)
    fn format_chatml(&self, messages: &[Message]) -> String {
        let mut prompt = String::new();

        for message in messages {
            match message.role {
                Role::System => {
                    prompt.push_str(&format!(
                        "<|im_start|>system\n{}<|im_end|>\n",
                        message.content
                    ));
                }
                Role::User => {
                    prompt.push_str(&format!(
                        "<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
                        message.content
                    ));
                }
                Role::Assistant => {
                    prompt.push_str(&format!("{}<|im_end|>\n", message.content));
                }
            }
        }

        prompt
    }

    /// Alpaca format
    fn format_alpaca(&self, messages: &[Message]) -> String {
        let mut prompt = String::new();

        for message in messages {
            match message.role {
                Role::System => {
                    prompt.push_str(&format!("### System:\n{}\n\n", message.content));
                }
                Role::User => {
                    prompt.push_str(&format!(
                        "### Instruction:\n{}\n\n### Response:\n",
                        message.content
                    ));
                }
                Role::Assistant => {
                    prompt.push_str(&format!("{}\n\n", message.content));
                }
            }
        }

        prompt
    }

    /// Vicuna format
    fn format_vicuna(&self, messages: &[Message]) -> String {
        let mut prompt = String::new();

        for message in messages {
            match message.role {
                Role::System => {
                    prompt.push_str(&format!("SYSTEM: {}\n", message.content));
                }
                Role::User => {
                    prompt.push_str(&format!("USER: {}\nASSISTANT: ", message.content));
                }
                Role::Assistant => {
                    prompt.push_str(&format!("{}\n", message.content));
                }
            }
        }

        prompt
    }

    /// Plain text format
    fn format_plain(&self, messages: &[Message]) -> String {
        messages
            .iter()
            .map(|m| format!("{:?}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Ensure the model is loaded and server is ready
    async fn ensure_model(&self) -> Result<(), ProviderError> {
        if !self.auto_start {
            return Ok(());
        }

        let mut supervisor = self.supervisor.lock().await;

        // Ensure model is loaded
        supervisor
            .ensure(&self.model)
            .map_err(|e| ProviderError::ApiError(format!("Failed to start model: {}", e)))?;

        Ok(())
    }

    /// Get the current base URL (from supervisor if auto-start is enabled)
    fn get_base_url(&self) -> &str {
        &self.base_url
    }

    /// Check if the server is running
    pub async fn is_running(&self) -> bool {
        let url = format!("{}/health", self.get_base_url());

        match self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

#[async_trait]
impl Provider for LlamaCppProvider {
    fn name(&self) -> &str {
        "llama.cpp"
    }

    fn models(&self) -> Vec<String> {
        self.models.clone()
    }

    fn current_model(&self) -> &str {
        &self.model
    }

    fn set_model(&mut self, model: String) {
        self.model = model;
    }

    fn is_configured(&self) -> bool {
        // llama.cpp doesn't need API key, just needs the server running
        // or the supervisor configured with model paths
        true
    }

    async fn send(&self, messages: Vec<Message>) -> Result<String, ProviderError> {
        // Ensure model is loaded
        self.ensure_model().await?;

        let prompt = self.format_prompt(&messages);
        let base_url = self.get_base_url();

        let request = CompletionRequest {
            prompt,
            n_predict: Some(2048),
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(40),
            stream: false,
        };

        let url = format!("{}/completion", base_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError(error_text));
        }

        let result: CompletionResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        Ok(result.content)
    }

    async fn send_with_system(
        &self,
        messages: Vec<Message>,
        system: Option<&str>,
    ) -> Result<String, ProviderError> {
        // Ensure model is loaded
        self.ensure_model().await?;

        // Prepend system message if provided
        let mut all_messages = Vec::new();
        if let Some(sys) = system {
            all_messages.push(Message {
                role: Role::System,
                content: sys.to_string(),
                name: None,
            });
        }
        all_messages.extend(messages);

        let prompt = self.format_prompt(&all_messages);
        let base_url = self.get_base_url();

        let request = CompletionRequest {
            prompt,
            n_predict: Some(2048),
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(40),
            stream: false,
        };

        let url = format!("{}/completion", base_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError(error_text));
        }

        let result: CompletionResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        Ok(result.content)
    }

    async fn send_stream(
        &self,
        messages: Vec<Message>,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>> {
        // Ensure model is loaded
        if let Err(e) = self.ensure_model().await {
            return Box::pin(futures::stream::once(async move { Err(e) }));
        }

        let prompt = self.format_prompt(&messages);
        let base_url = self.get_base_url().to_string();
        let client = self.client.clone();

        // Create streaming request
        let request = CompletionRequest {
            prompt,
            n_predict: Some(2048),
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(40),
            stream: true,
        };

        let url = format!("{}/completion", base_url);

        // Make the streaming request
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let error_text = resp.text().await.unwrap_or_default();
                    Box::pin(futures::stream::once(async move {
                        Err(ProviderError::ApiError(error_text))
                    }))
                } else {
                    // Parse SSE stream
                    Box::pin(parse_sse_stream(resp))
                }
            }
            Err(e) => Box::pin(futures::stream::once(async move {
                Err(ProviderError::NetworkError(e.to_string()))
            })),
        }
    }

    fn count_tokens(&self, text: &str) -> usize {
        // Rough approximation for llama models
        // Typically ~4 characters per token
        text.len() / 4
    }

    fn cost_per_million(&self) -> (f64, f64) {
        // llama.cpp is free (local inference)
        (0.0, 0.0)
    }
}

/// Parse SSE stream from llama.cpp server
fn parse_sse_stream(
    response: reqwest::Response,
) -> impl Stream<Item = Result<StreamChunk, ProviderError>> + Send {
    use futures::StreamExt;

    let mut buffer = String::new();

    async_stream::try_stream! {
        let mut stream = response.bytes_stream();

        while let Some(bytes_result) = stream.next().await {
            let bytes = bytes_result.map_err(|e| ProviderError::NetworkError(e.to_string()))?;
            let chunk = String::from_utf8_lossy(&bytes);
            buffer.push_str(&chunk);

            // Process complete lines
            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].to_string();
                buffer = buffer[pos + 1..].to_string();

                // Skip empty lines
                if line.trim().is_empty() {
                    continue;
                }

                // Parse SSE data lines
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        yield StreamChunk {
                            content: String::new(),
                            done: true,
                            tokens: None,
                        };
                        return;
                    }

                    // Parse JSON content
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = json.get("content").and_then(|c| c.as_str()) {
                            let is_done = json.get("stop")
                                .and_then(|s| s.as_bool())
                                .unwrap_or(false);

                            yield StreamChunk {
                                content: content.to_string(),
                                done: is_done,
                                tokens: None,
                            };

                            if is_done {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_llama() {
        let provider = LlamaCppProvider::standalone();
        let messages = vec![
            Message {
                role: Role::System,
                content: "You are helpful.".to_string(),
                name: None,
            },
            Message {
                role: Role::User,
                content: "Hello".to_string(),
                name: None,
            },
        ];

        let prompt = provider.format_prompt(&messages);
        assert!(prompt.contains("system"));
        assert!(prompt.contains("user"));
        assert!(prompt.contains("Hello"));
    }

    #[test]
    fn test_format_chatml() {
        let mut provider = LlamaCppProvider::standalone();
        provider.model = "qwen2.5".to_string();

        let messages = vec![Message {
            role: Role::User,
            content: "Hello".to_string(),
            name: None,
        }];

        let prompt = provider.format_prompt(&messages);
        assert!(prompt.contains("<|im_start|>user"));
        assert!(prompt.contains("<|im_end|>"));
    }

    #[test]
    fn test_detect_format() {
        let provider = LlamaCppProvider::standalone();

        assert!(matches!(
            provider.detect_format("llama3.2"),
            PromptFormat::Llama
        ));
        assert!(matches!(
            provider.detect_format("qwen2.5"),
            PromptFormat::ChatML
        ));
        assert!(matches!(
            provider.detect_format("vicuna"),
            PromptFormat::Vicuna
        ));
        assert!(matches!(
            provider.detect_format("alpaca"),
            PromptFormat::Alpaca
        ));
    }
}
