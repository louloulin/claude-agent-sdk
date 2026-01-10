// local_llm.rs - Local LLM integration with Ollama
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;

/// Local LLM provider configuration
#[derive(Debug, Clone)]
pub struct LocalLLMConfig {
    /// Ollama API endpoint
    pub endpoint: String,
    /// Model name
    pub model: String,
    /// Timeout for generation
    pub timeout: Duration,
    /// Temperature (0.0 - 1.0)
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: usize,
}

impl Default for LocalLLMConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434/api/generate".to_string(),
            model: "llama3.1".to_string(),
            timeout: Duration::from_secs(60),
            temperature: 0.7,
            max_tokens: 2000,
        }
    }
}

/// Ollama generation request
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

/// Ollama generation options
#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: usize,
}

/// Ollama generation response
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    model: String,
    response: String,
    context: Option<Vec<usize>>,
    total_duration: Option<u64>,
    load_duration: Option<u64>,
    prompt_eval_count: Option<usize>,
    eval_count: Option<usize>,
}

/// FinBERT sentiment analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    pub label: String,      // "positive", "negative", "neutral"
    pub score: f64,        // Confidence score
    pub probabilities: serde_json::Value,
}

/// Local LLM client for Ollama
pub struct LocalLLMClient {
    config: LocalLLMConfig,
    client: Client,
}

impl LocalLLMClient {
    /// Create a new local LLM client
    pub fn new(config: LocalLLMConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap();

        Self { config, client }
    }

    /// Check if Ollama is available
    pub async fn check_health(&self) -> Result<bool> {
        let url = self.config.endpoint.replace("/api/generate", "/api/tags");

        match timeout(Duration::from_secs(5), self.client.get(&url).send()).await {
            Ok(Ok(response)) => Ok(response.status().is_success()),
            _ => Ok(false),
        }
    }

    /// Generate text with local LLM
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaOptions {
                temperature: self.config.temperature,
                num_predict: self.config.max_tokens,
            },
        };

        let response = self
            .client
            .post(&self.config.endpoint)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("LLM request failed: {}", response.status()));
        }

        let ollama_response: OllamaResponse = response.json().await?;

        Ok(ollama_response.response)
    }

    /// Analyze sentiment using local LLM (FinBERT simulation)
    pub async fn analyze_sentiment(&self, text: &str) -> Result<SentimentResult> {
        let prompt = format!(
            "Analyze the sentiment of the following financial text. \
             Respond with ONLY one word: positive, negative, or neutral.\n\nText: {}",
            text
        );

        let response = self.generate(&prompt).await?;

        let response_lower = response.to_lowercase();
        let label = if response_lower.contains("positive") {
            "positive".to_string()
        } else if response_lower.contains("negative") {
            "negative".to_string()
        } else {
            "neutral".to_string()
        };

        // Simulate confidence score based on response clarity
        let score = if response.len() > 50 {
            0.5 // Uncertain
        } else {
            0.8 // Confident
        };

        Ok(SentimentResult {
            label,
            score,
            probabilities: serde_json::json!({
                "positive": if label == "positive" { score } else { (1.0 - score) / 2.0 },
                "negative": if label == "negative" { score } else { (1.0 - score) / 2.0 },
                "neutral": if label == "neutral" { score } else { (1.0 - score) / 2.0 },
            }),
        })
    }

    /// Generate investment analysis with local LLM
    pub async fn analyze_investment(&self, ticker: &str) -> Result<String> {
        let prompt = format!(
            "Provide a brief investment analysis for {}. Include:\n\
             1. Current trend (bullish/bearish/neutral)\n\
             2. Key support and resistance levels\n\
             3. Overall recommendation (Buy/Hold/Sell)\n\
             Keep your response under 200 words.",
            ticker
        );

        self.generate(&prompt).await
    }

    /// Summarize financial news
    pub async fn summarize_news(&self, news_text: &str) -> Result<String> {
        let prompt = format!(
            "Summarize the following financial news in 2-3 sentences. \
             Focus on key information that could impact investment decisions.\n\nNews: {}",
            news_text
        );

        self.generate(&prompt).await
    }

    /// Generate portfolio insights
    pub async fn generate_portfolio_insights(
        &self,
        portfolio: &str, // JSON string
    ) -> Result<String> {
        let prompt = format!(
            "Analyze the following portfolio and provide 3 key insights:\n1. Risk assessment\n2. Diversification analysis\n3. Improvement suggestions\n\nPortfolio: {}",
            portfolio
        );

        self.generate(&prompt).await
    }

    /// Get model information
    pub async fn get_model_info(&self) -> Result<ModelInfo> {
        let url = self.config.endpoint.replace("/api/generate", "/api/tags");

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get model info"));
        }

        #[derive(Deserialize)]
        struct TagsResponse {
            models: Vec<ModelInfo>,
        }

        let tags: TagsResponse = response.json().await?;

        tags.models
            .into_iter()
            .find(|m| m.name == self.config.model)
            .ok_or_else(|| anyhow!("Model {} not found", self.config.model))
    }
}

/// Model information
#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: Option<u64>,
    pub modified_at: Option<String>,
}

/// FinBERT client for specialized financial sentiment analysis
pub struct FinBERTClient {
    llm_client: LocalLLMClient,
}

