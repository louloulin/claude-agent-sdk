//! Type definitions for V2 API
//!
//! This module contains the simplified types used by the V2 API.

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Simplified session options for V2 API
///
/// `SessionOptions` contains only the most commonly used configuration parameters,
/// making it easier to configure Claude compared to the full `ClaudeAgentOptions`.
///
/// For advanced configuration, convert to `ClaudeAgentOptions` using `.into()`.
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk::v2::SessionOptions;
/// use claude_agent_sdk::v2::PermissionMode;
///
/// // Default configuration
/// let options = SessionOptions::default();
///
/// // Custom configuration
/// let options = SessionOptions::builder()
///     .model("claude-sonnet-4-20250514".to_string())
///     .max_turns(10)
///     .permission_mode(PermissionMode::BypassPermissions)
///     .build();
/// ```
#[derive(Debug, Clone, TypedBuilder, Serialize, Deserialize)]
#[derive(Default)]
pub struct SessionOptions {
    /// Model to use (None = system default)
    #[builder(default, setter(strip_option))]
    pub model: Option<String>,

    /// Permission mode for tool execution
    #[builder(default, setter(strip_option))]
    pub permission_mode: Option<PermissionMode>,

    /// Maximum budget in USD (None = no limit)
    #[builder(default, setter(strip_option))]
    pub max_budget_usd: Option<f64>,

    /// Maximum number of conversation turns
    #[builder(default, setter(strip_option))]
    pub max_turns: Option<u32>,

    /// Maximum thinking tokens for extended thinking
    #[builder(default, setter(strip_option))]
    pub max_thinking_tokens: Option<u32>,

    /// Custom system prompt
    #[builder(default, setter(strip_option))]
    pub system_prompt: Option<String>,

    /// Whether to include partial messages in stream
    #[builder(default = false)]
    pub include_partial_messages: bool,
}


impl From<SessionOptions> for crate::types::config::ClaudeAgentOptions {
    fn from(options: SessionOptions) -> Self {
        // Convert permission_mode if present
        let permission_mode: Option<crate::types::config::PermissionMode> =
            options.permission_mode.map(|pm| pm.into());

        // Convert system_prompt to SystemPrompt if present
        let system_prompt: Option<crate::types::config::SystemPrompt> =
            options.system_prompt.map(crate::types::config::SystemPrompt::Text);

        // Build ClaudeAgentOptions using builder with conditional field setting
        // Since we can't use if-else with builder reassignment due to TypedBuilder's type system,
        // we use a match to handle the different cases
        match (options.model, permission_mode, options.max_budget_usd) {
            (Some(model), Some(pm), Some(max_budget)) => {
                crate::types::config::ClaudeAgentOptions::builder()
                    .model(model)
                    .permission_mode(pm)
                    .max_budget_usd(max_budget)
                    .max_turns(options.max_turns.unwrap_or(0))
                    .max_thinking_tokens(options.max_thinking_tokens.unwrap_or(0))
                    .system_prompt(system_prompt.unwrap_or(
                        crate::types::config::SystemPrompt::Text(String::new())
                    ))
                    .include_partial_messages(options.include_partial_messages)
                    .build()
            }
            (Some(model), Some(pm), None) => {
                crate::types::config::ClaudeAgentOptions::builder()
                    .model(model)
                    .permission_mode(pm)
                    .max_turns(options.max_turns.unwrap_or(0))
                    .max_thinking_tokens(options.max_thinking_tokens.unwrap_or(0))
                    .system_prompt(system_prompt.unwrap_or(
                        crate::types::config::SystemPrompt::Text(String::new())
                    ))
                    .include_partial_messages(options.include_partial_messages)
                    .build()
            }
            (Some(model), None, Some(max_budget)) => {
                crate::types::config::ClaudeAgentOptions::builder()
                    .model(model)
                    .max_budget_usd(max_budget)
                    .max_turns(options.max_turns.unwrap_or(0))
                    .max_thinking_tokens(options.max_thinking_tokens.unwrap_or(0))
                    .system_prompt(system_prompt.unwrap_or(
                        crate::types::config::SystemPrompt::Text(String::new())
                    ))
                    .include_partial_messages(options.include_partial_messages)
                    .build()
            }
            (Some(model), None, None) => {
                crate::types::config::ClaudeAgentOptions::builder()
                    .model(model)
                    .max_turns(options.max_turns.unwrap_or(0))
                    .max_thinking_tokens(options.max_thinking_tokens.unwrap_or(0))
                    .system_prompt(system_prompt.unwrap_or(
                        crate::types::config::SystemPrompt::Text(String::new())
                    ))
                    .include_partial_messages(options.include_partial_messages)
                    .build()
            }
            (None, Some(pm), Some(max_budget)) => {
                crate::types::config::ClaudeAgentOptions::builder()
                    .permission_mode(pm)
                    .max_budget_usd(max_budget)
                    .max_turns(options.max_turns.unwrap_or(0))
                    .max_thinking_tokens(options.max_thinking_tokens.unwrap_or(0))
                    .system_prompt(system_prompt.unwrap_or(
                        crate::types::config::SystemPrompt::Text(String::new())
                    ))
                    .include_partial_messages(options.include_partial_messages)
                    .build()
            }
            (None, Some(pm), None) => {
                crate::types::config::ClaudeAgentOptions::builder()
                    .permission_mode(pm)
                    .max_turns(options.max_turns.unwrap_or(0))
                    .max_thinking_tokens(options.max_thinking_tokens.unwrap_or(0))
                    .system_prompt(system_prompt.unwrap_or(
                        crate::types::config::SystemPrompt::Text(String::new())
                    ))
                    .include_partial_messages(options.include_partial_messages)
                    .build()
            }
            (None, None, Some(max_budget)) => {
                crate::types::config::ClaudeAgentOptions::builder()
                    .max_budget_usd(max_budget)
                    .max_turns(options.max_turns.unwrap_or(0))
                    .max_thinking_tokens(options.max_thinking_tokens.unwrap_or(0))
                    .system_prompt(system_prompt.unwrap_or(
                        crate::types::config::SystemPrompt::Text(String::new())
                    ))
                    .include_partial_messages(options.include_partial_messages)
                    .build()
            }
            (None, None, None) => {
                crate::types::config::ClaudeAgentOptions::builder()
                    .max_turns(options.max_turns.unwrap_or(0))
                    .max_thinking_tokens(options.max_thinking_tokens.unwrap_or(0))
                    .system_prompt(system_prompt.unwrap_or(
                        crate::types::config::SystemPrompt::Text(String::new())
                    ))
                    .include_partial_messages(options.include_partial_messages)
                    .build()
            }
        }
    }
}

