//! Type definitions for Subagent system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A subagent - a specialized Claude instance with specific capabilities
///
/// # Example
///
/// ```
/// use claude_agent_sdk::subagents::Subagent;
///
/// let subagent = Subagent {
///     name: "code-reviewer".to_string(),
///     description: "Expert code reviewer".to_string(),
///     instructions: "Review code for bugs and best practices".to_string(),
///     allowed_tools: vec!["Read".to_string(), "Grep".to_string()],
///     max_turns: Some(5),
///     model: Some("claude-sonnet-4".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subagent {
    /// Unique name for this subagent
    pub name: String,

    /// Description of the subagent's purpose
    pub description: String,

    /// Specific instructions for the subagent
    pub instructions: String,

    /// Tools that this subagent is allowed to use
    pub allowed_tools: Vec<String>,

    /// Maximum number of turns (None = no limit)
    pub max_turns: Option<u32>,

    /// Model to use (None = use default)
    pub model: Option<String>,
}

/// Configuration for multiple subagents
///
/// # Example
///
/// ```
/// use claude_agent_sdk::subagents::{SubagentConfig, Subagent, DelegationStrategy};
///
/// let config = SubagentConfig {
///     subagents: vec![
///         Subagent {
///             name: "reviewer".to_string(),
///             description: "Code reviewer".to_string(),
///             instructions: "Review code".to_string(),
///             allowed_tools: vec!["Read".to_string()],
///             max_turns: Some(5),
///             model: None,
///         },
///     ],
///     delegation_strategy: DelegationStrategy::Auto,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentConfig {
    /// List of available subagents
    pub subagents: Vec<Subagent>,

    /// Strategy for delegating to subagents
    pub delegation_strategy: DelegationStrategy,
}

impl SubagentConfig {
    /// Create a new subagent configuration
    ///
    /// # Arguments
    ///
    /// * `delegation_strategy` - The delegation strategy to use
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::subagents::{DelegationStrategy, SubagentConfig};
    /// let config = SubagentConfig::new(DelegationStrategy::Auto);
    /// ```
    pub fn new(delegation_strategy: DelegationStrategy) -> Self {
        Self {
            subagents: Vec::new(),
            delegation_strategy,
        }
    }

    /// Add a subagent to the configuration
    ///
    /// # Arguments
    ///
    /// * `subagent` - The subagent to add
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::subagents::{SubagentConfig, Subagent, DelegationStrategy};
    /// # let mut config = SubagentConfig::new(DelegationStrategy::Auto);
    /// let subagent = Subagent {
    ///     name: "agent".to_string(),
    ///     description: "Description".to_string(),
    ///     instructions: "Instructions".to_string(),
    ///     allowed_tools: vec![],
    ///     max_turns: None,
    ///     model: None,
    /// };
    /// config.add_subagent(subagent);
    /// ```
    pub fn add_subagent(&mut self, subagent: Subagent) {
        self.subagents.push(subagent);
    }

    /// Get a subagent by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the subagent to retrieve
    ///
    /// # Returns
    ///
    /// `Some(subagent)` if found, `None` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::subagents::{SubagentConfig, Subagent, DelegationStrategy};
    /// # let mut config = SubagentConfig::new(DelegationStrategy::Auto);
    /// # let subagent = Subagent {
    /// #     name: "agent".to_string(),
    /// #     description: "Description".to_string(),
    /// #     instructions: "Instructions".to_string(),
    /// #     allowed_tools: vec![],
    /// #     max_turns: None,
    /// #     model: None,
    /// # };
    /// # config.add_subagent(subagent);
    /// if let Some(agent) = config.get_subagent("agent") {
    ///     println!("Found agent: {}", agent.name);
    /// }
    /// ```
    pub fn get_subagent(&self, name: &str) -> Option<&Subagent> {
        self.subagents.iter().find(|s| s.name == name)
    }

    /// Convert to a HashMap for efficient lookup
    ///
    /// # Returns
    ///
    /// A HashMap mapping subagent names to subagents
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::subagents::{SubagentConfig, DelegationStrategy};
    /// # let config = SubagentConfig::new(DelegationStrategy::Auto);
    /// let map = config.to_map();
    /// ```
    pub fn to_map(&self) -> HashMap<String, Subagent> {
        self.subagents
            .iter()
            .map(|s| (s.name.clone(), s.clone()))
            .collect()
    }
}

/// Strategy for delegating work to subagents
///
/// # Variants
///
/// * `Auto` - Claude automatically decides when to delegate
/// * `Manual` - Requires explicit SubagentTool calls
/// * `ToolCall` - Delegate through tool calls
///
/// # Example
///
/// ```
/// use claude_agent_sdk::subagents::DelegationStrategy;
///
/// let strategy = DelegationStrategy::Auto;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DelegationStrategy {
    /// Claude automatically decides when to delegate to subagents
    Auto,

    /// Manual delegation requires explicit SubagentTool calls
    Manual,

    /// Delegation happens through tool calls
    ToolCall,
}

/// Represents a single subagent execution call
///
/// # Example
///
/// ```
/// use claude_agent_sdk::subagents::SubagentCall;
///
/// let call = SubagentCall {
///     subagent_name: "reviewer".to_string(),
///     input: "Review this code".to_string(),
///     output: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentCall {
    /// Name of the subagent to call
    pub subagent_name: String,

    /// Input to provide to the subagent
    pub input: String,

    /// Output from the subagent (None if not yet executed)
    pub output: Option<String>,
}

