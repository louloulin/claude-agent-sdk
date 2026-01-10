//! Investment Analysis Hooks using Claude Agent SDK
//!
//! This module implements custom hooks for investment analysis:
//! - PreToolUse: Monitor and control tool usage
//! - PostToolUse: Track analysis results
//! - UserPromptSubmit: Validate user queries
//!
//! Uses the real Hooks system from Claude Agent SDK

use claude_agent_sdk_rs::{
    HookContext, HookInput, HookJsonOutput, HookSpecificOutput, Hooks,
    PostToolUseHookSpecificOutput, PreToolUseHookSpecificOutput, SyncHookJsonOutput,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Investment Analysis Security Hooks
///
/// Provides security and monitoring for investment analysis operations
pub struct InvestmentHooks {
    tool_usage_log: Arc<RwLock<Vec<ToolUsageLog>>>,
    analysis_stats: Arc<RwLock<AnalysisStats>>,
}

#[derive(Debug, Clone)]
struct ToolUsageLog {
    tool_name: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    allowed: bool,
    reason: Option<String>,
}

#[derive(Debug, Default)]
struct AnalysisStats {
    total_tools_called: usize,
    technical_analysis_count: usize,
    risk_analysis_count: usize,
    sentiment_analysis_count: usize,
}

impl InvestmentHooks {
    /// Create new investment hooks
    pub fn new() -> Self {
        Self {
            tool_usage_log: Arc::new(RwLock::new(Vec::new())),
            analysis_stats: Arc::new(RwLock::new(AnalysisStats::default())),
        }
    }

    /// Convert to SDK Hooks
    pub fn into_hooks(self) -> Hooks {
        let log = self.tool_usage_log.clone();
        let stats = self.analysis_stats.clone();

        let mut hooks = Hooks::new();

        // Add PreToolUse hook
        hooks.add_pre_tool_use(move |input, _id, _ctx| {
            let log = log.clone();
            let stats = stats.clone();
            async move {
                Self::handle_pre_tool_use(input, log, stats).await
            }
        });

        // Add PostToolUse hook
        hooks.add_post_tool_use(move |input, _id, _ctx| {
            let stats = stats.clone();
            async move {
                Self::handle_post_tool_use(input, stats).await
            }
        });

        hooks
    }

    /// Handle PreToolUse event
    async fn handle_pre_tool_use(
        input: HookInput,
        log: Arc<RwLock<Vec<ToolUsageLog>>>,
        stats: Arc<RwLock<AnalysisStats>>,
    ) -> HookJsonOutput {
        if let HookInput::PreToolUse(pre_tool) = input {
            let tool_name = pre_tool.tool_name.clone();
            let timestamp = chrono::Utc::now();

            // Log the tool usage
            let mut log_guard = log.write().await;
            log_guard.push(ToolUsageLog {
                tool_name: tool_name.clone(),
                timestamp,
                allowed: true,
                reason: None,
            });

            // Update stats
            let mut stats_guard = stats.write().await;
            stats_guard.total_tools_called += 1;

            match tool_name.as_str() {
                "technical_analysis" => {
                    stats_guard.technical_analysis_count += 1;
                }
                "var_calculation" | "stress_test" => {
                    stats_guard.risk_analysis_count += 1;
                }
                "sentiment_analysis" => {
                    stats_guard.sentiment_analysis_count += 1;
                }
                _ => {}
            }

            // Allow all tools for now
            HookJsonOutput::Sync(SyncHookJsonOutput {
                hook_specific_output: Some(HookSpecificOutput::PreToolUse(
                    PreToolUseHookSpecificOutput {
                        permission_decision: Some("allow".to_string()),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            })
        } else {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }
    }

    /// Handle PostToolUse event
    async fn handle_post_tool_use(
        input: HookInput,
        stats: Arc<RwLock<AnalysisStats>,
    ) -> HookJsonOutput {
        if let HookInput::PostToolUse(post_tool) = input {
            // Log successful completion
            println!("✅ Tool '{}' completed successfully", post_tool.tool_name);

            HookJsonOutput::Sync(SyncHookJsonOutput {
                hook_specific_output: Some(HookSpecificOutput::PostToolUse(
                    PostToolUseHookSpecificOutput::default(),
                )),
                ..Default::default()
            })
        } else {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }
    }

    /// Get usage statistics
    pub async fn get_stats(&self) -> AnalysisStats {
        self.analysis_stats.read().await.clone()
    }

    /// Get tool usage log
    pub async fn get_usage_log(&self) -> Vec<ToolUsageLog> {
        self.tool_usage_log.read().await.clone()
    }
}

/// Budget Control Hooks
///
/// Prevents exceeding API budget limits
pub struct BudgetControlHooks {
    max_budget_usd: f64,
    current_spent: Arc<RwLock<f64>>,
}

impl BudgetControlHooks {
    /// Create new budget control hooks
    pub fn new(max_budget_usd: f64) -> Self {
        Self {
            max_budget_usd,
            current_spent: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Convert to SDK Hooks
    pub fn into_hooks(self) -> Hooks {
        let spent = self.current_spent.clone();
        let max_budget = self.max_budget_usd;

        let mut hooks = Hooks::new();

        // Add PreToolUse hook to check budget
        hooks.add_pre_tool_use(move |input, _id, _ctx| {
            let spent = spent.clone();
            async move {
                if let HookInput::PreToolUse(pre_tool) = input {
                    let current_spent = *spent.read().await;

                    if current_spent >= max_budget {
                        // Budget exceeded
                        HookJsonOutput::Sync(SyncHookJsonOutput {
                            hook_specific_output: Some(HookSpecificOutput::PreToolUse(
                                PreToolUseHookSpecificOutput {
                                    permission_decision: Some("deny".to_string()),
                                    permission_decision_reason: Some(format!(
                                        "Budget exceeded: ${:.2} / ${:.2}",
                                        current_spent, max_budget
                                    )),
                                    ..Default::default()
                                },
                            )),
                            ..Default::default()
                        })
                    } else {
                        HookJsonOutput::Sync(SyncHookJsonOutput::default())
                    }
                } else {
                    HookJsonOutput::Sync(SyncHookJsonOutput::default())
                }
            }
        });

        hooks
    }
}

/// Analysis Logging Hooks
///
/// Logs all analysis operations for audit trail
pub struct AnalysisLoggingHooks {
    log_file: Arc<RwLock<Option<tokio::fs::File>>>,
}

impl AnalysisLoggingHooks {
    /// Create new logging hooks
    pub fn new(log_path: &str) -> Self {
        Self {
            log_file: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize logging
    pub async fn init(&self, log_path: &str) -> anyhow::Result<()> {
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .await?;

        let mut log_guard = self.log_file.write().await;
        *log_guard = Some(file);

        Ok(())
    }

    /// Convert to SDK Hooks
    pub fn into_hooks(self) -> Hooks {
        let log = self.log_file.clone();

        let mut hooks = Hooks::new();

        // Log all tool uses
        hooks.add_pre_tool_use(move |input, _id, _ctx| {
            let log = log.clone();
            async move {
                if let HookInput::PreToolUse(pre_tool) = input {
                    let log_entry = format!(
                        "[{}] Tool: {}, Input: {}\n",
                        chrono::Utc::now().to_rfc3339(),
                        pre_tool.tool_name,
                        serde_json::to_string(&pre_tool.tool_input).unwrap_or_default()
                    );

                    if let Some(ref mut file) = *log.write().await {
                        use tokio::io::AsyncWriteExt;
                        let _ = file.write_all(log_entry.as_bytes()).await;
                    }
                }

                HookJsonOutput::Sync(SyncHookJsonOutput::default())
            }
        });

        hooks
    }
}

/// Compliance Hooks
///
/// Ensures investment advice complies with regulations
pub struct ComplianceHooks;

impl ComplianceHooks {
    /// Create compliance hooks
    pub fn new() -> Self {
        Self
    }

    /// Convert to SDK Hooks
    pub fn into_hooks(self) -> Hooks {
        let mut hooks = Hooks::new();

        // Add UserPromptSubmit hook to validate queries
        hooks.add_user_prompt_submit(|input, _id, _ctx| {
            async move {
                if let HookInput::UserPromptSubmit(prompt) = input {
                    let content = prompt.content.to_lowercase();

                    // Check for disallowed content
                    let disallowed_patterns = vec![
                        "内幕交易",
                        "insider trading",
                        "市场操纵",
                        "market manipulation",
                    ];

                    for pattern in disallowed_patterns {
                        if content.contains(pattern) {
                            return HookJsonOutput::Sync(SyncHookJsonOutput {
                                system_message: Some(format!(
                                    "无法处理该请求: 涉及不允许的 Topics: {}",
                                    pattern
                                )),
                                ..Default::default()
                            });
                        }
                    }

                    // Add disclaimer
                    let disclaimer = "\n\n---\n\n⚠️ 免责声明: 以上分析仅供参考,不构成投资建议。投资有风险,请谨慎决策。";

                    HookJsonOutput::Sync(SyncHookJsonOutput {
                        system_message: Some(disclaimer.to_string()),
                        ..Default::default()
                    })
                } else {
                    HookJsonOutput::Sync(SyncHookJsonOutput::default())
                }
            }
        });

        hooks
    }
}

// ============================================================================
// Factory Functions
// ============================================================================

/// Create comprehensive investment hooks
pub fn create_investment_hooks() -> InvestmentHooks {
    InvestmentHooks::new()
}

/// Create budget control hooks
pub fn create_budget_hooks(max_budget_usd: f64) -> BudgetControlHooks {
    BudgetControlHooks::new(max_budget_usd)
}

/// Create analysis logging hooks
pub fn create_logging_hooks(log_path: &str) -> AnalysisLoggingHooks {
    AnalysisLoggingHooks::new(log_path)
}

/// Create compliance hooks
pub fn create_compliance_hooks() -> ComplianceHooks {
    ComplianceHooks::new()
}
