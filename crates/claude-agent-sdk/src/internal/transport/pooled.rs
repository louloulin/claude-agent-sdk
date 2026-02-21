//! Pooled transport implementation using connection pool

use async_trait::async_trait;
use futures::stream::Stream;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::errors::{ClaudeError, Result};
use crate::internal::pool::{get_global_pool, init_global_pool, ConnectionPool, PoolConfig, WorkerGuard};

use super::Transport;

/// Transport that uses a pooled worker from the connection pool.
///
/// This transport acquires a worker from the global connection pool on connect
/// and returns it to the pool when closed. This reduces process spawn overhead
/// for repeated queries.
pub struct PooledTransport {
    pool: Arc<ConnectionPool>,
    guard: Option<WorkerGuard>,
    options: crate::types::config::ClaudeAgentOptions,
    ready: bool,
}

impl PooledTransport {
    /// Create a new pooled transport
    ///
    /// # Arguments
    /// * `pool_config` - Configuration for the connection pool
    /// * `options` - SDK options for spawning workers
    pub fn new(
        pool_config: PoolConfig,
        options: crate::types::config::ClaudeAgentOptions,
    ) -> Self {
        let pool = Arc::new(ConnectionPool::new(pool_config, options.clone()));
        Self {
            pool,
            guard: None,
            options,
            ready: false,
        }
    }

    /// Create a pooled transport using an existing pool
    pub fn from_pool(pool: Arc<ConnectionPool>, options: crate::types::config::ClaudeAgentOptions) -> Self {
        Self {
            pool,
            guard: None,
            options,
            ready: false,
        }
    }

    /// Take the stdout reader for streaming (for bidirectional mode)
    pub fn take_stdout(&mut self) -> Option<Arc<Mutex<tokio::io::BufReader<tokio::process::ChildStdout>>>> {
        self.guard.as_ref().and_then(|g| g.stdout())
    }
}

#[async_trait]
impl Transport for PooledTransport {
    async fn connect(&mut self) -> Result<()> {
        if self.ready {
            return Ok(());
        }

        // Initialize pool if needed
        if !self.pool.is_enabled() {
            return Err(ClaudeError::Connection(crate::errors::ConnectionError::new(
                "Connection pool is not enabled".to_string(),
            )));
        }

        // Acquire a worker from the pool
        let guard = self.pool.acquire().await?;
        self.guard = Some(guard);
        self.ready = true;

        Ok(())
    }

    async fn write(&mut self, data: &str) -> Result<()> {
        let guard = self.guard.as_mut().ok_or_else(|| {
            ClaudeError::Transport("Transport not connected".to_string())
        })?;
        guard.write(data).await
    }

    fn read_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>> {
        Box::pin(async_stream::stream! {
            loop {
                let guard = match self.guard.as_mut() {
                    Some(g) => g,
                    None => {
                        yield Err(ClaudeError::Transport("Transport not connected".to_string()));
                        break;
                    }
                };

                let mut line = String::new();
                match guard.read_line(&mut line).await {
                    Ok(0) => {
                        // EOF
                        break;
                    }
                    Ok(_) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }

                        match serde_json::from_str::<serde_json::Value>(trimmed) {
                            Ok(json) => {
                                yield Ok(json);
                            }
                            Err(e) => {
                                yield Err(ClaudeError::Transport(format!(
                                    "Failed to parse JSON: {}",
                                    e
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(e);
                        break;
                    }
                }
            }
        })
    }

    async fn close(&mut self) -> Result<()> {
        // Drop the guard to return worker to pool
        self.guard = None;
        self.ready = false;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    async fn end_input(&mut self) -> Result<()> {
        // For pooled transport, we don't actually close stdin
        // The worker is returned to the pool for reuse
        // This is a no-op since pooled workers stay alive
        Ok(())
    }
}

/// Initialize the global connection pool
///
/// This should be called before creating any clients that use pooling.
/// If not called, the first client will initialize the pool lazily.
pub async fn init_pool(config: PoolConfig, options: crate::types::config::ClaudeAgentOptions) -> Result<()> {
    init_global_pool(config, options).await
}

/// Get the global connection pool if initialized
pub async fn get_pool() -> Option<Arc<ConnectionPool>> {
    get_global_pool().await
}
