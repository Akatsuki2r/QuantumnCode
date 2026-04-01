//! AI provider system
//!
//! Supports multiple AI providers: Anthropic Claude, OpenAI, and Ollama

pub mod provider_trait;
pub mod anthropic;
pub mod openai;
pub mod ollama;

pub use provider_trait::{Provider, ProviderError, Message, Role, StreamChunk};
pub use anthropic::AnthropicProvider;
pub use openai::OpenAIProvider;
pub use ollama::OllamaProvider;