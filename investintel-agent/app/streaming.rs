// streaming.rs - Advanced streaming API for real-time investment analysis
use anyhow::Result;
use claude_agent_sdk_rs::{
    query_stream, ContentBlock, ClaudeAgentOptions, Message, PermissionMode,
};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Streaming analysis request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingAnalysisRequest {
    pub query: String,
    pub tickers: Vec<String>,
    pub analysis_types: Vec<AnalysisType>,
    pub max_turns: Option<usize>,
}

/// Types of analysis to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    Technical,
    Fundamental,
    Sentiment,
    Risk,
    All,
}

/// Streaming analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingAnalysisResult {
    pub ticker: String,
    pub analysis_type: AnalysisType,
    pub content: String,
    pub confidence: f64,
    pub metadata: HashMap<String, String>,
}

/// Streaming message event
#[derive(Debug, Clone)]
pub enum StreamingEvent {
    /// Text content from Claude
    Text(String),
    /// Tool being executed
    ToolUse { name: String, input: serde_json::Value },
    /// Tool result received
    ToolResult { tool_id: String, content: String },
    /// Thinking process (extended thinking)
    Thinking { text: String, signature: String },
    /// Analysis complete for a ticker
    AnalysisComplete(StreamingAnalysisResult),
    /// Error occurred
    Error(String),
    /// Stream complete
    Complete,
}

/// Investment streaming analyzer
pub struct InvestmentStreamingAnalyzer {
    options: ClaudeAgentOptions,
}

impl InvestmentStreamingAnalyzer {
    /// Create a new streaming analyzer
    pub fn new() -> Self {
        let options = ClaudeAgentOptions::builder()
            .permission_mode(PermissionMode::BypassPermissions)
            .max_turns(5)
            .build();

        Self { options }
    }

    /// Analyze a single ticker with streaming updates
    pub async fn analyze_ticker_stream(
        &self,
        ticker: &str,
        analysis_types: Vec<AnalysisType>,
    ) -> Result<impl futures::Stream<Item = StreamingEvent>> {
        let prompt = self.build_analysis_prompt(ticker, &analysis_types);

        let stream = query_stream(prompt, Some(self.options.clone())).await?;

        Ok(async_stream::stream! {
            let mut stream = stream;
            let current_ticker = ticker.to_string();
            let mut current_content = String::new();
            let mut current_analysis_type = AnalysisType::All;

            while let Some(result) = stream.next().await {
                match result {
                    Ok(message) => match message {
                        Message::Assistant(msg) => {
                            for block in &msg.message.content {
                                match block {
                                    ContentBlock::Text(text) => {
                                        current_content.push_str(&text.text);
                                        yield StreamingEvent::Text(text.text.clone());
                                    }
                                    ContentBlock::ToolUse(tool) => {
                                        yield StreamingEvent::ToolUse {
                                            name: tool.name.clone(),
                                            input: tool.input.clone(),
                                        };
                                    }
                                    ContentBlock::ToolResult(result) => {
                                        let content = match &result.content {
                                            Some(content) => match content {
                                                claude_agent_sdk_rs::ToolResultContent::Text(t) => t.clone(),
                                                claude_agent_sdk_rs::ToolResultContent::Blocks(b) => {
                                                    format!("{} blocks", b.len())
                                                }
                                            },
                                            None => "No content".to_string(),
                                        };
                                        yield StreamingEvent::ToolResult {
                                            tool_id: result.tool_use_id.clone(),
                                            content,
                                        };
                                    }
                                    ContentBlock::Thinking(thinking) => {
                                        yield StreamingEvent::Thinking {
                                            text: thinking.thinking.clone(),
                                            signature: thinking.signature.clone(),
                                        };
                                    }
                                    ContentBlock::Image(_) => {
                                        // Ignore images for now
                                    }
                                }
                            }
                        }
                        Message::Result(_) => {
                            // Analysis complete
                            yield StreamingEvent::AnalysisComplete(
                                StreamingAnalysisResult {
                                    ticker: current_ticker.clone(),
                                    analysis_type: current_analysis_type.clone(),
                                    content: current_content.clone(),
                                    confidence: 0.85,
                                    metadata: HashMap::new(),
                                }
                            );
                            yield StreamingEvent::Complete;
                        }
                        _ => {}
                    },
                    Err(e) => {
                        yield StreamingEvent::Error(e.to_string());
                    }
                }
            }
        })
    }

