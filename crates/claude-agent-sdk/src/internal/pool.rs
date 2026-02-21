//! Connection pool for reusing CLI processes
//!
//! This module implements a connection pool that manages reusable CLI worker processes
//! to reduce the overhead of spawning new processes for each query.
//!
//! # Architecture
//!
//! The pool uses a channel-based distribution pattern:
//! 1. Workers are spawned and kept in a pool
//! 2. When a query arrives, an available worker is acquired
//! 3. After the query completes, the worker is returned to the pool
//! 4. Unhealthy workers are recycled and replaced
//!
//! # Performance Targets
//!
//! - Reduce query latency from ~300ms to <100ms by reusing processes
//! - Support concurrent queries with configurable pool size
//! - Automatic worker health monitoring and replacement

use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::time::timeout;

use crate::errors::{ClaudeError, ConnectionError, ProcessError, Result};
use crate::types::config::ClaudeAgentOptions;
use crate::version::{ENTRYPOINT, SDK_VERSION};

/// Default minimum pool size
pub const DEFAULT_MIN_POOL_SIZE: usize = 1;
/// Default maximum pool size
pub const DEFAULT_MAX_POOL_SIZE: usize = 10;
/// Default idle timeout for workers (seconds)
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 300; // 5 minutes
/// Default health check interval (seconds)
pub const DEFAULT_HEALTH_CHECK_INTERVAL_SECS: u64 = 60;
/// Worker acquisition timeout (seconds)
const ACQUIRE_TIMEOUT_SECS: u64 = 30;

/// Configuration for the connection pool
#[derive(Clone, Debug)]
pub struct PoolConfig {
    /// Minimum number of workers to maintain
    pub min_size: usize,
    /// Maximum number of workers allowed
    pub max_size: usize,
    /// Idle timeout before recycling a worker
    pub idle_timeout: Duration,
    /// Interval for health checks
    pub health_check_interval: Duration,
    /// Enable connection pooling
    pub enabled: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_size: DEFAULT_MIN_POOL_SIZE,
            max_size: DEFAULT_MAX_POOL_SIZE,
            idle_timeout: Duration::from_secs(DEFAULT_IDLE_TIMEOUT_SECS),
            health_check_interval: Duration::from_secs(DEFAULT_HEALTH_CHECK_INTERVAL_SECS),
            enabled: false, // Disabled by default for backward compatibility
        }
    }
}

impl PoolConfig {
    /// Create a new pool configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable the connection pool
    pub fn enabled(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Set minimum pool size
    pub fn min_size(mut self, size: usize) -> Self {
        self.min_size = size;
        self
    }

    /// Set maximum pool size
    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }

    /// Set idle timeout
    pub fn idle_timeout(mut self, duration: Duration) -> Self {
        self.idle_timeout = duration;
        self
    }
}

/// A pooled worker that wraps a CLI process
struct PooledWorker {
    /// Worker ID for tracking
    id: usize,
    /// The CLI process
    process: Child,
    /// Stdin for writing to the process
    stdin: ChildStdin,
    /// Stdout reader (wrapped in Arc<Mutex> for sharing)
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    /// Last activity timestamp
    last_activity: std::time::Instant,
    /// Whether this worker is healthy
    healthy: bool,
}

impl PooledWorker {
    /// Create a new pooled worker
    async fn new(id: usize, options: &ClaudeAgentOptions) -> Result<Self> {
        let (process, stdin, stdout) = Self::spawn_process(options).await?;

        Ok(Self {
            id,
            process,
            stdin,
            stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
            last_activity: std::time::Instant::now(),
            healthy: true,
        })
    }

    /// Spawn a new CLI process
    async fn spawn_process(
        options: &ClaudeAgentOptions,
    ) -> Result<(Child, ChildStdin, ChildStdout)> {
        use std::process::Stdio;

        let cli_path = if let Some(ref path) = options.cli_path {
            path.clone()
        } else {
            // Use the existing CLI finding logic
            return Err(ClaudeError::Connection(ConnectionError::new(
                "CLI path must be specified for pooled connections".to_string(),
            )));
        };

        // Build environment
        let mut env = options.env.clone();
        env.insert("CLAUDE_CODE_ENTRYPOINT".to_string(), ENTRYPOINT.to_string());
        env.insert(
            "CLAUDE_AGENT_SDK_VERSION".to_string(),
            SDK_VERSION.to_string(),
        );

        // Build command for streaming mode
        let mut cmd = Command::new(&cli_path);
        cmd.args(["--output-format", "stream-json", "--verbose", "--input-format", "stream-json"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // Suppress stderr for pooled workers
            .envs(&env);

        if let Some(ref cwd) = options.cwd {
            cmd.current_dir(cwd);
        }

        // Spawn process
        let mut child = cmd.spawn().map_err(|e| {
            ClaudeError::Process(ProcessError::new(
                format!("Failed to spawn CLI process for pool: {}", e),
                None,
                None,
            ))
        })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            ClaudeError::Connection(ConnectionError::new("Failed to get stdin".to_string()))
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            ClaudeError::Connection(ConnectionError::new("Failed to get stdout".to_string()))
        })?;

        Ok((child, stdin, stdout))
    }

