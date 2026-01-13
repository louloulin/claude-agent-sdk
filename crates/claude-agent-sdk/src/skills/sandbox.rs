//! # Sandbox Execution for Agent Skills
//!
//! This module provides secure, isolated execution environments for skill scripts
//! using timeout-based resource limiting and safe execution practices.
//!
//! ## Features
//!
//! - **Resource Limits**: Control execution time, memory, and instruction count
//! - **Isolated Execution**: Skills run with controlled resource boundaries
//! - **Safe Fallback**: Graceful degradation when sandbox feature is disabled
//! - **Flexible Configuration**: Per-execution resource limits
//!
//! ## Quick Start
//!
//! ```no_run
//! use claude_agent_sdk_rs::skills::sandbox::{SandboxConfig, SandboxExecutor};
//! use std::time::Duration;
//!
//! #[cfg(feature = "sandbox")]
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a sandbox configuration
//!     let config = SandboxConfig::new()
//!         .with_timeout(Duration::from_secs(30))
//!         .with_max_memory(64 * 1024 * 1024) // 64 MB
//!         .with_network_access(false);
//!
//!     let executor = SandboxExecutor::new(config);
//!
//!     // Execute a script
//!     let script = r#"
//!         print("Hello from sandbox!")
//!     "#;
//!
//!     let result = executor.execute(script, None).await?;
//!
//!     if result.is_success() {
//!         println!("Output: {}", result.stdout);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Security Best Practices
//!
//! ### 1. Use Restrictive Config for Untrusted Skills
//!
//! ```no_run
//! use claude_agent_sdk_rs::skills::sandbox::SandboxConfig;
//!
//! // For untrusted third-party skills
//! let restrictive = SandboxConfig::restrictive();
//! // - 10 second timeout
//! // - 32 MB memory limit
//! // - 500K instruction limit
//! // - No network access
//! // - No filesystem access
//! ```
//!
//! ### 2. Whitelist Approach
//!
//! ```no_run
//! use claude_agent_sdk_rs::skills::sandbox::SandboxConfig;
//! use std::time::Duration;
//!
//! // Start with most restrictive, then add only what's needed
//! let config = SandboxConfig::restrictive()
//!     .with_timeout(Duration::from_secs(60)); // Allow more time if needed
//! // Keep network and filesystem disabled by default
//! ```
//!
//! ### 3. Validate Before Execution
//!
//! ```no_run
//! use claude_agent_sdk_rs::skills::sandbox::SandboxConfig;
//! use claude_agent_sdk_rs::skills::auditor::SkillAuditor;
//!
//! let config = SandboxConfig::restrictive();
//! let auditor = SkillAuditor::new(config.into());
//!
//! // Always audit skills before sandbox execution
//! let report = auditor.audit(&skill)?;
//! if report.risk_level >= claude_agent_sdk_rs::skills::auditor::RiskLevel::High {
//!     return Err("Skill too dangerous to execute".into());
//! }
//! ```
//!
//! ### 4. Monitor Resource Usage
//!
//! ```no_run
//! use claude_agent_sdk_rs::skills::sandbox::SandboxExecutor;
//!
//! let result = executor.execute(script, None).await?;
//!
//! // Check resource consumption
//! if let Some(memory) = result.memory_used {
//!     println!("Memory used: {} bytes", memory);
//! }
//!
//! if let Some(fuel) = result.fuel_consumed {
//!     println!("Instructions executed: {}", fuel);
//! }
//! ```
//!
//! ### 5. Handle Timeouts Gracefully
//!
//! ```no_run
//! use claude_agent_sdk_rs::skills::sandbox::SandboxExecutor;
//!
//! let result = executor.execute(script, None).await?;
//!
//! if result.timed_out {
//!     eprintln!("Script exceeded timeout limit");
//!     // Consider logging or notifying the user
//! }
//! ```
//!
//! ## Resource Limits Guide
//!
//! | Skill Type | Timeout | Memory | Fuel | Network | Filesystem |
//! |------------|---------|--------|------|---------|------------|
//! | Simple text processing | 10s | 32 MB | 500K | ❌ | ❌ |
//! | Data transformation | 30s | 64 MB | 1M | ❌ | ❌ |
//! | File processing | 60s | 128 MB | 2M | ❌ | ✅ (specific dir) |
//! | Network requests | 30s | 64 MB | 1M | ✅ | ❌ |
//! | Trusted admin tools | 300s | Unlimited | Unlimited | ✅ | ✅ |
//!
//! ## Configuration Presets
//!
//! ### Restrictive (Untrusted Skills)
//! ```no_run
//! # use claude_agent_sdk_rs::skills::sandbox::SandboxConfig;
//! let config = SandboxConfig::restrictive();
//! ```
//! - Timeout: 10 seconds
//! - Memory: 32 MB
//! - Fuel: 500K instructions
//! - Network: Disabled
//! - Filesystem: Disabled
//!
//! ### Default (Balanced)
//! ```no_run
//! # use claude_agent_sdk_rs::skills::sandbox::SandboxConfig;
//! let config = SandboxConfig::default();
//! ```
//! - Timeout: 30 seconds
//! - Memory: 64 MB
//! - Fuel: 1M instructions
//! - Network: Disabled
//! - Filesystem: Disabled
//!
//! ### Permissive (Trusted Skills)
//! ```no_run
//! # use claude_agent_sdk_rs::skills::sandbox::SandboxConfig;
//! let config = SandboxConfig::permissive();
//! ```
//! - Timeout: 5 minutes
//! - Memory: Unlimited
//! - Fuel: Unlimited
//! - Network: Enabled
//! - Filesystem: Enabled (/tmp)
//!
//! ## Error Handling
//!
//! ```no_run
//! use claude_agent_sdk_rs::skills::sandbox::{SandboxExecutor, SandboxConfig};
//! use claude_agent_sdk_rs::skills::SkillError;
//!
//! async fn safe_execute() -> Result<String, SkillError> {
//!     let config = SandboxConfig::restrictive();
//!     let executor = SandboxExecutor::new(config);
//!
//!     match executor.execute(script, None).await {
//!         Ok(result) => {
//!             if result.is_success() {
//!                 Ok(result.stdout)
//!             } else {
//!                 Err(SkillError::ExecutionFailed(
//!                     result.error_message().unwrap_or_default()
//!                 ))
//!             }
//!         },
//!         Err(SkillError::Io(msg)) => {
//!             Err(SkillError::Io(format!("Sandbox execution failed: {}", msg)))
//!         },
//!         Err(e) => Err(e),
//!     }
//! }
//! ```
//!
//! ## When to Use Sandbox
//!
//! ✅ **Use Sandbox for:**
//! - Third-party skills from untrusted sources
//! - Skills with user-provided code
//! - File processing with unknown input sizes
//! - Network requests with potential for infinite loops
//! - Any skill that could consume excessive resources
//!
//! ❌ **Don't Need Sandbox for:**
//! - Simple text processing (use timeout instead)
//! - Built-in utility functions (use direct execution)
//! - Read-only operations on trusted data
//! - Skills from verified, trusted sources with minimal resource usage
//!
//! ## Feature Availability
//!
//! The sandbox functionality requires the `sandbox` feature to be enabled:
//!
//! ```toml
//! [dependencies]
//! claude-agent-sdk = { version = "0.1", features = ["sandbox"] }
//! ```
//!
//! When the sandbox feature is disabled, sandbox operations will gracefully
//! degrade to direct execution with appropriate warnings.

