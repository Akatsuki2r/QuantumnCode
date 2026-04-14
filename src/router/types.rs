//! Router type definitions
//!
//! Core data types for the 7-layer routing pipeline.

use serde::{Deserialize, Serialize};

// =============================================================================
// Intent Classification (16 intents via regex, < 1ms)
// =============================================================================

/// Intent - classified task type from user prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    // File operations
    Read,
    Write,
    Edit,
    Delete,
    // Shell operations
    Bash,
    Git,
    // Search operations
    Grep,
    Glob,
    Find,
    // Analysis operations
    Explain,
    Review,
    Debug,
    // Planning operations
    Plan,
    Design,
    // Meta operations
    Help,
    Chat,
    Unknown,
}

impl Intent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Intent::Read => "read",
            Intent::Write => "write",
            Intent::Edit => "edit",
            Intent::Delete => "delete",
            Intent::Bash => "bash",
            Intent::Git => "git",
            Intent::Grep => "grep",
            Intent::Glob => "glob",
            Intent::Find => "find",
            Intent::Explain => "explain",
            Intent::Review => "review",
            Intent::Debug => "debug",
            Intent::Plan => "plan",
            Intent::Design => "design",
            Intent::Help => "help",
            Intent::Chat => "chat",
            Intent::Unknown => "unknown",
        }
    }
}

// =============================================================================
// Complexity Estimation (5 levels)
// =============================================================================

/// Complexity - estimated task difficulty
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Complexity {
    Trivial = 0,
    Simple = 1,
    Moderate = 2,
    Complex = 3,
    Heavy = 4,
}

impl Complexity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Complexity::Trivial => "trivial",
            Complexity::Simple => "simple",
            Complexity::Moderate => "moderate",
            Complexity::Complex => "complex",
            Complexity::Heavy => "heavy",
        }
    }

    /// Numeric score for threshold comparisons
    pub fn score(&self) -> u8 {
        *self as u8
    }
}

// =============================================================================
// Mode Selection (5 modes)
// =============================================================================

/// AgentMode - execution mode for the current task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    /// Conversational - minimal tools, quick responses
    Chat,
    /// Planning - analysis only, no execution
    Plan,
    /// Building - full execution capabilities
    Build,
    /// Code review - read-only analysis
    Review,
    /// Debugging - diagnostic tools only
    Debug,
}

impl AgentMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentMode::Chat => "chat",
            AgentMode::Plan => "plan",
            AgentMode::Build => "build",
            AgentMode::Review => "review",
            AgentMode::Debug => "debug",
        }
    }

    /// State machine transition - check if transition is allowed
    pub fn can_transition_to(&self, target: AgentMode) -> bool {
        match (self, target) {
            // Forward transitions (typical workflow)
            (AgentMode::Chat, AgentMode::Plan) => true,
            (AgentMode::Chat, AgentMode::Build) => true,
            (AgentMode::Plan, AgentMode::Build) => true,
            // Backward transitions (replanning)
            (AgentMode::Build, AgentMode::Plan) => true,
            // Special cases
            (AgentMode::Chat, AgentMode::Review) => true,
            (AgentMode::Chat, AgentMode::Debug) => true,
            (AgentMode::Build, AgentMode::Debug) => true,
            // Same mode always allowed
            _ if *self == target => true,
            // All other transitions disallowed
            _ => false,
        }
    }

    /// Mode instruction for system prompt
    pub fn instruction(&self) -> &'static str {
        match self {
            AgentMode::Chat => "Answer directly. Suggest tools only if needed.",
            AgentMode::Plan => "Analyze and plan. Do NOT execute. Read-only.",
            AgentMode::Build => "Implement changes. Verify. Report progress.",
            AgentMode::Review => "Review code. Report issues and suggestions.",
            AgentMode::Debug => "Investigate. Find root cause. Suggest fix.",
        }
    }
}

// =============================================================================
// Model Tier Selection
// =============================================================================

/// ModelTier - capability level for model selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelTier {
    /// Ollama/llama.cpp - no API cost
    Local,
    /// claude-haiku, gpt-4o-mini - quick tasks
    Fast,
    /// claude-sonnet, gpt-4o - balanced
    Standard,
    /// claude-opus, gpt-4 - complex tasks
    Capable,
}

impl ModelTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelTier::Local => "local",
            ModelTier::Fast => "fast",
            ModelTier::Standard => "standard",
            ModelTier::Capable => "capable",
        }
    }

    /// Default model name for this tier
    pub fn default_model(&self) -> &'static str {
        match self {
            ModelTier::Local => "llama3.2:latest",
            ModelTier::Fast => "claude-haiku-4-20250514",
            ModelTier::Standard => "claude-sonnet-4-20250514",
            ModelTier::Capable => "claude-opus-4-20250514",
        }
    }
}

// =============================================================================
// Context Budget Strategy
// =============================================================================

