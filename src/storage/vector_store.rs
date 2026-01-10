// Vector store implementation using SQLite (rusqlite) with semantic search
use crate::observability::{Logger, MetricsCollector};
use crate::storage::{StorageError, StorageResult, Embedder};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

const EMPTY_LABELS: &[(&str, &str)] = &[];

/// Vector similarity metrics
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorMetric {
    /// Cosine similarity (default for text embeddings)
    Cosine,
    /// Euclidean distance (L2)
    L2,
    /// Dot product
    Dot,
}

impl Default for VectorMetric {
    fn default() -> Self {
        Self::Cosine
    }
}

/// Academic paper metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaperMetadata {
    /// Unique identifier (DOI, ArXiv ID, etc.)
    pub id: String,

    /// Paper title
    pub title: String,

    /// Author list
    pub authors: Vec<String>,

    /// Abstract text
    pub abstract_text: Option<String>,

    /// Publication year
    pub year: Option<i32>,

    /// Venue (conference/journal name)
    pub venue: Option<String>,

    /// DOI
    pub doi: Option<String>,

    /// PDF URL
    pub pdf_url: Option<String>,

    /// Citation count
    pub citation_count: Option<i32>,

    /// Data source (openalex, arxiv, semantic-scholar, etc.)
    pub source: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Simplified paper type for backward compatibility
pub type Paper = PaperMetadata;

/// Vector store configuration using builder pattern
#[derive(Clone, Debug, typed_builder::TypedBuilder)]
pub struct VectorStoreConfig {
    /// Path to the SQLite database file
    #[builder(default = "./vector_store.db".to_string())]
    pub db_path: String,

    /// Table name for storing papers
    #[builder(default = "papers".to_string())]
    pub table_name: String,

    /// Vector dimension
    #[builder(default = 384)]
    pub dimension: usize,

    /// Similarity metric
    #[builder(default)]
    pub metric: VectorMetric,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Vector search options
#[derive(Clone, Debug, Default)]
pub struct VectorSearchOptions {
    /// Maximum number of results
    pub limit: Option<usize>,

    /// Minimum similarity score (0-1)
    pub min_score: Option<f32>,

    /// Filter by year range
    pub year_from: Option<i32>,
    pub year_to: Option<i32>,

    /// Minimum citation count
    pub min_citations: Option<i32>,
}

/// Vector search result
#[derive(Clone, Debug)]
pub struct VectorSearchResult {
    /// Paper metadata
    pub paper: PaperMetadata,

    /// Similarity score (0-1)
    pub score: f32,
}

/// Vector store with SQLite backend
pub struct VectorStore {
    conn: Arc<RwLock<Connection>>,
    config: VectorStoreConfig,
    embedder: Arc<dyn Embedder>,
    logger: Logger,
    metrics: Arc<MetricsCollector>,
}

impl VectorStore {
    /// Create a new vector store
    pub async fn new(
        config: VectorStoreConfig,
        embedder: Arc<dyn Embedder>,
    ) -> StorageResult<Self> {
        let logger = Logger::new("VectorStore");
        let metrics = Arc::new(MetricsCollector::new());

        logger.info("Initializing SQLite vector store", &[
            ("db_path", config.db_path.as_str()),
            ("dimension", &config.dimension.to_string()),
        ]);

        // Create parent directory if needed
        let path = PathBuf::from(&config.db_path);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| StorageError::Io(e))?;
        }

        // Open SQLite database
        let conn = Connection::open(&config.db_path)
            .map_err(|e| StorageError::DatabaseConnection(
                format!("Failed to open database: {}", e)
            ))?;

        let store = Self {
            conn: Arc::new(RwLock::new(conn)),
            config,
            embedder,
            logger: logger.clone(),
            metrics,
        };

        // Initialize schema
        {
            let conn = store.conn.write().await;
            store.initialize_schema(&conn)?;
        }

        logger.info("Vector store initialized successfully", EMPTY_LABELS);

