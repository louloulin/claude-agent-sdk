#!/bin/bash
# 快速性能测试脚本

echo "🚀 Claude Agent SDK - 快速性能测试"
echo "========================================"

PROMPT="What is 2 + 2?"
ITERATIONS=5

echo ""
echo "测试场景: 简单查询"
echo "Prompt: $PROMPT"
echo "迭代次数: $ITERATIONS"
echo ""

TOTAL_TIME=0
for i in $(seq 1 $ITERATIONS); do
    echo "运行 $i/$ITERATIONS..."
    START=$(python3 -c 'import time; print(int(time.time() * 1000))')

    cargo run --release --example 01_hello_world <<< "$PROMPT" > /dev/null 2>&1

    END=$(python3 -c 'import time; print(int(time.time() * 1000))')
    ELAPSED=$((END - START))

    echo "  耗时: ${ELAPSED}ms"
    TOTAL_TIME=$((TOTAL_TIME + ELAPSED))
done

AVG=$((TOTAL_TIME / ITERATIONS))

echo ""
echo "========================================"
echo "📊 测试结果:"
echo "  总耗时: ${TOTAL_TIME}ms"
echo "  平均:   ${AVG}ms"
echo "  迭代:   ${ITERATIONS}次"
echo "========================================"

# 性能分析
if [ $AVG -gt 2000 ]; then
    echo "⚠️  性能较差 (>2000ms)"
    echo "   建议: 检查网络连接和CLI配置"
elif [ $AVG -gt 1000 ]; then
    echo "⚠️  性能一般 (>1000ms)"
    echo "   建议: 实施连接池优化"
elif [ $AVG -gt 500 ]; then
    echo "✓  性能良好 (500-1000ms)"
    echo "   建议: 可考虑服务器模式优化"
else
    echo "✅  性能优秀 (<500ms)"
    echo "   当前实现已经很快！"
fi