/// ContextBudget - token budget for conversation context
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContextBudget {
    /// ~4K tokens - trivial/simple tasks
    Minimal = 4_000,
    /// ~16K tokens - moderate tasks
    Relevant = 16_000,
    /// ~50K tokens - complex tasks
    Standard = 50_000,
    /// ~100K tokens - heavy/comprehensive tasks
    Comprehensive = 100_000,
}

impl ContextBudget {
    pub fn tokens(&self) -> usize {
        *self as usize
    }

    pub fn from_complexity(complexity: Complexity) -> Self {
        match complexity {
            Complexity::Trivial => ContextBudget::Minimal,
            Complexity::Simple => ContextBudget::Minimal,
            Complexity::Moderate => ContextBudget::Relevant,
            Complexity::Complex => ContextBudget::Standard,
            Complexity::Heavy => ContextBudget::Comprehensive,
        }
    }
}

// =============================================================================
// Memory Policy
// =============================================================================

/// MemoryPolicy - memory loading strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryPolicy {
    /// No memory loading
    None,
    /// Last N files modified
    Recent,
    /// Files relevant to current task
    Relevant,
    /// Load all recent context
    Full,
}

impl MemoryPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryPolicy::None => "none",
            MemoryPolicy::Recent => "recent",
            MemoryPolicy::Relevant => "relevant",
            MemoryPolicy::Full => "full",
        }
    }
}

// =============================================================================
// Tool Policy
// =============================================================================

/// ToolPolicy - per-intent tool permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPolicy {
    /// Tools allowed for this routing decision
    pub allowed_tools: Vec<String>,
    /// Tools explicitly disallowed
    pub disallowed_tools: Vec<String>,
    /// Whether destructive operations require confirmation
    pub require_confirmation: bool,
}

impl ToolPolicy {
    /// Default tool policy (all basic tools allowed)
    pub fn default_policy() -> Self {
        Self {
            allowed_tools: vec![
                "Read".to_string(),
                "Write".to_string(),
                "Bash".to_string(),
                "Grep".to_string(),
                "Glob".to_string(),
            ],
            disallowed_tools: vec![],
            require_confirmation: false,
        }
    }

    /// Read-only tool policy
    pub fn read_only() -> Self {
        Self {
            allowed_tools: vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string()],
            disallowed_tools: vec!["Write".to_string(), "Bash".to_string()],
            require_confirmation: false,
        }
    }

    /// Tool policy with confirmation required
    pub fn with_confirmation(base: Self) -> Self {
        Self {
            require_confirmation: true,
            ..base
        }
    }

    /// Check if a tool is allowed
    pub fn is_tool_allowed(&self, tool_name: &str) -> bool {
        self.allowed_tools
            .iter()
            .any(|t| t.to_lowercase() == tool_name.to_lowercase())
    }
}

// =============================================================================
// Routing Decision
// =============================================================================

/// RoutingDecision - complete output of the router's 7-layer pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Classified intent of the user prompt
    pub intent: Intent,
    /// Estimated complexity level
    pub complexity: Complexity,
    /// Selected operating mode
    pub mode: AgentMode,
    /// Selected model tier
    pub model_tier: ModelTier,
    /// Tool policy - which tools are allowed
    pub tools: ToolPolicy,
    /// Context budget allocation
    pub context_budget: ContextBudget,
    /// Memory loading policy
    pub memory_policy: MemoryPolicy,
    /// Confidence score (0.0 - 1.0) for the routing decision
    pub confidence: f32,
    /// Human-readable reasoning for debugging/tracing
    pub reasoning: String,
}

impl RoutingDecision {
    /// Create a new routing decision
    pub fn new(
        intent: Intent,
        complexity: Complexity,
        mode: AgentMode,
        model_tier: ModelTier,
        tools: ToolPolicy,
        context_budget: ContextBudget,
        memory_policy: MemoryPolicy,
        confidence: f32,
        reasoning: String,
    ) -> Self {
        Self {
            intent,
            complexity,
            mode,
            model_tier,
            tools,
            context_budget,
            memory_policy,
            confidence,
            reasoning,
        }
    }

    /// Create a default routing decision (chat mode, all tools allowed)
    pub fn default() -> Self {
        Self {
            intent: Intent::Chat,
            complexity: Complexity::Simple,
            mode: AgentMode::Chat,
            model_tier: ModelTier::Fast,
            tools: ToolPolicy::default_policy(),
            context_budget: ContextBudget::Minimal,
            memory_policy: MemoryPolicy::None,
            confidence: 0.5,
            reasoning: "default routing".to_string(),
        }
    }
}

// =============================================================================
// Router Configuration
// =============================================================================

/// RouterConfig - configuration for routing behavior
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Prefer local models (Ollama/llama.cpp) over API models
    pub prefer_local: bool,
    /// Maximum cost per million tokens
    pub cost_limit: f32,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            prefer_local: false,
            cost_limit: 1.0,
        }
    }
}
