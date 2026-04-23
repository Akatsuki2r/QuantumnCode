//! AI provider system
//!
//! Supports multiple AI providers: Anthropic Claude, OpenAI, Ollama, llama.cpp, LM Studio, and OpenCode Zen

pub mod anthropic;
pub mod llama_cpp;
pub mod lm_studio;
pub mod local_discover;
pub mod ollama;
pub mod openai;
pub mod opencode;
pub mod provider_trait;

pub use anthropic::AnthropicProvider;
pub use llama_cpp::LlamaCppProvider;
pub use lm_studio::LmStudioProvider;
pub use local_discover::{
    discover_all_models, format_size, get_all_models, LocalModel, LocalModelConfig,
};
pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
pub use opencode::OpenCodeProvider;
pub use provider_trait::{Message, Provider, ProviderError, Role, StreamChunk};
