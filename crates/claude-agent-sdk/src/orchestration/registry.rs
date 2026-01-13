//! # Agent Definitions Registry
//!
//! This module provides a centralized registry for agent definitions,
//! enabling dynamic agent discovery, registration, and management.
//!
//! ## Features
//!
//! - **Centralized Management**: Single source of truth for all agent definitions
//! - **Dynamic Registration**: Register agents at runtime
//! - **Type-Safe**: Strong typing with Rust's type system
//! - **Metadata Support**: Rich metadata for each agent (description, capabilities, etc.)
//! - **Observable**: Built-in metrics and logging support
//!
//! ## Example
//!
//! ```no_run
//! use claude_agent_sdk_rs::orchestration::registry::{AgentRegistry, AgentDefinition};
//! use claude_agent_sdk_rs::orchestration::agent::SimpleAgent;
//!
//! let mut registry = AgentRegistry::new();
//!
//! // Define an agent
//! let agent = SimpleAgent::new("researcher", "Academic researcher", |input| {
//!     Ok(AgentOutput::new(format!("Researched: {}", input.content)))
//! });
//!
//! // Register the agent
//! registry.register(Box::new(agent)).unwrap();
//!
//! // Retrieve and use the agent
//! let agent = registry.get("researcher").unwrap();
//! ```

use crate::orchestration::agent::{Agent, AgentError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Error type for agent registry operations
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Agent already registered: {0}")]
    AlreadyRegistered(String),

    #[error("Invalid agent definition: {0}")]
    InvalidDefinition(String),

    #[error("Registry error: {0}")]
    Other(String),
}

/// Result type for registry operations
pub type Result<T> = std::result::Result<T, RegistryError>;

/// Rich metadata for an agent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Unique identifier for this agent
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Detailed description of what the agent does
    pub description: String,

    /// Category/domain of the agent (e.g., "research", "analysis", "writing")
    pub category: String,

    /// Version of the agent definition
    pub version: String,

    /// List of tools this agent can use
    pub tools: Vec<String>,

    /// List of skills this agent possesses
    pub skills: Vec<String>,

    /// Tags for filtering and discovery
    #[serde(default)]
    pub tags: Vec<String>,

    /// Maximum retries for this agent
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,

    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Whether this agent is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_max_retries() -> usize {
    3
}

fn default_timeout() -> u64 {
    60
}

fn default_enabled() -> bool {
    true
}

impl AgentMetadata {
    /// Create new agent metadata
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            category: category.into(),
            version: "1.0.0".to_string(),
            tools: Vec::new(),
            skills: Vec::new(),
            tags: Vec::new(),
            max_retries: 3,
            timeout_secs: 60,
            enabled: true,
        }
    }

    /// Add a tool to the agent's capabilities
    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tools.push(tool.into());
        self
    }

    /// Add tools to the agent's capabilities
    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Add a skill to the agent's capabilities
    pub fn with_skill(mut self, skill: impl Into<String>) -> Self {
        self.skills.push(skill.into());
        self
    }

    /// Add a tag for filtering
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Enable or disable the agent
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Check if agent has a specific tool
    pub fn has_tool(&self, tool: &str) -> bool {
        self.tools.iter().any(|t| t == tool)
    }

    /// Check if agent has a specific skill
    pub fn has_skill(&self, skill: &str) -> bool {
        self.skills.iter().any(|s| s == skill)
    }

    /// Check if agent has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

/// Filter criteria for searching agents
#[derive(Debug, Clone, Default)]
pub struct AgentFilter {
    /// Filter by category
    pub category: Option<String>,

    /// Filter by tags (AND logic - agent must have all tags)
    pub tags: Vec<String>,

    /// Filter by tools (AND logic - agent must have all tools)
    pub tools: Vec<String>,

    /// Filter by skills (AND logic - agent must have all skills)
    pub skills: Vec<String>,

    /// Only return enabled agents
    pub enabled_only: bool,
}