    /// Analyze multiple tickers in parallel with streaming
    pub async fn analyze_multiple_stream(
        &self,
        request: StreamingAnalysisRequest,
    ) -> Result<impl futures::Stream<Item = StreamingEvent>> {
        let tickers = request.tickers.clone();
        let analysis_types = request.analysis_types.clone();
        let analyzers: Vec<_> = tickers
            .iter()
            .map(|ticker| self.analyze_ticker_stream(ticker, analysis_types.clone()))
            .collect();

        // Use join_all to process all streams concurrently
        let streams = futures::future::join_all(analyzers).await?;

        Ok(async_stream::stream! {
            for mut stream in streams {
                while let Some(event) = stream.next().await {
                    yield event;
                }
            }
        })
    }

    /// Real-time market monitoring with periodic updates
    pub async fn monitor_market_stream(
        &self,
        tickers: Vec<String>,
        interval: Duration,
    ) -> Result<impl futures::Stream<Item = StreamingEvent>> {
        Ok(async_stream::stream! {
            loop {
                for ticker in &tickers {
                    let mut stream = self.analyze_ticker_stream(
                        ticker,
                        vec![AnalysisType::Technical]
                    ).await.unwrap();

                    while let Some(event) = stream.next().await {
                        match event {
                            StreamingEvent::AnalysisComplete(result) => {
                                yield StreamingEvent::AnalysisComplete(result);
                            }
                            StreamingEvent::Error(e) => {
                                yield StreamingEvent::Error(e);
                            }
                            _ => {}
                        }
                    }
                }
                tokio::time::sleep(interval).await;
            }
        })
    }

    /// Build analysis prompt for a ticker
    fn build_analysis_prompt(&self, ticker: &str, analysis_types: &[AnalysisType]) -> String {
        let types_str = analysis_types
            .iter()
            .map(|t| match t {
                AnalysisType::Technical => "technical analysis",
                AnalysisType::Fundamental => "fundamental analysis",
                AnalysisType::Sentiment => "sentiment analysis",
                AnalysisType::Risk => "risk analysis",
                AnalysisType::All => "comprehensive analysis",
            })
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "Provide a detailed {} for {}. Include:\n\
             1. Current price and trend\n\
             2. Key technical indicators (RSI, MACD, MA)\n\
             3. Support and resistance levels\n\
             4. Overall recommendation (Buy/Hold/Sell)\n\
             5. Confidence level in your analysis",
            types_str, ticker
        )
    }
}

impl Default for InvestmentStreamingAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Example usage of streaming analyzer
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_streaming_analyzer() {
        let analyzer = InvestmentStreamingAnalyzer::new();
        let mut stream = analyzer
            .analyze_ticker_stream("AAPL", vec![AnalysisType::Technical])
            .await
            .unwrap();

        let mut event_count = 0;
        while let Some(event) = stream.next().await {
            match event {
                StreamingEvent::Text(text) => {
                    println!("Text: {}", text);
                    event_count += 1;
                }
                StreamingEvent::ToolUse { name, .. } => {
                    println!("Tool: {}", name);
                }
                StreamingEvent::AnalysisComplete(result) => {
                    println!("Analysis complete for {}: {}", result.ticker, result.content);
                }
                StreamingEvent::Complete => {
                    println!("Stream complete");
                    break;
                }
                _ => {}
            }
        }

        assert!(event_count > 0);
    }
}
