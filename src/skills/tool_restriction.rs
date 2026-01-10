//! Tool Restriction for Skills
//!
//! This module implements tool restriction based on the `allowed-tools` field
//! in SKILL.md metadata, as specified in Claude Code documentation.
//!
//! Based on: https://code.claude.com/docs/en/skills

use std::collections::HashSet;
use std::fmt;

/// Errors for tool restriction checking
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolRestrictionError {
    /// Tool is not in the allowed list
    ToolNotAllowed {
        /// The tool that was requested
        tool: String,
        /// Tools that are allowed
        allowed: Vec<String>,
    },

    /// Invalid tool specification format
    InvalidSpec(String),
}

impl fmt::Display for ToolRestrictionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolRestrictionError::ToolNotAllowed { tool, allowed } => {
                write!(
                    f,
                    "Tool '{}' is not allowed. Allowed tools: {:?}",
                    tool, allowed
                )
            }
            ToolRestrictionError::InvalidSpec(spec) => {
                write!(f, "Invalid tool specification: '{}'", spec)
            }
        }
    }
}

impl std::error::Error for ToolRestrictionError {}

/// Tool restriction enforcement for skills
///
/// # Overview
///
/// The `allowed-tools` field in SKILL.md metadata specifies which tools
/// a skill is allowed to use. This module enforces those restrictions.
///
/// # Tool Specifications
///
/// Tools can be specified in several ways:
///
/// ```text
/// allowed-tools:
///   - Read                    # Simple tool name
///   - Grep                    # Simple tool name
///   - "Bash(python:*)"        # Tool with parameter restrictions
///   - "*"                     # Wildcard (all tools)
/// ```
///
/// # Parameter Restrictions
///
/// Tools can have parameter restrictions like `Bash(python:*)` which means:
/// - The Bash tool is allowed
/// - Only when the command starts with "python:"
/// - Example: `Bash(command="python:script.py")` ✅
/// - Example: `Bash(command="node script.js")` ❌
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk_rs::skills::tool_restriction::ToolRestriction;
///
/// let restriction = ToolRestriction::new(Some(vec![
///     "Read".to_string(),
///     "Grep".to_string(),
///     "Bash(python:*)".to_string(),
/// ]));
///
/// // Check if tools are allowed
/// assert!(restriction.is_tool_allowed("Read"));
/// assert!(restriction.is_tool_allowed("Bash(python:script.py)"));
/// assert!(!restriction.is_tool_allowed("Write"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRestriction {
    /// Set of allowed tool specifications
    allowed_tools: Option<HashSet<String>>,
}

impl ToolRestriction {
    /// Create a new tool restriction
    ///
    /// # Arguments
    ///
    /// * `allowed_tools` - Optional list of allowed tool specifications
    ///   - None: No restrictions (all tools allowed)
    ///   - Some([]): No tools allowed
    ///   - Some([...]): Specific tools allowed
    ///
    /// # Example
    ///
    /// ```
    /// use claude_agent_sdk_rs::skills::tool_restriction::ToolRestriction;
    ///
    /// // No restrictions
    /// let unrestricted = ToolRestriction::new(None);
    /// assert!(unrestricted.is_tool_allowed("AnyTool"));
    ///
    /// // Specific restrictions
    /// let restricted = ToolRestriction::new(Some(vec![
    ///     "Read".to_string(),
    ///     "Grep".to_string(),
    /// ]));
    /// assert!(restricted.is_tool_allowed("Read"));
    /// assert!(!restricted.is_tool_allowed("Write"));
    /// ```
    pub fn new(allowed_tools: Option<Vec<String>>) -> Self {
        Self {
            allowed_tools: allowed_tools.map(|tools| tools.into_iter().collect()),
        }
    }

    /// Create a restriction with no tool limits
    ///
    /// # Example
    ///
    /// ```
    /// use claude_agent_sdk_rs::skills::tool_restriction::ToolRestriction;
    ///
    /// let restriction = ToolRestriction::unrestricted();
    /// assert!(restriction.is_tool_allowed("AnyTool"));
    /// assert!(restriction.is_tool_allowed("AnotherTool"));
    /// ```
    pub fn unrestricted() -> Self {
        Self { allowed_tools: None }
    }

