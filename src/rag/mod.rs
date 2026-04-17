//! RAG (Retrieval-Augmented Generation) Module
//!
//! Provides context-aware retrieval for enhanced responses.
//! Uses embedding-based similarity search over codebase.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RAG configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    /// Enable RAG retrieval
    pub enabled: bool,
    /// Maximum chunks to retrieve
    pub max_chunks: usize,
    /// Similarity threshold (0.0 - 1.0)
    pub similarity_threshold: f32,
    /// Chunk size in tokens
    pub chunk_size: usize,
    /// Chunk overlap in tokens
    pub chunk_overlap: usize,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_chunks: 5,
            similarity_threshold: 0.3,
            chunk_size: 512,
            chunk_overlap: 50,
        }
    }
}

/// A chunk of retrieved context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextChunk {
    /// Source file path
    pub file_path: String,
    /// Chunk content
    pub content: String,
    /// Start line in source file
    pub start_line: usize,
    /// End line in source file
    pub end_line: usize,
    /// Similarity score (0.0 - 1.0)
    pub similarity: f32,
    /// Embedding vector (simplified for now)
    pub embedding_hash: u64,
}

/// RAG retrieval result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagResult {
    /// Retrieved chunks
    pub chunks: Vec<ContextChunk>,
    /// Total retrieval time in ms
    pub retrieval_time_ms: u64,
    /// Whether RAG was actually used
    pub used: bool,
}

impl RagResult {
    pub fn empty() -> Self {
        Self {
            chunks: vec![],
            retrieval_time_ms: 0,
            used: false,
        }
    }

    pub fn format_context(&self) -> String {
        if self.chunks.is_empty() {
            return String::new();
        }

        let mut formatted = String::from("\n\n## Relevant Context\n\n");
        for (i, chunk) in self.chunks.iter().enumerate() {
            formatted.push_str(&format!(
                "### Chunk {} ({}:{}-{}) [similarity: {:.2}]\n```\n{}\n```\n\n",
                i + 1,
                chunk.file_path,
                chunk.start_line,
                chunk.end_line,
                chunk.similarity,
                chunk.content
            ));
        }
        formatted
    }
}

/// Simple keyword-based retriever (placeholder for full embedding search)
pub struct KeywordRetriever {
    config: RagConfig,
}

impl KeywordRetriever {
    pub fn new(config: RagConfig) -> Self {
        Self { config }
    }

    /// Retrieve relevant chunks based on keyword matching
    pub fn retrieve(&self, query: &str, documents: &[Document]) -> RagResult {
        let start = std::time::Instant::now();

        if !self.config.enabled || documents.is_empty() {
            return RagResult::empty();
        }

        let query_terms: Vec<&str> = query
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();

        let mut scored_chunks: Vec<(ContextChunk, f32)> = Vec::new();

        for doc in documents {
            for chunk in &doc.chunks {
                let score = self.compute_relevance(&query_terms, &chunk.content);
                if score >= self.config.similarity_threshold {
                    scored_chunks.push((chunk.clone(), score));
                }
            }
        }

        // Sort by score descending
        scored_chunks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N chunks
        let chunks: Vec<ContextChunk> = scored_chunks
            .into_iter()
            .take(self.config.max_chunks)
            .map(|(chunk, _)| chunk)
            .collect();

        let retrieval_time_ms = start.elapsed().as_millis() as u64;

        RagResult {
            chunks,
            retrieval_time_ms,
            used: true,
        }
    }

    /// Compute relevance score using TF-IDF-like keyword matching
    fn compute_relevance(&self, query_terms: &[&str], content: &str) -> f32 {
        let content_lower = content.to_lowercase();

        if query_terms.is_empty() {
            return 0.0;
        }

        let mut score = 0.0;
        for term in query_terms {
            if content_lower.contains(term) {
                // Bonus for exact matches
                score += 1.0;

                // Bonus for multiple occurrences
                let count = content_lower.matches(*term).count();
                score += (count as f32) * 0.5;

                // Bonus for term appearing in code identifiers (camelCase, snake_case)
                if content.contains(term) {
                    score += 0.5;
                }
            }
        }

        // Normalize by content length to avoid bias toward large files
        let length_factor = 1.0 / (1.0 + (content.len() as f32 / 1000.0));
        score * length_factor
    }
}

/// A document with pre-chunked content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// File path
    pub path: String,
    /// File content
    pub content: String,
    /// Pre-computed chunks
    pub chunks: Vec<ContextChunk>,
}

impl Document {
    /// Create a new document and chunk it
    pub fn new(path: String, content: String, chunk_size: usize, overlap: usize) -> Self {
        let chunks = Self::chunk_content(&path, &content, chunk_size, overlap);
        Self {
            path,
            content,
            chunks,
        }
    }

    /// Split content into overlapping chunks
    fn chunk_content(
        path: &str,
        content: &str,
        chunk_size: usize,
        overlap: usize,
    ) -> Vec<ContextChunk> {
        let mut chunks = Vec::new();

        // Simple line-based chunking
        let lines: Vec<&str> = content.lines().collect();
        let mut start = 0;

        while start < lines.len() {
            let mut end = start;
            let mut char_count = 0;

            // Accumulate lines until we hit chunk size
            while end < lines.len() && char_count < chunk_size {
                char_count += lines[end].len() + 1; // +1 for newline
                end += 1;
            }

            if end > start {
                let chunk_content = lines[start..end].join("\n");
                chunks.push(ContextChunk {
                    file_path: path.to_string(),
                    content: chunk_content,
                    start_line: start + 1, // 1-indexed
                    end_line: end,
                    similarity: 0.0, // Will be computed during retrieval
                    embedding_hash: 0, // Placeholder
                });
            }

            // Move start with overlap
            start = if end > overlap { end - overlap } else { end };
            if start >= end {
                break;
            }
        }

        chunks
    }
}

