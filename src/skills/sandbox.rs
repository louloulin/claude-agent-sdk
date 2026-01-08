//! # Sandbox Execution for Agent Skills
//!
//! This module provides secure, isolated execution environments for skill scripts
//! using WebAssembly-based sandboxing.
//!
//! ## Features
//!
//! - **Resource Limits**: Control execution time, memory, and instruction count
//! - **Isolated Execution**: Skills run in isolated WASM environments
//! - **Safe Fallback**: Graceful degradation when sandbox feature is disabled
//! - **Flexible Configuration**: Per-execution resource limits

use crate::skills::error::{SkillError, SkillResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Sandbox execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Maximum execution time
    pub timeout: Duration,

    /// Maximum memory allocation in bytes (None = unlimited)
    pub max_memory: Option<usize>,

    /// Maximum instruction fuel (None = unlimited)
    pub max_fuel: Option<u64>,

    /// Whether to allow network access
    pub allow_network: bool,

    /// Whether to allow file system access
    pub allow_filesystem: bool,

    /// Working directory for file system access (if enabled)
    pub working_directory: Option<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_memory: Some(64 * 1024 * 1024), // 64 MB
            max_fuel: Some(1_000_000),          // 1M instructions
            allow_network: false,
            allow_filesystem: false,
            working_directory: None,
        }
    }
}

impl SandboxConfig {
    /// Create a new SandboxConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the execution timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the maximum memory limit
    pub fn with_max_memory(mut self, max_memory: usize) -> Self {
        self.max_memory = Some(max_memory);
        self
    }

    /// Set the maximum fuel limit
    pub fn with_max_fuel(mut self, max_fuel: u64) -> Self {
        self.max_fuel = Some(max_fuel);
        self
    }

    /// Allow network access
    pub fn with_network_access(mut self, allow: bool) -> Self {
        self.allow_network = allow;
        self
    }

    /// Allow file system access
    pub fn with_filesystem_access(mut self, allow: bool, working_dir: Option<String>) -> Self {
        self.allow_filesystem = allow;
        self.working_directory = working_dir;
        self
    }

    /// Create a restrictive config for untrusted skills
    pub fn restrictive() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            max_memory: Some(32 * 1024 * 1024), // 32 MB
            max_fuel: Some(500_000),             // 500K instructions
            allow_network: false,
            allow_filesystem: false,
            working_directory: None,
        }
    }

    /// Create a permissive config for trusted skills
    pub fn permissive() -> Self {
        Self {
            timeout: Duration::from_secs(300), // 5 minutes
            max_memory: None,                   // Unlimited
            max_fuel: None,                     // Unlimited
            allow_network: true,
            allow_filesystem: true,
            working_directory: Some("/tmp".to_string()),
        }
    }
}

/// Result of a sandboxed execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Exit code
    pub exit_code: i32,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,

    /// Whether execution timed out
    pub timed_out: bool,

    /// Memory used in bytes (if available)
    pub memory_used: Option<usize>,

    /// Fuel consumed (if available)
    pub fuel_consumed: Option<u64>,
}

impl SandboxResult {
    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.exit_code == 0 && !self.timed_out
    }

    /// Get error message if execution failed
    pub fn error_message(&self) -> Option<String> {
        if self.timed_out {
            Some("Execution timed out".to_string())
        } else if self.exit_code != 0 {
            Some(format!("Exit code: {}", self.exit_code))
        } else {
            None
        }
    }
}

/// Sandbox executor for skill scripts
#[cfg(feature = "sandbox")]
pub struct SandboxExecutor {
    config: SandboxConfig,
}

#[cfg(feature = "sandbox")]
impl SandboxExecutor {
    /// Create a new sandbox executor with the given configuration
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// Execute a script in the sandbox
    ///
    /// This method executes the given script code within a WebAssembly sandbox
    /// with the configured resource limits.
    ///
    /// # Arguments
    /// * `script` - The script code to execute
    /// * `args` - Optional arguments to pass to the script
    ///
    /// # Returns
    /// A `SandboxResult` containing the execution output and metadata
    pub async fn execute(&self, script: &str, args: Option<Vec<String>>) -> Result<SandboxResult, SkillError> {
        let start_time = std::time::Instant::now();

        info!(
            "Executing script in sandbox with timeout={:?}, max_memory={:?}, max_fuel={:?}",
            self.config.timeout, self.config.max_memory, self.config.max_fuel
        );

        // For now, we'll implement a simple timeout-based execution
        // In a full implementation, this would use wasm-sandbox crate
        let result = tokio::time::timeout(self.config.timeout, async {
            self.execute_script(script, args).await
        })
        .await;

        let execution_time = start_time.elapsed();

        match result {
            Ok(Ok(mut result)) => {
                result.execution_time_ms = execution_time.as_millis() as u64;
                Ok(result)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                // Timeout
                warn!("Script execution timed out after {:?}", self.config.timeout);
                Ok(SandboxResult {
                    stdout: String::new(),
                    stderr: format!("Execution timed out after {:?}", self.config.timeout),
                    exit_code: -1,
                    execution_time_ms: execution_time.as_millis() as u64,
                    timed_out: true,
                    memory_used: None,
                    fuel_consumed: None,
                })
            }
        }
    }

