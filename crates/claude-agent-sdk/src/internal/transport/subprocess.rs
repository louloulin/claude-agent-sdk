//! Subprocess transport implementation for Claude Code CLI

use async_trait::async_trait;
use futures::stream::Stream;
use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Stdio;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tracing::warn;

use crate::errors::{
    ClaudeError, CliNotFoundError, ConnectionError, JsonDecodeError, ProcessError, Result,
};
use crate::types::config::{ClaudeAgentOptions, DynamicBufferConfig};
use crate::types::messages::UserContentBlock;
use crate::version::{
    ENTRYPOINT, MIN_CLI_VERSION, SDK_VERSION, SKIP_VERSION_CHECK_ENV, check_version,
};

use super::Transport;

use crate::internal::cli_installer::{CliInstaller, InstallProgress};

/// Thread-safe buffer usage metrics using atomic operations.
///
/// This struct tracks buffer statistics without requiring locks, using
/// atomic operations for thread safety.
#[derive(Debug, Default)]
pub struct AtomicBufferMetrics {
    /// Peak buffer size used during the session
    peak_size: AtomicUsize,
    /// Total number of messages processed
    message_count: AtomicUsize,
    /// Total bytes processed
    total_bytes: AtomicUsize,
    /// Number of buffer resizes
    resize_count: AtomicUsize,
}

impl AtomicBufferMetrics {
    /// Create new metrics with zero values
    pub fn new() -> Self {
        Self::default()
    }

