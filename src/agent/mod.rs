//! Minimalistic agentic workflow - Bear tools for AI
//!
//! Ultra-simple tool system that adapts to the AI's needs.
//! Just the essentials: read, write, bash, grep, glob.

mod executor;
mod parser;
mod tools;

pub use crate::router::{route, RouterConfig, RoutingDecision};
pub use executor::{run_agentic, AgentExecutor};
pub use parser::parse_tool_calls;
pub use tools::{get_tools, Tool, ToolCall, ToolHandler, ToolRegistry, ToolResult};

/// System prompt for agentic mode
pub const AGENT_SYSTEM_PROMPT: &str = r#"
AGENT MODE - You have tools. Use them.

RULES:
1. Think before you act - plan your approach
2. Use tools efficiently - don't repeat yourself
3. Report results clearly - show what changed
4. Ask if unsure - don't guess at destructive actions

AVAILABLE TOOLS:
{{TOOLS_LIST}}

{{TOOL_CALL_FORMAT}}

When done, respond with your final answer.
"#;