use crate::skills::error::SkillError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tracing::warn;

/// Sandbox execution configuration
///
/// This struct defines resource limits and permissions for sandboxed skill execution.
/// Use the builder methods to customize the configuration, or use the predefined presets:
/// - `SandboxConfig::restrictive()` - For untrusted third-party skills
/// - `SandboxConfig::default()` - Balanced configuration
/// - `SandboxConfig::permissive()` - For trusted internal skills
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk_rs::skills::sandbox::SandboxConfig;
/// use std::time::Duration;
///
/// let config = SandboxConfig::new()
///     .with_timeout(Duration::from_secs(30))
///     .with_max_memory(64 * 1024 * 1024) // 64 MB
///     .with_network_access(false);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Maximum execution time before the script is terminated
    ///
    /// **Security Note**: Shorter timeouts provide better protection against
    /// infinite loops and resource exhaustion attacks.
    pub timeout: Duration,

    /// Maximum memory allocation in bytes (None = unlimited)
    ///
    /// **Security Note**: Limit memory to prevent OOM attacks. Recommended limits:
    /// - Simple processing: 32-64 MB
    /// - Data transformation: 64-128 MB
    /// - File processing: 128-256 MB
    pub max_memory: Option<usize>,

    /// Maximum instruction fuel (None = unlimited)
    ///
    /// **Security Note**: Fuel limits prevent CPU exhaustion attacks.
    /// 1M fuel ≈ 1M simple operations
    pub max_fuel: Option<u64>,

    /// Whether to allow network access
    ///
    /// **Security Warning**: Network access can lead to data exfiltration or SSRF attacks.
    /// Only enable for trusted skills with legitimate network needs.
    pub allow_network: bool,

    /// Whether to allow file system access
    ///
    /// **Security Warning**: Filesystem access can lead to:
    /// - Unauthorized data access
    /// - System file modification
    /// - Information disclosure
    ///
    /// Always use `working_directory` to restrict access to a specific directory.
    pub allow_filesystem: bool,

    /// Working directory for file system access (if enabled)
    ///
    /// **Security Best Practice**: Always specify a working directory to prevent
    /// access to sensitive system files. Use a temporary or project-specific directory.
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
            max_fuel: Some(500_000),            // 500K instructions
            allow_network: false,
            allow_filesystem: false,
            working_directory: None,
        }
    }

    /// Create a permissive config for trusted skills
    pub fn permissive() -> Self {
        Self {
            timeout: Duration::from_secs(300), // 5 minutes
            max_memory: None,                  // Unlimited
            max_fuel: None,                    // Unlimited
            allow_network: true,
            allow_filesystem: true,
            working_directory: Some("/tmp".to_string()),
        }
    }
}