/// In-memory index for fast retrieval
pub struct RagIndex {
    documents: HashMap<String, Document>,
    config: RagConfig,
}

impl RagIndex {
    pub fn new(config: RagConfig) -> Self {
        Self {
            documents: HashMap::new(),
            config,
        }
    }

    /// Add a document to the index
    pub fn add_document(&mut self, path: String, content: String) {
        let doc = Document::new(
            path.clone(),
            content,
            self.config.chunk_size,
            self.config.chunk_overlap,
        );
        self.documents.insert(path, doc);
    }

    /// Remove a document from the index
    pub fn remove_document(&mut self, path: &str) {
        self.documents.remove(path);
    }

    /// Search the index
    pub fn search(&self, query: &str) -> RagResult {
        let retriever = KeywordRetriever::new(self.config.clone());
        let docs: Vec<&Document> = self.documents.values().collect();
        retriever.retrieve(
            query,
            &docs.into_iter().map(|d| (*d).clone()).collect::<Vec<_>>(),
        )
    }

    /// Get number of indexed documents
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// Get total number of chunks
    pub fn chunk_count(&self) -> usize {
        self.documents.values().map(|d| d.chunks.len()).sum()
    }
}

/// Compact prompts for efficient token usage
pub mod compact_prompts {
    /// Compact system prompt template
    pub const COMPACT_SYSTEM: &str = "QC: Local-first coding AI. Read/write/edit files, shell, analyze, search.
MODE: {mode} | TOOLS: read,write,bash,grep,glob | GIT: safe history
{context}";

    /// Ultra-compact for small context windows
    pub const ULTRA_COMPACT: &str = "QC AI: {mode}. Tools: read,write,bash,grep. {context}";

    /// Compress a prompt by removing redundancy
    pub fn compress_prompt(prompt: &str, target_tokens: usize) -> String {
        // Rough estimate: 4 chars per token
        let target_chars = target_tokens * 4;

        if prompt.len() <= target_chars {
            return prompt.to_string();
        }

        // Remove common filler phrases
        let compressed = prompt
            .replace("please ", "")
            .replace("could you ", "")
            .replace("i would like to ", "")
            .replace("i want to ", "")
            .replace("can you ", "")
            .replace("help me ", "")
            .replace("  ", " ");

        // If still too long, truncate with ellipsis
        if compressed.len() > target_chars {
            format!("{}...", &compressed[..target_chars.saturating_sub(3)])
        } else {
            compressed
        }
    }

    /// Format context chunks efficiently
    pub fn format_context_compact(chunks: &[crate::rag::ContextChunk]) -> String {
        if chunks.is_empty() {
            return String::new();
        }

        let mut formatted = String::with_capacity(chunks.len() * 200);
        for chunk in chunks {
            // Compact format: file:line-range + content preview
            let preview = if chunk.content.len() > 200 {
                format!("{}...", &chunk.content[..200])
            } else {
                chunk.content.clone()
            };
            formatted.push_str(&format!(
                "[{}:{}-{}] {}\n",
                chunk.file_path, chunk.start_line, chunk.end_line, preview
            ));
        }
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_chunking() {
        let content = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10";
        let doc = Document::new("test.txt".to_string(), content.to_string(), 50, 10);

        assert!(!doc.chunks.is_empty());
        assert!(doc.chunks.len() >= 2); // Should have multiple chunks
    }

    #[test]
    fn test_keyword_retriever() {
        let config = RagConfig::default();
        let retriever = KeywordRetriever::new(config);

        let docs = vec![Document::new(
            "test.rs".to_string(),
            "fn main() { println!(\"hello\"); }".to_string(),
            100,
            10,
        )];

        let result = retriever.retrieve("main function", &docs);
        assert!(result.used);
        assert!(!result.chunks.is_empty());
    }

    #[test]
    fn test_rag_index() {
        let mut index = RagIndex::new(RagConfig::default());

        index.add_document("file1.rs".to_string(), "fn test() {}".to_string());
        index.add_document("file2.rs".to_string(), "fn main() {}".to_string());

        assert_eq!(index.document_count(), 2);

        let result = index.search("main");
        assert!(result.used);
    }

    #[test]
    fn test_compact_prompts() {
        use compact_prompts::*;

        let long = "please could you help me to implement a new feature";
        let compressed = compress_prompt(long, 10);
        assert!(compressed.len() < long.len());
        assert!(!compressed.contains("please"));
    }

    #[test]
    fn test_context_formatting() {
        use compact_prompts::*;

        let chunks = vec![ContextChunk {
            file_path: "test.rs".to_string(),
            content: "fn main() {}".to_string(),
            start_line: 1,
            end_line: 1,
            similarity: 0.9,
            embedding_hash: 0,
        }];

        let compact = format_context_compact(&chunks);
        assert!(compact.contains("test.rs"));
        assert!(compact.contains("fn main"));
    }
}
