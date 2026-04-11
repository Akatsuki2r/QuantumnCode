//! LM Studio (local LLM) provider - Hybrid Mode
//!
//! Two modes:
//! 1. LM Studio API: Talk to LM Studio server at localhost:1234
//! 2. Local GGUF: Run GGUF files directly with turboquant llama-server
//!
//! Auto-detects which to use based on LM Studio API availability.

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::path::PathBuf;
use std::pin::Pin;
use std::process::{Child, Command};
use std::sync::Arc;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};

use super::provider_trait::{Provider, ProviderError, Message, Role, StreamChunk};

/// LM Studio provider mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LmStudioMode {
    /// Auto-detect: try LM Studio API first, fall back to local GGUF
    Auto,
    /// Force using LM Studio API server
    LmStudioApi,
    /// Force using local turboquant llama-server with GGUF files
    LocalLlamaCpp,
}

impl Default for LmStudioMode {
    fn default() -> Self {
        LmStudioMode::Auto
    }
}

/// Discovered model from LM Studio model directory
#[derive(Debug, Clone)]
struct DiscoveredModel {
    /// Model display name
    name: String,
    /// Full path to GGUF file
    path: PathBuf,
    /// Model ID (extracted from path)
    id: String,
}

/// LM Studio API client - Hybrid Mode
pub struct LmStudioProvider {
    base_url: String,
    model: String,
    api_token: Option<String>,
    client: reqwest::Client,
    mode: LmStudioMode,
    /// Path to turboquant llama-server binary
    turboquant_path: Option<PathBuf>,
    /// LM Studio models directory
    lm_studio_models_dir: PathBuf,
    /// Port for local llama-server
    local_port: u16,
    /// Cached discovered models
    discovered_models: Arc<Mutex<Vec<DiscoveredModel>>>,
    /// Currently running local server process (for LocalLlamaCpp mode)
    local_server: Arc<Mutex<Option<Child>>>,
}

/// LM Studio API request
#[derive(Debug, Serialize)]
struct LmStudioRequest {
    model: String,
    messages: Vec<LmStudioMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    context_length: Option<usize>,
}

/// LM Studio message format
#[derive(Debug, Serialize)]
struct LmStudioMessage {
    role: String,
    content: String,
}

/// LM Studio API response (non-streaming)
#[derive(Debug, Deserialize)]
struct LmStudioResponse {
    choices: Vec<LmStudioChoice>,
    #[serde(default)]
    stats: Option<LmStudioStats>,
}

#[derive(Debug, Deserialize)]
struct LmStudioChoice {
    message: LmStudioMessageResponse,
}