    /// Update peak size if the new value is larger
    pub fn update_peak(&self, size: usize) {
        use std::sync::atomic::Ordering;
        let mut current = self.peak_size.load(Ordering::Relaxed);
        while size > current {
            match self.peak_size.compare_exchange_weak(
                current,
                size,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }

    /// Increment message count
    pub fn inc_message_count(&self) {
        self.message_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Add bytes to total
    pub fn add_bytes(&self, bytes: usize) {
        self.total_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Increment resize count
    pub fn inc_resize_count(&self) {
        self.resize_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get snapshot of all metrics
    pub fn snapshot(&self) -> BufferMetricsSnapshot {
        BufferMetricsSnapshot {
            peak_size: self.peak_size.load(Ordering::Relaxed),
            message_count: self.message_count.load(Ordering::Relaxed),
            total_bytes: self.total_bytes.load(Ordering::Relaxed),
            resize_count: self.resize_count.load(Ordering::Relaxed),
        }
    }

    /// Reset all metrics to zero
    pub fn reset(&self) {
        self.peak_size.store(0, Ordering::Relaxed);
        self.message_count.store(0, Ordering::Relaxed);
        self.total_bytes.store(0, Ordering::Relaxed);
        self.resize_count.store(0, Ordering::Relaxed);
    }
}

/// Snapshot of buffer metrics at a point in time
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BufferMetricsSnapshot {
    /// Peak buffer size used during the session
    pub peak_size: usize,
    /// Total number of messages processed
    pub message_count: usize,
    /// Total bytes processed
    pub total_bytes: usize,
    /// Number of buffer resizes
    pub resize_count: usize,
}

impl BufferMetricsSnapshot {
    /// Get average message size
    pub fn average_message_size(&self) -> usize {
        if self.message_count == 0 {
            0
        } else {
            self.total_bytes / self.message_count
        }
    }
}

/// Query prompt type
#[derive(Clone)]
pub enum QueryPrompt {
    /// Text prompt (one-shot mode)
    Text(String),
    /// Structured content blocks (supports images and text)
    Content(Vec<UserContentBlock>),
    /// Streaming mode (no initial prompt)
    Streaming,
}

impl From<String> for QueryPrompt {
    fn from(text: String) -> Self {
        QueryPrompt::Text(text)
    }
}

impl From<&str> for QueryPrompt {
    fn from(text: &str) -> Self {
        QueryPrompt::Text(text.to_string())
    }
}

impl From<Vec<UserContentBlock>> for QueryPrompt {
    fn from(blocks: Vec<UserContentBlock>) -> Self {
        QueryPrompt::Content(blocks)
    }
}

/// Subprocess transport for communicating with Claude Code CLI
///
/// # Lock Optimization
///
/// This struct uses direct `Option<T>` for stdin/stdout instead of `Arc<Mutex<Option<T>>>`
/// because:
/// 1. The transport is owned by a single `InternalClient`
/// 2. All Transport trait methods take `&mut self`, providing exclusive access
/// 3. This eliminates lock contention on the hot path (read_messages/write)
///
/// The performance improvement is significant for high-frequency query scenarios:
/// - No lock acquisition overhead (~50-100ns per operation saved)
/// - No cache line bouncing between cores
/// - Simpler code with the same safety guarantees
///
/// For bidirectional control protocol (QueryFull), use `take_stdin_arc()` after connect()
/// to get a shared reference to stdin for concurrent writes.
///
/// # Dynamic Buffer Management
///
/// Uses adaptive buffer sizing that:
/// - Starts with a configurable initial size (default 64KB)
/// - Grows dynamically based on actual message sizes
/// - Tracks metrics for monitoring and tuning
/// - Protects against memory exhaustion with hard limits
pub struct SubprocessTransport {
    cli_path: PathBuf,
    cwd: Option<PathBuf>,
    options: ClaudeAgentOptions,
    prompt: QueryPrompt,
    process: Option<Child>,
    /// Direct stdin access - owned for simple query mode
    stdin: Option<ChildStdin>,
    /// Shared stdin for bidirectional mode - set when take_stdin_arc() is called
    stdin_arc: Option<Arc<Mutex<Option<ChildStdin>>>>,
    /// Direct stdout access - no lock needed as we have exclusive mutable access
    stdout: Option<BufReader<ChildStdout>>,
    /// Dynamic buffer configuration
    buffer_config: DynamicBufferConfig,
    /// Buffer usage metrics (thread-safe atomic counters)
    buffer_metrics: AtomicBufferMetrics,
    ready: bool,
}

impl SubprocessTransport {
    /// Create a new subprocess transport
    pub fn new(prompt: QueryPrompt, options: ClaudeAgentOptions) -> Result<Self> {
        // Validate cwd early, before CLI lookup, for better error messages
        if let Some(ref cwd) = options.cwd {
            if !cwd.exists() {
                return Err(ClaudeError::InvalidConfig(format!(
                    "Working directory does not exist: {}. Please ensure the directory exists before connecting.",
                    cwd.display()
                )));
            }
            if !cwd.is_dir() {
                return Err(ClaudeError::InvalidConfig(format!(
                    "Working directory path is not a directory: {}",
                    cwd.display()
                )));
            }
        }

        let cli_path = if let Some(ref path) = options.cli_path {
            path.clone()
        } else {
            // 尝试查找 CLI，如果失败且启用自动安装，则尝试安装
            Self::find_cli_with_auto_install(&options)?
        };

        let cwd = options.cwd.clone().or_else(|| std::env::current_dir().ok());

        // Resolve buffer configuration with backward compatibility
        let buffer_config = if let Some(ref config) = options.buffer_config {
            config.clone()
        } else if let Some(max_size) = options.max_buffer_size {
            // Backward compatibility: use max_buffer_size as max_message_size
            DynamicBufferConfig {
                max_message_size: max_size,
                ..DynamicBufferConfig::default()
            }
        } else {
            DynamicBufferConfig::default()
        };

        Ok(Self {
            cli_path,
            cwd,
            options,
            prompt,
            process: None,
            stdin: None,
            stdin_arc: None,
            stdout: None,
            buffer_config,
            buffer_metrics: AtomicBufferMetrics::new(),
            ready: false,
        })
    }

    /// Take stdin as Arc<Mutex> for bidirectional control protocol.
    ///
    /// This method transfers ownership of stdin to a shared reference that can be
    /// used for concurrent writes while the transport is reading messages.
    /// This is used by QueryFull for bidirectional communication.
    ///
    /// # Returns
    /// - `Some(Arc<Mutex<Option<ChildStdin>>>)` if stdin was available
    /// - `None` if stdin was already taken or not connected
    ///
    /// # Note
    /// After calling this method, direct stdin access via `write()` will fail
    /// because stdin ownership has been transferred to the shared reference.
    pub fn take_stdin_arc(&mut self) -> Option<Arc<Mutex<Option<ChildStdin>>>> {
        if let Some(stdin) = self.stdin.take() {
            let arc = Arc::new(Mutex::new(Some(stdin)));
            self.stdin_arc = Some(Arc::clone(&arc));
            Some(arc)
        } else {
            // Already taken, return the existing arc if available
            self.stdin_arc.clone()
        }
    }

    /// Get buffer usage metrics for monitoring and tuning.
    ///
    /// Returns metrics about buffer usage including peak size, message count,
    /// and resize operations. Only meaningful if `enable_metrics` is true in
    /// the buffer configuration.
    ///
    /// # Example
    /// ```ignore
    /// let metrics = transport.get_buffer_metrics();
    /// println!("Peak buffer size: {} bytes", metrics.peak_size);
    /// println!("Average message size: {} bytes", metrics.average_message_size());
    /// ```
    pub fn get_buffer_metrics(&self) -> BufferMetricsSnapshot {
        self.buffer_metrics.snapshot()
    }

    /// Reset buffer metrics.
    ///
    /// Useful when starting a new session to get fresh metrics.
    pub fn reset_buffer_metrics(&self) {
        self.buffer_metrics.reset();
    }

    /// Find the Claude CLI executable
    fn find_cli() -> Result<PathBuf> {
        // Strategy 1: Try executing 'claude' directly from PATH
        // This is the most reliable method as it respects the shell's PATH resolution
        if let Ok(output) = std::process::Command::new("claude")
            .arg("--version")
            .output()
            && output.status.success()
        {
            // 'claude' is in PATH and executable, return it as-is
            // The OS will resolve it when we spawn the process
            return Ok(PathBuf::from("claude"));
        }

        // Strategy 2: Use 'which' command to locate claude in PATH (Unix-like systems)
        #[cfg(not(target_os = "windows"))]
        if let Ok(output) = std::process::Command::new("which").arg("claude").output()
            && output.status.success()
        {
            let path_str = String::from_utf8_lossy(&output.stdout);
            let path = PathBuf::from(path_str.trim());
            // Verify the path exists and is executable
            if path.exists() && path.is_file() {
                return Ok(path);
            }
        }

        // Strategy 3: Use 'where' command on Windows
        #[cfg(target_os = "windows")]
        if let Ok(output) = std::process::Command::new("where").arg("claude").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                // 'where' returns all matches, take the first one
                if let Some(first_line) = path_str.lines().next() {
                    let path = PathBuf::from(first_line.trim());
                    if path.exists() && path.is_file() {
                        return Ok(path);
                    }
                }
            }
        }

        // Strategy 4: Check common installation locations
        // Get home directory for path expansion
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE")) // Windows fallback
            .ok()
            .map(PathBuf::from);

        // Common installation locations
        let mut common_paths: Vec<PathBuf> = vec![];

        // Unix-like paths
        #[cfg(not(target_os = "windows"))]
        {
            common_paths.extend(vec![
                PathBuf::from("/usr/local/bin/claude"),
                PathBuf::from("/opt/homebrew/bin/claude"),
                PathBuf::from("/usr/bin/claude"),
            ]);

            // Add home-relative paths if home directory is available
            if let Some(ref home) = home_dir {
                common_paths.push(home.join(".local/bin/claude"));
                common_paths.push(home.join("bin/claude"));
            }
        }

        // Windows paths
        #[cfg(target_os = "windows")]
        {
            if let Some(ref home) = home_dir {
                common_paths.extend(vec![
                    home.join("AppData\\Local\\Programs\\Claude\\claude.exe"),
                    home.join("AppData\\Roaming\\npm\\claude.cmd"),
                    home.join("AppData\\Roaming\\npm\\claude.exe"),
                ]);
            }
            common_paths.extend(vec![
                PathBuf::from("C:\\Program Files\\Claude\\claude.exe"),
                PathBuf::from("C:\\Program Files (x86)\\Claude\\claude.exe"),
            ]);
        }

        // Check each common path
        for path in common_paths {
            if path.exists() && path.is_file() {
                return Ok(path);
            }
        }

        // Strategy 5: Check if CLAUDE_CLI_PATH environment variable is set
        if let Ok(cli_path) = std::env::var("CLAUDE_CLI_PATH") {
            let path = PathBuf::from(cli_path);
            if path.exists() && path.is_file() {
                return Ok(path);
            }
        }

        Err(ClaudeError::CliNotFound(CliNotFoundError::new(
            "Claude Code CLI not found. Please ensure 'claude' is in your PATH or set CLAUDE_CLI_PATH environment variable.",
            None,
        )))
    }

    /// 查找 CLI，支持自动安装
    ///
    /// 首先尝试标准查找，如果失败且启用自动安装，则尝试自动安装
    fn find_cli_with_auto_install(options: &ClaudeAgentOptions) -> Result<PathBuf> {
        // 首先尝试标准查找
        match Self::find_cli() {
            Ok(path) => return Ok(path),
            Err(_) => {
                // CLI 未找到，检查是否启用自动安装
                let auto_install = options.auto_install_cli
                    || std::env::var("CLAUDE_AUTO_INSTALL_CLI")
                        .ok()
                        .and_then(|v| {
                            let v_lower = v.to_lowercase();
                            if v_lower == "true" || v_lower == "1" || v_lower == "yes" {
                                Some(true)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(false);

                if !auto_install {
                    // 未启用自动安装，返回原始错误
                    return Err(ClaudeError::CliNotFound(CliNotFoundError::new(
                        "Claude Code CLI not found. Please ensure 'claude' is in your PATH or set CLAUDE_CLI_PATH environment variable.",
                        None,
                    )));
                }

                // 启用自动安装
                tracing::info!("🔧 CLI not found, auto-install enabled - attempting installation...");
            }
        }

        // 使用 runtime executor 执行异步安装
        // 注意：我们在独立线程中运行，以避免在已有的 tokio runtime 中调用 block_on 导致 panic
        let installer_options = options.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| ClaudeError::InternalError(format!("Failed to create runtime: {}", e)))?;

            let installer = CliInstaller::new(true);
            let installer = if let Some(ref callback) = installer_options.cli_install_callback {
                installer.with_progress_callback(callback.clone())
            } else {
                // 默认进度回调：记录日志
                let default_callback = std::sync::Arc::new(|event: InstallProgress| {
                    match event {
                        InstallProgress::Checking(msg) => {
                            tracing::info!("🔍 {}", msg);
                        }
                        InstallProgress::Downloading { current, total } => {
                            if let Some(total) = total {
                                let progress = (current as f64 / total as f64 * 100.0) as u32;
                                tracing::info!("⬇️  Downloading: {}% ({}/{})", progress, current, total);
                            } else {
                                tracing::info!("⬇️  Downloading: {} bytes", current);
                            }
                        }
                        InstallProgress::Installing(msg) => {
                            tracing::info!("🔧 {}", msg);
                        }
                        InstallProgress::Done(path) => {
                            tracing::info!("✅ Installation complete: {}", path.display());
                        }
                        InstallProgress::Failed(err) => {
                            tracing::error!("❌ {}", err);
                        }
                    }
                });
                installer.with_progress_callback(default_callback)
            };

            rt.block_on(installer.install_if_needed())
                .map_err(|e| ClaudeError::InternalError(format!("Auto-install failed: {}", e)))
        })
        .join()
        .map_err(|_| ClaudeError::InternalError("Auto-install thread panicked".to_string()))?
    }

    /// Build command arguments from options
    fn build_command(&self) -> Vec<String> {
        let mut args = vec![
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
        ];

        // For streaming mode or content mode, enable stream-json input
        if matches!(
            self.prompt,
            QueryPrompt::Streaming | QueryPrompt::Content(_)
        ) {
            args.push("--input-format".to_string());
            args.push("stream-json".to_string());
        }

        // Add system prompt
        // Note: Python SDK behavior (lines 91-102 of subprocess_cli.py):
        // - If None: skip
        // - If string: use --system-prompt
        // - If preset with append: use --append-system-prompt (NOT --system-prompt-preset)
        //   This relies on default Claude Code prompt and just appends to it
        if let Some(ref system_prompt) = self.options.system_prompt {
            match system_prompt {
                crate::types::config::SystemPrompt::Text(text) => {
                    args.push("--system-prompt".to_string());
                    args.push(text.clone());
                },
                crate::types::config::SystemPrompt::Preset(preset) => {
                    // Only add append if present (uses default Claude Code prompt)
                    if let Some(ref append) = preset.append {
                        args.push("--append-system-prompt".to_string());
                        args.push(append.clone());
                    }
                    // Note: preset.preset field is ignored - CLI uses default prompt
                },
            }
        }

        // Add tools configuration
        if let Some(ref tools) = self.options.tools {
            match tools {
                crate::types::config::Tools::List(tool_list) => {
                    if tool_list.is_empty() {
                        args.push("--tools".to_string());
                        args.push(String::new());
                    } else {
                        args.push("--tools".to_string());
                        args.push(tool_list.join(","));
                    }
                },
                crate::types::config::Tools::Preset(_) => {
                    // Preset object - 'claude_code' preset maps to 'default'
                    args.push("--tools".to_string());
                    args.push("default".to_string());
                },
            }
        }

        // Add permission mode
        if let Some(mode) = self.options.permission_mode {
            let mode_str = match mode {
                crate::types::config::PermissionMode::Default => "default",
                crate::types::config::PermissionMode::AcceptEdits => "acceptEdits",
                crate::types::config::PermissionMode::Plan => "plan",
                crate::types::config::PermissionMode::BypassPermissions => "bypassPermissions",
            };
            args.push("--permission-mode".to_string());
            args.push(mode_str.to_string());
        }

        // Add allowed tools (Python SDK uses --allowedTools with comma-separated values)
        if !self.options.allowed_tools.is_empty() {
            args.push("--allowedTools".to_string());
            args.push(self.options.allowed_tools.join(","));
        }

        // Add disallowed tools (Python SDK uses --disallowedTools with comma-separated values)
        if !self.options.disallowed_tools.is_empty() {
            args.push("--disallowedTools".to_string());
            args.push(self.options.disallowed_tools.join(","));
        }

        // Add model
        if let Some(ref model) = self.options.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        // Add fallback model
        if let Some(ref fallback_model) = self.options.fallback_model {
            args.push("--fallback-model".to_string());
            args.push(fallback_model.clone());
        }

        // Add beta features
        if !self.options.betas.is_empty() {
            let betas: Vec<String> = self
                .options
                .betas
                .iter()
                .map(|b| match b {
                    crate::types::config::SdkBeta::Context1M => "context-1m-2025-08-07".to_string(),
                })
                .collect();
            args.push("--betas".to_string());
            args.push(betas.join(","));
        }

        // Add max budget USD
        if let Some(max_budget) = self.options.max_budget_usd {
            args.push("--max-budget-usd".to_string());
            args.push(max_budget.to_string());
        }

        // Add max thinking tokens
        if let Some(max_thinking) = self.options.max_thinking_tokens {
            args.push("--max-thinking-tokens".to_string());
            args.push(max_thinking.to_string());
        }

        // Add permission prompt tool name
        if let Some(ref tool_name) = self.options.permission_prompt_tool_name {
            args.push("--permission-prompt-tool".to_string());
            args.push(tool_name.clone());
        }

        // Add output format (structured outputs / JSON schema)
        // Expected format: {"type": "json_schema", "schema": {...}}
        if let Some(ref output_format) = self.options.output_format
            && output_format.get("type") == Some(&serde_json::json!("json_schema"))
            && let Some(schema) = output_format.get("schema")
        {
            args.push("--json-schema".to_string());
            args.push(schema.to_string());
        }

        // Add max turns
        if let Some(max_turns) = self.options.max_turns {
            args.push("--max-turns".to_string());
            args.push(max_turns.to_string());
        }

        // Add resume session
        if let Some(ref session_id) = self.options.resume {
            args.push("--resume".to_string());
            args.push(session_id.clone());
        }

        // Add continue conversation
        if self.options.continue_conversation {
            args.push("--continue".to_string());
        }

        // Add settings (combined with sandbox if both are provided)
        let settings_value = self.build_settings_value();
        if let Some(ref settings) = settings_value {
            args.push("--settings".to_string());
            args.push(settings.clone());
        }

        // Add additional directories
        for dir in &self.options.add_dirs {
            args.push("--add-dir".to_string());
            args.push(dir.display().to_string());
        }

        // Add include partial messages
        if self.options.include_partial_messages {
            args.push("--include-partial-messages".to_string());
        }

        // Add fork session
        if self.options.fork_session {
            args.push("--fork-session".to_string());
        }

        // Add agent definitions
        if let Some(ref agents) = self.options.agents
            && !agents.is_empty()
        {
            let agents_json = serde_json::to_string(agents).unwrap_or_default();
            args.push("--agents".to_string());
            args.push(agents_json);
        }

        // Add setting sources
        if let Some(ref sources) = self.options.setting_sources {
            let sources_str: Vec<&str> = sources
                .iter()
                .map(|s| match s {
                    crate::types::config::SettingSource::User => "user",
                    crate::types::config::SettingSource::Project => "project",
                    crate::types::config::SettingSource::Local => "local",
                })
                .collect();
            args.push("--setting-sources".to_string());
            args.push(sources_str.join(","));
        }

        // Add plugins
        for plugin in &self.options.plugins {
            if let Some(path) = plugin.path() {
                args.push("--plugin-dir".to_string());
                args.push(path.display().to_string());
            }
        }

        // Add additional directories
        for dir in &self.options.add_dirs {
            args.push("--add-dir".to_string());
            args.push(dir.display().to_string());
        }

        // Add extra args
        for (key, value) in &self.options.extra_args {
            args.push(format!("--{}", key));
            if let Some(v) = value {
                args.push(v.clone());
            }
        }

        args
    }

    /// Build settings value, merging sandbox settings if provided.
    ///
    /// Returns the settings value as either:
    /// - A JSON string (if sandbox is provided or settings is JSON)
    /// - A file path (if only settings path is provided without sandbox)
    /// - None if neither settings nor sandbox is provided
    fn build_settings_value(&self) -> Option<String> {
        let has_settings = self.options.settings.is_some();
        let has_sandbox = self.options.sandbox.is_some();

        if !has_settings && !has_sandbox {
            return None;
        }

        // If only settings path and no sandbox, pass through as-is
        if has_settings && !has_sandbox {
            return self.options.settings.clone();
        }

        // If we have sandbox settings, we need to merge into a JSON object
        let mut settings_obj = serde_json::Map::new();

        if let Some(settings_str) = &self.options.settings {
            let trimmed = settings_str.trim();
            // Check if settings is a JSON string or a file path
            if trimmed.starts_with('{') && trimmed.ends_with('}') {
                // Parse JSON string
                if let Ok(serde_json::Value::Object(obj)) =
                    serde_json::from_str::<serde_json::Value>(trimmed)
                {
                    settings_obj = obj;
                }
            } else {
                // It's a file path - try to read and parse
                if let Ok(content) = std::fs::read_to_string(trimmed)
                    && let Ok(serde_json::Value::Object(obj)) =
                        serde_json::from_str::<serde_json::Value>(&content)
                {
                    settings_obj = obj;
                }
            }
        }

        // Merge sandbox settings
        if let Some(sandbox) = &self.options.sandbox
            && let Ok(sandbox_value) = serde_json::to_value(sandbox)
        {
            settings_obj.insert("sandbox".to_string(), sandbox_value);
        }

        Some(serde_json::to_string(&serde_json::Value::Object(settings_obj)).unwrap_or_default())
    }

    /// Check Claude CLI version
    async fn check_claude_version(&self) -> Result<()> {
        // Skip if environment variable is set
        if std::env::var(SKIP_VERSION_CHECK_ENV).is_ok() {
            return Ok(());
        }

        let output = Command::new(&self.cli_path)
            .arg("--version")
            .output()
            .await
            .map_err(|e| {
                ClaudeError::Connection(ConnectionError::new(format!(
                    "Failed to get Claude version: {}",
                    e
                )))
            })?;

        let version_output = String::from_utf8_lossy(&output.stdout);
        let version = version_output
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().next())
            .unwrap_or("")
            .trim();

        if !check_version(version) {
            warn!(
                "Claude Code CLI ({}) version {} is below minimum required version {}. Some features may not work correctly.",
                self.cli_path.display(),
                version,
                MIN_CLI_VERSION
            );
        }

        Ok(())
    }

    /// Build environment variables
    fn build_env(&self) -> HashMap<String, String> {
        let mut env = self.options.env.clone();
        env.insert("CLAUDE_CODE_ENTRYPOINT".to_string(), ENTRYPOINT.to_string());
        env.insert(
            "CLAUDE_AGENT_SDK_VERSION".to_string(),
            SDK_VERSION.to_string(),
        );

        // Enable file checkpointing if requested
        if self.options.enable_file_checkpointing {
            env.insert(
                "CLAUDE_CODE_ENABLE_SDK_FILE_CHECKPOINTING".to_string(),
                "true".to_string(),
            );
        }

        env
    }
}

#[async_trait]
impl Transport for SubprocessTransport {
    async fn connect(&mut self) -> Result<()> {
        // Note: cwd validation is done in new() for early error detection

        // Check version
        self.check_claude_version().await?;

        // Build command
        let args = self.build_command();
        let env = self.build_env();

        // Build command
        let mut cmd = Command::new(&self.cli_path);
        cmd.args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(&env);

        if let Some(ref cwd) = self.cwd {
            cmd.current_dir(cwd);
        }

        // Spawn process
        let mut child = cmd.spawn().map_err(|e| {
            ClaudeError::Process(ProcessError::new(
                format!("Failed to spawn Claude CLI process: {}", e),
                None,
                None,
            ))
        })?;

        // Take stdin and stdout
        let stdin = child.stdin.take().ok_or_else(|| {
            ClaudeError::Connection(ConnectionError::new("Failed to get stdin".to_string()))
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            ClaudeError::Connection(ConnectionError::new("Failed to get stdout".to_string()))
        })?;

        let stderr = child.stderr.take();

        // Spawn stderr handler if callback is provided
        if let (Some(stderr), Some(callback)) = (stderr, &self.options.stderr_callback) {
            let callback = Arc::clone(callback);
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break;
                    }
                    callback(line.clone());
                    line.clear();
                }
            });
        }

        self.stdin = Some(stdin);
        self.stdout = Some(BufReader::new(stdout));
        self.process = Some(child);
        self.ready = true;

        // Send initial prompt based on type
        match &self.prompt {
            QueryPrompt::Text(text) => {
                let text_owned = text.clone();
                self.write(&text_owned).await?;
                self.end_input().await?;
            },
            QueryPrompt::Content(blocks) => {
                // Format as JSON user message for stream-json input format
                let user_message = serde_json::json!({
                    "type": "user",
                    "message": {
                        "role": "user",
                        "content": blocks
                    }
                });
                let content_json = serde_json::to_string(&user_message).map_err(|e| {
                    ClaudeError::Transport(format!("Failed to serialize content blocks: {}", e))
                })?;
                self.write(&content_json).await?;
                self.end_input().await?;
            },
            QueryPrompt::Streaming => {
                // Don't send initial prompt or close stdin - leave it open for streaming
            },
        }

        Ok(())
    }

    async fn write(&mut self, data: &str) -> Result<()> {
        // Try direct stdin first (simple mode)
        if let Some(ref mut stdin) = self.stdin {
            stdin
                .write_all(data.as_bytes())
                .await
                .map_err(|e| ClaudeError::Transport(format!("Failed to write to stdin: {}", e)))?;
            stdin
                .write_all(b"\n")
                .await
                .map_err(|e| ClaudeError::Transport(format!("Failed to write newline: {}", e)))?;
            stdin
                .flush()
                .await
                .map_err(|e| ClaudeError::Transport(format!("Failed to flush stdin: {}", e)))?;
            return Ok(());
        }

        // Fall back to shared stdin (bidirectional mode)
        if let Some(ref stdin_arc) = self.stdin_arc {
            let mut stdin_guard = stdin_arc.lock().await;
            if let Some(ref mut stdin) = *stdin_guard {
                stdin
                    .write_all(data.as_bytes())
                    .await
                    .map_err(|e| ClaudeError::Transport(format!("Failed to write to stdin: {}", e)))?;
                stdin
                    .write_all(b"\n")
                    .await
                    .map_err(|e| ClaudeError::Transport(format!("Failed to write newline: {}", e)))?;
                stdin
                    .flush()
                    .await
                    .map_err(|e| ClaudeError::Transport(format!("Failed to flush stdin: {}", e)))?;
                return Ok(());
            }
        }

        Err(ClaudeError::Transport("stdin not available".to_string()))
    }

    fn read_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>> {
        let max_message_size = self.buffer_config.max_message_size;
        let enable_metrics = self.buffer_config.enable_metrics;

        Box::pin(async_stream::stream! {
            if let Some(ref mut reader) = self.stdout {
                let mut line = String::with_capacity(self.buffer_config.initial_size);
                let mut current_capacity = self.buffer_config.initial_size;

                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => {
                            // EOF
                            break;
                        }
                        Ok(bytes_read) => {
                            // Per-message size check (not cumulative)
                            if bytes_read > max_message_size {
                                yield Err(ClaudeError::Transport(format!(
                                    "Message size {} bytes exceeded maximum of {} bytes",
                                    bytes_read, max_message_size
                                )));
                                break;
                            }

                            // Track metrics if enabled
                            if enable_metrics {
                                self.buffer_metrics.inc_message_count();
                                self.buffer_metrics.add_bytes(bytes_read);
                                self.buffer_metrics.update_peak(bytes_read);
                                // Check if we need to grow the buffer
                                if bytes_read > current_capacity {
                                    let new_capacity = (current_capacity as f64 * self.buffer_config.growth_factor) as usize;
                                    current_capacity = new_capacity.min(max_message_size);
                                    self.buffer_metrics.inc_resize_count();
                                }
                            }

                            let trimmed = line.trim();
                            if trimmed.is_empty() {
                                continue;
                            }

                            match serde_json::from_str::<serde_json::Value>(trimmed) {
                                Ok(json) => {
                                    yield Ok(json);
                                }
                                Err(e) => {
                                    yield Err(ClaudeError::JsonDecode(JsonDecodeError::new(
                                        format!("Failed to parse JSON: {}", e),
                                        trimmed.to_string(),
                                    )));
                                }
                            }
                        }
                        Err(e) => {
                            yield Err(ClaudeError::Transport(format!("Failed to read line: {}", e)));
                            break;
                        }
                    }
                }
            }
        })
    }

    fn read_raw_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send + '_>> {
        let max_message_size = self.buffer_config.max_message_size;
        let enable_metrics = self.buffer_config.enable_metrics;

        Box::pin(async_stream::stream! {
            if let Some(ref mut reader) = self.stdout {
                let mut line = String::with_capacity(self.buffer_config.initial_size);
                let mut current_capacity = self.buffer_config.initial_size;

                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => {
                            // EOF
                            break;
                        }
                        Ok(bytes_read) => {
                            // Per-message size check (not cumulative)
                            if bytes_read > max_message_size {
                                yield Err(ClaudeError::Transport(format!(
                                    "Message size {} bytes exceeded maximum of {} bytes",
                                    bytes_read, max_message_size
                                )));
                                break;
                            }

                            // Track metrics if enabled
                            if enable_metrics {
                                self.buffer_metrics.inc_message_count();
                                self.buffer_metrics.add_bytes(bytes_read);
                                self.buffer_metrics.update_peak(bytes_read);
                                // Check if we need to grow the buffer
                                if bytes_read > current_capacity {
                                    let new_capacity = (current_capacity as f64 * self.buffer_config.growth_factor) as usize;
                                    current_capacity = new_capacity.min(max_message_size);
                                    self.buffer_metrics.inc_resize_count();
                                }
                            }

                            let trimmed = line.trim();
                            if trimmed.is_empty() {
                                continue;
                            }

                            // Return the raw string - caller will parse
                            yield Ok(trimmed.to_string());
                        }
                        Err(e) => {
                            yield Err(ClaudeError::Transport(format!("Failed to read line: {}", e)));
                            break;
                        }
                    }
                }
            }
        })
    }

    async fn close(&mut self) -> Result<()> {
        // Close direct stdin (simple mode)
        if let Some(mut stdin) = self.stdin.take() {
            let _ = stdin.shutdown().await;
        }

        // Close shared stdin (bidirectional mode)
        if let Some(ref stdin_arc) = self.stdin_arc {
            let mut stdin_guard = stdin_arc.lock().await;
            if let Some(mut stdin) = stdin_guard.take() {
                let _ = stdin.shutdown().await;
            }
        }

        // Wait for process to exit
        if let Some(mut process) = self.process.take() {
            let status = process.wait().await.map_err(|e| {
                ClaudeError::Process(ProcessError::new(
                    format!("Failed to wait for process: {}", e),
                    None,
                    None,
                ))
            })?;

            if !status.success() {
                return Err(ClaudeError::Process(ProcessError::new(
                    "Claude CLI exited with non-zero status".to_string(),
                    status.code(),
                    None,
                )));
            }
        }

        self.ready = false;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    async fn end_input(&mut self) -> Result<()> {
        // Close direct stdin (simple mode)
        if let Some(mut stdin) = self.stdin.take() {
            stdin
                .shutdown()
                .await
                .map_err(|e| ClaudeError::Transport(format!("Failed to close stdin: {}", e)))?;
            return Ok(());
        }

        // Close shared stdin (bidirectional mode)
        if let Some(ref stdin_arc) = self.stdin_arc {
            let mut stdin_guard = stdin_arc.lock().await;
            if let Some(mut stdin) = stdin_guard.take() {
                stdin
                    .shutdown()
                    .await
                    .map_err(|e| ClaudeError::Transport(format!("Failed to close stdin: {}", e)))?;
            }
        }
        Ok(())
    }

    fn get_buffer_metrics(&self) -> Option<BufferMetricsSnapshot> {
        Some(self.buffer_metrics.snapshot())
    }
}

impl Drop for SubprocessTransport {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.start_kill();
        }
    }
}
