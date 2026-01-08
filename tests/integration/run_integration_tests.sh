#!/bin/bash
# Integration Test Runner
# This script runs all integration tests for the Claude Agent SDK

set -e

echo "=== Claude Agent SDK - Integration Tests ==="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name=$1
    local test_command=$2

    TESTS_RUN=$((TESTS_RUN + 1))

    echo -e "${YELLOW}Running: ${test_name}${NC}"

    if eval "$test_command"; then
        echo -e "${GREEN}✓ PASSED${NC}: ${test_name}\n"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}: ${test_name}\n"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found${NC}"
    exit 1
fi

if ! command -v claude &> /dev/null; then
    echo -e "${YELLOW}Warning: claude command not found${NC}"
    echo "Some tests may be skipped"
fi

echo ""

# Run unit tests
echo -e "${GREEN}=== Unit Tests ===${NC}"
run_test "Unit tests" "cargo test --lib"

# Run example programs as integration tests
echo ""
echo -e "${GREEN}=== Example Integration Tests ===${NC}"

# Basic examples
run_test "01_hello_world" "cargo run --example 01_hello_world"
run_test "02_limit_tool_use" "cargo run --example 02_limit_tool_use"
run_test "03_monitor_tools" "cargo run --example 03_monitor_tools"

# Streaming examples
run_test "06_bidirectional_client" "cargo run --example 06_bidirectional_client"
run_test "14_streaming_mode" "cargo run --example 14_streaming_mode"
run_test "20_query_stream" "cargo run --example 20_query_stream"

# Advanced examples
run_test "08_mcp_server_integration" "cargo run --example 08_mcp_server_integration"
run_test "16_session_management" "cargo run --example 16_session_management"
run_test "17_fallback_model" "cargo run --example 17_fallback_model"
run_test "23_image_input" "cargo run --example 23_image_input"

# Skills examples
run_test "30_agent_skills_simple" "cargo run --example 30_agent_skills_simple"
run_test "31_agent_skills_persistence" "cargo run --example 31_agent_skills_persistence"
run_test "36_agent_skills_tags" "cargo run --example 36_agent_skills_tags"

# MCP examples
run_test "42_mcp_async_tasks" "cargo run --example 42_mcp_async_tasks"

# Advanced examples
run_test "43_error_handling" "cargo run --example 43_error_handling"
run_test "44_concurrent_queries" "cargo run --example 44_concurrent_queries"
run_test "45_real_world_use_cases" "cargo run --example 45_real_world_use_cases"
run_test "46_advanced_configuration" "cargo run --example 46_advanced_configuration"

# Testing examples
run_test "47_testing_patterns" "cargo run --example 47_testing_patterns"

# Performance examples
run_test "48_performance_benchmarking" "cargo run --example 48_performance_benchmarking"

# Best practices
run_test "49_best_practices" "cargo run --example 49_best_practices"

# Integration test suite
run_test "50_integration_tests" "cargo run --example 50_integration_tests"

# Run doctests
echo ""
echo -e "${GREEN}=== Documentation Tests ===${NC}"
run_test "Doctests" "cargo test --doc"

# Summary
echo ""
echo "=== Test Summary ==="
echo "Total:   $TESTS_RUN"
echo -e "Passed:  ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed:  ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
