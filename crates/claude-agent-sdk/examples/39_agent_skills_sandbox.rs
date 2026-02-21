//! Agent Skills Sandbox Execution Examples
//!
//! This example demonstrates how to use the sandbox execution system
//! for secure, isolated skill script execution.

use claude_agent_sdk::skills::sandbox::{
    SandboxConfig, SandboxExecutor, SandboxResult, SandboxUtils,
};
use std::time::Duration;

#[cfg(feature = "sandbox")]
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    // tracing_subscriber::fmt::init();

    println!("=== Agent Skills Sandbox Execution Demo ===\n");

    // Example 1: Default sandbox configuration
    println!("1. Default Sandbox Configuration");
    println!("-----------------------------------");
    let config = SandboxConfig::default();
    println!("Timeout: {:?}", config.timeout);
    println!("Max Memory: {:?}", config.max_memory);
    println!("Max Fuel: {:?}", config.max_fuel);
    println!("Network Access: {}", config.allow_network);
    println!("Filesystem Access: {}", config.allow_filesystem);
    println!();

    // Example 2: Restrictive configuration for untrusted code
    println!("2. Restrictive Configuration (Untrusted Skills)");
    println!("----------------------------------------------");
    let restrictive = SandboxConfig::restrictive();
    println!("Timeout: {:?}", restrictive.timeout);
    println!("Max Memory: {:?}", restrictive.max_memory);
    println!("Max Fuel: {:?}", restrictive.max_fuel);
    println!(
        "Safe for untrusted code: {}",
        SandboxUtils::is_safe_config(&restrictive)
    );
    println!();

    // Example 3: Permissive configuration for trusted skills
    println!("3. Permissive Configuration (Trusted Skills)");
    println!("--------------------------------------------");
    let permissive = SandboxConfig::permissive();
    println!("Timeout: {:?}", permissive.timeout);
    println!("Max Memory: {:?}", permissive.max_memory);
    println!("Max Fuel: {:?}", permissive.max_fuel);
    println!("Network Access: {}", permissive.allow_network);
    println!("Filesystem Access: {}", permissive.allow_filesystem);
    println!(
        "Safe for untrusted code: {}",
        SandboxUtils::is_safe_config(&permissive)
    );
    println!();

    // Example 4: Custom configuration using builder pattern
    println!("4. Custom Configuration (Builder Pattern)");
    println!("-----------------------------------------");
    let custom = SandboxConfig::new()
        .with_timeout(Duration::from_secs(60))
        .with_max_memory(128 * 1024 * 1024) // 128 MB
        .with_max_fuel(2_000_000)
        .with_network_access(false)
        .with_filesystem_access(true, Some("/tmp/skills".to_string()));

    println!("Timeout: {:?}", custom.timeout);
    println!("Max Memory: {:?}", custom.max_memory);
    println!("Max Fuel: {:?}", custom.max_fuel);
    println!("Network: {}", custom.allow_network);
    println!(
        "Filesystem: {} (dir: {:?})",
        custom.allow_filesystem, custom.working_directory
    );
    println!();

    // Example 5: Script validation
    println!("5. Script Validation");
    println!("--------------------");
    let valid_script = "print('Hello, World!')";
    match SandboxUtils::validate_script(valid_script) {
        Ok(()) => println!("✓ Valid script"),
        Err(e) => println!("✗ Invalid: {}", e),
    }

    let empty_script = "";
    match SandboxUtils::validate_script(empty_script) {
        Ok(()) => println!("✓ Valid script"),
        Err(e) => println!("✗ Invalid: {}", e),
    }

    let large_script = "x".repeat(11 * 1024 * 1024);
    match SandboxUtils::validate_script(&large_script) {
        Ok(()) => println!("✓ Valid script"),
        Err(e) => println!("✗ Invalid: {}", e),
    }
    println!();

    // Example 6: Memory requirement estimation
    println!("6. Memory Requirement Estimation");
    println!("---------------------------------");
    let small_script = "print('small')";
    let estimated = SandboxUtils::estimate_memory_requirement(small_script);
    println!("Small script estimated memory: {} bytes", estimated);

    let large_script = "x".repeat(100 * 1024);
    let estimated = SandboxUtils::estimate_memory_requirement(&large_script);
    println!(
        "Large script estimated memory: {} bytes ({} MB)",
        estimated,
        estimated / (1024 * 1024)
    );
    println!();

    // Example 7: Recommended configuration based on script size
    println!("7. Recommended Configuration");
    println!("----------------------------");
    let tiny_script = "print('tiny')";
    let config1 = SandboxUtils::recommended_config_for_script(tiny_script);
    println!("Tiny script → Timeout: {:?}", config1.timeout);

    let huge_script = "x".repeat(3 * 1024 * 1024);
    let config2 = SandboxUtils::recommended_config_for_script(&huge_script);
    println!("Huge script → Timeout: {:?}", config2.timeout);
    println!();

    // Example 8: Create sandbox executor and execute (fallback mode)
    println!("8. Sandbox Execution (Fallback Mode)");
    println!("------------------------------------");
    #[cfg(feature = "sandbox")]
    {
        let executor = SandboxExecutor::new(SandboxConfig::default());
        let script = "print('Hello from sandbox!')";

        match executor.execute(script, None).await {
            Ok(result) => {
                println!("✓ Execution successful");
                println!("  Exit code: {}", result.exit_code);
                println!("  Time: {} ms", result.execution_time_ms);
                println!("  Timed out: {}", result.timed_out);
                println!("  Stdout: {}", result.stdout);
                if !result.stderr.is_empty() {
                    println!("  Stderr: {}", result.stderr);
                }
            },
            Err(e) => {
                println!("✗ Execution failed: {}", e);
            },
        }
    }

    #[cfg(not(feature = "sandbox"))]
    {
        println!("Note: Enable with --features sandbox for actual execution");
        let executor = SandboxExecutor::new(SandboxConfig::default());
        let script = "print('Hello from sandbox!')";

        match executor.execute(script, None).await {
            Ok(_) => println!("✓ Unexpected success (should fail without feature)"),
            Err(e) => println!("Expected error: {}", e),
        }
    }
    println!();

    // Example 9: SandboxResult analysis
    println!("9. SandboxResult Analysis");
    println!("-------------------------");
    let success_result = SandboxResult {
        stdout: "Operation completed".to_string(),
        stderr: String::new(),
        exit_code: 0,
        execution_time_ms: 150,
        timed_out: false,
        memory_used: Some(1024),
        fuel_consumed: Some(5000),
    };

    println!("Success check: {}", success_result.is_success());
    println!("Error message: {:?}", success_result.error_message());

    let failure_result = SandboxResult {
        stdout: String::new(),
        stderr: "Something went wrong".to_string(),
        exit_code: 1,
        execution_time_ms: 50,
        timed_out: false,
        memory_used: None,
        fuel_consumed: None,
    };

    println!("Failure check: {}", failure_result.is_success());
    println!("Error message: {:?}", failure_result.error_message());

    let timeout_result = SandboxResult {
        stdout: String::new(),
        stderr: String::new(),
        exit_code: -1,
        execution_time_ms: 10000,
        timed_out: true,
        memory_used: None,
        fuel_consumed: None,
    };

    println!("Timeout check: {}", timeout_result.is_success());
    println!("Error message: {:?}", timeout_result.error_message());
    println!();

    // Example 10: Execute from file
    println!("10. Execute Script File");
    println!("----------------------");
    #[cfg(feature = "sandbox")]
    {
        let executor = SandboxExecutor::new(SandboxConfig::default());
        let script_path = Path::new("/tmp/test_skill.js");

        // Create a test script file
        std::fs::write(script_path, "console.log('Hello from file!');")?;

        match executor.execute_file(script_path, None).await {
            Ok(result) => {
                println!("✓ File execution successful");
                println!("  Stdout: {}", result.stdout);
            },
            Err(e) => {
                println!("✗ File execution failed: {}", e);
            },
        }

        // Clean up
        std::fs::remove_file(script_path)?;
    }

    #[cfg(not(feature = "sandbox"))]
    {
        println!("Note: Enable with --features sandbox for file execution");
    }
    println!();

    // Example 11: Timeout demonstration
    println!("11. Timeout Configuration");
    println!("-------------------------");
    let quick_config = SandboxConfig::new().with_timeout(Duration::from_millis(100));
    println!("Quick timeout: {:?}", quick_config.timeout);

    let slow_config = SandboxConfig::new().with_timeout(Duration::from_secs(120));
    println!("Slow timeout: {:?}", slow_config.timeout);
    println!();

    // Example 12: Resource limits
    println!("12. Resource Limits Configuration");
    println!("---------------------------------");
    let memory_limited = SandboxConfig::new().with_max_memory(16 * 1024 * 1024); // 16 MB
    println!("Memory limit: {:?} (16 MB)", memory_limited.max_memory);

    let fuel_limited = SandboxConfig::new().with_max_fuel(100_000); // 100K instructions
    println!(
        "Fuel limit: {:?} (100K instructions)",
        fuel_limited.max_fuel
    );

    let unlimited = SandboxConfig {
        timeout: Duration::from_secs(30),
        max_memory: None,
        max_fuel: None,
        allow_network: false,
        allow_filesystem: false,
        working_directory: None,
    };
    println!("Unlimited memory: {:?}", unlimited.max_memory);
    println!("Unlimited fuel: {:?}", unlimited.max_fuel);
    println!();

    println!("=== Demo Complete ===");
    println!();
    println!("Key Takeaways:");
    println!("- SandboxConfig provides flexible execution controls");
    println!("- Restrictive config for untrusted, Permissive for trusted");
    println!("- Builder pattern allows custom configurations");
    println!("- Validation and estimation tools help planning");
    println!("- Graceful fallback when sandbox feature is disabled");

    Ok(())
}
