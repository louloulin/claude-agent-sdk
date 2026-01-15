# Justfile for Claude Agent SDK Examples

set shell := ["bash", "-cu"]

EXAMPLES_API := "01_hello_world 02_limit_tool_use 03_monitor_tools 04_permission_callbacks 05_hooks_pretooluse 06_bidirectional_client 07_dynamic_control 08_mcp_server_integration 09_agents 10_include_partial_messages 12_stderr_callback 13_system_prompt 16_session_management 17_fallback_model 18_max_budget_usd 19_max_thinking_tokens 20_query_stream 21_custom_plugins 22_plugin_integration 23_image_input 43_error_handling 44_concurrent_queries 45_stream_processing 46_advanced_errors 47_concurrency_patterns 48_memory_management 44_comprehensive_skill_md_test 55_real_skill_md_verification"
EXAMPLES_CLI := "11_setting_sources 14_streaming_mode 15_hooks_comprehensive"
EXAMPLES_STANDALONE := "30_agent_skills 30_agent_skills_simple 31_agent_skills_persistence 32_agent_skills_discovery 33_agent_skills_resources 34_agent_skills_dependency 35_agent_skills_version 36_agent_skills_tags 37_agent_skills_yaml 38_agent_skills_hot_reload 39_agent_skills_sandbox 40_agent_skills_performance 41_agent_skills_vscode 42_mcp_async_tasks 42_skill_md_integration 43_skill_md_real_world_examples 45_real_world_use_cases 46_advanced_configuration 47_testing_patterns 48_performance_benchmarking 49_testing_strategies 50_integration_tests 50_production_deployment 50_verify_skill_md 51_orchestration"
ALL_EXAMPLES := EXAMPLES_API + " " + EXAMPLES_CLI + " " + EXAMPLES_STANDALONE

default:
    @just --list --unstable

build-examples:
    @echo "Building all examples..."
    @cargo build --examples 2>&1 | grep -E "(Compiling|Finished|error)" || true

check-examples:
    @echo "Checking examples for errors..."
    @for ex in {{ALL_EXAMPLES}}; do echo -n "$ex... "; cargo check --example "$ex" --quiet 2>&1 >/dev/null && echo "OK" || echo "FAIL"; done

list-examples:
    @echo "API-Required:"
    @for ex in {{EXAMPLES_API}}; do echo "  - $ex"; done
    @echo "CLI-Argument:"
    @for ex in {{EXAMPLES_CLI}}; do echo "  - $ex"; done
    @echo "Standalone:"
    @for ex in {{EXAMPLES_STANDALONE}}; do echo "  - $ex"; done

build example:
    @cargo build --example "{{example}}"

run example:
    @cargo run --example "{{example}}"

test-standalone:
    @echo "Testing standalone examples..."
    @for ex in {{EXAMPLES_STANDALONE}}; do echo -n "$ex... "; cargo run --quiet --example "$ex" 2>/dev/null >/dev/null && echo "OK" || echo "FAIL"; done

verify: build-examples test-standalone
    @echo "Verification complete!"

warnings:
    @cargo build --examples 2>&1 | grep "warning:" | wc -l | xargs echo "Warnings:"

clean:
    @cargo clean --examples
    @echo "Clean complete"

ci:
    @echo "CI check..."
    @cargo build --examples --quiet
    @echo "CI passed"

validate:
    @echo "Validating..."
    @for ex in {{ALL_EXAMPLES}}; do cargo check --example "$ex" --quiet 2>&1 || echo "FAILED: $ex"; done
    @echo "Validation complete!"

smoke-test:
    @echo "Smoke test..."
    @cargo run --quiet --example 30_agent_skills_simple
    @cargo run --quiet --example 36_agent_skills_tags
    @echo "Smoke test passed"

stats:
    @echo "Statistics:"
    @echo "Total: $(echo {{ALL_EXAMPLES}} | wc -w)"
    @echo "API: $(echo {{EXAMPLES_API}} | wc -w)"
    @echo "CLI: $(echo {{EXAMPLES_CLI}} | wc -w)"
    @echo "Standalone: $(echo {{EXAMPLES_STANDALONE}} | wc -w)"

fmt-examples:
    @cargo fmt --examples

check-fmt:
    @cargo fmt --examples -- --check

lint-examples:
    @cargo clippy --examples -- -W clippy::all

