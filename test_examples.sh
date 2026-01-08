#!/bin/bash
# 测试所有示例文件并记录结果

RESULT_FILE="/tmp/example_test_results.txt"
echo "示例测试结果 - $(date)" > "$RESULT_FILE"
echo "========================================" >> "$RESULT_FILE"

# 统计变量
total=0
passed=0
failed=0
compile_errors=0
runtime_errors=0

# 测试函数
test_example() {
    local example=$1
    local total_count=$2
    local current=$3

    echo ""
    echo "[$current/$total_count] 测试: $example"
    echo "----------------------------------------" >> "$RESULT_FILE"
    echo "示例: $example" >> "$RESULT_FILE"

    # 尝试编译
    if cargo build --example "$example" 2>&1 | grep -q "error\[E"; then
        echo "状态: ❌ 编译失败" >> "$RESULT_FILE"
        cargo build --example "$example" 2>&1 | grep -E "error\[E[0-9]+\]:" | head -3 >> "$RESULT_FILE"
        ((compile_errors++))
        ((failed++))
        echo "  ❌ 编译失败"
        return 1
    fi

    # 如果需要特殊feature
    if [[ "$example" == *"yaml"* ]]; then
        cargo build --example "$example" --features yaml 2>&1 | grep -q "error\[E" && {
            echo "状态: ❌ 编译失败（带yaml feature）" >> "$RESULT_FILE"
            ((compile_errors++))
            ((failed++))
            echo "  ❌ 编译失败（带yaml feature）"
            return 1
        }
    fi

    echo "状态: ✅ 编译成功" >> "$RESULT_FILE"
    ((passed++))
    echo "  ✅ 编译成功"
    return 0
}

# 获取所有示例文件
examples=$(ls examples/*.rs | sed 's/examples\//' | sed 's/\.rs$//' | sort)

# 执行测试
for example in $examples; do
    ((total++))
    test_example "$example" ${total} ${total}
done

# 生成摘要
echo "" >> "$RESULT_FILE"
echo "========================================" >> "$RESULT_FILE"
echo "测试摘要" >> "$RESULT_FILE"
echo "========================================" >> "$RESULT_FILE"
echo "总计: $total" >> "$RESULT_FILE"
echo "通过: $passed" >> "$RESULT_FILE"
echo "失败: $failed" >> "$RESULT_FILE"
echo "编译错误: $compile_errors" >> "$RESULT_FILE"
echo "运行时错误: $runtime_errors" >> "$RESULT_FILE"
echo "成功率: $(echo "scale=1; $passed * 100 / $total" | bc)%" >> "$RESULT_FILE"

# 输出结果
cat "$RESULT_FILE"
echo ""
echo "详细结果已保存到: $RESULT_FILE"
