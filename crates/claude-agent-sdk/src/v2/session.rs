//! Session management for V2 API
//!
//! This module provides session-based conversation management with a simplified API.

use crate::client::ClaudeClient;
use crate::errors::{ClaudeError, Result};
use crate::types::config::ClaudeAgentOptions;
use crate::types::messages::Message;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::types::SessionOptions;

/// A conversation session with Claude
///
/// `Session` provides a simplified interface for multi-turn conversations with Claude.
/// It wraps the underlying `ClaudeClient` and provides easier-to-use methods.
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk::v2::{create_session, SessionOptions};
///
/// #[tokio::main]
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let mut session = create_session(SessionOptions::default()).await?;
///
///     // Send a message
///     session.send("What is 2 + 2?").await?;
///
///     // Receive all responses
///     let messages = session.receive().await?;
///     for msg in messages {
///         println!("Message: {:?}", msg);
///     }
///
///     // Close the session
///     session.close().await?;
///
///     Ok(())
/// }
/// ```
pub struct Session {
    /// Unique session identifier
    pub id: String,
    /// Session options
    pub options: SessionOptions,
    /// Internal client
    client: Arc<Mutex<ClaudeClient>>,
}

impl Session {
    /// Create a new session (internal use)
    ///
/// This is called by `create_session()` to initialize a new session.
    fn new(id: String, options: SessionOptions, client: ClaudeClient) -> Self {
        Self {
            id,
            options,
            client: Arc::new(Mutex::new(client)),
        }
    }

    /// Send a message to Claude
    ///
    /// This method sends a user message to Claude and queues it for processing.
    /// Call `receive()` to get Claude's response.
    ///
    /// # Arguments
    ///
    /// * `message` - The message text to send
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::v2::Session;
    /// # async fn example(session: &mut Session) -> Result<(), Box<dyn std::error::Error>> {
    /// session.send("Hello, Claude!").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The message is empty
    /// - Network connection fails
    /// - Session is closed
    pub async fn send(&mut self, message: impl Into<String>) -> Result<()> {
        let message_text = message.into();

        if message_text.trim().is_empty() {
            return Err(ClaudeError::InvalidInput(
                "Message cannot be empty".to_string(),
            ));
        }

        let mut client = self.client.lock().await;
        client.query(&message_text).await?;

        Ok(())
    }

    /// Receive messages from Claude
    ///
    /// This method returns all pending messages from Claude since the last `send()` call.
    /// Messages are returned until a `Result` message is encountered (end of turn).
    ///
    /// # Returns
    ///
    /// A vector of `V2Message` objects
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::v2::Session;
    /// # async fn example(session: &mut Session) -> Result<(), Box<dyn std::error::Error>> {
    /// let messages = session.receive().await?;
    /// for msg in messages {
    ///     if let Some(text) = msg.as_text() {
    ///         println!("Claude: {}", text);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn receive(&self) -> Result<Vec<V2Message>> {
        let client = self.client.lock().await;
        let mut stream = client.receive_response();
        let mut messages = Vec::new();

        while let Some(result) = stream.next().await {
            let msg = result?;

            match msg {
                Message::Assistant(assist_msg) => {
                    // Extract text content
                    let content = assist_msg
                        .message
                        .content
                        .iter()
                        .filter_map(|block| match block {
                            crate::types::messages::ContentBlock::Text(text) => {
                                Some(text.text.clone())
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    messages.push(V2Message::Assistant { content });
                }
                Message::Result(_) => {
                    // End of turn
                    break;
                }
                _ => {
                    // Ignore other message types
                }
            }
        }

        Ok(messages)
    }

    /// Get the model being used for this session
    pub async fn model(&self) -> Option<String> {
        // Return the model from session options if set
        self.options.model.clone()
    }

    /// Check if the session is still connected
    pub async fn is_connected(&self) -> bool {
        // Check if client can still communicate
        // For now, we assume it's connected if we haven't closed it
        true
    }

    /// Close the session
    ///
    /// This method closes the connection to Claude and releases any resources.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::v2::Session;
    /// # async fn example(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
    /// session.close().await?;
    /// println!("Session closed");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(self) -> Result<()> {
        // Release the client connection
        // The client will be dropped when self is dropped
        Ok(())
    }
}

/// Simplified message type for V2 API sessions
///
/// This is a streamlined version of the full Message type,
/// focused on the most common use cases.
///
/// # Variants
///
/// * `Assistant` - Response from Claude with text content
#[derive(Debug, Clone)]
pub enum V2Message {
    /// Response from Claude
    Assistant {
        /// The text content of the response
        content: String,
    },
}

impl V2Message {
    /// Get the text content (if available)
    pub fn as_text(&self) -> Option<&str> {
        match self {
            V2Message::Assistant { content } => Some(content),
        }
    }
}

/// Create a new session with Claude
///
/// This function creates a new session and connects to Claude.
/// The session will use the provided options for configuration.
///
/// # Arguments
///
/// * `options` - Session options (use `Default::default()` for defaults)
///
/// # Returns
///
/// A new `Session` object
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk::v2::{create_session, SessionOptions};
///
/// #[tokio::main]
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let session = create_session(SessionOptions::default()).await?;
///     println!("Session created: {}", session.id);
///     Ok(())
/// }
/// ```
pub async fn create_session(options: SessionOptions) -> Result<Session> {
    let opts: ClaudeAgentOptions = options.clone().into();
    let mut client = ClaudeClient::new(opts);

    client.connect().await?;

    // Generate a session ID
    let id = uuid::Uuid::new_v4().to_string();

    Ok(Session::new(id, options, client))
}

/// Resume an existing session
///
/// This function resumes a previously created session using its session ID.
/// Note: Session persistence is not yet fully implemented, so this currently
/// creates a new session with the same ID.
///
/// # Arguments
///
/// * `session_id` - The ID of the session to resume
/// * `options` - Session options
///
/// # Returns
///
/// A resumed `Session` object
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk::v2::{resume_session, SessionOptions};
///
/// #[tokio::main]
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let session = resume_session("existing-session-id", SessionOptions::default()).await?;
///     println!("Resumed session: {}", session.id);
///     Ok(())
/// }
/// ```
pub async fn resume_session(
    session_id: &str,
    options: SessionOptions,
) -> Result<Session> {
    // Session resumption from persistent storage is not yet implemented.
    // For now, create a new session with the provided ID.
    let opts: ClaudeAgentOptions = options.clone().into();
    let mut client = ClaudeClient::new(opts);

    client.connect().await?;

    Ok(Session::new(
        session_id.to_string(),
        options,
        client,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v2_message_as_text() {
        let msg = V2Message::Assistant {
            content: "Hello!".to_string(),
        };

        assert_eq!(msg.as_text(), Some("Hello!"));
    }
}