impl AgentFilter {
    /// Create a new filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Add a tag requirement
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add a tool requirement
    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tools.push(tool.into());
        self
    }

    /// Add a skill requirement
    pub fn with_skill(mut self, skill: impl Into<String>) -> Self {
        self.skills.push(skill.into());
        self
    }

    /// Only return enabled agents
    pub fn enabled_only(mut self) -> Self {
        self.enabled_only = true;
        self
    }

    /// Check if metadata matches this filter
    pub fn matches(&self, metadata: &AgentMetadata) -> bool {
        // Check enabled status
        if self.enabled_only && !metadata.enabled {
            return false;
        }

        // Check category
        if let Some(ref category) = self.category {
            if &metadata.category != category {
                return false;
            }
        }

        // Check tags (agent must have all specified tags)
        for tag in &self.tags {
            if !metadata.has_tag(tag) {
                return false;
            }
        }

        // Check tools (agent must have all specified tools)
        for tool in &self.tools {
            if !metadata.has_tool(tool) {
                return false;
            }
        }

        // Check skills (agent must have all specified skills)
        for skill in &self.skills {
            if !metadata.has_skill(skill) {
                return false;
            }
        }

        true
    }
}

/// Centralized registry for agent definitions
pub struct AgentRegistry {
    /// Map of agent ID to (agent, metadata) pairs
    agents: Arc<RwLock<HashMap<String, (Box<dyn Agent>, AgentMetadata)>>>,

    /// Registry name for logging
    name: String,
}

impl AgentRegistry {
    /// Create a new agent registry
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            name: "AgentRegistry".to_string(),
        }
    }

    /// Create a new named registry
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            name: name.into(),
        }
    }

    /// Register an agent with metadata
    pub async fn register(
        &self,
        agent: Box<dyn Agent>,
        metadata: AgentMetadata,
    ) -> Result<()> {
        let id = metadata.id.clone();

        // Validate metadata
        if id.is_empty() {
            return Err(RegistryError::InvalidDefinition("Agent ID cannot be empty".to_string()));
        }

        let mut agents = self.agents.write().await;

        // Check for duplicate
        if agents.contains_key(&id) {
            return Err(RegistryError::AlreadyRegistered(id));
        }

        // Validate that agent name matches metadata
        if agent.name() != metadata.name {
            return Err(RegistryError::InvalidDefinition(format!(
                "Agent name '{}' does not match metadata name '{}'",
                agent.name(),
                metadata.name
            )));
        }

        agents.insert(id.clone(), (agent, metadata.clone()));

        tracing::info!(
            registry = self.name,
            agent_id = id,
            agent_name = metadata.name,
            category = metadata.category,
            "Agent registered"
        );

        Ok(())
    }

    /// Register or update an agent
    pub async fn register_or_update(
        &self,
        agent: Box<dyn Agent>,
        metadata: AgentMetadata,
    ) -> Result<()> {
        let id = metadata.id.clone();

        let mut agents = self.agents.write().await;

        let is_update = agents.contains_key(&id);

        agents.insert(id.clone(), (agent, metadata));

        if is_update {
            tracing::info!(
                registry = self.name,
                agent_id = id,
                "Agent updated"
            );
        } else {
            tracing::info!(
                registry = self.name,
                agent_id = id,
                "Agent registered"
            );
        }

        Ok(())
    }

    /// Unregister an agent
    pub async fn unregister(&self, id: &str) -> Result<()> {
        let mut agents = self.agents.write().await;

        if agents.remove(id).is_some() {
            tracing::info!(
                registry = self.name,
                agent_id = id,
                "Agent unregistered"
            );
            Ok(())
        } else {
            Err(RegistryError::AgentNotFound(id.to_string()))
        }
    }

    /// Get an agent by ID
    pub async fn get(&self, id: &str) -> Result<Arc<dyn Agent>> {
        let agents = self.agents.read().await;

        let (_agent, metadata) = agents.get(id).ok_or_else(|| RegistryError::AgentNotFound(id.to_string()))?;

        // Check if agent is enabled
        if !metadata.enabled {
            return Err(RegistryError::AgentNotFound(format!("{} (disabled)", id)));
        }

        // We can't return the agent directly, so we need to clone it
        // This is a limitation - in practice, you'd want a different approach
        // For now, we'll return an error indicating this
        Err(RegistryError::Other("Direct agent retrieval not supported - use get_metadata or list_agents".to_string()))
    }

    /// Get agent metadata by ID
    pub async fn get_metadata(&self, id: &str) -> Result<AgentMetadata> {
        let agents = self.agents.read().await;

        let (_agent, metadata) = agents.get(id)
            .ok_or_else(|| RegistryError::AgentNotFound(id.to_string()))?;

        Ok(metadata.clone())
    }

    /// Check if an agent exists
    pub async fn contains(&self, id: &str) -> bool {
        let agents = self.agents.read().await;
        agents.contains_key(id)
    }

    /// Check if an agent exists and is enabled
    pub async fn is_enabled(&self, id: &str) -> bool {
        let agents = self.agents.read().await;
        agents.get(id).map(|(_, m)| m.enabled).unwrap_or(false)
    }

    /// List all agent IDs
    pub async fn list_ids(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }

    /// List all agent metadata
    pub async fn list_metadata(&self) -> Vec<AgentMetadata> {
        let agents = self.agents.read().await;
        agents.values().map(|(_, m)| m.clone()).collect()
    }

    /// Find agents matching a filter
    pub async fn find(&self, filter: &AgentFilter) -> Vec<AgentMetadata> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|(_, metadata)| filter.matches(metadata))
            .map(|(_, metadata)| metadata.clone())
            .collect()
    }

    /// Count total agents
    pub async fn count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.len()
    }

    /// Count enabled agents
    pub async fn count_enabled(&self) -> usize {
        let agents = self.agents.read().await;
        agents.values().filter(|(_, m)| m.enabled).count()
    }

    /// Clear all agents
    pub async fn clear(&self) {
        let mut agents = self.agents.write().await;
        let count = agents.len();
        agents.clear();

        tracing::info!(
            registry = self.name,
            count = count,
            "Registry cleared"
        );
    }

    /// Execute an agent by ID
    pub async fn execute_agent(
        &self,
        id: &str,
        input: crate::orchestration::agent::AgentInput,
    ) -> std::result::Result<crate::orchestration::agent::AgentOutput, AgentError> {
        let agents = self.agents.read().await;

        let (agent, metadata) = agents.get(id)
            .ok_or_else(|| AgentError::InvalidInput(format!("Agent not found: {}", id)))?;

        if !metadata.enabled {
            return Err(AgentError::InvalidInput(format!("Agent is disabled: {}", id)));
        }

        agent.execute(input).await
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating agent registries with predefined agents
pub struct AgentRegistryBuilder {
    registry: AgentRegistry,
}

impl AgentRegistryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            registry: AgentRegistry::new(),
        }
    }

    /// Create a new builder with a named registry
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            registry: AgentRegistry::with_name(name),
        }
    }

    /// Add an agent to the registry
    pub async fn with_agent(
        self,
        agent: Box<dyn Agent>,
        metadata: AgentMetadata,
    ) -> Result<Self> {
        self.registry.register(agent, metadata).await?;
        Ok(self)
    }

    /// Build the registry
    pub fn build(self) -> AgentRegistry {
        self.registry
    }
}

