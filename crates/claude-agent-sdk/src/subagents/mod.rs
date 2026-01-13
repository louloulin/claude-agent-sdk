//! Subagent system for Claude Agent SDK
//!
//! This module provides functionality for creating and managing subagents,
//! which are specialized Claude instances with specific capabilities and instructions.

mod types;

pub use types::{
    DelegationStrategy, Subagent, SubagentCall, SubagentConfig, SubagentError,
    SubagentOutput,
};

/// Subagent executor for managing and executing subagents
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk::subagents::{SubagentExecutor, Subagent, DelegationStrategy};
///
/// #[tokio::main]
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let executor = SubagentExecutor::new(DelegationStrategy::Auto);
///
///     // Register a subagent
///     let subagent = Subagent {
///         name: "code-reviewer".to_string(),
///         description: "Expert code reviewer".to_string(),
///         instructions: "Review code for bugs and best practices".to_string(),
///         allowed_tools: vec!["Read".to_string(), "Grep".to_string()],
///         max_turns: Some(5),
///         model: Some("claude-sonnet-4".to_string()),
///     };
///
///     executor.register(subagent)?;
///
///     // Execute the subagent
///     let output = executor.execute("code-reviewer", "Review this file").await?;
///     println!("Output: {:?}", output);
///
///     Ok(())
/// }
/// ```
pub struct SubagentExecutor {
    subagents: std::collections::HashMap<String, Subagent>,
    strategy: DelegationStrategy,
}

impl SubagentExecutor {
    /// Create a new subagent executor
    ///
    /// # Arguments
    ///
    /// * `strategy` - The delegation strategy to use
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::subagents::{SubagentExecutor, DelegationStrategy};
    /// let executor = SubagentExecutor::new(DelegationStrategy::Auto);
    /// ```
    pub fn new(strategy: DelegationStrategy) -> Self {
        Self {
            subagents: std::collections::HashMap::new(),
            strategy,
        }
    }