#[derive(Debug, Deserialize)]
struct LmStudioMessageResponse {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct LmStudioStats {
    tokens_per_second: Option<f64>,
    time_to_first_token: Option<f64>,
    generation_time: Option<f64>,
}

impl LmStudioProvider {
    /// Create a new LM Studio provider (hybrid mode)
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:1234".to_string(),
            model: "llama3.2".to_string(),
            api_token: None,
            client: reqwest::Client::new(),
            mode: LmStudioMode::Auto,
            turboquant_path: Some(PathBuf::from(
                "/home/nahanomark/llama-cpp-turboquant/build/bin/llama-server"
            )),
            lm_studio_models_dir: PathBuf::from(
                std::env::var("HOME").unwrap_or_else(|_| "~".to_string()) + "/.lmstudio/models"
            ),
            local_port: 8080,
            discovered_models: Arc::new(Mutex::new(Vec::new())),
            local_server: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the provider mode
    pub fn with_mode(mut self, mode: LmStudioMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set turboquant path
    pub fn with_turboquant_path(mut self, path: PathBuf) -> Self {
        self.turboquant_path = Some(path);
        self
    }

    /// Set local server port
    pub fn with_local_port(mut self, port: u16) -> Self {
        self.local_port = port;
        self
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

    /// Create with API token
    pub fn with_api_token(mut self, token: String) -> Self {
        self.api_token = Some(token);
        self
    }

    /// Check if LM Studio API is running
    pub async fn is_lm_studio_api_running(&self) -> bool {
        self.client
            .get(format!("{}/api/v1/models", self.base_url))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Scan ~/.lmstudio/models/ for GGUF files
    pub async fn scan_lm_studio_models(&self) -> Result<Vec<DiscoveredModel>, ProviderError> {
        let mut cached = self.discovered_models.lock().await;
        if !cached.is_empty() {
            return Ok(cached.clone());
        }

        let models_dir = &self.lm_studio_models_dir;
        let mut models = Vec::new();

        if models_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(models_dir) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            // Scan subdirectories for GGUF files
                            if let Ok(sub_entries) = std::fs::read_dir(entry.path()) {
                                for sub_entry in sub_entries.flatten() {
                                    if let Some(ext) = sub_entry.path().extension() {
                                        if ext == "gguf" {
                                            if let Some(file_name) = sub_entry.path().file_name() {
                                                let name = file_name.to_string_lossy().to_string();
                                                let model_id = sub_entry.path()
                                                    .file_stem()
                                                    .map(|s| s.to_string_lossy().to_string())
                                                    .unwrap_or_else(|| name.clone());

                                                models.push(DiscoveredModel {
                                                    name: name.clone(),
                                                    path: sub_entry.path(),
                                                    id: model_id,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        *cached = models.clone();
        Ok(models)
    }

    /// Ask user which mode to use (for interactive inquiry)
    pub fn ask_user_mode(&self) -> LmStudioMode {
        // In non-interactive context, default to Auto
        // The actual user inquiry would be handled by the chat UI
        LmStudioMode::Auto
    }

    /// Determine which mode to use based on settings and availability
    async fn determine_mode(&self) -> LmStudioMode {
        match self.mode {
            LmStudioMode::LmStudioApi => LmStudioMode::LmStudioApi,
            LmStudioMode::LocalLlamaCpp => LmStudioMode::LocalLlamaCpp,
            LmStudioMode::Auto => {
                if self.is_lm_studio_api_running().await {
                    LmStudioMode::LmStudioApi
                } else {
                    LmStudioMode::LocalLlamaCpp
                }
            }
        }
    }

    /// Get available models from LM Studio API
    pub async fn list_models_from_api(&self) -> Result<Vec<String>, ProviderError> {
        #[derive(Debug, Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelInfo>,
        }

        #[derive(Debug, Deserialize)]
        struct ModelInfo {
            id: String,
        }

        let mut request = self.client.get(format!("{}/api/v1/models", self.base_url));
        if let Some(ref token) = &self.api_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError("Failed to list models".to_string()));
        }

        let models: ModelsResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        Ok(models.data.into_iter().map(|m| m.id).collect())
    }

    /// Start local llama-server for a model
    async fn start_local_server(&self, model_path: &PathBuf) -> Result<String, ProviderError> {
        let mut server = self.local_server.lock().await;

        // Kill existing server if running
        if server.is_some() {
            if let Some(ref mut child) = *server {
                let _ = child.kill();
            }
            *server = None;
        }

        let turboquant = self.turboquant_path.as_ref()
            .ok_or_else(|| ProviderError::ConfigError("Turboquant path not set".to_string()))?;

        let port_arg = format!("--port={}", self.local_port);
        let model_arg = format!("--model={}", model_path.display());
        let ctx_arg = "--ctx-size=8192";

        let child = Command::new(turboquant)
            .arg(&model_arg)
            .arg(&port_arg)
            .arg(ctx_arg)
            .spawn()
            .map_err(|e| ProviderError::ConfigError(format!("Failed to start llama-server: {}", e)))?;

        *server = Some(child);

        // Wait for server to be ready
        let url = format!("http://localhost:{}/health", self.local_port);
        let start = std::time::Instant::now();

        while start.elapsed().as_secs() < 30 {
            if self.client.get(&url).send().await.is_ok() {
                return Ok(format!("http://localhost:{}", self.local_port));
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        Err(ProviderError::ConfigError("Local server failed to start".to_string()))
    }

    /// Stop local llama-server
    async fn stop_local_server(&self) {
        if let Some(ref mut child) = *self.local_server.lock().await {
            let _ = child.kill();
        }
    }

    /// Convert messages to LM Studio format
    fn convert_messages(&self, messages: Vec<Message>) -> Vec<LmStudioMessage> {
        messages
            .into_iter()
            .map(|m| LmStudioMessage {
                role: match m.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: m.content,
            })
            .collect()
    }

    /// Build request builder with optional auth
    fn build_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut builder = self.client.post(url);
        if let Some(ref token) = &self.api_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }
        builder.header("Content-Type", "application/json")
    }

    /// Check if LM Studio (API or local) is available
    pub async fn is_available(&self) -> bool {
        self.is_lm_studio_api_running().await || self.scan_lm_studio_models().await.is_ok()
    }
}

impl Default for LmStudioProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for LmStudioProvider {
    fn name(&self) -> &str {
        "lm_studio"
    }

    fn models(&self) -> Vec<String> {
        vec![
            "llama3.2".to_string(),
            "llama3.1".to_string(),
            "mistral".to_string(),
            "qwen2.5".to_string(),
            "gemma-4".to_string(),
            "deepseek-coder".to_string(),
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
        let mode = self.determine_mode().await;

        match mode {
            LmStudioMode::LmStudioApi => {
                let request = LmStudioRequest {
                    model: self.model.clone(),
                    messages: self.convert_messages(messages),
                    stream: false,
                    context_length: None,
                };

                let response = self.build_request(&format!("{}/api/v1/chat", self.base_url))
                    .json(&request)
                    .timeout(std::time::Duration::from_secs(120))
                    .send()
                    .await
                    .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

                if !response.status().is_success() {
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(ProviderError::ApiError(error_text));
                }

                let result: LmStudioResponse = response
                    .json()
                    .await
                    .map_err(|e| ProviderError::ApiError(e.to_string()))?;

                Ok(result.choices.into_iter()
                    .next()
                    .map(|c| c.message.content)
                    .unwrap_or_default())
            }
            LmStudioMode::LocalLlamaCpp | LmStudioMode::Auto => {
                // Find model path from discovered models
                let discovered = self.scan_lm_studio_models().await?;
                let model_path = discovered.iter()
                    .find(|m| m.id.to_lowercase().contains(&self.model.to_lowercase()) ||
                             m.name.to_lowercase().contains(&self.model.to_lowercase()))
                    .map(|m| m.path.clone())
                    .ok_or_else(|| ProviderError::ModelNotFound(self.model.clone()))?;

                // Start local server
                let base_url = self.start_local_server(&model_path).await?;

                // Send request to local server
                let request = LmStudioRequest {
                    model: self.model.clone(),
                    messages: self.convert_messages(messages),
                    stream: false,
                    context_length: None,
                };

                let response = self.client
                    .post(&format!("{}/completion", base_url))
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

                #[derive(Deserialize)]
                struct CompletionResponse {
                    content: String,
                }

                let result: CompletionResponse = response
                    .json()
                    .await
                    .map_err(|e| ProviderError::ApiError(e.to_string()))?;

                Ok(result.content)
            }
        }
    }

    async fn send_with_system(&self, messages: Vec<Message>, system: Option<&str>) -> Result<String, ProviderError> {
        let mut all_messages = Vec::new();
        if let Some(sys) = system {
            all_messages.push(Message {
                role: Role::System,
                content: sys.to_string(),
                name: None,
            });
        }
        all_messages.extend(messages);

        self.send(all_messages).await
    }

    async fn send_stream(
        &self,
        messages: Vec<Message>,
    ) -> Pin<Box<dyn Stream<Item = std::result::Result<StreamChunk, ProviderError>> + Send>> {
        // Capture all needed values before await
        let base_url = self.base_url.clone();
        let client = self.client.clone();
        let api_token = self.api_token.clone();
        let model = self.model.clone();
        let messages_for_request = self.convert_messages(messages);

        let mode = self.determine_mode().await;

        match mode {
            LmStudioMode::LmStudioApi => {
                let request = LmStudioRequest {
                    model: model.clone(),
                    messages: messages_for_request,
                    stream: true,
                    context_length: None,
                };

                let mut req_builder = client.post(&format!("{}/api/v1/chat", base_url));
                if let Some(ref token) = api_token {
                    req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
                }

                let response = req_builder
                    .header("Content-Type", "application/json")
                    .json(&request)
                    .timeout(std::time::Duration::from_secs(120))
                    .send()
                    .await;

                match response {
                    Ok(resp) if resp.status().is_success() => {
                        Box::pin(parse_lm_studio_sse_stream(resp))
                    }
                    Ok(resp) => {
                        let error_text = resp.text().await.unwrap_or_default();
                        Box::pin(futures::stream::once(async move {
                            Err(ProviderError::ApiError(error_text))
                        }))
                    }
                    Err(e) => {
                        Box::pin(futures::stream::once(async move {
                            Err(ProviderError::NetworkError(e.to_string()))
                        }))
                    }
                }
            }
            LmStudioMode::LocalLlamaCpp | LmStudioMode::Auto => {
                // Fall back to non-streaming for local mode (simpler for now)
                // Clone what we need to avoid self capture issues
                let base_url = self.base_url.clone();
                let model = self.model.clone();
                let turboquant = self.turboquant_path.clone();
                let lm_studio_dir = self.lm_studio_models_dir.clone();
                let local_port = self.local_port;
                let client = self.client.clone();
                let messages_json = serde_json::to_string(&messages_for_request).ok();

                Box::pin(futures::stream::once(async move {
                    // Scan for models
                    let mut discovered = Vec::new();
                    if lm_studio_dir.exists() {
                        if let Ok(entries) = std::fs::read_dir(&lm_studio_dir) {
                            for entry in entries.flatten() {
                                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                                    if let Ok(sub) = std::fs::read_dir(entry.path()) {
                                        for sub_entry in sub.flatten() {
                                            if sub_entry.path().extension().map(|e| e == "gguf").unwrap_or(false) {
                                                let name = sub_entry.file_name().to_string_lossy().to_string();
                                                discovered.push((name, sub_entry.path()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let model_path = discovered.iter()
                        .find(|(name, _)| name.to_lowercase().contains(&model.to_lowercase()))
                        .map(|(_, path)| path.clone())
                        .ok_or_else(|| ProviderError::ModelNotFound(model.clone()))?;

                    // Start local server
                    let turboquant = turboquant.as_ref()
                        .ok_or_else(|| ProviderError::ConfigError("Turboquant path not set".to_string()))?;

                    let mut child = std::process::Command::new(turboquant)
                        .arg(format!("--model={}", model_path.display()))
                        .arg(format!("--port={}", local_port))
                        .arg("--ctx-size=8192")
                        .spawn()
                        .map_err(|e| ProviderError::ConfigError(format!("Failed to start llama-server: {}", e)))?;

                    // Wait for server
                    let url = format!("http://localhost:{}/health", local_port);
                    let start = std::time::Instant::now();
                    while start.elapsed().as_secs() < 30 {
                        if client.get(&url).send().await.is_ok() {
                            break;
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }

                    // Send completion request
                    let request_json = messages_json.ok_or_else(|| ProviderError::ApiError("Failed to serialize messages".to_string()))?;
                    let response = client
                        .post(&format!("http://localhost:{}/completion", local_port))
                        .header("Content-Type", "application/json")
                        .body(request_json)
                        .timeout(std::time::Duration::from_secs(120))
                        .send()
                        .await
                        .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

                    let _ = child.kill();

                    if !response.status().is_success() {
                        return Err(ProviderError::ApiError("Completion request failed".to_string()));
                    }

                    #[derive(serde::Deserialize)]
                    struct CompletionResponse { content: String }

                    let result: CompletionResponse = response.json().await
                        .map_err(|e| ProviderError::ApiError(e.to_string()))?;

                    Ok(StreamChunk {
                        content: result.content,
                        done: true,
                        tokens: None,
                    })
                }))
            }
        }
    }

    fn count_tokens(&self, text: &str) -> usize {
        text.len() / 4
    }

    fn cost_per_million(&self) -> (f64, f64) {
        (0.0, 0.0)
    }
}

/// Parse SSE stream from LM Studio v1 API
fn parse_lm_studio_sse_stream(
    response: reqwest::Response,
) -> impl Stream<Item = std::result::Result<StreamChunk, ProviderError>> + Send {
    let mut buffer = String::new();
    let mut event_type = String::new();

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

                let trimmed = line.trim();
                if trimmed.is_empty() {
                    event_type.clear();
                    continue;
                }

                if let Some(type_value) = trimmed.strip_prefix("event: ") {
                    event_type = type_value.to_string();
                } else if let Some(data_value) = trimmed.strip_prefix("data: ") {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data_value) {
                        match event_type.as_str() {
                            "message.delta" => {
                                if let Some(content) = json.get("content").and_then(|c| c.as_str()) {
                                    yield StreamChunk {
                                        content: content.to_string(),
                                        done: false,
                                        tokens: None,
                                    };
                                }
                            }
                            "message.stop" | "chat.done" => {
                                yield StreamChunk {
                                    content: String::new(),
                                    done: true,
                                    tokens: None,
                                };
                                return;
                            }
                            "error" => {
                                let error_msg = json.get("error")
                                    .and_then(|e| e.get("message"))
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("Unknown error");
                                yield StreamChunk {
                                    content: format!("[LM Studio Error: {}]", error_msg),
                                    done: true,
                                    tokens: None,
                                };
                                return;
                            }
                            _ => {}
                        }
                    }
                    event_type.clear();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_messages() {
        let provider = LmStudioProvider::new();
        let messages = vec![
            Message { role: Role::System, content: "You are helpful.".to_string(), name: None },
            Message { role: Role::User, content: "Hello".to_string(), name: None },
        ];

        let converted = provider.convert_messages(messages);
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].role, "system");
        assert_eq!(converted[0].content, "You are helpful.");
        assert_eq!(converted[1].role, "user");
        assert_eq!(converted[1].content, "Hello");
    }
}