impl FinBERTClient {
    /// Create a new FinBERT client
    pub fn new(config: LocalLLMConfig) -> Self {
        let mut finbert_config = config;
        finbert_config.model = "finbert".to_string(); // Assumes finbert is available in Ollama

        Self {
            llm_client: LocalLLMClient::new(finbert_config),
        }
    }

    /// Analyze sentiment with FinBERT (local or remote)
    pub async fn analyze(&self, text: &str) -> Result<SentimentResult> {
        // Try local FinBERT first
        match self.llm_client.check_health().await {
            Ok(true) => {
                // Use local FinBERT model via Ollama
                self.llm_client.analyze_sentiment(text).await
            }
            _ => {
                // Fallback to Hugging Face API
                self.analyze_with_huggingface(text).await
            }
        }
    }

    /// Analyze sentiment using Hugging Face API (fallback)
    async fn analyze_with_huggingface(&self, text: &str) -> Result<SentimentResult> {
        let client = Client::new();
        let url = "https://api-inference.huggingface.co/models/ProsusAI/finbert";

        let response = client
            .post(url)
            .header("Authorization", "Bearer YOUR_API_KEY") // User would need to provide this
            .json(&serde_json::json!({ "inputs": text }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Hugging Face API request failed"));
        }

        #[derive(Deserialize)]
        struct HFResponse {
            #[serde(rename = "0")]
            label: String,
            #[serde(rename = "1")]
            score: f64,
        }

        let hf_response: Vec<HFResponse> = response.json().await?;

        let result = hf_response
            .into_iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
            .ok_or_else(|| anyhow!("No sentiment results"))?;

        Ok(SentimentResult {
            label: result.label,
            score: result.score,
            probabilities: serde_json::json!({}),
        })
    }

    /// Batch analyze multiple texts
    pub async fn batch_analyze(&self, texts: &[String]) -> Result<Vec<SentimentResult>> {
        let mut results = Vec::new();

        for text in texts {
            let result = self.analyze(text).await?;
            results.push(result);
        }

        Ok(results)
    }
}

/// LLM router for intelligent model selection
pub struct LLMRouter {
    local_llm: LocalLLMClient,
    claude_api_key: Option<String>,
    use_local_first: bool,
}

impl LLMRouter {
    /// Create a new LLM router
    pub fn new(local_llm: LocalLLMClient, claude_api_key: Option<String>) -> Self {
        Self {
            local_llm,
            claude_api_key,
            use_local_first: true,
        }
    }

    /// Route request to appropriate LLM
    pub async fn route_request(&self, prompt: &str, prefer_local: bool) -> Result<String> {
        // Try local first if preferred
        if prefer_local || self.use_local_first {
            match self.local_llm.check_health().await {
                Ok(true) => {
                    return self.local_llm.generate(prompt).await;
                }
                _ => {
                    // Fall back to Claude API if available
                    if let Some(_key) = &self.claude_api_key {
                        // Use Claude API
                        return Err(anyhow!("Claude API fallback not implemented yet"));
                    }
                }
            }
        }

        // Use Claude API if local not available or not preferred
        if let Some(_key) = &self.claude_api_key {
            return Err(anyhow!("Claude API not implemented yet"));
        }

        Err(anyhow!("No LLM available"))
    }

    /// Analyze with automatic model selection
    pub async fn smart_analyze(&self, task_type: &str, input: &str) -> Result<String> {
        // For complex analysis, prefer Claude API
        // For simple tasks, use local LLM
        let prefer_local = match task_type {
            "sentiment" | "summarize" => true,
            "complex_analysis" | "strategy" => false,
            _ => true,
        };

        let prompt = match task_type {
            "sentiment" => format!("Analyze sentiment: {}", input),
            "summarize" => format!("Summarize: {}", input),
            "complex_analysis" => format!("Analyze in detail: {}", input),
            _ => input.to_string(),
        };

        self.route_request(&prompt, prefer_local).await
    }

    /// Set local-first preference
    pub fn set_local_first(&mut self, local_first: bool) {
        self.use_local_first = local_first;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_llm_client() {
        let config = LocalLLMConfig::default();
        let client = LocalLLMClient::new(config);

        // Check if Ollama is running
        match client.check_health().await {
            Ok(true) => {
                println!("✅ Ollama is available");

                // Test generation
                match client.generate("Say hello in one sentence.").await {
                    Ok(response) => {
                        println!("Generated: {}", response);
                    }
                    Err(e) => {
                        println!("Generation failed: {}", e);
                    }
                }
            }
            Ok(false) => {
                println!("⚠️  Ollama is not available (this is expected if not installed)");
            }
            Err(e) => {
                println!("Health check error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_finbert_client() {
        let config = LocalLLMConfig::default();
        let client = FinBERTClient::new(config);

        // Test sentiment analysis
        let result = client
            .analyze("Apple reports record quarterly earnings, stock surges 5%.")
            .await;

        match result {
            Ok(sentiment) => {
                println!("Sentiment: {} (confidence: {:.2})", sentiment.label, sentiment.score);
            }
            Err(e) => {
                println!("Sentiment analysis failed: {}", e);
            }
        }
    }
}
