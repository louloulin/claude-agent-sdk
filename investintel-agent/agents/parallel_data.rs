//! 并行数据获取优化
//!
//! 提供并行获取多个数据源的能力，显著提升性能
//!
//! ## 核心功能
//!
//! 1. **并行报价获取** - 同时获取多个股票的实时报价
//! 2. **并行基本面获取** - 同时获取多个股票的基本面数据
//! 3. **智能聚合** - 合并来自不同数据源的结果
//! 4. **错误容忍** - 部分失败不影响整体结果

use crate::agents::{StockQuote, FundamentalData, MarketDataProvider};
use anyhow::{Context, Result};
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// 并行数据获取器
pub struct ParallelDataFetcher {
    /// 市场数据提供者
    provider: Arc<MarketDataProvider>,

    /// 最大并发数
    max_concurrent: usize,

    /// 信号量（用于控制并发）
    semaphore: Arc<Semaphore>,
}

impl ParallelDataFetcher {
    /// 创建新的并行数据获取器
    pub fn new(provider: Arc<MarketDataProvider>) -> Self {
        let max_concurrent = 10; // 默认最大10个并发请求

        Self {
            provider,
            max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    /// 设置最大并发数
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self.semaphore = Arc::new(Semaphore::new(max));
        self
    }

    /// 并行获取多个股票报价
    ///
    /// 返回: HashMap<symbol, Result<StockQuote>>
    pub async fn fetch_quotes_parallel(&self, symbols: &[&str]) -> HashMap<String, Result<StockQuote>> {
        let futures = symbols.iter().map(|symbol| {
            let provider = self.provider.clone();
            let semaphore = self.semaphore.clone();
            let symbol = symbol.to_string();

            async move {
                let _permit = semaphore.acquire().await.unwrap();
                let result = provider.get_quote(&symbol).await;
                (symbol, result)
            }
        });

        let results = join_all(futures).await;
        results.into_iter().collect()
    }

    /// 并行获取多个股票的基本面数据
    ///
    /// 返回: HashMap<symbol, Result<FundamentalData>>
    pub async fn fetch_fundamentals_parallel(&self, symbols: &[&str]) -> HashMap<String, Result<FundamentalData>> {
        let futures = symbols.iter().map(|symbol| {
            let provider = self.provider.clone();
            let semaphore = self.semaphore.clone();
            let symbol = symbol.to_string();

            async move {
                let _permit = semaphore.acquire().await.unwrap();
                let result = provider.get_fundamental(&symbol).await;
                (symbol, result)
            }
        });

        let results = join_all(futures).await;
        results.into_iter().collect()
    }

    /// 并行获取报价和基本面数据
    ///
    /// 返回: (quotes, fundamentals)
    pub async fn fetch_all_parallel(&self, symbols: &[&str]) -> (
        HashMap<String, Result<StockQuote>>,
        HashMap<String, Result<FundamentalData>>
    ) {
        let quotes_future = self.fetch_quotes_parallel(symbols);
        let fundamentals_future = self.fetch_fundamentals_parallel(symbols);

        let (quotes, fundamentals) = tokio::join!(quotes_future, fundamentals_future);
        (quotes, fundamentals)
    }

    /// 获取成功的报价（过滤掉失败的）
    pub async fn fetch_quotes_successful(&self, symbols: &[&str]) -> HashMap<String, StockQuote> {
        let all_results = self.fetch_quotes_parallel(symbols).await;

        all_results
            .into_iter()
            .filter_map(|(symbol, result)| result.ok().map(|quote| (symbol, quote)))
            .collect()
    }

    /// 获取成功的基本面数据（过滤掉失败的）
    pub async fn fetch_fundamentals_successful(&self, symbols: &[&str]) -> HashMap<String, FundamentalData> {
        let all_results = self.fetch_fundamentals_parallel(symbols).await;

        all_results
            .into_iter()
            .filter_map(|(symbol, result)| result.ok().map(|data| (symbol, data)))
            .collect()
    }

    /// 批量获取并返回统计信息
    pub async fn fetch_with_stats(&self, symbols: &[&str]) -> FetchStats {
        let start = std::time::Instant::now();

        let (quotes, fundamentals) = self.fetch_all_parallel(symbols).await;

        let elapsed = start.elapsed();

        let successful_quotes = quotes.values().filter(|r| r.is_ok()).count();
        let successful_fundamentals = fundamentals.values().filter(|r| r.is_ok()).count();

        FetchStats {
            total_symbols: symbols.len(),
            successful_quotes,
            successful_fundamentals,
            failed_quotes: symbols.len() - successful_quotes,
            failed_fundamentals: symbols.len() - successful_fundamentals,
            elapsed_ms: elapsed.as_millis(),
        }
    }
}

/// 获取统计信息
#[derive(Debug, Clone)]
pub struct FetchStats {
    /// 总股票数
    pub total_symbols: usize,