    /// Check if a tool is allowed
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Tool name with optional parameters (e.g., "Bash(python:script.py)")
    ///
    /// # Returns
    ///
    /// true if tool is allowed, false otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use claude_agent_sdk_rs::skills::tool_restriction::ToolRestriction;
    ///
    /// let restriction = ToolRestriction::new(Some(vec![
    ///     "Read".to_string(),
    ///     "Bash(python:*)".to_string(),
    /// ]));
    ///
    /// assert!(restriction.is_tool_allowed("Read"));
    /// assert!(restriction.is_tool_allowed("Bash(python:script.py)"));
    /// assert!(!restriction.is_tool_allowed("Write"));
    /// assert!(!restriction.is_tool_allowed("Bash(node:script.js)"));
    /// ```
    pub fn is_tool_allowed(&self, tool_name: &str) -> bool {
        match &self.allowed_tools {
            None => true, // No restrictions
            Some(allowed) => {
                // Check for wildcard
                if allowed.contains("*") {
                    return true;
                }

                // Parse the tool name
                let (base_tool, params) = Self::parse_tool_spec(tool_name);

                // Check for exact match
                if allowed.contains(tool_name) {
                    return true;
                }

                // Check for base tool match (without parameters)
                if allowed.contains(&base_tool) {
                    return true;
                }

                // Check for parameter-restricted tools
                for allowed_spec in allowed {
                    if let Some((allowed_base, allowed_pattern)) =
                        Self::parse_tool_spec_with_pattern(allowed_spec)
                    {
                        if allowed_base == base_tool {
                            // Check if parameters match the pattern
                            if let Some(params) = &params {
                                if Self::pattern_matches(&allowed_pattern, params) {
                                    return true;
                                }
                            }
                        }
                    }
                }

                false
            }
        }
    }

    /// Validate a tool and return an error if not allowed
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Tool name with optional parameters
    ///
    /// # Returns
    ///
    /// Ok(()) if tool is allowed
    /// Err(ToolRestrictionError) if not allowed
    ///
    /// # Example
    ///
    /// ```
    /// use claude_agent_sdk_rs::skills::tool_restriction::ToolRestriction;
    ///
    /// let restriction = ToolRestriction::new(Some(vec!["Read".to_string()]));
    ///
    /// assert!(restriction.validate_tool("Read").is_ok());
    /// assert!(restriction.validate_tool("Write").is_err());
    /// ```
    pub fn validate_tool(&self, tool_name: &str) -> Result<(), ToolRestrictionError> {
        if self.is_tool_allowed(tool_name) {
            Ok(())
        } else {
            Err(ToolRestrictionError::ToolNotAllowed {
                tool: tool_name.to_string(),
                allowed: self
                    .allowed_tools
                    .as_ref()
                    .map(|s| s.iter().cloned().collect())
                    .unwrap_or_default(),
            })
        }
    }

    /// Get list of allowed tools
    ///
    /// # Returns
    ///
    /// None if unrestricted, Some(VEC) if restricted
    ///
    /// # Example
    ///
    /// ```
    /// use claude_agent_sdk_rs::skills::tool_restriction::ToolRestriction;
    ///
    /// let restriction = ToolRestriction::new(Some(vec![
    ///     "Read".to_string(),
    ///     "Grep".to_string(),
    /// ]));
    ///
    /// let allowed = restriction.get_allowed_tools();
    /// assert!(allowed.is_some());
    /// assert_eq!(allowed.unwrap().len(), 2);
    /// ```
    pub fn get_allowed_tools(&self) -> Option<Vec<String>> {
        self.allowed_tools
            .as_ref()
            .map(|s| s.iter().cloned().collect())
    }

