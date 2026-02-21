//! Internal client implementation

use futures::stream::StreamExt;

use crate::errors::Result;
use crate::types::config::ClaudeAgentOptions;
use crate::types::messages::Message;

use super::message_parser::{MessageParser, ParsingMode, parse_with_mode};
use super::transport::subprocess::QueryPrompt;
use super::transport::{SubprocessTransport, Transport};

/// Internal client for processing queries
pub struct InternalClient {
    transport: SubprocessTransport,
    parsing_mode: ParsingMode,
}

impl InternalClient {
    /// Create a new client
    pub fn new(prompt: QueryPrompt, options: ClaudeAgentOptions) -> Result<Self> {
        let parsing_mode = options.parsing_mode;
        let transport = SubprocessTransport::new(prompt, options)?;
        Ok(Self { transport, parsing_mode })
    }

    /// Connect and get messages
    pub async fn execute(mut self) -> Result<Vec<Message>> {
        // Connect
        self.transport.connect().await?;

        // Collect all messages using the configured parsing mode
        let mut messages = Vec::new();

        match self.parsing_mode {
            ParsingMode::ZeroCopy => {
                // Use zero-copy parsing (raw strings)
                let mut stream = self.transport.read_raw_messages();
                while let Some(result) = stream.next().await {
                    let json = result?;
                    let message = parse_with_mode(&json, ParsingMode::ZeroCopy)?;
                    messages.push(message);
                }
            }
            ParsingMode::Traditional => {
                // Use traditional parsing (serde_json::Value)
                let mut stream = self.transport.read_messages();
                while let Some(result) = stream.next().await {
                    let json = result?;
                    let message = MessageParser::parse(json)?;
                    messages.push(message);
                }
            }
        }

        // Close transport
        self.transport.close().await?;

        Ok(messages)
    }
}