/// Permission mode for tool execution
///
/// Controls how Claude requests permission to use tools.
///
/// # Variants
///
/// * `Default` - Default permission mode
/// * `AcceptEdits` - Accept edits automatically
/// * `Plan` - Plan mode
/// * `BypassPermissions` - Auto-approve all tool usage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionMode {
    /// Default permission mode
    Default,

    /// Accept edits automatically
    AcceptEdits,

    /// Plan mode
    Plan,

    /// Auto-approve all tool usage
    BypassPermissions,
}

impl From<PermissionMode> for crate::types::config::PermissionMode {
    fn from(mode: PermissionMode) -> Self {
        match mode {
            PermissionMode::Default => Self::Default,
            PermissionMode::AcceptEdits => Self::AcceptEdits,
            PermissionMode::Plan => Self::Plan,
            PermissionMode::BypassPermissions => Self::BypassPermissions,
        }
    }
}

/// Result of a one-shot prompt
///
/// Contains the response text and metadata from a `prompt()` call.
///
/// # Example
///
/// ```no_run
/// # use claude_agent_sdk::v2::PromptResult;
/// let result = PromptResult {
///     content: "The answer is 4".to_string(),
///     input_tokens: 15,
///     output_tokens: 5,
///     model: Some("claude-sonnet-4-20250514".to_string()),
/// };
///
/// println!("Response: {}", result.content);
/// println!("Total tokens: {}", result.total_tokens());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResult {
    /// The text content of Claude's response
    pub content: String,

    /// Number of tokens in the input
    pub input_tokens: u64,

    /// Number of tokens in the output
    pub output_tokens: u64,

    /// Model used for generation (if known)
    pub model: Option<String>,

    /// Buffer metrics from the transport layer (if available)
    ///
    /// This provides insight into buffer usage during the query,
    /// including peak size, message count, and resize operations.
    /// Only available when using `SubprocessTransport` without pooling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer_metrics: Option<crate::internal::transport::BufferMetricsSnapshot>,
}

impl PromptResult {
    /// Get the total token usage (input + output)
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }

    /// Get the cost in USD (approximate)
    ///
    /// This is a rough estimate based on public pricing.
    /// Actual costs may vary.
    pub fn estimated_cost_usd(&self) -> f64 {
        // Rough pricing (subject to change)
        // Input: $3/M tokens, Output: $15/M tokens
        let input_cost = (self.input_tokens as f64) / 1_000_000.0 * 3.0;
        let output_cost = (self.output_tokens as f64) / 1_000_000.0 * 15.0;
        input_cost + output_cost
    }
}