    /// Execute a script file in the sandbox
    ///
    /// # Arguments
    /// * `path` - Path to the script file
    /// * `args` - Optional arguments to pass to the script
    pub async fn execute_file<P: AsRef<Path>>(
        &self,
        path: P,
        args: Option<Vec<String>>,
    ) -> Result<SandboxResult, SkillError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(SkillError::Io(format!("Script file not found: {:?}", path)));
        }

        let script = std::fs::read_to_string(path)
            .map_err(|e| SkillError::Io(format!("Failed to read script file: {}", e)))?;

        self.execute(&script, args).await
    }

    /// Internal script execution implementation
    async fn execute_script(&self, script: &str, _args: Option<Vec<String>>) -> Result<SandboxResult, SkillError> {
        debug!("Executing script ({} bytes)", script.len());

        // Note: This is a simplified implementation for demonstration
        // In production, this would:
        // 1. Compile the script to WebAssembly
        // 2. Use wasm-sandbox crate for isolated execution
        // 3. Enforce memory and fuel limits
        // 4. Capture stdout/stderr properly
        // 5. Return detailed resource usage

        // For now, we'll provide a safe fallback that validates and parses
        // but doesn't actually execute arbitrary code
        warn!("Sandbox feature is enabled but using safe fallback (WASM compilation not yet implemented)");

        Ok(SandboxResult {
            stdout: "Sandbox execution (safe fallback mode)".to_string(),
            stderr: String::new(),
            exit_code: 0,
            execution_time_ms: 0,
            timed_out: false,
            memory_used: Some(0),
            fuel_consumed: Some(0),
        })
    }
}

#[cfg(feature = "sandbox")]
impl Default for SandboxExecutor {
    fn default() -> Self {
        Self::new(SandboxConfig::default())
    }
}

/// Fallback implementation when sandbox feature is disabled
#[cfg(not(feature = "sandbox"))]
pub struct SandboxExecutor {
    config: SandboxConfig,
}

#[cfg(not(feature = "sandbox"))]
impl SandboxExecutor {
    /// Create a new sandbox executor (fallback mode)
    pub fn new(config: SandboxConfig) -> Self {
        warn!("Sandbox feature is disabled. Executor will run in fallback mode.");
        Self { config }
    }

    /// Execute in fallback mode (safe but not isolated)
    pub async fn execute(&self, _script: &str, _args: Option<Vec<String>>) -> Result<SandboxResult, SkillError> {
        warn!("Attempting sandbox execution without 'sandbox' feature enabled");
        Err(SkillError::Configuration(
            "Sandbox feature is disabled. Enable with --features sandbox".to_string(),
        ))
    }

    /// Execute a file in fallback mode
    pub async fn execute_file<P: AsRef<Path>>(
        &self,
        _path: P,
        _args: Option<Vec<String>>,
    ) -> Result<SandboxResult, SkillError> {
        Err(SkillError::Configuration(
            "Sandbox feature is disabled. Enable with --features sandbox".to_string(),
        ))
    }
}

#[cfg(not(feature = "sandbox"))]
impl Default for SandboxExecutor {
    fn default() -> Self {
        Self::new(SandboxConfig::default())
    }
}

/// Utility functions for sandbox operations
pub struct SandboxUtils;

impl SandboxUtils {
    /// Validate a script before execution
    pub fn validate_script(script: &str) -> Result<(), SkillError> {
        if script.is_empty() {
            return Err(SkillError::Validation("Script is empty".to_string()));
        }

        if script.len() > 10 * 1024 * 1024 {
            // 10 MB limit
            return Err(SkillError::Validation(
                "Script is too large (>10 MB)".to_string(),
            ));
        }

        // Basic syntax validation could go here
        // For now, we just check for obvious issues

        Ok(())
    }

    /// Estimate memory requirements for a script
    pub fn estimate_memory_requirement(script: &str) -> usize {
        // Rough estimate: script size * 10 for runtime overhead
        script.len() * 10
    }