    /// Check if this restriction allows all tools
    ///
    /// # Returns
    ///
    /// true if no restrictions are in place
    ///
    /// # Example
    ///
    /// ```
    /// use claude_agent_sdk_rs::skills::tool_restriction::ToolRestriction;
    ///
    /// let unrestricted = ToolRestriction::unrestricted();
    /// assert!(unrestricted.is_unrestricted());
    ///
    /// let restricted = ToolRestriction::new(Some(vec!["Read".to_string()]));
    /// assert!(!restricted.is_unrestricted());
    /// ```
    pub fn is_unrestricted(&self) -> bool {
        self.allowed_tools.is_none()
    }

    /// Parse tool specification into base tool and parameters
    ///
    /// # Examples
    ///
    /// - "Bash(python:script.py)" -> ("Bash", Some("python:script.py"))
    /// - "Read" -> ("Read", None)
    fn parse_tool_spec(tool_spec: &str) -> (String, Option<String>) {
        if let Some(params) = tool_spec.strip_suffix(')') {
            if let Some((base, args)) = params.split_once('(') {
                return (base.to_string(), Some(args.to_string()));
            }
        }
        (tool_spec.to_string(), None)
    }

    /// Parse tool specification with pattern matching support
    ///
    /// # Examples
    ///
    /// - "Bash(python:*)" -> Some(("Bash", "python:*"))
    /// - "Read" -> None
    fn parse_tool_spec_with_pattern(spec: &str) -> Option<(String, String)> {
        if let Some(params) = spec.strip_suffix(')') {
            if let Some((base, pattern)) = params.split_once('(') {
                if pattern.contains('*') {
                    return Some((base.to_string(), pattern.to_string()));
                }
            }
        }
        None
    }

    /// Check if parameters match a pattern
    ///
    /// # Patterns
    ///
    /// - "*" matches anything
    /// - "python:*" matches anything starting with "python:"
    /// - "*" matches all parameters
    ///
    /// # Examples
    ///
    /// - pattern_matches("python:*", "python:script.py") -> true
    /// - pattern_matches("python:*", "node:script.js") -> false
    /// - pattern_matches("*", "anything") -> true
    fn pattern_matches(pattern: &str, params: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if let Some(prefix) = pattern.strip_suffix('*') {
            return params.starts_with(prefix);
        }

        pattern == params
    }

    /// Add an allowed tool to the restriction
    ///
    /// # Example
    ///
    /// ```
    /// use claude_agent_sdk_rs::skills::tool_restriction::ToolRestriction;
    ///
    /// let mut restriction = ToolRestriction::new(Some(vec!["Read".to_string()]));
    /// restriction.add_tool("Grep".to_string());
    ///
    /// assert!(restriction.is_tool_allowed("Grep"));
    /// ```
    pub fn add_tool(&mut self, tool: String) {
        self.allowed_tools
            .get_or_insert_with(HashSet::new)
            .insert(tool);
    }

    /// Remove an allowed tool from the restriction
    ///
    /// # Example
    ///
    /// ```
    /// use claude_agent_sdk_rs::skills::tool_restriction::ToolRestriction;
    ///
    /// let mut restriction = ToolRestriction::new(Some(vec![
    ///     "Read".to_string(),
    ///     "Grep".to_string(),
    /// ]));
    ///
    /// restriction.remove_tool("Grep");
    /// assert!(!restriction.is_tool_allowed("Grep"));
    /// ```
    pub fn remove_tool(&mut self, tool: &str) {
        if let Some(allowed) = &mut self.allowed_tools {
            allowed.remove(tool);
        }
    }
}