    /// 成功获取报价数
    pub successful_quotes: usize,

    /// 成功获取基本面数
    pub successful_fundamentals: usize,

    /// 失败获取报价数
    pub failed_quotes: usize,

    /// 失败获取基本面数
    pub failed_fundamentals: usize,

    /// 耗时（毫秒）
    pub elapsed_ms: u128,
}

impl FetchStats {
    /// 报价成功率
    pub fn quote_success_rate(&self) -> f64 {
        if self.total_symbols == 0 {
            return 0.0;
        }
        self.successful_quotes as f64 / self.total_symbols as f64
    }

    /// 基本面数据成功率
    pub fn fundamental_success_rate(&self) -> f64 {
        if self.total_symbols == 0 {
            return 0.0;
        }
        self.successful_fundamentals as f64 / self.total_symbols as f64
    }

    /// 平均每只股票耗时（毫秒）
    pub fn avg_ms_per_symbol(&self) -> f64 {
        if self.total_symbols == 0 {
            return 0.0;
        }
        self.elapsed_ms as f64 / self.total_symbols as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_fetcher_creation() {
        let provider = Arc::new(MarketDataProvider::new());
        let fetcher = ParallelDataFetcher::new(provider);

        assert_eq!(fetcher.max_concurrent, 10);
    }

    #[tokio::test]
    async fn test_custom_max_concurrent() {
        let provider = Arc::new(MarketDataProvider::new());
        let fetcher = ParallelDataFetcher::new(provider)
            .with_max_concurrent(5);

        assert_eq!(fetcher.max_concurrent, 5);
    }

    #[tokio::test]
    async fn test_fetch_empty_symbols() {
        let provider = Arc::new(MarketDataProvider::new());
        let fetcher = ParallelDataFetcher::new(provider);

        let quotes = fetcher.fetch_quotes_parallel(&[]).await;
        assert_eq!(quotes.len(), 0);

        let fundamentals = fetcher.fetch_fundamentals_parallel(&[]).await;
        assert_eq!(fundamentals.len(), 0);
    }

    #[tokio::test]
    async fn test_fetch_stats_calculation() {
        let stats = FetchStats {
            total_symbols: 10,
            successful_quotes: 8,
            successful_fundamentals: 7,
            failed_quotes: 2,
            failed_fundamentals: 3,
            elapsed_ms: 1000,
        };

        assert_eq!(stats.quote_success_rate(), 0.8);
        assert_eq!(stats.fundamental_success_rate(), 0.7);
        assert_eq!(stats.avg_ms_per_symbol(), 100.0);
    }

    #[tokio::test]
    async fn test_fetch_stats_zero_symbols() {
        let stats = FetchStats {
            total_symbols: 0,
            successful_quotes: 0,
            successful_fundamentals: 0,
            failed_quotes: 0,
            failed_fundamentals: 0,
            elapsed_ms: 0,
        };

        assert_eq!(stats.quote_success_rate(), 0.0);
        assert_eq!(stats.fundamental_success_rate(), 0.0);
        assert_eq!(stats.avg_ms_per_symbol(), 0.0);
    }
}
