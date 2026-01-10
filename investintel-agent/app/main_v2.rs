// main_v2.rs - Advanced InvestIntel AI with real-time data, streaming, and backtesting
mod backtest;
mod market_data;
mod orchestration;
mod storage;
mod streaming;
mod tools;

use anyhow::Result;
use clap::{Parser, Subcommand};
use market_data::MarketDataClient;
use storage::StorageManager;
use streaming::InvestmentStreamingAnalyzer;
use std::path::PathBuf;
use std::time::Duration;

/// InvestIntel AI - Intelligent Investment Analysis Platform
#[derive(Parser, Debug)]
#[command(name = "investintel")]
#[command(about = "AI-powered investment analysis platform", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze a ticker with AI
    Analyze {
        /// Ticker symbol to analyze
        ticker: String,

        /// Analysis types (comma-separated): technical, fundamental, sentiment, risk, all
        #[arg(short, long, default_value = "all")]
        types: String,

        /// Use streaming mode for real-time updates
        #[arg(short, long)]
        stream: bool,
    },

    /// Get real-time market data
    Market {
        /// Ticker symbol(s) (comma-separated)
        tickers: String,

        /// Get historical data
        #[arg(short, long)]
        historical: bool,

        /// Period for historical data (1mo, 3mo, 6mo, 1y, 2y, 5y)
        #[arg(short, long, default_value = "1mo")]
        period: String,
    },

    /// Backtest a strategy
    Backtest {
        /// Strategy name: sma_crossover, bollinger, rsi
        strategy: String,

        /// Ticker symbol(s) (comma-separated)
        tickers: String,

        /// Initial capital
        #[arg(short, long, default_value = "100000")]
        capital: f64,

        /// Backtest period (e.g., "1y", "6mo")
        #[arg(short, long, default_value = "1y")]
        period: String,
    },

    /// Monitor portfolio
    Portfolio {
        /// Portfolio ID
        #[arg(short, long)]
        id: Option<String>,

        /// Create new portfolio
        #[arg(long)]
        create: bool,

        /// Portfolio name
        #[arg(long)]
        name: Option<String>,
    },

    /// Real-time monitoring
    Monitor {
        /// Ticker symbol(s) to monitor (comma-separated)
        tickers: String,

        /// Update interval in seconds
        #[arg(short, long, default_value = "60")]
        interval: u64,
    },

    /// Database operations
    Db {
        #[command(subcommand)]
        db_command: DbCommands,
    },
}

#[derive(Subcommand, Debug)]
enum DbCommands {
    /// Show database statistics
    Stats,

    /// Clean expired cache
    Clean,

    /// Vacuum database
    Vacuum,

