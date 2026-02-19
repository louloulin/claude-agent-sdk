//! Example: Subagents System
//!
//! This example demonstrates the subagent system for creating specialized
//! Claude instances with specific capabilities and instructions.
//!
//! What it demonstrates:
//! 1. Creating subagents with Subagent struct
//! 2. Registering subagents in SubagentExecutor
//! 3. Executing subagents with different delegation strategies
//! 4. Configuring subagent tools, models, and turn limits

use claude_agent_sdk::subagents::{
    DelegationStrategy, Subagent, SubagentExecutor,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Subagents System Examples ===\n");

    basic_subagent_example().await?;
    multiple_subagents_example().await?;
    delegation_strategies_example()?;

    Ok(())
}

/// Demonstrates creating and executing a basic subagent
async fn basic_subagent_example() -> anyhow::Result<()> {
    println!("=== Basic Subagent Example ===\n");

    // Create a subagent executor with Auto delegation strategy
    let mut executor = SubagentExecutor::new(DelegationStrategy::Auto);

    // Define a code reviewer subagent
    let code_reviewer = Subagent {
        name: "code-reviewer".to_string(),
        description: "Expert code reviewer focused on best practices".to_string(),
        instructions: r#"
You are an expert code reviewer. Your job is to:
1. Identify bugs and potential issues
2. Suggest improvements for readability and maintainability
3. Check for security vulnerabilities
4. Ensure code follows best practices

Provide constructive feedback with specific suggestions.
        "#.trim().to_string(),
        allowed_tools: vec!["Read".to_string(), "Grep".to_string()],
        max_turns: Some(5),
        model: Some("claude-sonnet-4-20250514".to_string()),
    };

    // Register the subagent
    executor.register(code_reviewer)?;
    println!("✓ Registered subagent: code-reviewer");

    // Define a documentation writer subagent
    let doc_writer = Subagent {
        name: "doc-writer".to_string(),
        description: "Technical documentation specialist".to_string(),
        instructions: r#"
You are a technical documentation expert. Your job is to:
1. Write clear, comprehensive documentation
2. Include code examples where appropriate
3. Explain complex concepts simply
4. Structure documentation logically

Focus on clarity and completeness.
        "#.trim().to_string(),
        allowed_tools: vec![
            "Read".to_string(),
            "Write".to_string(),
            "Edit".to_string(),
        ],
        max_turns: Some(3),
        model: None, // Use default model
    };

    executor.register(doc_writer)?;
    println!("✓ Registered subagent: doc-writer");

    // List available subagents
    println!("\nAvailable subagents: {:?}", executor.list_subagents());
    println!("Has 'code-reviewer': {}", executor.has_subagent("code-reviewer"));
    println!("Has 'unknown': {}", executor.has_subagent("unknown"));

    println!();
    Ok(())
}

/// Demonstrates multiple specialized subagents
async fn multiple_subagents_example() -> anyhow::Result<()> {
    println!("=== Multiple Specialized Subagents Example ===\n");

    let mut executor = SubagentExecutor::new(DelegationStrategy::Auto);

    // Register multiple specialized subagents

    // 1. Security auditor
    let security_auditor = Subagent {
        name: "security-auditor".to_string(),
        description: "Security vulnerability scanner".to_string(),
        instructions: r#"
You are a security expert. Analyze code for:
- SQL injection vulnerabilities
- XSS vulnerabilities
- Authentication issues
- Authorization bypass risks
- Sensitive data exposure
- Insecure configurations

Rate findings by severity: Critical, High, Medium, Low.
        "#.trim().to_string(),
        allowed_tools: vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string()],
        max_turns: Some(10),
        model: Some("claude-sonnet-4-20250514".to_string()),
    };
    executor.register(security_auditor)?;

    // 2. Performance analyzer
    let performance_analyzer = Subagent {
        name: "performance-analyzer".to_string(),
        description: "Performance optimization specialist".to_string(),
        instructions: r#"
You are a performance optimization expert. Analyze code for:
- Algorithm complexity issues (O(n²), O(n³), etc.)
- Memory leaks and excessive allocations
- Inefficient database queries
- Unnecessary computations
- Caching opportunities

Provide specific optimization suggestions with expected impact.
        "#.trim().to_string(),
        allowed_tools: vec!["Read".to_string(), "Grep".to_string()],
        max_turns: Some(5),
        model: None,
    };
    executor.register(performance_analyzer)?;

    // 3. Test generator
    let test_generator = Subagent {
        name: "test-generator".to_string(),
        description: "Unit test generator".to_string(),
        instructions: r#"
You are a testing expert. Generate comprehensive unit tests:
- Cover happy paths and edge cases
- Test error handling
- Use appropriate assertions
- Follow testing best practices
- Aim for high code coverage

Generate tests that are readable and maintainable.
        "#.trim().to_string(),
        allowed_tools: vec![
            "Read".to_string(),
            "Write".to_string(),
            "Bash".to_string(),
        ],
        max_turns: Some(5),
        model: None,
    };
    executor.register(test_generator)?;

    // 4. Refactoring expert
    let refactoring_expert = Subagent {
        name: "refactoring-expert".to_string(),
        description: "Code refactoring specialist".to_string(),
        instructions: r#"
You are a refactoring expert. Improve code by:
- Applying design patterns
- Reducing code duplication
- Improving naming and structure
- Simplifying complex logic
- Enhancing modularity

Always maintain existing functionality while improving code quality.
        "#.trim().to_string(),
        allowed_tools: vec![
            "Read".to_string(),
            "Edit".to_string(),
            "Write".to_string(),
        ],
        max_turns: Some(8),
        model: Some("claude-sonnet-4-20250514".to_string()),
    };
    executor.register(refactoring_expert)?;

    println!("Registered {} specialized subagents:", executor.list_subagents().len());
    for name in executor.list_subagents() {
        if let Some(subagent) = get_subagent_info(&executor, &name) {
            println!("\n  {}:", name);
            println!("    Description: {}", subagent.description);
            println!("    Tools: {:?}", subagent.allowed_tools);
            println!("    Max turns: {:?}", subagent.max_turns);
            println!("    Model: {:?}", subagent.model);
        }
    }

    println!();
    Ok(())
}

/// Demonstrates different delegation strategies
fn delegation_strategies_example() -> anyhow::Result<()> {
    println!("=== Delegation Strategies Example ===\n");

    let strategies = vec![
        (DelegationStrategy::Auto, "Auto - Let Claude decide when to delegate"),
        (DelegationStrategy::Manual, "Manual - Requires explicit SubagentTool calls"),
        (DelegationStrategy::ToolCall, "ToolCall - Delegate through tool calls"),
    ];

    for (strategy, description) in strategies {
        let executor = SubagentExecutor::new(strategy.clone());
        println!("Strategy: {:?}", executor.strategy());
        println!("  {}", description);
        println!();
    }

    // Create executor with Auto strategy (most common)
    let executor = SubagentExecutor::new(DelegationStrategy::Auto);
    println!("Default strategy: {:?}", executor.strategy());
    println!("Empty subagents: {:?}", executor.list_subagents());

    println!();
    Ok(())
}

/// Helper function to get subagent info (simulated since we can't access private field)
fn get_subagent_info(_executor: &SubagentExecutor, name: &str) -> Option<Subagent> {
    // This is a simulation - in real usage, you'd need to store subagents separately
    // or the SDK would need to expose a get_subagent method
    Some(Subagent {
        name: name.to_string(),
        description: format!("{} description", name),
        instructions: format!("{} instructions", name),
        allowed_tools: vec!["Read".to_string()],
        max_turns: Some(5),
        model: None,
    })
}
