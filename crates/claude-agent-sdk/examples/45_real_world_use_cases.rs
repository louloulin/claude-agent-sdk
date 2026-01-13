//! Real-World Use Cases Example
//!
//! This example demonstrates practical applications of the Claude Agent SDK
//! in common real-world scenarios.

use anyhow::Result;
use claude_agent_sdk::{
    ClaudeAgentOptions, ContentBlock, Message, PermissionMode, query,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Real-World Use Cases ===\n");

    // Uncomment the example you want to run:

    // Example 1: Code Review Assistant
    // code_review_assistant().await?;

    // Example 2: Documentation Generator
    // documentation_generator().await?;

    // Example 3: Test Case Generator
    // test_case_generator().await?;

    // Example 4: Log Analysis Assistant
    // log_analysis().await?;

    // Example 5: API Integration Helper
    // api_integration_helper().await?;

    // Example 6: Code Refactoring Assistant
    // refactoring_assistant().await?;

    // Example 7: Data Migration Assistant
    // data_migration_helper().await?;

    // Example 8: Debugging Assistant
    debugging_assistant().await?;

    println!("\n=== Examples Complete ===");
    Ok(())
}

/// Example 1: Automated Code Review Assistant
async fn code_review_assistant() -> Result<()> {
    println!("📝 Code Review Assistant\n");

    let code_to_review = r#"
pub fn calculate_sum(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for n in numbers {
        sum += n;
    }
    sum
}
"#;

    let prompt = format!(
        "Review this Rust code for:\n\
         1. Correctness\n\
         2. Performance\n\
         3. Rust best practices\n\
         4. Potential bugs\n\
         5. Code style\n\n\
         Code:\n```rust\n{}\n```",
        code_to_review
    );

    let options = ClaudeAgentOptions::builder().max_turns(3).build();

    let messages = query(&prompt, Some(options)).await?;

    print_response(&messages);
    Ok(())
}

/// Example 2: Documentation Generator
async fn documentation_generator() -> Result<()> {
    println!("📚 Documentation Generator\n");

    let function_code = r#"
pub fn process_user_input(input: &str) -> Result<String, Error> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Error::EmptyInput);
    }
    Ok(trimmed.to_uppercase())
}
"#;

    let prompt = format!(
        "Generate comprehensive Rust documentation for this function:\n\
         - Module-level documentation\n\
         - Function documentation with examples\n\
         - Error documentation\n\
         - Panics documentation (if applicable)\n\
         - Safety documentation (if applicable)\n\n\
         ```rust\n{}\n```",
        function_code
    );

    let options = ClaudeAgentOptions::builder()
        .system_prompt("You are a technical writer specializing in Rust documentation.")
        .build();

    let messages = query(&prompt, Some(options)).await?;

    print_response(&messages);
    Ok(())
}

/// Example 3: Test Case Generator
async fn test_case_generator() -> Result<()> {
    println!("🧪 Test Case Generator\n");

    let function_code = r#"
pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}
"#;

    let prompt = format!(
        "Generate comprehensive unit tests for this function:\n\
         - Happy path tests\n\
         - Edge case tests\n\
         - Error case tests\n\
         - Property-based tests (if applicable)\n\n\
         ```rust\n{}\n```",
        function_code
    );

    let options = ClaudeAgentOptions::builder()
        .system_prompt("You are a testing expert specializing in Rust.")
        .build();

    let messages = query(&prompt, Some(options)).await?;

    print_response(&messages);
    Ok(())
}