impl Default for AgentRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::agent::{AgentInput, AgentOutput, SimpleAgent};

    #[tokio::test]
    async fn test_agent_metadata_creation() {
        let metadata = AgentMetadata::new("test-agent", "Test Agent", "A test agent", "test")
            .with_tool("web-search")
            .with_skill("research")
            .with_tag("experimental")
            .with_version("2.0.0")
            .with_max_retries(5)
            .with_timeout(120);

        assert_eq!(metadata.id, "test-agent");
        assert_eq!(metadata.name, "Test Agent");
        assert_eq!(metadata.category, "test");
        assert!(metadata.has_tool("web-search"));
        assert!(metadata.has_skill("research"));
        assert!(metadata.has_tag("experimental"));
        assert_eq!(metadata.version, "2.0.0");
        assert_eq!(metadata.max_retries, 5);
        assert_eq!(metadata.timeout_secs, 120);
    }

    #[tokio::test]
    async fn test_agent_registry_register() {
        let registry = AgentRegistry::new();

        let agent = SimpleAgent::new("TestAgent", "A test agent", |input| {
            Ok(AgentOutput::new(format!("Processed: {}", input.content)))
        });

        let metadata = AgentMetadata::new("test-agent", "TestAgent", "Description", "test");

        assert!(registry.register(Box::new(agent), metadata).await.is_ok());
        assert!(registry.contains("test-agent").await);
    }

    #[tokio::test]
    async fn test_agent_registry_duplicate() {
        let registry = AgentRegistry::new();

        let agent1 = SimpleAgent::new("TestAgent", "A test agent", |input| {
            Ok(AgentOutput::new(format!("1: {}", input.content)))
        });

        let agent2 = SimpleAgent::new("TestAgent", "Another test agent", |input| {
            Ok(AgentOutput::new(format!("2: {}", input.content)))
        });

        let metadata = AgentMetadata::new("test-agent", "TestAgent", "Description", "test");

        assert!(registry.register(Box::new(agent1), metadata.clone()).await.is_ok());
        assert!(registry.register(Box::new(agent2), metadata).await.is_err());
    }

    #[tokio::test]
    async fn test_agent_registry_filter() {
        let registry = AgentRegistry::new();

        // Register multiple agents
        for i in 0..3 {
            let agent = SimpleAgent::new(
                format!("Agent{}", i),
                format!("Agent number {}", i),
                move |input| Ok(AgentOutput::new(format!("{}: {}", i, input.content)))
            );

            let metadata = AgentMetadata::new(
                format!("agent-{}", i),
                format!("Agent{}", i),
                format!("Description {}", i),
                if i % 2 == 0 { "research" } else { "analysis" }
            )
                .with_tool(if i % 2 == 0 { "web-search" } else { "data-analysis" })
                .with_tag("test");

            registry.register(Box::new(agent), metadata).await.unwrap();
        }

        // Filter by category
        let filter = AgentFilter::new().with_category("research");
        let results = registry.find(&filter).await;
        assert_eq!(results.len(), 2);

        // Filter by tool
        let filter = AgentFilter::new().with_tool("data-analysis");
        let results = registry.find(&filter).await;
        assert_eq!(results.len(), 1);

        // Filter by tag
        let filter = AgentFilter::new().with_tag("test");
        let results = registry.find(&filter).await;
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_agent_registry_unregister() {
        let registry = AgentRegistry::new();

        let agent = SimpleAgent::new("TestAgent", "A test agent", |_input| {
            Ok(AgentOutput::new("Done"))
        });

        let metadata = AgentMetadata::new("test-agent", "TestAgent", "Description", "test");

        registry.register(Box::new(agent), metadata).await.unwrap();
        assert!(registry.contains("test-agent").await);

        registry.unregister("test-agent").await.unwrap();
        assert!(!registry.contains("test-agent").await);

        assert!(registry.unregister("test-agent").await.is_err());
    }

    #[tokio::test]
    async fn test_agent_registry_enabled() {
        let registry = AgentRegistry::new();

        let agent = SimpleAgent::new("TestAgent", "A test agent", |_input| {
            Ok(AgentOutput::new("Done"))
        });

        let metadata = AgentMetadata::new("test-agent", "TestAgent", "Description", "test")
            .with_enabled(false);

        registry.register(Box::new(agent), metadata).await.unwrap();

        // Agent exists but is disabled
        assert!(registry.contains("test-agent").await);
        assert!(!registry.is_enabled("test-agent").await);
    }

    #[tokio::test]
    async fn test_agent_registry_execute() {
        let registry = AgentRegistry::new();

        let agent = SimpleAgent::new("TestAgent", "A test agent", |input| {
            Ok(AgentOutput::new(format!("Echo: {}", input.content)))
        });

        let metadata = AgentMetadata::new("test-agent", "TestAgent", "Description", "test");

        registry.register(Box::new(agent), metadata).await.unwrap();

        let input = AgentInput::new("Hello");
        let output = registry.execute_agent("test-agent", input).await.unwrap();

        assert_eq!(output.content, "Echo: Hello");
    }

    #[tokio::test]
    async fn test_agent_registry_builder() {
        let registry = AgentRegistryBuilder::new()
            .with_agent(
                Box::new(SimpleAgent::new("Agent1", "First", |input| {
                    Ok(AgentOutput::new(format!("1: {}", input.content)))
                })),
                AgentMetadata::new("agent-1", "Agent1", "First", "test")
            )
            .await
            .unwrap()
            .with_agent(
                Box::new(SimpleAgent::new("Agent2", "Second", |input| {
                    Ok(AgentOutput::new(format!("2: {}", input.content)))
                })),
                AgentMetadata::new("agent-2", "Agent2", "Second", "test")
            )
            .await
            .unwrap()
            .build();

        assert_eq!(registry.count().await, 2);
    }
}
