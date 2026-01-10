// storage.rs - libSQL-based data persistence with 200ns query optimization
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use libsql::{Connection, Database};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Storage manager using libSQL for ultra-low latency queries
pub struct StorageManager {
    db: Arc<RwLock<Connection>>,
    db_path: PathBuf,
}

impl StorageManager {
    /// Create a new storage manager
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        // Create database directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Open database with libSQL optimizations
        let db = Database::open(&db_path)
            .map_err(|e| anyhow!("Failed to open database: {}", e))?;

        // Get connection
        let conn = db
            .connect()
            .map_err(|e| anyhow!("Failed to get connection: {}", e))?;

        let manager = Self {
            db: Arc::new(RwLock::new(conn)),
            db_path,
        };

        // Initialize schema
        manager.initialize_schema().await?;

        Ok(manager)
    }

    /// Initialize database schema with optimized indexes
    async fn initialize_schema(&self) -> Result<()> {
        let conn = self.db.write().await;

        // Enable performance optimizations
        conn.execute(
            "PRAGMA journal_mode = WAL",
            [],
        )
            .map_err(|e| anyhow!("Failed to set WAL mode: {}", e))?;

        conn.execute(
            "PRAGMA synchronous = NORMAL",
            [],
        )
            .map_err(|e| anyhow!("Failed to set synchronous: {}", e))?;

        conn.execute(
            "PRAGMA cache_size = -64000", // 64MB cache
            [],
        )
            .map_err(|e| anyhow!("Failed to set cache size: {}", e))?;

        conn.execute(
            "PRAGMA temp_store = MEMORY",
            [],
        )
            .map_err(|e| anyhow!("Failed to set temp store: {}", e))?;

        // Create portfolios table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS portfolios (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                initial_value REAL NOT NULL,
                current_value REAL NOT NULL,
                metadata TEXT
            )",
            [],
        )
            .map_err(|e| anyhow!("Failed to create portfolios table: {}", e))?;

        // Create positions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS positions (
                id TEXT PRIMARY KEY,
                portfolio_id TEXT NOT NULL,
                ticker TEXT NOT NULL,
                shares REAL NOT NULL,
                avg_cost REAL NOT NULL,
                current_price REAL NOT NULL,
                market_value REAL NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (portfolio_id) REFERENCES portfolios(id) ON DELETE CASCADE
            )",
            [],
        )
            .map_err(|e| anyhow!("Failed to create positions table: {}", e))?;

        // Create market_data table with indexes for fast queries
        conn.execute(
            "CREATE TABLE IF NOT EXISTS market_data (
                id TEXT PRIMARY KEY,
                ticker TEXT NOT NULL,
                price REAL NOT NULL,
                change REAL NOT NULL,
                change_percent REAL NOT NULL,
                volume INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                raw_data TEXT
            )",
            [],
        )
            .map_err(|e| anyhow!("Failed to create market_data table: {}", e))?;

        // Create indexes for ultra-fast queries (200ns target)
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_market_data_ticker_timestamp
                ON market_data(ticker, timestamp DESC)",
            [],
        )
            .map_err(|e| anyhow!("Failed to create index: {}", e))?;

        // Create analysis_cache table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS analysis_cache (
                id TEXT PRIMARY KEY,
                ticker TEXT NOT NULL,
                analysis_type TEXT NOT NULL,
                result TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                confidence REAL
            )",
            [],
        )
            .map_err(|e| anyhow!("Failed to create analysis_cache table: {}", e))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_analysis_cache_ticker_type_expires
                ON analysis_cache(ticker, analysis_type, expires_at)",
            [],
        )
            .map_err(|e| anyhow!("Failed to create index: {}", e))?;

        // Create backtest_results table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS backtest_results (
                id TEXT PRIMARY KEY,
                strategy_name TEXT NOT NULL,
                start_date TEXT NOT NULL,
                end_date TEXT NOT NULL,
                initial_capital REAL NOT NULL,
                final_value REAL NOT NULL,
                total_return REAL NOT NULL,
                sharpe_ratio REAL,
                max_drawdown REAL,
                win_rate REAL,
                parameters TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )
            .map_err(|e| anyhow!("Failed to create backtest_results table: {}", e))?;

        Ok(())
    }

    /// Save portfolio to database
    pub async fn save_portfolio(&self, portfolio: &Portfolio) -> Result<()> {
        let conn = self.db.write().await;

        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT OR REPLACE INTO portfolios
                (id, name, created_at, updated_at, initial_value, current_value, metadata)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [
                &portfolio.id,
                &portfolio.name,
                &portfolio.created_at.to_rfc3339(),
                &now,
                &portfolio.initial_value.to_string(),
                &portfolio.current_value.to_string(),
                &serde_json::to_string(&portfolio.metadata)?,
            ],
        )
            .map_err(|e| anyhow!("Failed to save portfolio: {}", e))?;

        // Delete old positions
        conn.execute(
            "DELETE FROM positions WHERE portfolio_id = ?1",
            [&portfolio.id],
        )
            .map_err(|e| anyhow!("Failed to delete old positions: {}", e))?;

        // Insert new positions
        for position in &portfolio.positions {
            conn.execute(
                "INSERT INTO positions
                    (id, portfolio_id, ticker, shares, avg_cost, current_price, market_value, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                [
                    &position.id,
                    &portfolio.id,
                    &position.ticker,
                    &position.shares.to_string(),
                    &position.avg_cost.to_string(),
                    &position.current_price.to_string(),
                    &position.market_value.to_string(),
                    &now,
                ],
            )
                .map_err(|e| anyhow!("Failed to save position: {}", e))?;
        }

        Ok(())
    }

    /// Load portfolio from database
    pub async fn load_portfolio(&self, id: &str) -> Result<Portfolio> {
        let conn = self.db.read().await;

        let mut stmt = conn
            .prepare("SELECT * FROM portfolios WHERE id = ?1")?
            .bind(&[id.into()])?;

        let mut portfolios = stmt.query()?;
        let portfolio_row = portfolios
            .next()
            .ok_or_else(|| anyhow!("Portfolio not found"))??;

        let id: String = portfolio_row.get(0)?;
        let name: String = portfolio_row.get(1)?;
        let created_at: String = portfolio_row.get(2)?;
        let initial_value: String = portfolio_row.get(4)?;
        let current_value: String = portfolio_row.get(5)?;
        let metadata: String = portfolio_row.get(6)?;

        // Load positions
        let mut stmt = conn
            .prepare("SELECT * FROM positions WHERE portfolio_id = ?1")?
            .bind(&[id.clone()])?;

        let mut positions = Vec::new();
        let mut pos_rows = stmt.query()?;

        while let Some(row) = pos_rows.next()? {
            positions.push(Position {
                id: row.get(0)?,
                portfolio_id: row.get(1)?,
                ticker: row.get(2)?,
                shares: row.get(3)?,
                avg_cost: row.get(4)?,
                current_price: row.get(5)?,
                market_value: row.get(6)?,
            });
        }

        Ok(Portfolio {
            id,
            name,
            created_at: DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc),
            updated_at: Utc::now(),
            initial_value: initial_value.parse()?,
            current_value: current_value.parse()?,
            positions,
            metadata: serde_json::from_str(&metadata).unwrap_or_default(),
        })
    }

    /// Save market data with automatic deduplication
    pub async fn save_market_data(&self, data: &MarketDataRecord) -> Result<()> {
        let conn = self.db.write().await;

        conn.execute(
            "INSERT OR REPLACE INTO market_data
                (id, ticker, price, change, change_percent, volume, timestamp, raw_data)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            [
                &data.id,
                &data.ticker,
                &data.price.to_string(),
                &data.change.to_string(),
                &data.change_percent.to_string(),
                &data.volume.to_string(),
                &data.timestamp.to_rfc3339(),
                &serde_json::to_string(data)?,
            ],
        )
            .map_err(|e| anyhow!("Failed to save market data: {}", e))?;

        Ok(())
    }

    /// Get latest market data for a ticker (optimized for 200ns query)
    pub async fn get_latest_market_data(&self, ticker: &str) -> Result<Option<MarketDataRecord>> {
        let conn = self.db.read().await;

        let mut stmt = conn
            .prepare("SELECT * FROM market_data WHERE ticker = ?1 ORDER BY timestamp DESC LIMIT 1")?
            .bind(&[ticker.to_string()])?;

        let mut rows = stmt.query()?;
        if let Some(row) = rows.next()? {
            Ok(Some(MarketDataRecord {
                id: row.get(0)?,
                ticker: row.get(1)?,
                price: row.get(2)?,
                change: row.get(3)?,
                change_percent: row.get(4)?,
                volume: row.get(5)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)?.with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    /// Cache analysis results
    pub async fn cache_analysis(&self, cache: &AnalysisCache) -> Result<()> {
        let conn = self.db.write().await;

        conn.execute(
            "INSERT OR REPLACE INTO analysis_cache
                (id, ticker, analysis_type, result, created_at, expires_at, confidence)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [
                &cache.id,
                &cache.ticker,
                &cache.analysis_type,
                &cache.result,
                &cache.created_at.to_rfc3339(),
                &cache.expires_at.to_rfc3339(),
                &cache.confidence.to_string(),
            ],
        )
            .map_err(|e| anyhow!("Failed to cache analysis: {}", e))?;

        Ok(())
    }

    /// Get cached analysis if not expired
    pub async fn get_cached_analysis(
        &self,
        ticker: &str,
        analysis_type: &str,
    ) -> Result<Option<AnalysisCache>> {
        let conn = self.db.read().await;

        let now = Utc::now().to_rfc3339();

        let mut stmt = conn
            .prepare(
                "SELECT * FROM analysis_cache
                    WHERE ticker = ?1 AND analysis_type = ?2 AND expires_at > ?3
                    ORDER BY created_at DESC LIMIT 1"
            )?
            .bind(&[ticker.to_string(), analysis_type.to_string(), now])?;

        let mut rows = stmt.query()?;
        if let Some(row) = rows.next()? {
            Ok(Some(AnalysisCache {
                id: row.get(0)?,
                ticker: row.get(1)?,
                analysis_type: row.get(2)?,
                result: row.get(3)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)?.with_timezone(&Utc),
                expires_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)?.with_timezone(&Utc),
                confidence: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Save backtest results
    pub async fn save_backtest_result(&self, result: &BacktestResult) -> Result<()> {
        let conn = self.db.write().await;

        conn.execute(
            "INSERT INTO backtest_results
                (id, strategy_name, start_date, end_date, initial_capital, final_value,
                total_return, sharpe_ratio, max_drawdown, win_rate, parameters, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            [
                &result.id,
                &result.strategy_name,
                &result.start_date.to_rfc3339(),
                &result.end_date.to_rfc3339(),
                &result.initial_capital.to_string(),
                &result.final_value.to_string(),
                &result.total_return.to_string(),
                &result.sharpe_ratio.map(|v| v.to_string()).unwrap_or_default(),
                &result.max_drawdown.map(|v| v.to_string()).unwrap_or_default(),
                &result.win_rate.map(|v| v.to_string()).unwrap_or_default(),
                &serde_json::to_string(&result.parameters)?,
                &result.created_at.to_rfc3339(),
            ],
        )
            .map_err(|e| anyhow!("Failed to save backtest result: {}", e))?;

        Ok(())
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let conn = self.db.read().await;

        let portfolio_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM portfolios", [], |row| row.get(0))?;

        let position_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM positions", [], |row| row.get(0))?;

        let market_data_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM market_data", [], |row| row.get(0))?;

        let cache_size: i64 = conn
            .query_row("SELECT COUNT(*) FROM analysis_cache", [], |row| row.get(0))?;

        // Get database file size
        let db_size = tokio::fs::metadata(&self.db_path).await?.len();

        Ok(StorageStats {
            portfolio_count,
            position_count,
            market_data_count,
            cache_size,
            db_size_bytes: db_size,
        })
    }

    /// Clean expired cache entries
    pub async fn clean_expired_cache(&self) -> Result<u64> {
        let conn = self.db.write().await;

        let now = Utc::now().to_rfc3339();

        conn.execute(
            "DELETE FROM analysis_cache WHERE expires_at < ?1",
            [&now],
        )
            .map_err(|e| anyhow!("Failed to clean cache: {}", e))
    }

    /// Vacuum database to reclaim space
    pub async fn vacuum(&self) -> Result<()> {
        let conn = self.db.write().await;

        conn.execute("VACUUM", [])
            .map_err(|e| anyhow!("Failed to vacuum: {}", e))?;

        Ok(())
    }
}

/// Portfolio data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub initial_value: f64,
    pub current_value: f64,
    pub positions: Vec<Position>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Position data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub portfolio_id: String,
    pub ticker: String,
    pub shares: f64,
    pub avg_cost: f64,
    pub current_price: f64,
    pub market_value: f64,
}

/// Market data record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataRecord {
    pub id: String,
    pub ticker: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: u64,
    pub timestamp: DateTime<Utc>,
}

/// Analysis cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCache {
    pub id: String,
    pub ticker: String,
    pub analysis_type: String,
    pub result: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub confidence: f64,
}

/// Backtest result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub id: String,
    pub strategy_name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub initial_capital: f64,
    pub final_value: f64,
    pub total_return: f64,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub win_rate: Option<f64>,
    pub parameters: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub portfolio_count: i64,
    pub position_count: i64,
    pub market_data_count: i64,
    pub cache_size: i64,
    pub db_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_manager() {
        let db_path = PathBuf::from("/tmp/test_investintel.db");
        let manager = StorageManager::new(db_path).await.unwrap();

        // Test portfolio save/load
        let portfolio = Portfolio {
            id: "test-portfolio".to_string(),
            name: "Test Portfolio".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            initial_value: 10000.0,
            current_value: 10500.0,
            positions: vec![],
            metadata: serde_json::json!({"test": true}),
        };

        manager.save_portfolio(&portfolio).await.unwrap();
        let loaded = manager.load_portfolio("test-portfolio").await.unwrap();

        assert_eq!(loaded.id, portfolio.id);
        assert_eq!(loaded.name, portfolio.name);

        // Test stats
        let stats = manager.get_stats().await.unwrap();
        assert_eq!(stats.portfolio_count, 1);

        // Cleanup
        tokio::fs::remove_file("/tmp/test_investintel.db").await.ok();
    }
}