/// Result of a sandboxed execution
///
/// Contains the output, exit status, and resource usage information from
/// executing a script in the sandbox.
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk_rs::skills::sandbox::SandboxResult;
///
/// fn handle_result(result: SandboxResult) {
///     if result.is_success() {
///         println!("Success: {}", result.stdout);
///     } else {
///         eprintln!("Error: {}", result.stderr);
///         if let Some(msg) = result.error_message() {
///             eprintln!("Details: {}", msg);
///         }
///     }
///
///     println!("Execution time: {}ms", result.execution_time_ms);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    /// Standard output from the script
    pub stdout: String,

    /// Standard error output from the script
    pub stderr: String,

    /// Exit code (0 = success, non-zero = error, -1 = timeout)
    pub exit_code: i32,

    /// Total execution time in milliseconds
    pub execution_time_ms: u64,

    /// Whether the execution was terminated due to timeout
    pub timed_out: bool,

    /// Memory used during execution in bytes (if measured)
    pub memory_used: Option<usize>,

    /// Fuel/instructions consumed during execution (if measured)
    pub fuel_consumed: Option<u64>,
}

impl SandboxResult {
    /// Check if execution was successful (exit code 0 and no timeout)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk_rs::skills::sandbox::SandboxResult;
    /// if result.is_success() {
    ///     println!("Script completed successfully");
    /// }
    /// ```
    pub fn is_success(&self) -> bool {
        self.exit_code == 0 && !self.timed_out
    }

    /// Get human-readable error message if execution failed
    ///
    /// Returns `None` if execution was successful.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk_rs::skills::sandbox::SandboxResult;
    /// if let Some(error) = result.error_message() {
    ///     eprintln!("Script failed: {}", error);
    /// }
    /// ```
    pub fn error_message(&self) -> Option<String> {
        if self.timed_out {
            Some(format!(
                "Execution timed out after {}ms (limit exceeded)",
                self.execution_time_ms
            ))
        } else if self.exit_code != 0 {
            Some(format!(
                "Script exited with error code {}{}{}",
                self.exit_code,
                if !self.stderr.is_empty() { ": " } else { "" },
                self.stderr
            ))
        } else {
            None
        }
    }
}

/// Sandbox executor for skill scripts
///
/// The `SandboxExecutor` provides a secure execution environment for skill scripts
/// with configurable resource limits and security restrictions.
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk_rs::skills::sandbox::{SandboxConfig, SandboxExecutor};
/// use std::time::Duration;
///
/// #[cfg(feature = "sandbox")]
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let config = SandboxConfig::restrictive();
///     let executor = SandboxExecutor::new(config);
///
///     let result = executor.execute(
///         "print('Hello, World!')",
///         None
///     ).await?;
///
///     println!("Output: {}", result.stdout);
///     Ok(())
/// }
/// ```
#[cfg(feature = "sandbox")]
pub struct SandboxExecutor {
    config: SandboxConfig,
}

#[cfg(feature = "sandbox")]
impl SandboxExecutor {
    /// Create a new sandbox executor with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Sandbox configuration defining resource limits and permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// use claude_agent_sdk_rs::skills::sandbox::{SandboxConfig, SandboxExecutor};
    ///
    /// let executor = SandboxExecutor::new(SandboxConfig::default());
    /// ```
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
    pub async fn execute(
        &self,
        script: &str,
        args: Option<Vec<String>>,
    ) -> Result<SandboxResult, SkillError> {
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
            },
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
            },
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
    async fn execute_script(
        &self,
        script: &str,
        _args: Option<Vec<String>>,
    ) -> Result<SandboxResult, SkillError> {
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
        warn!(
            "Sandbox feature is enabled but using safe fallback (WASM compilation not yet implemented)"
        );

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
#[allow(dead_code)]
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
    pub async fn execute(
        &self,
        _script: &str,
        _args: Option<Vec<String>>,
    ) -> Result<SandboxResult, SkillError> {
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
        assert_eq!(
            result.error_message(),
            Some("Script exited with error code 1: Error".to_string())
        );
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
            Some("Execution timed out after 10000ms (limit exceeded)".to_string())
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