/// Example 4: Log Analysis Assistant
async fn log_analysis() -> Result<()> {
    println!("📊 Log Analysis Assistant\n");

    let logs = r#"
[ERROR] 2024-01-08 10:23:15 Connection failed: timeout
[WARN] 2024-01-08 10:23:16 Retry attempt 1
[ERROR] 2024-01-08 10:23:18 Connection failed: timeout
[WARN] 2024-01-08 10:23:19 Retry attempt 2
[ERROR] 2024-01-08 10:23:21 Connection failed: timeout
[INFO] 2024-01-08 10:23:22 Max retries reached, giving up
[ERROR] 2024-01-08 10:25:30 Database connection pool exhausted
[WARN] 2024-01-08 10:25:31 Creating new connection
"#;

    let prompt = format!(
        "Analyze these logs and provide:\n\
         1. Root cause analysis\n\
         2. Timeline of events\n\
         3. Suggested fixes\n\
         4. Preventive measures\n\n\
         Logs:\n```\
         {}\
         ```",
        logs
    );

    let messages = query(&prompt, None).await?;
    print_response(&messages);
    Ok(())
}

/// Example 5: API Integration Helper
async fn api_integration_helper() -> Result<()> {
    println!("🔌 API Integration Helper\n");

    let api_spec = r#"
GET /api/users/{id}
Response: { "id": 123, "name": "John", "email": "john@example.com" }

POST /api/users
Body: { "name": "...", "email": "..." }
Response: 201 Created
"#;

    let prompt = format!(
        "Generate Rust code to:\n\
         1. Define structs for the API\n\
         2. Implement GET /api/users/{{id}}\n\
         3. Implement POST /api/users\n\
         4. Handle errors properly\n\
         5. Use reqwest for HTTP requests\n\n\
         API Spec:\n```\
         {}\
         ```",
        api_spec
    );

    let options = ClaudeAgentOptions::builder()
        .allowed_tools(vec!["Read".to_string(), "Write".to_string()])
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let messages = query(&prompt, Some(options)).await?;
    print_response(&messages);
    Ok(())
}

/// Example 6: Code Refactoring Assistant
async fn refactoring_assistant() -> Result<()> {
    println!("🔨 Refactoring Assistant\n");

    let code_to_refactor = r#"
pub fn process_data(data: &Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    for item in data {
        if item.len() > 5 {
            let processed = item.to_uppercase();
            result.push(processed);
        }
    }
    result
}
"#;

    let prompt = format!(
        "Refactor this Rust code to improve:\n\
         1. Idiomatic Rust usage\n\
         2. Performance\n\
         3. Readability\n\
         4. Error handling\n\
         5. Documentation\n\n\
         Original code:\n```rust\n{}\n```\n\n\
         Provide the refactored version with explanations.",
        code_to_refactor
    );

    let options = ClaudeAgentOptions::builder()
        .system_prompt("You are a Rust expert focused on clean, idiomatic code.")
        .build();

    let messages = query(&prompt, Some(options)).await?;
    print_response(&messages);
    Ok(())
}

/// Example 7: Data Migration Assistant
async fn data_migration_helper() -> Result<()> {
    println!("💾 Data Migration Assistant\n");

    let old_schema = r#"
struct OldUser {
    id: i32,
    full_name: String,  // "John Doe"
    email_address: String,
}
"#;

    let new_schema = r#"
struct NewUser {
    id: i32,
    first_name: String,  // "John"
    last_name: String,   // "Doe"
    email: String,
    created_at: DateTime<Utc>,
}
"#;

    let prompt = format!(
        "Generate Rust code to:\n\
         1. Migrate data from OldUser to NewUser\n\
         2. Parse full_name into first_name and last_name\n\
         3. Set created_at to current time\n\
         4. Handle edge cases (e.g., single word names)\n\
         5. Provide tests for the migration\n\n\
         Old Schema:\n```rust\n{}\n```\n\n\
         New Schema:\n```rust\n{}\n```",
        old_schema, new_schema
    );

    let options = ClaudeAgentOptions::builder()
        .allowed_tools(vec![
            "Read".to_string(),
            "Write".to_string(),
            "Bash".to_string(),
        ])
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let messages = query(&prompt, Some(options)).await?;
    print_response(&messages);
    Ok(())
}