impl Default for ToolRestriction {
    fn default() -> Self {
        Self::unrestricted()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unrestricted() {
        let restriction = ToolRestriction::unrestricted();
        assert!(restriction.is_tool_allowed("AnyTool"));
        assert!(restriction.is_tool_allowed("AnotherTool"));
        assert!(restriction.is_unrestricted());
    }

    #[test]
    fn test_specific_tools() {
        let restriction = ToolRestriction::new(Some(vec![
            "Read".to_string(),
            "Grep".to_string(),
        ]));

        assert!(restriction.is_tool_allowed("Read"));
        assert!(restriction.is_tool_allowed("Grep"));
        assert!(!restriction.is_tool_allowed("Write"));
        assert!(!restriction.is_tool_allowed("Bash"));
    }

    #[test]
    fn test_tool_with_parameters() {
        let restriction = ToolRestriction::new(Some(vec![
            "Bash(python:*)".to_string(),
            "Read".to_string(),
        ]));

        // Bash with python: should be allowed
        assert!(restriction.is_tool_allowed("Bash(python:script.py)"));
        assert!(restriction.is_tool_allowed("Bash(python:-m pytest)"));

        // Bash with other commands should not be allowed
        assert!(!restriction.is_tool_allowed("Bash(node:script.js)"));
        assert!(!restriction.is_tool_allowed("Bash(ls -la)"));

        // Read should be allowed
        assert!(restriction.is_tool_allowed("Read"));
    }

    #[test]
    fn test_wildcard() {
        let restriction = ToolRestriction::new(Some(vec!["*".to_string()]));

        assert!(restriction.is_tool_allowed("AnyTool"));
        assert!(restriction.is_tool_allowed("Bash(anything)"));
        assert!(restriction.is_tool_allowed("Read"));
    }

    #[test]
    fn test_validate_tool() {
        let restriction = ToolRestriction::new(Some(vec!["Read".to_string()]));

        assert!(restriction.validate_tool("Read").is_ok());

        let result = restriction.validate_tool("Write");
        assert!(result.is_err());
        if let Err(ToolRestrictionError::ToolNotAllowed { tool, .. }) = result {
            assert_eq!(tool, "Write");
        } else {
            panic!("Expected ToolNotAllowed error");
        }
    }

    #[test]
    fn test_parse_tool_spec() {
        assert_eq!(
            ToolRestriction::parse_tool_spec("Bash(python:script.py)"),
            ("Bash".to_string(), Some("python:script.py".to_string()))
        );

        assert_eq!(
            ToolRestriction::parse_tool_spec("Read"),
            ("Read".to_string(), None)
        );
    }

    #[test]
    fn test_pattern_matches() {
        assert!(ToolRestriction::pattern_matches("python:*", "python:script.py"));
        assert!(ToolRestriction::pattern_matches("python:*", "python:-m pytest"));
        assert!(!ToolRestriction::pattern_matches("python:*", "node:script.js"));

        assert!(ToolRestriction::pattern_matches("*", "anything"));
        assert!(ToolRestriction::pattern_matches("*", ""));
    }

    #[test]
    fn test_add_tool() {
        let mut restriction = ToolRestriction::new(Some(vec!["Read".to_string()]));
        assert!(!restriction.is_tool_allowed("Grep"));

        restriction.add_tool("Grep".to_string());
        assert!(restriction.is_tool_allowed("Grep"));
    }

    #[test]
    fn test_remove_tool() {
        let mut restriction = ToolRestriction::new(Some(vec![
            "Read".to_string(),
            "Grep".to_string(),
        ]));

        assert!(restriction.is_tool_allowed("Grep"));

        restriction.remove_tool("Grep");
        assert!(!restriction.is_tool_allowed("Grep"));
        assert!(restriction.is_tool_allowed("Read"));
    }

    #[test]
    fn test_get_allowed_tools() {
        let unrestricted = ToolRestriction::unrestricted();
        assert!(unrestricted.get_allowed_tools().is_none());

        let restricted = ToolRestriction::new(Some(vec![
            "Read".to_string(),
            "Grep".to_string(),
        ]));
        let allowed = restricted.get_allowed_tools();
        assert!(allowed.is_some());
        assert_eq!(allowed.unwrap().len(), 2);
    }

    #[test]
    fn test_default() {
        let restriction = ToolRestriction::default();
        assert!(restriction.is_unrestricted());
        assert!(restriction.is_tool_allowed("AnyTool"));
    }

    #[test]
    fn test_empty_allowed_list() {
        let restriction = ToolRestriction::new(Some(vec![]));
        assert!(!restriction.is_tool_allowed("Read"));
        assert!(!restriction.is_tool_allowed("AnyTool"));
    }
}
