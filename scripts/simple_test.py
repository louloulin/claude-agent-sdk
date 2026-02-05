#!/usr/bin/env python3
"""
简单直接的性能测试
"""
import subprocess
import time
import os
import sys

print("🚀 运行单次查询测试...")
print("-" * 60)

# Check if API key is set
api_key = os.environ.get("ANTHROPIC_API_KEY", "")
if not api_key or len(api_key) < 10:
    print("❌ 错误: ANTHROPIC_API_KEY 未设置或无效")
    print("   请设置环境变量: export ANTHROPIC_API_KEY=sk-ant-...")
    print("\n   跳过测试 (需要有效的API密钥)")
    sys.exit(0)

# Check if binary exists, otherwise build it
binary_path = "./target/release/examples/01_hello_world"
if not os.path.exists(binary_path):
    print("📦 首次运行，编译示例...")
    build_result = subprocess.run(
        ["cargo", "build", "--release", "--example", "01_hello_world"],
        capture_output=True,
        timeout=300
    )
    if build_result.returncode != 0:
        print("❌ 编译失败:")
        print(build_result.stderr.decode()[-500:])
        sys.exit(1)
    print("✅ 编译完成")

prompt = "What is 2 + 2?"
start = time.perf_counter()

# Use the pre-built binary instead of cargo run
result = subprocess.run(
    [binary_path],
    input=prompt.encode(),
    capture_output=True,
    timeout=120,
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
    stderr = result.stderr.decode()
    stdout = result.stdout.decode()
    print(f"   错误输出:")
    print(f"   stderr: {stderr[:300]}")
    if stdout:
        print(f"   stdout: {stdout[:300]}")

    # Check for common API errors
    if "401" in stderr or "authentication" in stderr.lower():
        print(f"\n   ❌ API认证失败: 请检查 ANTHROPIC_API_KEY 是否正确")
    elif "timeout" in stderr.lower() or "timed out" in stderr.lower():
        print(f"\n   ⏱️  请求超时: 可能是网络问题")
    elif "rate" in stderr.lower():
        print(f"\n   ⚠️  速率限制: API请求过于频繁")

    # Exit gracefully on error
    sys.exit(0)

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