        Ok(store)
    }

    /// Initialize database schema
    fn initialize_schema(&self, conn: &Connection) -> StorageResult<()> {
        // Performance optimizations (PRAGMA statements may return rows, ignore errors)
        let _ = conn.execute("PRAGMA journal_mode = WAL", []);
        let _ = conn.execute("PRAGMA synchronous = NORMAL", []);
        let _ = conn.execute("PRAGMA cache_size = -64000", []);

        // Create papers table
        let create_table = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                authors TEXT NOT NULL,
                abstract_text TEXT,
                year INTEGER,
                venue TEXT,
                doi TEXT UNIQUE,
                pdf_url TEXT,
                citation_count INTEGER DEFAULT 0,
                embedding BLOB,
                source TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            self.config.table_name
        );

        conn.execute(&create_table, [])
            .map_err(|e| StorageError::QueryExecution(format!("Failed to create table: {}", e)))?;

        // Create indexes for efficient queries
        let idx_year = format!(
            "CREATE INDEX IF NOT EXISTS idx_{}_year ON {}(year)",
            self.config.table_name, self.config.table_name
        );
        conn.execute(&idx_year, [])
            .map_err(|e| StorageError::QueryExecution(format!("Failed to create year index: {}", e)))?;

        let idx_venue = format!(
            "CREATE INDEX IF NOT EXISTS idx_{}_venue ON {}(venue)",
            self.config.table_name, self.config.table_name
        );
        conn.execute(&idx_venue, [])
            .map_err(|e| StorageError::QueryExecution(format!("Failed to create venue index: {}", e)))?;

        let idx_citations = format!(
            "CREATE INDEX IF NOT EXISTS idx_{}_citations ON {}(citation_count)",
            self.config.table_name, self.config.table_name
        );
        conn.execute(&idx_citations, [])
            .map_err(|e| StorageError::QueryExecution(format!("Failed to create citation index: {}", e)))?;

        Ok(())
    }

    /// Insert a paper with its embedding
    pub async fn insert(&self, paper: PaperMetadata) -> StorageResult<()> {
        let _timer = self.metrics.time(
            "vector_store_insert",
            &[("table", self.config.table_name.as_str())],
            || {}
        );

        // Generate embedding from title + abstract
        let text = format!("{}\n{}", paper.title, paper.abstract_text.as_deref().unwrap_or(""));
        let embedding = self.embedder.embed(&text).await?;

        if embedding.len() != self.config.dimension {
            return Err(StorageError::InvalidDimension {
                expected: self.config.dimension,
                actual: embedding.len(),
            });
        }

        // Convert embedding to bytes
        let embedding_bytes = self.embedding_to_bytes(&embedding);

        let conn = self.conn.write().await;
        let authors_json = serde_json::to_string(&paper.authors)?;
        let now = Utc::now().to_rfc3339();

        let query = format!(
            "INSERT OR REPLACE INTO {} (
                id, title, authors, abstract_text, year, venue, doi,
                pdf_url, citation_count, embedding, source, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            self.config.table_name
        );

        conn.execute(&query, params![
            paper.id,
            paper.title,
            authors_json,
            paper.abstract_text,
            paper.year,
            paper.venue,
            paper.doi,
            paper.pdf_url,
            paper.citation_count,
            embedding_bytes,
            paper.source,
            paper.created_at.to_rfc3339(),
            now,
        ]).map_err(|e| StorageError::QueryExecution(format!("Failed to insert paper: {}", e)))?;

        self.metrics.increment("vector_store_insert_total", &[("table", self.config.table_name.as_str())]);

        Ok(())
    }

    /// Batch insert papers
    pub async fn insert_batch(&self, papers: Vec<PaperMetadata>) -> StorageResult<()> {
        let count = papers.len();
        let _timer = self.metrics.time(
            "vector_store_insert_batch",
            &[("count", &count.to_string())],
            || {}
        );

        self.logger.info("Batch inserting papers", &[
            ("count", &count.to_string()),
        ]);

        for paper in papers {
            self.insert(paper).await?;
        }

        self.metrics.increment("vector_store_insert_batch_total", &[("table", self.config.table_name.as_str())]);

        Ok(())
    }

    /// Vector similarity search
    pub async fn similarity_search(
        &self,
        query: &str,
        options: Option<VectorSearchOptions>,
    ) -> StorageResult<Vec<VectorSearchResult>> {
        let opts = options.unwrap_or_default();
        let limit = opts.limit.unwrap_or(10);

        let _timer = self.metrics.time(
            "vector_store_search",
            &[("limit", &limit.to_string())],
            || {}
        );

        // Generate query embedding
        let query_embedding = self.embedder.embed(query).await?;

        // Fetch all papers with embeddings
        let conn = self.conn.read().await;
        let select_query = format!(
            "SELECT id, title, authors, abstract_text, year, venue, doi,
                    pdf_url, citation_count, source, created_at, updated_at, embedding
             FROM {}",
            self.config.table_name
        );

        let mut stmt = conn.prepare(&select_query)
            .map_err(|e| StorageError::QueryExecution(format!("Search failed: {}", e)))?;

        let paper_rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>("id")?,
                row.get::<_, String>("title")?,
                row.get::<_, String>("authors")?,
                row.get::<_, Option<String>>("abstract_text")?,
                row.get::<_, Option<i32>>("year")?,
                row.get::<_, Option<String>>("venue")?,
                row.get::<_, Option<String>>("doi")?,
                row.get::<_, Option<String>>("pdf_url")?,
                row.get::<_, Option<i32>>("citation_count")?,
                row.get::<_, String>("source")?,
                row.get::<_, String>("created_at")?,
                row.get::<_, String>("updated_at")?,
                row.get::<_, Vec<u8>>("embedding")?,
            ))
        }).map_err(|e| StorageError::QueryExecution(format!("Query failed: {}", e)))?;

        let mut results = Vec::new();

        for paper_row in paper_rows {
            let (id, title, authors_json, abstract_text, year, venue, doi, pdf_url,
                 citation_count, source, created_at, updated_at, embedding_bytes) = paper_row
                .map_err(|e| StorageError::QueryExecution(format!("Row iteration failed: {}", e)))?;

            // Parse authors
            let authors: Vec<String> = serde_json::from_str(&authors_json)
                .map_err(|e| StorageError::Serialization(format!("Failed to parse authors: {}", e)))?;

            // Parse embedding
            let paper_embedding = self.bytes_to_embedding(&embedding_bytes)?;

            // Calculate similarity
            let score = match self.config.metric {
                VectorMetric::Cosine => self.cosine_similarity(&query_embedding, &paper_embedding),
                VectorMetric::L2 => self.euclidean_distance(&query_embedding, &paper_embedding),
                VectorMetric::Dot => self.dot_product(&query_embedding, &paper_embedding),
            };

            // Apply filters
            if let Some(min_score) = opts.min_score {
                if score < min_score {
                    continue;
                }
            }

            if let Some(year_from) = opts.year_from {
                if let Some(year) = year {
                    if year < year_from {
                        continue;
                    }
                }
            }

            if let Some(year_to) = opts.year_to {
                if let Some(year) = year {
                    if year > year_to {
                        continue;
                    }
                }
            }

            if let Some(min_citations) = opts.min_citations {
                if let Some(citations) = citation_count {
                    if citations < min_citations {
                        continue;
                    }
                }
            }

            // Parse timestamps
            let created_dt = DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| StorageError::Serialization(format!("Invalid created_at: {}", e)))?
                .with_timezone(&Utc);
            let updated_dt = DateTime::parse_from_rfc3339(&updated_at)
                .map_err(|e| StorageError::Serialization(format!("Invalid updated_at: {}", e)))?
                .with_timezone(&Utc);

            let paper = PaperMetadata {
                id, title, authors, abstract_text, year, venue, doi, pdf_url,
                citation_count, source,
                created_at: created_dt,
                updated_at: updated_dt,
            };

            results.push(VectorSearchResult { paper, score });
        }

        // Sort by score and limit
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        self.metrics.increment("vector_store_search_total", EMPTY_LABELS);

        Ok(results)
    }

    /// Get paper by ID
    pub async fn get(&self, id: &str) -> StorageResult<Option<PaperMetadata>> {
        let conn = self.conn.read().await;
        let query = format!(
            "SELECT id, title, authors, abstract_text, year, venue, doi,
                    pdf_url, citation_count, source, created_at, updated_at
             FROM {} WHERE id = ?1",
            self.config.table_name
        );

        let mut stmt = conn.prepare(&query)
            .map_err(|e| StorageError::QueryExecution(format!("Get paper failed: {}", e)))?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok((
                row.get::<_, String>("id")?,
                row.get::<_, String>("title")?,
                row.get::<_, String>("authors")?,
                row.get::<_, Option<String>>("abstract_text")?,
                row.get::<_, Option<i32>>("year")?,
                row.get::<_, Option<String>>("venue")?,
                row.get::<_, Option<String>>("doi")?,
                row.get::<_, Option<String>>("pdf_url")?,
                row.get::<_, Option<i32>>("citation_count")?,
                row.get::<_, String>("source")?,
                row.get::<_, String>("created_at")?,
                row.get::<_, String>("updated_at")?,
            ))
        }).map_err(|e| StorageError::QueryExecution(format!("Query failed: {}", e)))?;

        if let Some(paper_row) = rows.next() {
            let (id, title, authors_json, abstract_text, year, venue, doi, pdf_url,
                 citation_count, source, created_at, updated_at) = paper_row
                .map_err(|e| StorageError::QueryExecution(format!("Row iteration failed: {}", e)))?;

            let authors: Vec<String> = serde_json::from_str(&authors_json)
                .map_err(|e| StorageError::Serialization(format!("Failed to parse authors: {}", e)))?;

            let created_dt = DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| StorageError::Serialization(format!("Invalid created_at: {}", e)))?
                .with_timezone(&Utc);
            let updated_dt = DateTime::parse_from_rfc3339(&updated_at)
                .map_err(|e| StorageError::Serialization(format!("Invalid updated_at: {}", e)))?
                .with_timezone(&Utc);

            Ok(Some(PaperMetadata {
                id, title, authors, abstract_text, year, venue, doi, pdf_url,
                citation_count, source,
                created_at: created_dt,
                updated_at: updated_dt,
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete paper by ID
    pub async fn delete(&self, id: &str) -> StorageResult<bool> {
        let conn = self.conn.write().await;
        let query = format!("DELETE FROM {} WHERE id = ?1", self.config.table_name);

        let rows_affected = conn.execute(&query, params![id])
            .map_err(|e| StorageError::QueryExecution(format!("Delete failed: {}", e)))?;

        Ok(rows_affected > 0)
    }

    /// Count total papers
    pub async fn count(&self) -> StorageResult<usize> {
        let conn = self.conn.read().await;
        let query = format!("SELECT COUNT(*) as count FROM {}", self.config.table_name);

        let mut stmt = conn.prepare(&query)
            .map_err(|e| StorageError::QueryExecution(format!("Count failed: {}", e)))?;

        let count: i64 = stmt.query_row([], |row| row.get(0))
            .map_err(|e| StorageError::QueryExecution(format!("Count query failed: {}", e)))?;

        Ok(count as usize)
    }

    // Helper methods

    fn embedding_to_bytes(&self, embedding: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(embedding.len() * 4);
        for &val in embedding {
            bytes.extend_from_slice(&val.to_le_bytes());
        }
        bytes
    }

    fn bytes_to_embedding(&self, bytes: &[u8]) -> StorageResult<Vec<f32>> {
        if bytes.len() % 4 != 0 {
            return Err(StorageError::Internal(
                "Invalid embedding byte length".to_string()
            ));
        }

        let mut embedding = Vec::with_capacity(bytes.len() / 4);
        for chunk in bytes.chunks_exact(4) {
            let val = f32::from_le_bytes(chunk.try_into().unwrap());
            embedding.push(val);
        }

        Ok(embedding)
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        let sum_sq: f32 = a.iter().zip(b.iter())
            .map(|(x, y)| (x - y) * (x - y))
            .sum();

        1.0 / (1.0 + sum_sq.sqrt())
    }

    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::embedders::LocalEmbedder;
    use std::sync::Arc;
    use chrono::Utc;
    use tempfile::TempDir;

    async fn create_test_store() -> (VectorStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();

        let embedder = Arc::new(LocalEmbedder::new(384));
        let config = VectorStoreConfig::builder()
            .db_path(db_path)
            .dimension(384)
            .build();

        let store = VectorStore::new(config, embedder).await.unwrap();
        (store, temp_dir)
    }

    #[tokio::test]
    async fn test_vector_store_creation() {
        let (store, _temp) = create_test_store().await;
        assert_eq!(store.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_paper_insertion() {
        let (store, _temp) = create_test_store().await;

        let paper = PaperMetadata {
            id: "test-paper-1".to_string(),
            title: "Test Paper".to_string(),
            authors: vec!["Author One".to_string(), "Author Two".to_string()],
            abstract_text: Some("This is a test abstract.".to_string()),
            year: Some(2024),
            venue: Some("Test Conference".to_string()),
            doi: Some("10.1234/test".to_string()),
            pdf_url: None,
            citation_count: Some(10),
            source: "test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        store.insert(paper).await.unwrap();
        assert_eq!(store.count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_paper_retrieval() {
        let (store, _temp) = create_test_store().await;

        let paper = PaperMetadata {
            id: "test-paper-1".to_string(),
            title: "Machine Learning in Academia".to_string(),
            authors: vec!["Researcher One".to_string()],
            abstract_text: Some("A paper about machine learning.".to_string()),
            year: Some(2024),
            venue: Some("AI Conference".to_string()),
            doi: None,
            pdf_url: None,
            citation_count: Some(5),
            source: "test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        store.insert(paper.clone()).await.unwrap();

        let retrieved = store.get("test-paper-1").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.title, "Machine Learning in Academia");
        assert_eq!(retrieved.authors.len(), 1);
    }

    #[tokio::test]
    async fn test_similarity_search() {
        let (store, _temp) = create_test_store().await;

        // Insert test papers
        for i in 0..5 {
            let paper = PaperMetadata {
                id: format!("paper-{}", i),
                title: format!("Research Paper {}", i),
                authors: vec![format!("Author {}", i)],
                abstract_text: Some(format!("Abstract for paper {}", i)),
                year: Some(2020 + i),
                venue: Some("Test Venue".to_string()),
                doi: None,
                pdf_url: None,
                citation_count: Some(i * 10),
                source: "test".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            store.insert(paper).await.unwrap();
        }

        let results = store.similarity_search("research paper", None).await.unwrap();
        assert!(!results.is_empty());
        assert!(results.len() <= 10);

        // Check scores are valid
        for result in &results {
            assert!(result.score >= 0.0 && result.score <= 1.0);
        }
    }

    #[tokio::test]
    async fn test_paper_deletion() {
        let (store, _temp) = create_test_store().await;

        let paper = PaperMetadata {
            id: "test-paper-1".to_string(),
            title: "Test Paper".to_string(),
            authors: vec!["Author".to_string()],
            abstract_text: Some("Abstract".to_string()),
            year: Some(2024),
            venue: None,
            doi: None,
            pdf_url: None,
            citation_count: None,
            source: "test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        store.insert(paper).await.unwrap();
        assert_eq!(store.count().await.unwrap(), 1);

        store.delete("test-paper-1").await.unwrap();
        assert_eq!(store.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_vector_search_filters() {
        let (store, _temp) = create_test_store().await;

        // Insert papers with different years
        for year in &[2020, 2021, 2022, 2023, 2024] {
            let paper = PaperMetadata {
                id: format!("paper-{}", year),
                title: format!("Paper from {}", year),
                authors: vec!["Author".to_string()],
                abstract_text: Some("Abstract".to_string()),
                year: Some(*year),
                venue: None,
                doi: None,
                pdf_url: None,
                citation_count: Some(100),
                source: "test".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            store.insert(paper).await.unwrap();
        }

        let opts = VectorSearchOptions {
            year_from: Some(2022),
            year_to: Some(2024),
            ..Default::default()
        };

        let results = store.similarity_search("paper", Some(opts)).await.unwrap();

        // Should only return papers from 2022-2024
        for result in &results {
            if let Some(year) = result.paper.year {
                assert!(year >= 2022 && year <= 2024);
            }
        }
    }
}
