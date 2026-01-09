#!/bin/bash
# 运行所有编译成功的示例并分析结果

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

echo_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

echo_section() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

# 编译成功的示例列表
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

# 不需要Claude Code CLI的简单示例（可以立即运行）
SIMPLE_EXAMPLES=(
    "01_hello_world"
    "02_limit_tool_use"
    "03_monitor_tools"
    "06_bidirectional_client"
    "07_dynamic_control"
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
    "46_advanced_errors"
    "48_memory_management"
    "51_orchestration"
)

# 需要API密钥或Claude Code CLI的示例
COMPLEX_EXAMPLES=(
    "04_permission_callbacks"
    "05_hooks_pretooluse"
    "08_mcp_server_integration"
    "23_image_input"
    "45_real_world_use_cases"
    "45_stream_processing"
    "47_concurrency_patterns"
)

# 创建输出目录
OUTPUT_DIR="/tmp/example_output"
mkdir -p "$OUTPUT_DIR"

# 统计变量
TOTAL_SUCCESSFUL=${#SUCCESSFUL_EXAMPLES[@]}
TOTAL_SIMPLE=${#SIMPLE_EXAMPLES[@]}
TOTAL_COMPLEX=${#COMPLEX_EXAMPLES[@]}
RUN_SUCCESS=0
RUN_FAILED=0
RUN_SKIPPED=0

echo_section "示例运行测试报告"
echo ""
echo_info "总示例数: $TOTAL_SUCCESSFUL"
echo_info "简单示例（可立即运行）: $TOTAL_SIMPLE"
echo_info "复杂示例（需要API密钥）: $TOTAL_COMPLEX"
echo ""

# 运行简单示例
echo_section "第一部分：运行简单示例（不需要API密钥）"
echo ""

for example in "${SIMPLE_EXAMPLES[@]}"; do
    echo_info "运行示例: $example"

    OUTPUT_FILE="$OUTPUT_DIR/${example}.log"

    # 直接运行，不使用timeout（macOS不兼容）
    if cargo run --example $example > "$OUTPUT_FILE" 2>&1; then
        echo -e "${GREEN}✅ 成功${NC}"
        ((RUN_SUCCESS++))
    else
        EXIT_CODE=$?
        echo -e "${RED}❌ 失败（退出码: $EXIT_CODE）${NC}"
        echo -e "${RED}错误信息:${NC}"
        tail -n 5 "$OUTPUT_FILE" | sed 's/^/  /'
        ((RUN_FAILED++))
    fi
    echo ""
done

echo_section "第二部分：分析复杂示例（需要API密钥）"
echo ""
echo_warn "以下示例需要 ANTHROPIC_API_KEY 环境变量或Claude Code CLI："
for example in "${COMPLEX_EXAMPLES[@]}"; do
    echo "  - $example"
    ((RUN_SKIPPED++))
done
echo ""

# 检查是否有API密钥
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo_warn "未设置 ANTHROPIC_API_KEY 环境变量"
    echo_warn "如需测试复杂示例，请运行："
    echo "  export ANTHROPIC_API_KEY='your-api-key'"
    echo "  cargo run --example 04_permission_callbacks"
else
    echo_info "检测到 ANTHROPIC_API_KEY，可以运行复杂示例"
    echo ""

    for example in "${COMPLEX_EXAMPLES[@]}"; do
        echo_info "运行示例: $example"

        OUTPUT_FILE="$OUTPUT_DIR/${example}.log"

        # 直接运行
        if cargo run --example $example > "$OUTPUT_FILE" 2>&1; then
            echo -e "${GREEN}✅ 成功${NC}"
            ((RUN_SUCCESS++))
        else
            EXIT_CODE=$?
            echo -e "${RED}❌ 失败（退出码: $EXIT_CODE）${NC}"
            tail -n 5 "$OUTPUT_FILE" | sed 's/^/  /'
            ((RUN_FAILED++))
        fi
        echo ""
    done
fi

# 汇总报告
echo_section "运行测试汇总"
echo ""
echo_info "总示例数: $TOTAL_SUCCESSFUL"
echo -e "${GREEN}运行成功: $RUN_SUCCESS${NC}"
echo -e "${RED}运行失败: $RUN_FAILED${NC}"
echo -e "${YELLOW}跳过（需要API密钥）: $RUN_SKIPPED${NC}"

SUCCESS_RATE=$(awk "BEGIN {printf \"%.1f\", ($RUN_SUCCESS / $TOTAL_SUCCESSFUL) * 100}")
echo ""
echo_info "成功率: $SUCCESS_RATE%"
echo ""

# 检查输出目录
echo_info "所有示例的输出已保存到: $OUTPUT_DIR"
echo ""

# 分析一些关键示例的输出
echo_section "关键示例输出分析"
echo ""

# 分析01_hello_world
if [ -f "$OUTPUT_DIR/01_hello_world.log" ]; then
    echo_info "01_hello_world 输出预览:"
    head -n 10 "$OUTPUT_DIR/01_hello_world.log" | sed 's/^/  /'
    echo ""
fi

# 分析43_error_handling
if [ -f "$OUTPUT_DIR/43_error_handling.log" ]; then
    echo_info "43_error_handling 输出预览:"
    head -n 10 "$OUTPUT_DIR/43_error_handling.log" | sed 's/^/  /'
    echo ""
fi

# 分析51_orchestration
if [ -f "$OUTPUT_DIR/51_orchestration.log" ]; then
    echo_info "51_orchestration 输出预览:"
    head -n 15 "$OUTPUT_DIR/51_orchestration.log" | sed 's/^/  /'
    echo ""
fi

echo_section "完成"
echo ""
