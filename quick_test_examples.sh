#!/bin/bash
# 快速测试所有编译成功的示例

echo "╔════════════════════════════════════════════════════════════╗"
echo "║     Claude Agent SDK Rust - 示例执行测试报告               ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# 编译成功的示例
SUCCESSFUL_EXAMPLES=(
    "01_hello_world"
    "02_limit_tool_use"
    "03_monitor_tools"
    "04_permission_callbacks"
    "05_hooks_pretooluse"
    "06_bidirectional_client"
    "07_dynamic_control"
    "08_mcp_server_integration"
    "09_agents"
    "10_include_partial_messages"
    "11_setting_sources"
    "12_stderr_callback"
    "13_system_prompt"
    "14_streaming_mode"
    "15_hooks_comprehensive"
    "16_session_management"
    "17_fallback_model"
    "18_max_budget_usd"
    "19_max_thinking_tokens"
    "20_query_stream"
    "21_custom_plugins"
    "22_plugin_integration"
    "23_image_input"
    "30_agent_skills_simple"
    "31_agent_skills_persistence"
    "32_agent_skills_discovery"
    "33_agent_skills_resources"
    "34_agent_skills_dependency"
    "35_agent_skills_version"
    "36_agent_skills_tags"
    "39_agent_skills_sandbox"
    "40_agent_skills_performance"
    "41_agent_skills_vscode"
    "42_mcp_async_tasks"
    "43_error_handling"
    "45_real_world_use_cases"
    "45_stream_processing"
    "46_advanced_errors"
    "47_concurrency_patterns"
    "48_memory_management"
    "51_orchestration"
)

TOTAL=${#SUCCESSFUL_EXAMPLES[@]}
SUCCESS=0
FAILED=0

echo "📊 总示例数: $TOTAL"
echo ""

# 测试每个示例（只编译不运行，避免需要API密钥）
echo "🔍 测试示例编译状态..."
echo ""

for example in "${SUCCESSFUL_EXAMPLES[@]}"; do
    printf "%-35s" "  $example"

    # 使用 cargo check 来验证编译（比 run 快）
    if cargo check --example $example --quiet 2>&1 | grep -q "Finished"; then
        echo "✅ 编译成功"
        ((SUCCESS++))
    else
        # 可能生成了警告，尝试实际编译
        if cargo build --example $example --quiet 2>&1 | grep -q "Finished"; then
            echo "✅ 编译成功（有警告）"
            ((SUCCESS++))
        else
            echo "❌ 编译失败"
            ((FAILED++))
        fi
    fi
done

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                        测试汇总                            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "  总示例数:     $TOTAL"
echo "  编译成功:     $SUCCESS"
echo "  编译失败:     $FAILED"

if [ $TOTAL -gt 0 ]; then
    SUCCESS_RATE=$(awk "BEGIN {printf \"%.1f\", ($SUCCESS / $TOTAL) * 100}")
    echo "  成功率:       $SUCCESS_RATE%"
fi

echo ""
echo "✅ 所有示例已完成编译测试"
echo ""
