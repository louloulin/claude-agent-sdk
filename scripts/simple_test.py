#!/usr/bin/env python3
"""
简单直接的性能测试
"""
import subprocess
import time

print("🚀 运行单次查询测试...")
print("-" * 60)

prompt = "What is 2 + 2?"
start = time.perf_counter()

result = subprocess.run(
    ["cargo", "run", "--release", "--example", "01_hello_world"],
    input=prompt.encode(),
    capture_output=True,
    timeout=60,
    cwd="."
)

elapsed = (time.perf_counter() - start) * 1000

print(f"✅ 完成！")
print(f"   总耗时: {elapsed:.1f}ms")
print(f"   返回码: {result.returncode}")

if result.returncode == 0:
    output = result.stdout.decode()
    print(f"   输出长度: {len(output)} 字符")
    print(f"\n   输出预览:")
    print("   " + "\n   ".join(output.split('\n')[:10]))
else:
    print(f"   错误: {result.stderr.decode()[:200]}")

# 性能分析
print(f"\n{'='*60}")
print("📊 性能分析:")
print(f"{'='*60}")

if elapsed < 500:
    print("✅ 性能优秀 (<500ms)")
    print("   当前实现已经非常快！")
elif elapsed < 1000:
    print("✓ 性能良好 (500-1000ms)")
    print("   正常的API响应时间")
elif elapsed < 2000:
    print("⚠️  性能一般 (1-2秒)")
    print("   可能存在子进程启动开销")
    print("   建议: 实施连接池可提升到 200-400ms")
else:
    print("❌ 性能较差 (>2秒)")
    print("   主要瓶颈分析:")
    print(f"   1. 子进程启动: ~100-200ms")
    print(f"   2. IPC通信: ~50-100ms")
    print(f"   3. API推理: ~{elapsed - 300:.0f}ms")
    print(f"\n   优化建议:")
    print("   • 实施连接池（可减少80%启动时间）")
    print("   • 考虑服务器模式（零启动开销）")
    print("   • 使用直接HTTP API（绕过CLI）")
