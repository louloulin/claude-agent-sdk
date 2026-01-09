#!/bin/bash
# 检查所有示例的编译状态

echo "========================================="
echo "示例编译状态检查"
echo "========================================="
echo ""

RESULT_FILE="/tmp/example_compile_status.txt"
echo "示例编译状态 - $(date)" > "$RESULT_FILE"

total=0
compiled=0
failed=0

for example in examples/*.rs; do
    name=$(basename "$example" .rs)
    ((total++))
    
    echo -n "[$total] $name ... "
    
    if cargo build --example "$name" 2>&1 | grep -q "Finished"; then
        echo "✅"
        echo "$name: ✅ 编译成功" >> "$RESULT_FILE"
        ((compiled++))
    else
        echo "❌"
        echo "$name: ❌ 编译失败" >> "$RESULT_FILE"
        ((failed++))
    fi
done

echo "" >> "$RESULT_FILE"
echo "========================================" >> "$RESULT_FILE"
echo "总计: $total" >> "$RESULT_FILE"
echo "编译成功: $compiled" >> "$RESULT_FILE"
echo "编译失败: $failed" >> "$RESULT_FILE"
echo "成功率: $(echo "scale=1; $compiled * 100 / $total" | bc)%" >> "$RESULT_FILE"

echo ""
cat "$RESULT_FILE"