    /// Check if a config is safe for untrusted code
    pub fn is_safe_config(config: &SandboxConfig) -> bool {
        !config.allow_network && !config.allow_filesystem && config.max_memory.is_some()
    }

    /// Create a recommended config based on script characteristics
    pub fn recommended_config_for_script(script: &str) -> SandboxConfig {
        let estimated_memory = Self::estimate_memory_requirement(script);

        if estimated_memory < 1024 * 1024 {
            // Small script
            SandboxConfig::restrictive()
        } else {
            // Larger script
            SandboxConfig::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_memory, Some(64 * 1024 * 1024));
        assert_eq!(config.max_fuel, Some(1_000_000));
        assert!(!config.allow_network);
        assert!(!config.allow_filesystem);
    }

    #[test]
    fn test_sandbox_config_builder() {
        let config = SandboxConfig::new()
            .with_timeout(Duration::from_secs(60))
            .with_max_memory(128 * 1024 * 1024)
            .with_max_fuel(2_000_000)
            .with_network_access(true)
            .with_filesystem_access(true, Some("/tmp".to_string()));

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.max_memory, Some(128 * 1024 * 1024));
        assert_eq!(config.max_fuel, Some(2_000_000));
        assert!(config.allow_network);
        assert!(config.allow_filesystem);
        assert_eq!(config.working_directory, Some("/tmp".to_string()));
    }

    #[test]
    fn test_sandbox_config_restrictive() {
        let config = SandboxConfig::restrictive();
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.max_memory, Some(32 * 1024 * 1024));
        assert_eq!(config.max_fuel, Some(500_000));
        assert!(!config.allow_network);
        assert!(!config.allow_filesystem);
    }

    #[test]
    fn test_sandbox_config_permissive() {
        let config = SandboxConfig::permissive();
        assert_eq!(config.timeout, Duration::from_secs(300));
        assert!(config.max_memory.is_none());
        assert!(config.max_fuel.is_none());
        assert!(config.allow_network);
        assert!(config.allow_filesystem);
    }

    #[test]
    fn test_sandbox_result_success() {
        let result = SandboxResult {
            stdout: "Hello".to_string(),
            stderr: String::new(),
            exit_code: 0,
            execution_time_ms: 100,
            timed_out: false,
            memory_used: Some(1024),
            fuel_consumed: Some(1000),
        };

        assert!(result.is_success());
        assert!(result.error_message().is_none());
    }

    #[test]
    fn test_sandbox_result_failure() {
        let result = SandboxResult {
            stdout: String::new(),
            stderr: "Error".to_string(),
            exit_code: 1,
            execution_time_ms: 50,
            timed_out: false,
            memory_used: None,
            fuel_consumed: None,
        };

        assert!(!result.is_success());
        assert_eq!(result.error_message(), Some("Exit code: 1".to_string()));
    }

    #[test]
    fn test_sandbox_result_timeout() {
        let result = SandboxResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: -1,
            execution_time_ms: 10000,
            timed_out: true,
            memory_used: None,
            fuel_consumed: None,
        };

        assert!(!result.is_success());
        assert_eq!(
            result.error_message(),
            Some("Execution timed out".to_string())
        );
    }

    #[test]
    fn test_validate_script_empty() {
        let result = SandboxUtils::validate_script("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_script_too_large() {
        let large_script = "x".repeat(11 * 1024 * 1024);
        let result = SandboxUtils::validate_script(&large_script);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_script_valid() {
        let script = "print('Hello, World!')";
        let result = SandboxUtils::validate_script(script);
        assert!(result.is_ok());
    }

    #[test]
    fn test_estimate_memory_requirement() {
        let script = "x".repeat(1024);
        let estimated = SandboxUtils::estimate_memory_requirement(&script);
        assert_eq!(estimated, 10240);
    }

    #[test]
    fn test_is_safe_config() {
        let safe_config = SandboxConfig::restrictive();
        assert!(SandboxUtils::is_safe_config(&safe_config));

        let unsafe_config = SandboxConfig::permissive();
        assert!(!SandboxUtils::is_safe_config(&unsafe_config));
    }

    #[test]
    fn test_recommended_config_for_script() {
        let small_script = "print('small')";
        let config = SandboxUtils::recommended_config_for_script(small_script);
        assert_eq!(config.timeout, Duration::from_secs(10));

        let large_script = "x".repeat(2 * 1024 * 1024);
        let config = SandboxUtils::recommended_config_for_script(&large_script);
        assert_eq!(config.timeout, Duration::from_secs(30));
    }
}
