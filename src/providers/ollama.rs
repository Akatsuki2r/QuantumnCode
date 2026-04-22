//! Ollama (local LLM) provider

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::path::PathBuf;
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

/// Detailed Ollama model information from /api/tags
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaModelDetail {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
    pub details: Option<ModelDetails>,
}

/// Model details nested within OllamaModelDetail
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelDetails {
    pub format: Option<String>,
    pub family: Option<String>,
    pub families: Option<Vec<String>>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

/// Running model information from /api/ps
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaRunningModel {
    pub name: String,
    pub model: String,
    pub size: u64,
    pub digest: String,
    pub options: Option<serde_json::Value>,
    pub expires_at: Option<String>,
    pub size_vram: Option<u64>,
}

/// Detailed model info from /api/show
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaModelInfo {
    pub license: Option<String>,
    pub modelfile: Option<String>,
    pub parameters: Option<String>,
    pub template: Option<String>,
    pub system: Option<String>,
    pub details: Option<ModelDetails>,
    pub messages: Option<Vec<serde_json::Value>>,
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

    /// List available models via API
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

    /// Get detailed model information via API
    pub async fn list_models_detailed(&self) -> Result<Vec<OllamaModelDetail>, ProviderError> {
        #[derive(Debug, Deserialize)]
        struct TagsResponse {
            models: Vec<OllamaModelDetail>,
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

        Ok(tags.models)
    }

    /// List models via CLI parsing (fallback when API not available)
    pub fn list_models_cli() -> Result<Vec<String>, String> {
        let output = std::process::Command::new("ollama")
            .args(["list"])
            .output()
            .map_err(|e| format!("Failed to execute ollama list: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ollama list failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut models = Vec::new();

        // Parse ollama list output
        // Format: NAME                    ID          SIZE      MODIFIED
        for line in stdout.lines().skip(1) {
            // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                models.push(parts[0].to_string());
            }
        }

        Ok(models)
    }

    /// Get Ollama models directory path
    /// Checks OLLAMA_MODELS env var first, then uses platform defaults
    pub fn get_models_path() -> Option<PathBuf> {
        use std::env;

        // Check environment variable first
        if let Ok(ollama_models) = env::var("OLLAMA_MODELS") {
            if !ollama_models.is_empty() {
                return Some(PathBuf::from(ollama_models));
            }
        }

        // Platform-specific defaults
        #[cfg(target_os = "windows")]
        {
            if let Ok(home) = env::var("USERPROFILE") {
                return Some(PathBuf::from(home).join(".ollama").join("models"));
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = env::var("HOME") {
                return Some(PathBuf::from(home).join(".ollama").join("models"));
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Try user home first
            if let Ok(home) = env::var("HOME") {
                let user_path = PathBuf::from(&home).join(".ollama").join("models");
                if user_path.exists() {
                    return Some(user_path);
                }
            }
            // Fall back to system path
            let system_path = PathBuf::from("/usr/share/ollama/models");
            if system_path.exists() {
                return Some(system_path);
            }
        }

        None
    }

    /// Comprehensive Ollama model detection
    /// Tries API first, then CLI, then filesystem scan
    /// Returns (models_list, detailed_info, is_running)
    pub fn detect_models_comprehensive() -> (Vec<String>, Vec<OllamaModelDetail>, bool) {
        // Method 1: Try API (most reliable, gives detailed info)
        // Spawn a new thread with its own runtime to avoid blocking the current one
        let handle = std::thread::spawn(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create runtime");
            let provider = Self::new();
            rt.block_on(async { provider.list_models_detailed().await })
        });
        let api_result = handle.join().ok().and_then(|r| r.ok());

        if let Some(models) = api_result {
            let names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
            tracing::debug!("Detected {} Ollama models via API", names.len());
            return (names, models, true);
        }

        // Method 2: Try CLI parsing
        let cli_result = Self::list_models_cli();
        if let Ok(models) = cli_result {
            tracing::debug!("Detected {} Ollama models via CLI", models.len());
            // Create basic details from CLI output
            let details: Vec<OllamaModelDetail> = models
                .iter()
                .map(|name| OllamaModelDetail {
                    name: name.clone(),
                    modified_at: "unknown".to_string(),
                    size: 0,
                    digest: "unknown".to_string(),
                    details: None,
                })
                .collect();
            return (models, details, false);
        }

        // Method 3: Filesystem scan (last resort)
        let fs_result = Self::scan_models_filesystem();
        if let Ok(models) = fs_result {
            let names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
            tracing::debug!("Detected {} Ollama models via filesystem", names.len());
            return (names, models, false);
        }

        tracing::debug!("No Ollama models detected");
        (Vec::new(), Vec::new(), false)
    }

    /// Scan filesystem for Ollama models (fallback when server not running)
    /// Reads manifest files to get human-readable model names
    pub fn scan_models_filesystem() -> Result<Vec<OllamaModelDetail>, String> {
        use std::fs;

        let models_path = Self::get_models_path()
            .ok_or_else(|| "Could not determine Ollama models path".to_string())?;

        if !models_path.exists() {
            return Ok(Vec::new());
        }

        let mut models = Vec::new();
        let manifests_dir = models_path.join("manifests");

        if !manifests_dir.exists() {
            return Ok(Vec::new());
        }

        // Walk through registry/ollama.com/library/ structure
        let library_dir = manifests_dir
            .join("registry")
            .join("ollama.com")
            .join("library");
        if !library_dir.exists() {
            return Ok(Vec::new());
        }

        // Each subdirectory is a model namespace (e.g., "llama3", "mistral")
        if let Ok(namespace_entries) = fs::read_dir(&library_dir) {
            for namespace_entry in namespace_entries.flatten() {
                let namespace_path = namespace_entry.path();
                if !namespace_path.is_dir() {
                    continue;
                }

                // Each sub-subdirectory is a model tag (e.g., "latest", "7b")
                if let Ok(tag_entries) = fs::read_dir(&namespace_path) {
                    for tag_entry in tag_entries.flatten() {
                        let tag_path = tag_entry.path();
                        if !tag_path.is_dir() {
                            continue;
                        }

                        // Read the manifest file
                        let manifest_path = tag_path.join("latest");
                        if !manifest_path.exists() {
                            continue;
                        }

                        if let Ok(manifest_content) = fs::read_to_string(&manifest_path) {
                            if let Ok(manifest_json) =
                                serde_json::from_str::<serde_json::Value>(&manifest_content)
                            {
                                let name = format!(
                                    "{}:latest",
                                    namespace_path
                                        .file_name()
                                        .map(|s| s.to_string_lossy())
                                        .unwrap_or_default()
                                );

                                let size = manifest_json
                                    .get("total_size")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);

                                let digest = manifest_json
                                    .get("schema_version")
                                    .map(|v| v.to_string())
                                    .unwrap_or_else(|| "unknown".to_string());

                                let modified = fs::metadata(&manifest_path)
                                    .ok()
                                    .and_then(|m| m.modified().ok())
                                    .map(|t| {
                                        format!(
                                            "{:04}-{:02}-{:02}",
                                            t.elapsed().map(|d| d.as_secs() / 86400).unwrap_or(0)
                                                / 365,
                                            1,
                                            1
                                        )
                                    })
                                    .unwrap_or_else(|| "unknown".to_string());

                                models.push(OllamaModelDetail {
                                    name,
                                    modified_at: modified,
                                    size,
                                    digest,
                                    details: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(models)
    }

    /// Get currently running/loading models via /api/ps endpoint
    pub async fn list_running_models(&self) -> Result<Vec<OllamaRunningModel>, ProviderError> {
        #[derive(Debug, Deserialize)]
        struct ProcessResponse {
            models: Vec<OllamaRunningModel>,
        }

        let response = self
            .client
            .get(format!("{}/api/ps", self.base_url))
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError(
                "Failed to list running models".to_string(),
            ));
        }

        let result: ProcessResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        Ok(result.models)
    }

    /// Show detailed model information
    pub async fn show_model(&self, model: &str) -> Result<OllamaModelInfo, ProviderError> {
        #[derive(Debug, Serialize)]
        struct ShowRequest {
            name: String,
        }

        let request = ShowRequest {
            name: model.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/api/show", self.base_url))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::ApiError(format!(
                "Failed to show model: {}",
                model
            )));
        }

        let info: OllamaModelInfo = response
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        Ok(info)
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
            "ollama_isnt_running".to_string(),
            "llama2".to_string(),
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
        let client = self.client.clone();
        let url = format!("{}/api/chat", self.base_url);
        let request = OllamaRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            stream: true,
        };

        let stream = async_stream::try_stream! {
            let response = client
                .post(url)
                .json(&request)
                .send()
                .await
                .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

            let mut bytes_stream = response.bytes_stream();
            while let Some(chunk_result) = bytes_stream.next().await {
                let bytes = chunk_result.map_err(|e| ProviderError::ApiError(e.to_string()))?;
                let chunk: OllamaResponse = serde_json::from_slice(&bytes)
                    .map_err(|e| ProviderError::ApiError(e.to_string()))?;
                
                yield StreamChunk {
                    content: chunk.message.content,
                    done: chunk.done,
                    tokens: None,
                };
            }
        };

        Box::pin(stream)
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