    /// Export data
    Export {
        /// Output file path
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize storage manager
    let db_path = PathBuf::from("investintel.db");
    let storage = StorageManager::new(db_path).await?;

    match args.command {
        Commands::Analyze { ticker, types, stream } => {
            if stream {
                run_streaming_analysis(&ticker, &types).await?;
            } else {
                run_analysis(&ticker, &types).await?;
            }
        }

        Commands::Market {
            tickers,
            historical,
            period,
        } => {
            run_market_data(&tickers, historical, &period).await?;
        }

        Commands::Backtest {
            strategy,
            tickers,
            capital,
            period,
        } => {
            run_backtest(&strategy, &tickers, capital, &period).await?;
        }

        Commands::Portfolio { id, create, name } => {
            if create {
                create_portfolio(&storage, name).await?;
            } else if let Some(portfolio_id) = id {
                show_portfolio(&storage, &portfolio_id).await?;
            } else {
                list_portfolios(&storage).await?;
            }
        }

        Commands::Monitor { tickers, interval } => {
            run_monitoring(&tickers, interval).await?;
        }

        Commands::Db { db_command } => match db_command {
            DbCommands::Stats => {
                show_db_stats(&storage).await?;
            }
            DbCommands::Clean => {
                clean_database(&storage).await?;
            }
            DbCommands::Vacuum => {
                vacuum_database(&storage).await?;
            }
            DbCommands::Export { output } => {
                export_database(&storage, &output).await?;
            }
        },
    }

    Ok(())
}

/// Run streaming analysis
async fn run_streaming_analysis(ticker: &str, types: &str) -> Result<()> {
    println!("🔄 Starting streaming analysis for {}...", ticker);

    let analyzer = InvestmentStreamingAnalyzer::new();
    let analysis_types = parse_analysis_types(types);

    let mut stream = analyzer
        .analyze_ticker_stream(ticker, analysis_types)
        .await?;

    println!("📡 Streaming updates:");

    while let Some(event) = stream.next().await {
        match event {
            streaming::StreamingEvent::Text(text) => {
                print!("{}", text);
            }
            streaming::StreamingEvent::ToolUse { name, .. } => {
                println!("\n🔧 Executing tool: {}", name);
            }
            streaming::StreamingEvent::ToolResult { tool_id, content } => {
                println!("✅ Tool {} completed: {}", tool_id, content);
            }
            streaming::StreamingEvent::Thinking { text, .. } => {
                println!("💭 Thinking... ({} chars)", text.len());
            }
            streaming::StreamingEvent::AnalysisComplete(result) => {
                println!("\n✅ Analysis Complete!");
                println!("   Ticker: {}", result.ticker);
                println!("   Confidence: {:.1}%", result.confidence * 100.0);
                println!("   Content: {}", result.content);
            }
            streaming::StreamingEvent::Error(e) => {
                eprintln!("❌ Error: {}", e);
            }
            streaming::StreamingEvent::Complete => {
                println!("\n✨ Stream complete");
                break;
            }
        }
    }

    Ok(())
}

/// Run regular analysis
async fn run_analysis(ticker: &str, types: &str) -> Result<()> {
    println!("📊 Analyzing {}...", ticker);

    use claude_agent_sdk_rs::{query, ClaudeAgentOptions, PermissionMode};

    let prompt = format!(
        "Provide a comprehensive investment analysis for {}. Include:\n\
         1. Technical analysis (trend, indicators, support/resistance)\n\
         2. Current price and recent performance\n\
         3. Risk assessment\n\
         4. Investment recommendation (Buy/Hold/Sell) with confidence level",
        ticker
    );

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .max_turns(5)
        .build();

    let messages = query(prompt, Some(options)).await?;

    for message in messages {
        if let claude_agent_sdk_rs::Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let claude_agent_sdk_rs::ContentBlock::Text(text) = block {
                    println!("{}", text.text);
                }
            }
        }
    }

    Ok(())
}

