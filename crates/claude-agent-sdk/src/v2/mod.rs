//! # Claude Agent SDK V2 API
//!
//! This module provides a simplified, TypeScript-inspired API for interacting with Claude.
//! The V2 API offers a more ergonomic interface compared to V1, with:
//!
//! - **One-shot prompts**: Simple `prompt()` function for single queries
//! - **Session-based API**: `create_session()` and `resume_session()` for multi-turn conversations
//! - **Simplified options**: `SessionOptions` with only the most commonly used parameters
//! - **TypeScript-style naming**: `prompt`, `send`, `receive` instead of `query`, `query_with_prompt`, `receive_response`
//!
//! ## Quick Start
//!
//! ```no_run
//! use claude_agent_sdk::v2::{prompt, create_session};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // One-shot prompt
//!     let result = prompt("What is 2 + 2?", Default::default()).await?;
//!     println!("Answer: {}", result.content);
//!
//!     // Session-based conversation
//!     let mut session = create_session(Default::default()).await?;
//!     session.send("Hello, Claude!").await?;
//!
//!     for message in session.receive().await? {
//!         println!("Message: {:?}", message);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## V1 vs V2 API Comparison
//!
//! ### V1 API (Current/Existing)
//! ```no_run
//! # use claude_agent_sdk::{query, ClaudeClient, ClaudeAgentOptions};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // One-shot
//! let messages = query("What is 2 + 2?", None).await?;
//!
//! // Session-based
//! let options = ClaudeAgentOptions::builder().build();
//! let mut client = ClaudeClient::new(options);
//! client.connect().await?;
//! client.query("Hello").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### V2 API (Simplified)
//! ```no_run
//! # use claude_agent_sdk::v2::{prompt, create_session};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // One-shot
//! let result = prompt("What is 2 + 2?", Default::default()).await?;
//!
//! // Session-based
//! let mut session = create_session(Default::default()).await?;
//! session.send("Hello").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Key Differences
//!
//! 1. **Simplified Options**: `SessionOptions` has fewer fields than `ClaudeAgentOptions`
//! 2. **Direct Results**: `prompt()` returns `PromptResult` directly, not a stream
//! 3. **TypeScript Naming**: `send/receive` instead of `query/receive_response`
//! 4. **No Connect**: Sessions are auto-connected on creation
//!
//! ## Migration Guide
//!
//! ### From V1 to V2
//!
//! **One-shot queries**:
//! ```rust,ignore
//! // V1
//! let messages = query("Question", None).await?;
//! for msg in messages {
//!     if let Message::Assistant(assist_msg) = msg {
//!         // process...
//!     }
//! }
//!
//! // V2
//! let result = prompt("Question", Default::default()).await?;
//! // result.content has the answer text
//! ```
//!
//! **Session-based**:
//! ```rust,ignore
//! // V1
//! let mut client = ClaudeClient::new(options);
//! client.connect().await?;
//! client.query("Hello").await?;
//! let stream = client.receive_response();
//!
//! // V2
//! let mut session = create_session(Default::default()).await?;
//! session.send("Hello").await?;
//! let messages = session.receive().await?;
//! ```
//!
//! ## Feature Parity
//!
//! V2 provides the same functionality as V1, just with a simpler API. For advanced use cases,
//! you can still use the V1 API directly.
//!
//! - ✅ One-shot queries
//! - ✅ Multi-turn sessions
//! - ✅ Streaming responses
//! - ✅ Permission management
//! - ✅ Cost control
//! - ✅ Custom tools
//! - ✅ Hooks
//! - ✅ Session resumption

mod session;
mod types;

pub use session::{create_session, resume_session, Session};
pub use types::{Message, PermissionMode, PromptResult, SessionOptions};

use crate::errors::Result;
use crate::types::config::ClaudeAgentOptions;
use crate::client::ClaudeClient;

/// One-shot prompt API - simplified interface for single queries
///
/// This function sends a single prompt to Claude and returns the complete response.
/// It's the simplest way to interact with Claude for one-off queries.
///
/// # Arguments
///
/// * `prompt` - The prompt text to send to Claude
/// * `options` - Session options (use `Default::default()` for defaults)
///
/// # Returns
///
/// A `PromptResult` containing the response text and metadata
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk::v2::prompt;
///
/// #[tokio::main]
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let result = prompt("What is 2 + 2?", Default::default()).await?;
///     println!("Claude says: {}", result.content);
///     println!("Used {} tokens", result.input_tokens + result.output_tokens);
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The prompt is empty
/// - Network connection fails
/// - API returns an error
/// - Response parsing fails
pub async fn prompt(
    prompt: impl Into<String>,
    options: SessionOptions,
) -> Result<PromptResult> {
    let prompt_text = prompt.into();
    let opts: ClaudeAgentOptions = options.into();

    // Create client and send query
    let mut client = ClaudeClient::new(opts);
    client.connect().await?;

    client.query(&prompt_text).await?;

    // Collect all messages
    let mut content = String::new();
    let mut input_tokens = 0;
    let mut output_tokens = 0;
    let mut stream = client.receive_response();

    use futures::StreamExt;
    while let Some(result) = stream.next().await {
        let message = result?;

        match message {
            crate::types::messages::Message::Assistant(assist_msg) => {
                // Extract text content
                for block in &assist_msg.message.content {
                    if let crate::types::messages::ContentBlock::Text(text_block) = block {
                        content.push_str(&text_block.text);
                    }
                }

                // Track token usage if available
                if let Some(usage) = &assist_msg.message.usage {
                    // Usage is a JSON value, parse it to extract token counts
                    if let Some(input) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                        input_tokens = input;
                    }
                    if let Some(output) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                        output_tokens = output;
                    }
                }
            }
            crate::types::messages::Message::Result(_) => {
                // End of conversation
                break;
            }
            _ => {
                // Ignore other message types for one-shot API
            }
        }
    }

    Ok(PromptResult {
        content,
        input_tokens,
        output_tokens,
        model: None, // Model info not available from current client implementation
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are placeholder tests that verify the API structure
    // Integration tests would require actual Claude API access

    #[test]
    fn test_prompt_result_structure() {
        let result = PromptResult {
            content: "Test response".to_string(),
            input_tokens: 10,
            output_tokens: 20,
            model: Some("claude-sonnet-4-20250514".to_string()),
        };

        assert_eq!(result.content, "Test response");
        assert_eq!(result.total_tokens(), 30);
    }

    #[test]
    fn test_session_options_default() {
        let options = SessionOptions::default();
        // Verify default options are created successfully
        assert!(options.model.is_none()); // Default to None (uses system default)
    }

    #[test]
    fn test_session_options_builder() {
        let options = SessionOptions::builder()
            .model("claude-sonnet-4-20250514".to_string())
            .max_turns(5)
            .build();

        assert_eq!(options.model, Some("claude-sonnet-4-20250514".to_string()));
        assert_eq!(options.max_turns, Some(5));
    }
}
