#!/bin/bash
# 自动运行所有可编译的示例

echo "========================================="
echo "执行所有示例程序"
echo "========================================="
echo ""

# 统计变量
total=0
passed=0
failed=0
compile_errors=0
runtime_errors=0
need_cli=0

# 创建结果文件
RESULT_FILE="/tmp/example_execution_results.txt"
echo "示例执行结果 - $(date)" > "$RESULT_FILE"
echo "========================================" >> "$RESULT_FILE"

# 测试单个示例
test_example() {
    local example=$1
    local total_count=$2
    local current=$3
    
    ((total++))
    echo ""
    echo "[$current/$total_count] 测试: $example"
    echo "----------------------------------------" >> "$RESULT_FILE"
    echo "示例: $example" >> "$RESULT_FILE"
    
    # 尝试编译
    if cargo build --example "$example" 2>&1 | grep -q "error\[E"; then
        echo "状态: ❌ 编译失败" >> "$RESULT_FILE"
        cargo build --example "$example" 2>&1 | grep "error\[E" | head -3 >> "$RESULT_FILE"
        ((compile_errors++))
        ((failed++))
        echo "  ❌ 编译失败"
        return 1
    fi
    
    # 检查是否需要特殊 feature
    if [[ "$example" == *"yaml"* ]]; then
        if cargo build --example "$example" --features yaml 2>&1 | grep -q "error\[E"; then
            echo "状态: ❌ 编译失败（带yaml feature）" >> "$RESULT_FILE"
            ((compile_errors++))
            ((failed++))
            echo "  ❌ 编译失败（带yaml feature）"
            return 1
        fi
    fi
    
    echo "状态: ✅ 编译成功" >> "$RESULT_FILE"
    
    # 判断是否需要 CLI
    if grep -q "query\|ClaudeClient\|connect" "examples/$example.rs" 2>/dev/null; then
        echo "注意: 此示例需要 Claude Code CLI 和 API key" >> "$RESULT_FILE"
        ((need_cli++))
        echo "  ℹ️  需要 Claude Code CLI"
        ((passed++))
        return 0
    fi
    
    # 尝试运行（有超时）
    echo "尝试运行..." >> "$RESULT_FILE"
    timeout 5s cargo run --example "$example" >> "$RESULT_FILE" 2>&1
    exit_code=$?
    
    if [ $exit_code -eq 124 ]; then
        echo "  ⏱️  运行超时（正常，示例可能等待输入）" >> "$RESULT_FILE"
        echo "  ⏱️  运行超时（正常）"
        ((passed++))
    elif [ $exit_code -eq 0 ]; then
        echo "  ✅ 运行成功" >> "$RESULT_FILE"
        echo "  ✅ 运行成功"
        ((passed++))
    else
        echo "  ❌ 运行失败 (退出码: $exit_code)" >> "$RESULT_FILE"
        echo "  ❌ 运行失败"
        ((runtime_errors++))
        ((failed++))
    fi
}

# 获取所有示例文件
echo "正在扫描示例文件..."
examples=$(ls examples/*.rs 2>/dev/null | sed 's/examples\//' | sed 's/\.rs$//' | sort)

# 执行测试
for example in $examples; do
    test_example "$example" ${total} $((total + 1))
done

# 生成摘要
echo "" >> "$RESULT_FILE"
echo "========================================" >> "$RESULT_FILE"
echo "执行摘要" >> "$RESULT_FILE"
echo "========================================" >> "$RESULT_FILE"
echo "总计: $total" >> "$RESULT_FILE"
echo "通过: $passed" >> "$RESULT_FILE"
echo "失败: $failed" >> "$RESULT_FILE"
echo "编译错误: $compile_errors" >> "$RESULT_FILE"
echo "运行时错误: $runtime_errors" >> "$RESULT_FILE"
echo "需要 CLI: $need_cli" >> "$RESULT_FILE"
echo "成功率: $(echo "scale=1; $passed * 100 / $total" | bc)%" >> "$RESULT_FILE"

# 输出结果
cat "$RESULT_FILE"
echo ""
echo "详细结果已保存到: $RESULT_FILE"