impl SubagentCall {
    /// Create a new subagent call
    ///
    /// # Arguments
    ///
    /// * `subagent_name` - Name of the subagent to call
    /// * `input` - Input for the subagent
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::subagents::SubagentCall;
    /// let call = SubagentCall::new("reviewer", "Review this code");
    /// ```
    pub fn new(subagent_name: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            subagent_name: subagent_name.into(),
            input: input.into(),
            output: None,
        }
    }

    /// Check if the call has been executed
    ///
    /// # Returns
    ///
    /// `true` if the call has an output, `false` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// # use claude_agent_sdk::subagents::SubagentCall;
    /// let call = SubagentCall::new("agent", "input");
    /// assert!(!call.is_executed());
    /// ```
    pub fn is_executed(&self) -> bool {
        self.output.is_some()
    }
}

/// Output from a subagent execution
///
/// # Example
///
/// ```
/// use claude_agent_sdk::subagents::SubagentOutput;
///
/// let output = SubagentOutput {
///     subagent_name: "reviewer".to_string(),
///     messages: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentOutput {
    /// Name of the subagent that produced this output
    pub subagent_name: String,

    /// Messages produced by the subagent
    /// Note: This is a placeholder - in a full implementation,
    /// this would contain actual Message types from the SDK
    pub messages: Vec<serde_json::Value>,
}

/// Errors that can occur in subagent operations
///
/// # Variants
///
/// * `NotFound` - Subagent not found
/// * `AlreadyExists` - Subagent with this name already exists
/// * `ExecutionFailed` - Subagent execution failed
/// * `InvalidInput` - Invalid input provided
///
/// # Example
///
/// ```
/// use claude_agent_sdk::subagents::SubagentError;
///
/// let error = SubagentError::NotFound("my-agent".to_string());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubagentError {
    /// Subagent not found
    NotFound(String),

    /// Subagent with this name already exists
    AlreadyExists(String),

    /// Subagent execution failed
    ExecutionFailed(String),

    /// Invalid input provided
    InvalidInput(String),
}

impl std::fmt::Display for SubagentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubagentError::NotFound(name) => write!(f, "Subagent not found: {}", name),
            SubagentError::AlreadyExists(name) => write!(f, "Subagent already exists: {}", name),
            SubagentError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            SubagentError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for SubagentError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subagent_creation() {
        let subagent = Subagent {
            name: "test-agent".to_string(),
            description: "Test".to_string(),
            instructions: "Instructions".to_string(),
            allowed_tools: vec!["Read".to_string()],
            max_turns: Some(5),
            model: Some("claude-sonnet-4".to_string()),
        };

        assert_eq!(subagent.name, "test-agent");
        assert_eq!(subagent.max_turns, Some(5));
    }

    #[test]
    fn test_subagent_config_new() {
        let config = SubagentConfig::new(DelegationStrategy::Auto);
        assert!(config.subagents.is_empty());
        assert_eq!(config.delegation_strategy, DelegationStrategy::Auto);
    }

    #[test]
    fn test_subagent_config_add() {
        let mut config = SubagentConfig::new(DelegationStrategy::Manual);

        let subagent = Subagent {
            name: "agent".to_string(),
            description: "Description".to_string(),
            instructions: "Instructions".to_string(),
            allowed_tools: vec![],
            max_turns: None,
            model: None,
        };

        config.add_subagent(subagent);
        assert_eq!(config.subagents.len(), 1);
    }

    #[test]
    fn test_subagent_config_get() {
        let mut config = SubagentConfig::new(DelegationStrategy::Auto);

        let subagent = Subagent {
            name: "agent".to_string(),
            description: "Description".to_string(),
            instructions: "Instructions".to_string(),
            allowed_tools: vec![],
            max_turns: None,
            model: None,
        };

        config.add_subagent(subagent);
        assert!(config.get_subagent("agent").is_some());
        assert!(config.get_subagent("nonexistent").is_none());
    }

    #[test]
    fn test_subagent_config_to_map() {
        let mut config = SubagentConfig::new(DelegationStrategy::ToolCall);

        config.add_subagent(Subagent {
            name: "agent1".to_string(),
            description: "Agent 1".to_string(),
            instructions: "Instructions 1".to_string(),
            allowed_tools: vec![],
            max_turns: None,
            model: None,
        });

        config.add_subagent(Subagent {
            name: "agent2".to_string(),
            description: "Agent 2".to_string(),
            instructions: "Instructions 2".to_string(),
            allowed_tools: vec![],
            max_turns: None,
            model: None,
        });

        let map = config.to_map();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("agent1"));
        assert!(map.contains_key("agent2"));
    }

    #[test]
    fn test_delegation_strategy_equality() {
        assert_eq!(DelegationStrategy::Auto, DelegationStrategy::Auto);
        assert_ne!(DelegationStrategy::Auto, DelegationStrategy::Manual);
    }

    #[test]
    fn test_subagent_call_new() {
        let call = SubagentCall::new("agent", "input");
        assert_eq!(call.subagent_name, "agent");
        assert_eq!(call.input, "input");
        assert!(!call.is_executed());
    }

    #[test]
    fn test_subagent_call_executed() {
        let mut call = SubagentCall::new("agent", "input");
        assert!(!call.is_executed());

        call.output = Some("output".to_string());
        assert!(call.is_executed());
    }

    #[test]
    fn test_subagent_error_display() {
        let error = SubagentError::NotFound("agent".to_string());
        assert_eq!(format!("{}", error), "Subagent not found: agent");

        let error = SubagentError::AlreadyExists("agent".to_string());
        assert_eq!(format!("{}", error), "Subagent already exists: agent");
    }

    #[test]
    fn test_subagent_output() {
        let output = SubagentOutput {
            subagent_name: "agent".to_string(),
            messages: vec![],
        };

        assert_eq!(output.subagent_name, "agent");
        assert!(output.messages.is_empty());
    }
}