/// Simplified message type for V2 API
///
/// This is a simplified version of the full `Message` type,
/// containing only the most essential information.
///
/// # Variants
///
/// * `User` - Message from the user
/// * `Assistant` - Response from Claude
/// * `ToolResult` - Result from a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Message from the user
    User {
        /// The text content
        content: String,
    },

    /// Response from Claude
    Assistant {
        /// The text content
        content: String,
    },

    /// Result from a tool execution
    ToolResult {
        /// Tool name
        tool_name: String,
        /// Tool result
        result: String,
    },
}

impl Message {
    /// Get the content text as a string (if applicable)
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Message::User { content } => Some(content),
            Message::Assistant { content } => Some(content),
            Message::ToolResult { .. } => None,
        }
    }

    /// Check if this is a user message
    pub fn is_user(&self) -> bool {
        matches!(self, Message::User { .. })
    }

    /// Check if this is an assistant message
    pub fn is_assistant(&self) -> bool {
        matches!(self, Message::Assistant { .. })
    }

    /// Check if this is a tool result
    pub fn is_tool_result(&self) -> bool {
        matches!(self, Message::ToolResult { .. })
    }
}

impl From<crate::types::messages::Message> for Message {
    fn from(msg: crate::types::messages::Message) -> Self {
        match msg {
            crate::types::messages::Message::User(user_msg) => {
                // Extract text from user message
                // UserMessage has text: Option<String> and content: Option<Vec<ContentBlock>>
                let content = if let Some(text) = user_msg.text {
                    text
                } else if let Some(blocks) = user_msg.content {
                    blocks
                        .iter()
                        .filter_map(|block| match block {
                            crate::types::messages::ContentBlock::Text(text) => Some(text.text.clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    String::new()
                };

                Message::User { content }
            }
            crate::types::messages::Message::Assistant(assist_msg) => {
                // Extract text from assistant message
                let content = assist_msg
                    .message
                    .content
                    .iter()
                    .filter_map(|block| match block {
                        crate::types::messages::ContentBlock::Text(text) => Some(text.text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                Message::Assistant { content }
            }
            _ => Message::Assistant {
                content: String::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_options_builder() {
        let options = SessionOptions::builder()
            .model("claude-sonnet-4-20250514".to_string())
            .max_turns(5)
            .build();

        assert_eq!(options.model, Some("claude-sonnet-4-20250514".to_string()));
        assert_eq!(options.max_turns, Some(5));
    }

    #[test]
    fn test_permission_mode_conversion() {
        let mode = PermissionMode::BypassPermissions;
        let converted: crate::types::config::PermissionMode = mode.into();
        assert!(matches!(
            converted,
            crate::types::config::PermissionMode::BypassPermissions
        ));
    }

    #[test]
    fn test_prompt_result_total_tokens() {
        let result = PromptResult {
            content: "Test".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            model: None,
            buffer_metrics: None,
        };

        assert_eq!(result.total_tokens(), 150);
    }

    #[test]
    fn test_message_is_user() {
        let msg = Message::User {
            content: "Hello".to_string(),
        };

        assert!(msg.is_user());
        assert!(!msg.is_assistant());
        assert_eq!(msg.as_text(), Some("Hello"));
    }

    #[test]
    fn test_message_is_assistant() {
        let msg = Message::Assistant {
            content: "Hi there!".to_string(),
        };

        assert!(!msg.is_user());
        assert!(msg.is_assistant());
        assert_eq!(msg.as_text(), Some("Hi there!"));
    }

    #[test]
    fn test_message_is_tool_result() {
        let msg = Message::ToolResult {
            tool_name: "calculator".to_string(),
            result: "42".to_string(),
        };

        assert!(msg.is_tool_result());
        assert!(!msg.is_user());
        assert!(!msg.is_assistant());
        assert_eq!(msg.as_text(), None);
    }

    #[test]
    fn test_prompt_result_cost_estimation() {
        let result = PromptResult {
            content: "Test".to_string(),
            input_tokens: 1_000_000, // 1M input tokens
            output_tokens: 1_000_000, // 1M output tokens
            model: None,
            buffer_metrics: None,
        };

        // 1M input * $3/M = $3
        // 1M output * $15/M = $15
        // Total = $18
        let cost = result.estimated_cost_usd();
        assert!((cost - 18.0).abs() < 0.01); // Allow small floating point error
    }
}
