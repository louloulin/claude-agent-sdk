//! Transport trait definition

use async_trait::async_trait;
use futures::stream::Stream;
use std::pin::Pin;

use crate::errors::Result;

/// Transport trait for communicating with Claude Code CLI
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect the transport
    async fn connect(&mut self) -> Result<()>;

    /// Write raw data to the transport
    async fn write(&mut self, data: &str) -> Result<()>;

    /// Read messages as a stream of JSON values
    ///
    /// This is the traditional method that returns parsed `serde_json::Value`.
    /// For better performance, consider using [`read_raw_messages`] with zero-copy parsing.
    fn read_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>>;

    /// Read messages as a stream of raw JSON strings
    ///
    /// This method returns raw JSON lines for use with zero-copy parsing.
    /// Use [`crate::internal::message_parser::ZeroCopyMessageParser::parse`] to parse
    /// the raw strings into `Message` types.
    ///
    /// # Performance
    ///
    /// This method avoids allocating intermediate `serde_json::Value` objects,
    /// reducing memory usage by ~30-50% for large messages.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use claude_agent_sdk::internal::message_parser::ZeroCopyMessageParser;
    ///
    /// let mut stream = transport.read_raw_messages();
    /// while let Some(result) = stream.next().await {
    ///     let json = result?;
    ///     let message = ZeroCopyMessageParser::parse(&json)?;
    ///     // Process message
    /// }
    /// ```
    fn read_raw_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send + '_>>;

    /// Close the transport
    async fn close(&mut self) -> Result<()>;

    /// Check if the transport is ready
    #[allow(dead_code)]
    fn is_ready(&self) -> bool;

    /// End input stream (close stdin)
    async fn end_input(&mut self) -> Result<()>;
}