sizes:
    @ls -lh crates/claude-agent-sdk/examples/*.rs | awk '{print $5, $9}' | sort -h

search keyword:
    @grep -l "{{keyword}}" crates/claude-agent-sdk/examples/*.rs | xargs basename -s .rs 2>/dev/null || echo "No matches"

# ============================================================================
# Performance Testing Commands
# ============================================================================

# Build the project in release mode for performance testing
bench-build:
    @echo "Building release version..."
    @cargo build --release 2>&1 | tail -5

# Run a quick single performance test
bench-quick:
    @echo "Running quick performance test..."
    @python3 scripts/simple_test.py

# Run detailed performance statistics (5 iterations)
bench-detailed:
    @echo "Running detailed performance benchmark..."
    @python3 scripts/detailed_benchmark.py

# Run Criterion benchmarks
bench-criterion:
    @echo "Running Criterion benchmarks..."
    @cargo bench --bench query_performance

# Run all performance tests
bench-all: bench-build bench-quick
    @echo ""
    @echo "Running all performance tests..."
    @echo "=========================================="
    @echo ""
    @python3 scripts/detailed_benchmark.py

# Generate performance comparison report (requires Python and Node.js SDKs)
bench-compare:
    @echo "Running cross-language performance comparison..."
    @python3 scripts/benchmark_sdk_comparison.py

# Show performance summary
bench-summary:
    @echo "Performance Test Summary"
    @echo "======================="
    @echo ""
    @echo "Available reports:"
    @echo "  - QUICK_START.md (快速开始指南)"
    @echo "  - bench.md (完整性能分析)"
    @echo ""
    @echo "Quick test results (latest):"
    @if [ -f bench.md ]; then \
        grep -A 10 "测试结果汇总" bench.md | head -11; \
    else \
        echo "No test results found. Run 'just bench-all' first."; \
    fi

# Clean benchmark artifacts
bench-clean:
    @echo "Cleaning benchmark artifacts..."
    @cargo clean
    @rm -rf target/criterion
    @echo "Clean complete"

# ============================================================================
# Analysis Commands
# ============================================================================

# Show performance bottleneck analysis
analyze-bottlenecks:
    @echo "Performance Bottleneck Analysis"
    @echo "==============================="
    @echo ""
    @grep -A 20 "性能瓶颈分解" bench.md 2>/dev/null || echo "Run benchmarks first with 'just bench-all'"

# Show optimization recommendations
analyze-recommendations:
    @echo "Optimization Recommendations"
    @echo "============================"
    @echo ""
    @grep -A 30 "最终建议" bench.md 2>/dev/null || echo "Run benchmarks first with 'just bench-all'"

# Show test statistics
analyze-stats:
    @echo "Test Statistics"
    @echo "==============="
    @echo ""
    @grep -A 15 "5次测试结果" bench.md 2>/dev/null || echo "Run benchmarks first with 'just bench-all'"

# ============================================================================
# Documentation Commands
# ============================================================================

# Show all available documentation
docs-list:
    @echo "Available Documentation:"
    @echo "========================"
    @echo ""
    @echo "Performance Documents:"
    @echo "  - QUICK_START.md (快速开始指南)"
    @echo "  - bench.md (完整性能分析)"
    @echo ""
    @ls -lh *.md 2>/dev/null | awk '{print $5, $9}' | grep -E "(bench|QUICK)" || true

# View main performance analysis
docs-view-main:
    @less -R bench.md

# View final test report
docs-view-final:
    @less -R bench.md

# ============================================================================
# Quick Reference
# ============================================================================

# Show performance testing quick reference
bench-help:
    @echo "Performance Testing Commands"
    @echo "============================"
    @echo ""
    @echo "Quick Tests:"
    @echo "  just bench-build       - Build release version"
    @echo "  just bench-quick       - Run single quick test"
    @echo "  just bench-detailed    - Run 5 iterations with statistics"
    @echo "  just bench-all         - Run complete performance test suite"
    @echo ""
    @echo "Analysis:"
    @echo "  just analyze-bottlenecks   - Show performance bottleneck analysis"
    @echo "  just analyze-recommendations - Show optimization recommendations"
    @echo "  just analyze-stats         - Show test statistics"
    @echo ""
    @echo "Documentation:"
    @echo "  just docs-list         - List all performance docs"
    @echo "  just docs-view-main    - View main analysis"
    @echo "  just docs-view-final   - View final report"
    @echo ""
    @echo "Examples:"
    @echo "  just bench-quick | grep '总耗时'  - Get total time"
    @echo "  just bench-detailed | grep P95     - Get P95 latency"
    @echo ""
    @echo "For more info, see: bench.md"
