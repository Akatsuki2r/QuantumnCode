//! Agent executor - runs the agentic loop

use color_eyre::eyre::Result;
use futures::StreamExt;

use super::tools::{execute_tool, ToolCall, ToolResult};
use super::AGENT_SYSTEM_PROMPT;
use crate::providers::{Message, Provider, Role, StreamChunk};
use crate::router::{route, RouterConfig, RoutingDecision};

/// Max iterations to prevent infinite loops
const MAX_ITERATIONS: usize = 50;

/// Agent executor that handles tool calling loop
pub struct AgentExecutor {
    messages: Vec<Message>,
    iteration: usize,
    routing: Option<RoutingDecision>,
    cwd: String,
}

impl AgentExecutor {
    /// Create new executor with initial user message
    pub fn new(user_message: &str, cwd: &str) -> Self {
        let mut messages = vec![
            Message {
                role: Role::System,
                content: AGENT_SYSTEM_PROMPT.to_string(),
                name: None,
            },
            Message {
                role: Role::User,
                content: user_message.to_string(),
                name: None,
            },
        ];

        Self {
            messages,
            iteration: 0,
            routing: None,
            cwd: cwd.to_string(),
        }
    }

    /// Run the agent loop until done
    pub async fn run(&mut self, provider: &dyn Provider) -> Result<String> {
        // Initialize routing on first iteration if not set
        if self.routing.is_none() {
            let user_msg = self
                .messages
                .last()
                .filter(|m| m.role == Role::User)
                .map(|m| m.content.clone())
                .unwrap_or_default();

            let config = RouterConfig::default();
            self.routing = Some(route(&user_msg, &self.cwd, &config));

            // Log routing decision for debugging
            let routing = self.routing.as_ref().unwrap();
            tracing::debug!(
                target: "router",
                "Routing decision: intent={}, complexity={}, mode={}, model={}, confidence={:.2}",
                routing.intent.as_str(),
                routing.complexity.as_str(),
                routing.mode.as_str(),
                routing.model_tier.as_str(),
                routing.confidence
            );
        }

        loop {
            self.iteration += 1;

            if self.iteration > MAX_ITERATIONS {
                return Ok(
                    "Error: Max iterations exceeded. The AI may be in an infinite loop."
                        .to_string(),
                );
            }

            // Get routing decision
            let routing = self.routing.as_ref().unwrap();

            // Get response from AI
            let response = match self.get_ai_response(provider).await {
                Ok(r) => r,
                Err(e) => {
                    return Ok(format!(
                        "Error: AI provider error: {}\n\n\
                        Tips:\n\
                        - If using Anthropic/OpenAI: ensure your API key is set (export ANTHROPIC_API_KEY=...)\n\
                        - If using local: ensure Ollama is running (ollama serve)\n\
                        - Try switching to Ollama: quantumn model ollama",
                        e
                    ));
                }
            };

            // Add assistant message
            self.messages.push(Message {
                role: Role::Assistant,
                content: response.clone(),
                name: None,
            });

            // Parse tool calls
            let tool_calls = super::parse_tool_calls(&response);

            if tool_calls.is_empty() {
                // No more tool calls, we're done
                return Ok(response);
            }

            // Filter tool calls against routing policy
            let allowed = &routing.tools.allowed_tools;
            let filtered_calls: Vec<ToolCall> = tool_calls
                .into_iter()
                .filter(|call| {
                    allowed
                        .iter()
                        .any(|t| t.to_lowercase() == call.name.to_lowercase())
                })
                .collect();

            if filtered_calls.is_empty() {
                // All tools filtered out - add message and continue
                self.messages.push(Message {
                    role: Role::User,
                    content: "Note: No allowed tools for this operation. Respond with suggestions instead.".to_string(),
                    name: None,
                });
                continue;
            }

            let tool_calls = filtered_calls;

            // Execute tools and add results
            let tool_results = self.execute_tools(&tool_calls);

            for (call, result) in tool_calls.iter().zip(tool_results.iter()) {
                let result_text = format!(
                    "\n[Tool: {}]\nstdout:\n{}\nstderr:\n{}\nsuccess: {}\n",
                    call.name, result.stdout, result.stderr, result.success
                );

                self.messages.push(Message {
                    role: Role::User,
                    content: result_text,
                    name: None,
                });
            }
        }
    }

    /// Get response from AI
    async fn get_ai_response(
        &self,
        provider: &dyn Provider,
    ) -> std::result::Result<String, crate::providers::ProviderError> {
        // Use streaming for better UX but collect full response
        let stream = provider.send_stream(self.messages.clone()).await;

        let mut full_response = String::new();

        futures::pin_mut!(stream);
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    print!("{}", chunk.content);
                    std::io::Write::flush(&mut std::io::stdout()).ok();
                    full_response.push_str(&chunk.content);
                    if chunk.done {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("\nAPI Error: {}", e);
                    return Err(e);
                }
            }
        }

        println!(); // newline after streaming
        Ok(full_response)
    }

    /// Execute tools and return results
    fn execute_tools(&self, calls: &[ToolCall]) -> Vec<ToolResult> {
        calls.iter().map(execute_tool).collect()
    }

    /// Get conversation history
    pub fn get_history(&self) -> &[Message] {
        &self.messages
    }
}

/// Simple one-shot agentic request
pub async fn run_agentic(prompt: &str, provider: &dyn Provider) -> Result<String> {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    let mut executor = AgentExecutor::new(prompt, &cwd);
    executor.run(provider).await
}