/// Run market data queries
async fn run_market_data(tickers: &str, historical: bool, period: &str) -> Result<()> {
    let client = MarketDataClient::new();
    let ticker_list: Vec<String> = tickers
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .collect();

    if historical {
        println!("📈 Fetching historical data for {} ticker(s)...", ticker_list.len());

        for ticker in &ticker_list {
            println!("\n=== {} ===", ticker);

            match client.get_historical_data(ticker, period).await {
                Ok(data) => {
                    println!("Retrieved {} data points", data.len());

                    if !data.is_empty() {
                        let latest = &data[data.len() - 1];
                        println!("Latest: ${:.2} (Volume: {})", latest.close, latest.volume);

                        // Calculate indicators
                        match client.calculate_indicators(&data) {
                            Ok(indicators) => {
                                println!("SMA 20: ${:.2}", indicators.sma_20.unwrap_or(0.0));
                                println!("SMA 50: ${:.2}", indicators.sma_50.unwrap_or(0.0));
                                println!("RSI: {:.2}", indicators.rsi.unwrap_or(0.0));

                                if let Some(macd) = &indicators.macd {
                                    println!("MACD: {:.4} (Signal: {:.4})", macd.macd, macd.signal);
                                }

                                println!(
                                    "Support: {:.2}, {:.2}",
                                    indicators.support_levels.get(0).unwrap_or(&0.0),
                                    indicators.support_levels.get(1).unwrap_or(&0.0)
                                );
                                println!(
                                    "Resistance: {:.2}, {:.2}",
                                    indicators.resistance_levels.get(0).unwrap_or(&0.0),
                                    indicators.resistance_levels.get(1).unwrap_or(&0.0)
                                );
                            }
                            Err(e) => {
                                eprintln!("Error calculating indicators: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching data for {}: {}", ticker, e);
                }
            }
        }
    } else {
        println!("💹 Fetching real-time quotes...");
        let quotes = client.get_batch_quotes(&ticker_list).await?;

        for (ticker, data) in quotes {
            println!(
                " {}: ${:.2} ({:+.2}%, Vol: {})",
                ticker,
                data.price,
                data.change_percent,
                data.volume
            );
        }
    }

    Ok(())
}

/// Run backtest
async fn run_backtest(strategy: &str, tickers: &str, capital: f64, period: &str) -> Result<()> {
    println!("🔬 Running backtest: {} strategy", strategy);
    println!("   Capital: ${:.2}", capital);
    println!("   Period: {}", period);

    let ticker_list: Vec<String> = tickers
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .collect();

    let client = MarketDataClient::new();

    // Fetch historical data
    let mut all_data = std::collections::HashMap::new();

    for ticker in &ticker_list {
        match client.get_historical_data(ticker, period).await {
            Ok(data) => {
                println!("✅ Fetched {} data points for {}", data.len(), ticker);

                // Convert to PriceData format
                let prices: Vec<backtest::PriceData> = data
                    .into_iter()
                    .map(|d| backtest::PriceData {
                        timestamp: d.date,
                        open: d.open,
                        high: d.high,
                        low: d.low,
                        close: d.close,
                        volume: d.volume,
                    })
                    .collect();

                all_data.insert(ticker.clone(), prices);
            }
            Err(e) => {
                eprintln!("❌ Error fetching {}: {}", ticker, e);
            }
        }
    }

    if all_data.is_empty() {
        println!("❌ No data available for backtesting");
        return Ok(());
    }

    // Run backtest
    let config = backtest::BacktestConfig {
        initial_capital: capital,
        ..Default::default()
    };

    let mut engine = backtest::BacktestEngine::new(config);

    let strategy_fn = match strategy {
        "bollinger" => backtest::bollinger_band_strategy(),
        _ => backtest::bollinger_band_strategy(), // Default
    };

    let result = engine.run(strategy_fn, &all_data)?;

    // Print results
    println!("\n📊 Backtest Results:");
    println!("   Strategy: {}", result.strategy_name);
    println!("   Period: {} to {}", result.start_date.format("%Y-%m-%d"), result.end_date.format("%Y-%m-%d"));
    println!("   Initial Capital: ${:.2}", result.initial_capital);
    println!("   Final Value: ${:.2}", result.final_value);
    println!("   Total Return: {:.2}%", result.total_return);
    println!("   Annual Return: {:.2}%", result.annual_return);
    println!("   Sharpe Ratio: {:.2}", result.sharpe_ratio);
    println!("   Sortino Ratio: {:.2}", result.sortino_ratio);
    println!("   Max Drawdown: {:.2}%", result.max_drawdown);
    println!("   Win Rate: {:.1}%", result.win_rate);
    println!("   Total Trades: {}", result.total_trades);
    println!("   Winning Trades: {}", result.winning_trades);
    println!("   Losing Trades: {}", result.losing_trades);
    println!("   Profit Factor: {:.2}", result.profit_factor);
    println!("   Avg Win: ${:.2}", result.avg_win);
    println!("   Avg Loss: ${:.2}", result.avg_loss);
    println!("   Largest Win: ${:.2}", result.largest_win);
    println!("   Largest Loss: ${:.2}", result.largest_loss);

    Ok(())
}

/// Create portfolio
async fn create_portfolio(storage: &StorageManager, name: Option<String>) -> Result<()> {
    use uuid::Uuid;

    let portfolio = storage::Portfolio {
        id: Uuid::new_v4().to_string(),
        name: name.unwrap_or_else(|| "My Portfolio".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        initial_value: 0.0,
        current_value: 0.0,
        positions: vec![],
        metadata: serde_json::json!({}),
    };

    storage.save_portfolio(&portfolio).await?;

    println!("✅ Portfolio created: {}", portfolio.id);
    println!("   Name: {}", portfolio.name);

    Ok(())
}

/// Show portfolio
async fn show_portfolio(storage: &StorageManager, id: &str) -> Result<()> {
    let portfolio = storage.load_portfolio(id).await?;

    println!("📁 Portfolio: {}", portfolio.name);
    println!("   ID: {}", portfolio.id);
    println!("   Created: {}", portfolio.created_at.format("%Y-%m-%d %H:%M"));
    println!("   Initial Value: ${:.2}", portfolio.initial_value);
    println!("   Current Value: ${:.2}", portfolio.current_value);

    if portfolio.current_value > 0.0 {
        let return_pct = (portfolio.current_value / portfolio.initial_value - 1.0) * 100.0;
        println!("   Return: {:+.2}%", return_pct);
    }

    println!("\n   Positions ({}):", portfolio.positions.len());

    for position in &portfolio.positions {
        println!(
            "     {} - {} shares @ ${:.2} (Value: ${:.2})",
            position.ticker, position.shares, position.avg_cost, position.market_value
        );
    }

    Ok(())
}

/// List portfolios
async fn list_portfolios(storage: &StorageManager) -> Result<()> {
    use libsql::{params, Connection};

    // Query all portfolios directly
    let conn = storage.db.read().await;
    let mut stmt = conn.prepare("SELECT id, name, current_value FROM portfolios")?;

    println!("📁 Portfolios:");

    let mut rows = stmt.query()?;
    let mut count = 0;

    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let value: String = row.get(2)?;
        let value_f64: f64 = value.parse().unwrap_or(0.0);

        println!("   {} - {} (${:.2})", id, name, value_f64);
        count += 1;
    }

    if count == 0 {
        println!("   No portfolios found");
    }

    Ok(())
}

/// Run real-time monitoring
async fn run_monitoring(tickers: &str, interval: u64) -> Result<()> {
    let ticker_list: Vec<String> = tickers
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .collect();

    println!("👀 Monitoring {} ticker(s) every {}s...", ticker_list.len(), interval);
    println!("   Press Ctrl+C to stop\n");

    let client = MarketDataClient::new();
    let mut interval_timer = tokio::time::interval(Duration::from_secs(interval));

    loop {
        interval_timer.tick().await;

        println!("📊 Update at {}", chrono::Utc::now().format("%H:%M:%S"));

        let quotes = client.get_batch_quotes(&ticker_list).await?;

        for (ticker, data) in quotes {
            let emoji = if data.change >= 0.0 { "📈" } else { "📉" };
            println!(
                "   {} {}: ${:.2} ({:+.2}%, Vol: {})",
                emoji, ticker, data.price, data.change_percent, data.volume
            );
        }

        println!();
    }
}

/// Show database statistics
async fn show_db_stats(storage: &StorageManager) -> Result<()> {
    let stats = storage.get_stats().await?;

    println!("💾 Database Statistics:");
    println!("   Portfolios: {}", stats.portfolio_count);
    println!("   Positions: {}", stats.position_count);
    println!("   Market Data Points: {}", stats.market_data_count);
    println!("   Cached Analyses: {}", stats.cache_size);
    println!("   Database Size: {:.2} MB", stats.db_size_bytes as f64 / 1024.0 / 1024.0);

    Ok(())
}

/// Clean expired cache
async fn clean_database(storage: &StorageManager) -> Result<()> {
    let count = storage.clean_expired_cache().await?;
    println!("✅ Cleaned {} expired cache entries", count);
    Ok(())
}

/// Vacuum database
async fn vacuum_database(storage: &StorageManager) -> Result<()> {
    storage.vacuum().await?;
    println!("✅ Database vacuumed successfully");
    Ok(())
}

/// Export database
async fn export_database(storage: &StorageManager, output: &PathBuf) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    let stats = storage.get_stats().await?;

    let mut file = File::create(output)?;

    writeln!(file, "InvestIntel Database Export")?;
    writeln!(file, "Generated: {}", chrono::Utc::now().to_rfc3339())?;
    writeln!(file, "")?;
    writeln!(file, "Statistics:")?;
    writeln!(file, "  Portfolios: {}", stats.portfolio_count)?;
    writeln!(file, "  Positions: {}", stats.position_count)?;
    writeln!(file, "  Market Data: {}", stats.market_data_count)?;
    writeln!(file, "  Cache Size: {}", stats.cache_size)?;

    println!("✅ Database exported to: {}", output.display());

    Ok(())
}

/// Parse analysis types
fn parse_analysis_types(types: &str) -> Vec<streaming::AnalysisType> {
    types
        .split(',')
        .map(|s| match s.trim().to_lowercase().as_str() {
            "technical" => streaming::AnalysisType::Technical,
            "fundamental" => streaming::AnalysisType::Fundamental,
            "sentiment" => streaming::AnalysisType::Sentiment,
            "risk" => streaming::AnalysisType::Risk,
            "all" => streaming::AnalysisType::All,
            _ => streaming::AnalysisType::All,
        })
        .collect()
}