    /// Check if the worker is still healthy
    fn is_healthy(&self) -> bool {
        self.healthy && self.process.id().is_some()
    }

    /// Update last activity timestamp
    fn touch(&mut self) {
        self.last_activity = std::time::Instant::now();
    }

    /// Check if worker has been idle too long
    fn is_idle_timeout(&self, timeout_dur: Duration) -> bool {
        self.last_activity.elapsed() > timeout_dur
    }

    /// Write data to the worker's stdin
    async fn write(&mut self, data: &str) -> Result<()> {
        self.stdin
            .write_all(data.as_bytes())
            .await
            .map_err(|e| ClaudeError::Transport(format!("Failed to write to pooled worker: {}", e)))?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|e| ClaudeError::Transport(format!("Failed to write newline: {}", e)))?;
        self.stdin
            .flush()
            .await
            .map_err(|e| ClaudeError::Transport(format!("Failed to flush pooled worker: {}", e)))?;
        self.touch();
        Ok(())
    }

    /// Read a line from the worker's stdout
    async fn read_line(&mut self, line: &mut String) -> Result<usize> {
        let mut stdout = self.stdout.lock().await;
        let n = stdout
            .read_line(line)
            .await
            .map_err(|e| ClaudeError::Transport(format!("Failed to read from pooled worker: {}", e)))?;
        drop(stdout); // Release lock before touching
        self.touch();
        Ok(n)
    }
}

impl Drop for PooledWorker {
    fn drop(&mut self) {
        if let Some(pid) = self.process.id() {
            tracing::debug!("Dropping pooled worker with PID {}", pid);
            let _ = self.process.start_kill();
        }
    }
}

/// A guard that returns the worker to the pool when dropped
pub struct WorkerGuard {
    worker: Option<PooledWorker>,
    return_tx: mpsc::Sender<PooledWorker>,
    _permit: Option<tokio::sync::OwnedSemaphorePermit>,
}

impl WorkerGuard {
    /// Write data to the worker
    pub async fn write(&mut self, data: &str) -> Result<()> {
        if let Some(ref mut worker) = self.worker {
            worker.write(data).await
        } else {
            Err(ClaudeError::Transport("Worker not available".to_string()))
        }
    }

    /// Read a line from the worker
    pub async fn read_line(&mut self, line: &mut String) -> Result<usize> {
        if let Some(ref mut worker) = self.worker {
            worker.read_line(line).await
        } else {
            Err(ClaudeError::Transport("Worker not available".to_string()))
        }
    }

    /// Get the stdout reader for streaming
    pub fn stdout(&self) -> Option<Arc<Mutex<BufReader<ChildStdout>>>> {
        self.worker.as_ref().map(|w| Arc::clone(&w.stdout))
    }
}

impl Drop for WorkerGuard {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            // Try to return the worker to the pool (non-blocking)
            let _ = self.return_tx.try_send(worker);
        }
        // Permit is released when _permit is dropped
    }
}

/// The connection pool for managing CLI worker processes
pub struct ConnectionPool {
    /// Pool configuration
    config: PoolConfig,
    /// SDK options for spawning workers
    options: ClaudeAgentOptions,
    /// Channel for returning workers to the pool
    return_tx: mpsc::Sender<PooledWorker>,
    /// Channel for receiving returned workers (stored in mutex for interior mutability)
    return_rx: Mutex<mpsc::Receiver<PooledWorker>>,
    /// Semaphore for limiting concurrent workers
    semaphore: Arc<Semaphore>,
    /// Counter for worker IDs
    next_worker_id: Mutex<usize>,
    /// Pool state
    state: Mutex<PoolState>,
}