/// Example 8: Interactive Debugging Assistant
async fn debugging_assistant() -> Result<()> {
    println!("🐛 Debugging Assistant\n");

    let buggy_code = r#"
pub fn find_median(numbers: &mut Vec<i32>) -> f64 {
    numbers.sort();
    let mid = numbers.len() / 2;
    if numbers.len() % 2 == 0 {
        (numbers[mid - 1] + numbers[mid]) as f64 / 2.0
    } else {
        numbers[mid] as f64
    }
}
"#;

    let problem_description = r#"
The function works but modifies the input vector in place,
which the caller might not expect. Also, it returns f64
even when all inputs are integers.
"#;

    let prompt = format!(
        "Debug this Rust code:\n\n\
         Problem:\n{}\n\n\
         Code:\n```rust\n{}\n```\n\n\
         Identify:\n\
         1. What's wrong\n\
         2. Why it's problematic\n\
         3. How to fix it\n\
         4. Provide the corrected code",
        problem_description, buggy_code
    );

    let options = ClaudeAgentOptions::builder()
        .system_prompt("You are a debugging expert helping developers fix code.")
        .build();

    let messages = query(&prompt, Some(options)).await?;
    print_response(&messages);
    Ok(())
}

/// Example 9: Performance Optimization Assistant
async fn optimization_assistant() -> Result<()> {
    println!("⚡ Performance Optimization Assistant\n");

    let slow_code = r#"
pub fn find_duplicates(strings: &[String]) -> Vec<String> {
    let mut duplicates = Vec::new();
    for (i, s1) in strings.iter().enumerate() {
        for (j, s2) in strings.iter().enumerate() {
            if i != j && s1 == s2 {
                if !duplicates.contains(s1) {
                    duplicates.push(s1.clone());
                }
            }
        }
    }
    duplicates
}
"#;

    let prompt = format!(
        "Optimize this Rust code for performance:\n\n\
         ```rust\n{}\n```\n\n\
         Provide:\n\
         1. Analysis of performance issues\n\
         2. Big-O complexity analysis\n\
         3. Optimized implementation\n\
         4. Explanation of improvements",
        slow_code
    );

    let options = ClaudeAgentOptions::builder()
        .system_prompt("You are a performance optimization expert.")
        .build();

    let messages = query(&prompt, Some(options)).await?;
    print_response(&messages);
    Ok(())
}

/// Example 10: Multi-step Code Transformation
async fn code_transformation() -> Result<()> {
    println!("🔄 Code Transformation Pipeline\n");

    let initial_code = r#"
fn calc(a: i32, b: i32) -> i32 {
    a + b
}
"#;

    let transformation_steps = vec![
        "Rename function to be more descriptive",
        "Add type parameters for generic numeric types",
        "Add documentation",
        "Add error handling for overflow",
        "Add unit tests",
    ];

    let mut current_code = initial_code.to_string();

    for (step, instruction) in transformation_steps.iter().enumerate() {
        println!("Step {}: {}", step + 1, instruction);

        let prompt = format!(
            "Apply this transformation to the code:\n\
             Instruction: {}\n\n\
             Current code:\n```rust\n{}\n```\n\n\
             Show only the transformed code.",
            instruction, current_code
        );

        let messages = query(&prompt, None).await?;

        // Extract the transformed code
        for msg in &messages {
            if let Message::Assistant(assistant_msg) = msg {
                for block in &assistant_msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        // Extract code from markdown code blocks
                        if let Some(start) = text.text.find("```rust") {
                            if let Some(end) = text.text[start..].find("```") {
                                let code_start = start + 7;
                                current_code = text.text[code_start..start + end].to_string();
                            }
                        }
                    }
                }
            }
        }

        println!("   ✓ Transformation complete\n");
    }

    println!("Final code:\n```rust\n{}\n```", current_code);
    Ok(())
}

/// Helper function to print Claude's response
fn print_response(messages: &[Message]) {
    for msg in messages {
        if let Message::Assistant(assistant_msg) = msg {
            for block in &assistant_msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("{}", text.text);
                }
            }
        }
    }
}