    /// Register a subagent
    ///
    /// # Arguments
    ///
    /// * `subagent` - The subagent to register
    ///
    /// # Errors
    ///
    /// Returns an error if a subagent with the same name already exists
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::subagents::{SubagentExecutor, Subagent, DelegationStrategy};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let executor = SubagentExecutor::new(DelegationStrategy::Auto);
    /// let subagent = Subagent {
    ///     name: "my-agent".to_string(),
    ///     description: "Description".to_string(),
    ///     instructions: "Instructions".to_string(),
    ///     allowed_tools: vec![],
    ///     max_turns: Some(5),
    ///     model: None,
    /// };
    /// executor.register(subagent)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn register(&mut self, subagent: Subagent) -> Result<(), SubagentError> {
        if self.subagents.contains_key(&subagent.name) {
            return Err(SubagentError::AlreadyExists(subagent.name));
        }
        self.subagents.insert(subagent.name.clone(), subagent);
        Ok(())
    }

    /// Execute a subagent by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the subagent to execute
    /// * `input` - The input to provide to the subagent
    ///
    /// # Errors
    ///
    /// Returns an error if the subagent is not found or execution fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::subagents::{SubagentExecutor, DelegationStrategy};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let executor = SubagentExecutor::new(DelegationStrategy::Auto);
    /// # // ... register subagent ...
    /// let output = executor.execute("my-agent", "Hello").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute(
        &self,
        name: &str,
        input: &str,
    ) -> Result<SubagentOutput, SubagentError> {
        let subagent = self
            .subagents
            .get(name)
            .ok_or_else(|| SubagentError::NotFound(name.to_string()))?;

        // Build system prompt from description and instructions
        let system_prompt = format!(
            "{}\n\nInstructions:\n{}",
            subagent.description, subagent.instructions
        );

        // Build ClaudeAgentOptions using match to handle conditional fields
        let options = match (&subagent.model, subagent.max_turns) {
            (Some(model), Some(max_turns)) => {
                crate::types::config::ClaudeAgentOptions::builder()
                    .system_prompt(crate::types::config::SystemPrompt::Text(
                        system_prompt,
                    ))
                    .allowed_tools(subagent.allowed_tools.clone())
                    .model(model.clone())
                    .max_turns(max_turns)
                    .build()
            }
            (Some(model), None) => {
                crate::types::config::ClaudeAgentOptions::builder()
                    .system_prompt(crate::types::config::SystemPrompt::Text(
                        system_prompt,
                    ))
                    .allowed_tools(subagent.allowed_tools.clone())
                    .model(model.clone())
                    .build()
            }
            (None, Some(max_turns)) => {
                crate::types::config::ClaudeAgentOptions::builder()
                    .system_prompt(crate::types::config::SystemPrompt::Text(
                        system_prompt,
                    ))
                    .allowed_tools(subagent.allowed_tools.clone())
                    .max_turns(max_turns)
                    .build()
            }
            (None, None) => {
                crate::types::config::ClaudeAgentOptions::builder()
                    .system_prompt(crate::types::config::SystemPrompt::Text(
                        system_prompt,
                    ))
                    .allowed_tools(subagent.allowed_tools.clone())
                    .build()
            }
        };

        // Execute query
        let messages = crate::query::query(input, Some(options))
            .await
            .map_err(|e| SubagentError::ExecutionFailed(format!("Query failed: {}", e)))?;

        // Convert messages to JSON values for SubagentOutput
        let json_messages = messages
            .into_iter()
            .map(|msg| serde_json::to_value(msg).map_err(|e| {
                SubagentError::ExecutionFailed(format!("Failed to serialize message: {}", e))
            }))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SubagentOutput {
            subagent_name: name.to_string(),
            messages: json_messages,
        })
    }

    /// Get all registered subagent names
    ///
    /// # Returns
    ///
    /// A vector of subagent names
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::subagents::{SubagentExecutor, DelegationStrategy};
    /// # let executor = SubagentExecutor::new(DelegationStrategy::Auto);
    /// let names = executor.list_subagents();
    /// println!("Available subagents: {:?}", names);
    /// ```
    pub fn list_subagents(&self) -> Vec<String> {
        self.subagents.keys().cloned().collect()
    }

    /// Check if a subagent exists
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the subagent to check
    ///
    /// # Returns
    ///
    /// `true` if the subagent exists, `false` otherwise
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::subagents::{SubagentExecutor, DelegationStrategy};
    /// # let executor = SubagentExecutor::new(DelegationStrategy::Auto);
    /// if executor.has_subagent("my-agent") {
    ///     println!("Agent exists");
    /// }
    /// ```
    pub fn has_subagent(&self, name: &str) -> bool {
        self.subagents.contains_key(name)
    }

    /// Get the delegation strategy
    ///
    /// # Returns
    ///
    /// The current delegation strategy
    pub fn strategy(&self) -> &DelegationStrategy {
        &self.strategy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = SubagentExecutor::new(DelegationStrategy::Auto);
        assert!(matches!(executor.strategy(), &DelegationStrategy::Auto));
    }

    #[test]
    fn test_register_subagent() {
        let mut executor = SubagentExecutor::new(DelegationStrategy::Auto);

        let subagent = Subagent {
            name: "test-agent".to_string(),
            description: "Test agent".to_string(),
            instructions: "Test instructions".to_string(),
            allowed_tools: vec![],
            max_turns: Some(5),
            model: None,
        };

        assert!(executor.register(subagent).is_ok());
        assert!(executor.has_subagent("test-agent"));
    }

    #[test]
    fn test_register_duplicate_fails() {
        let mut executor = SubagentExecutor::new(DelegationStrategy::Auto);

        let subagent = Subagent {
            name: "test-agent".to_string(),
            description: "Test agent".to_string(),
            instructions: "Test instructions".to_string(),
            allowed_tools: vec![],
            max_turns: Some(5),
            model: None,
        };

        assert!(executor.register(subagent.clone()).is_ok());
        assert!(matches!(
            executor.register(subagent),
            Err(SubagentError::AlreadyExists(_))
        ));
    }

    #[test]
    fn test_list_subagents() {
        let mut executor = SubagentExecutor::new(DelegationStrategy::Auto);

        let subagent1 = Subagent {
            name: "agent1".to_string(),
            description: "Agent 1".to_string(),
            instructions: "Instructions 1".to_string(),
            allowed_tools: vec![],
            max_turns: Some(5),
            model: None,
        };

        let subagent2 = Subagent {
            name: "agent2".to_string(),
            description: "Agent 2".to_string(),
            instructions: "Instructions 2".to_string(),
            allowed_tools: vec![],
            max_turns: Some(10),
            model: Some("claude-sonnet-4".to_string()),
        };

        executor.register(subagent1).unwrap();
        executor.register(subagent2).unwrap();

        let names = executor.list_subagents();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"agent1".to_string()));
        assert!(names.contains(&"agent2".to_string()));
    }

    #[test]
    fn test_execute_not_found() {
        let executor = SubagentExecutor::new(DelegationStrategy::Auto);

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(executor.execute("nonexistent", "input"));

        assert!(matches!(result, Err(SubagentError::NotFound(_))));
    }
}