struct PoolState {
    /// Total workers created
    total_created: usize,
    /// Active workers
    active_count: usize,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: PoolConfig, options: ClaudeAgentOptions) -> Self {
        let (return_tx, return_rx) = mpsc::channel(config.max_size);
        let semaphore = Arc::new(Semaphore::new(config.max_size));

        Self {
            config,
            options,
            return_tx,
            return_rx: Mutex::new(return_rx),
            semaphore,
            next_worker_id: Mutex::new(0),
            state: Mutex::new(PoolState {
                total_created: 0,
                active_count: 0,
            }),
        }
    }

    /// Initialize the pool with minimum workers
    pub async fn initialize(&self) -> Result<()> {
        for _ in 0..self.config.min_size {
            let worker = self.create_worker().await?;
            let _ = self.return_tx.try_send(worker);
        }
        Ok(())
    }

    /// Create a new worker
    async fn create_worker(&self) -> Result<PooledWorker> {
        let id = {
            let mut guard = self.next_worker_id.lock().await;
            *guard += 1;
            *guard
        };

        let worker = PooledWorker::new(id, &self.options).await?;

        let mut state = self.state.lock().await;
        state.total_created += 1;
        state.active_count += 1;

        tracing::debug!("Created pooled worker {} (total: {}, active: {})",
            id, state.total_created, state.active_count);

        Ok(worker)
    }

    /// Acquire a worker from the pool
    pub async fn acquire(&self) -> Result<WorkerGuard> {
        // Try to acquire with timeout
        let permit = timeout(
            Duration::from_secs(ACQUIRE_TIMEOUT_SECS),
            Arc::clone(&self.semaphore).acquire_owned(),
        )
        .await
        .map_err(|_| {
            ClaudeError::Connection(ConnectionError::new(
                "Timeout acquiring worker from pool".to_string(),
            ))
        })?
        .map_err(|e| {
            ClaudeError::Connection(ConnectionError::new(format!(
                "Failed to acquire semaphore: {}",
                e
            )))
        })?;

        // Try to get a worker from the return channel
        let worker = {
            let mut rx = self.return_rx.lock().await;
            match rx.try_recv() {
                Ok(worker) => {
                    if worker.is_healthy() && !worker.is_idle_timeout(self.config.idle_timeout) {
                        Some(worker)
                    } else {
                        // Worker is unhealthy or timed out, create new one
                        tracing::debug!("Recycling unhealthy/timed-out worker {}", worker.id);
                        None
                    }
                }
                Err(_) => None,
            }
        };

        // If no worker available, create a new one
        let worker = match worker {
            Some(w) => w,
            None => self.create_worker().await?,
        };

        Ok(WorkerGuard {
            worker: Some(worker),
            return_tx: self.return_tx.clone(),
            _permit: Some(permit),
        })
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let state = self.state.lock().await;
        PoolStats {
            total_created: state.total_created,
            active_count: state.active_count,
            available_permits: self.semaphore.available_permits(),
        }
    }

    /// Check if the pool is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Total workers created
    pub total_created: usize,
    /// Currently active workers
    pub active_count: usize,
    /// Available permits (slots for new workers)
    pub available_permits: usize,
}

/// Global connection pool singleton
static POOL: std::sync::OnceLock<Arc<Mutex<Option<Arc<ConnectionPool>>>>> = std::sync::OnceLock::new();

fn get_pool_singleton() -> &'static Arc<Mutex<Option<Arc<ConnectionPool>>>> {
    POOL.get_or_init(|| Arc::new(Mutex::new(None)))
}

/// Initialize the global connection pool
pub async fn init_global_pool(config: PoolConfig, options: ClaudeAgentOptions) -> Result<()> {
    let pool = Arc::new(ConnectionPool::new(config, options));

    if pool.is_enabled() {
        pool.initialize().await?;
    }

    let global = get_pool_singleton();
    let mut guard = global.lock().await;
    *guard = Some(pool);

    Ok(())
}

/// Get the global connection pool
pub async fn get_global_pool() -> Option<Arc<ConnectionPool>> {
    let global = get_pool_singleton();
    let guard = global.lock().await;
    guard.clone()
}

/// Shutdown the global connection pool
pub async fn shutdown_global_pool() {
    let global = get_pool_singleton();
    let mut guard = global.lock().await;
    *guard = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.min_size, DEFAULT_MIN_POOL_SIZE);
        assert_eq!(config.max_size, DEFAULT_MAX_POOL_SIZE);
        assert!(!config.enabled);
    }

    #[test]
    fn test_pool_config_builder() {
        let config = PoolConfig::new()
            .enabled()
            .min_size(2)
            .max_size(5);

        assert!(config.enabled);
        assert_eq!(config.min_size, 2);
        assert_eq!(config.max_size, 5);
    }
}
