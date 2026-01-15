#!/usr/bin/env python3
"""
收集多次测试的统计数据
"""
import subprocess
import time
import statistics

def run_test(iterations=5):
    """运行多次测试"""
    print(f"🚀 运行 {iterations} 次性能测试...")
    print("=" * 70)

    prompt = "What is 2 + 2?"
    times = []

    for i in range(iterations):
        print(f"\n测试 {i+1}/{iterations}...", end=" ")

        start = time.perf_counter()

        result = subprocess.run(
            ["cargo", "run", "--release", "--example", "01_hello_world"],
            input=prompt.encode(),
            capture_output=True,
            timeout=60,
            cwd="."
        )

        elapsed = (time.perf_counter() - start) * 1000
        times.append(elapsed)

        if result.returncode == 0:
            print(f"✅ {elapsed:.1f}ms")
        else:
            print(f"❌ 失败 ({elapsed:.1f}ms)")

    # 分析结果
    print("\n" + "=" * 70)
    print("📊 统计分析")
    print("=" * 70)

    sorted_times = sorted(times)
    n = len(times)

    mean = statistics.mean(times)
    median = statistics.median(times)
    min_t = min(times)
    max_t = max(times)
    std_dev = statistics.stdev(times) if n > 1 else 0
    p95 = sorted_times[int(n * 0.95)] if n >= 20 else max_t
    p99 = sorted_times[int(n * 0.99)] if n >= 100 else max_t

    print(f"\n延迟统计:")
    print(f"  平均值:     {mean:.1f}ms")
    print(f"  中位数:     {median:.1f}ms")
    print(f"  最小值:     {min_t:.1f}ms")
    print(f"  最大值:     {max_t:.1f}ms")
    print(f"  标准差:     {std_dev:.1f}ms")
    print(f"  P95:        {p95:.1f}ms")
    print(f"  P99:        {p99:.1f}ms")
    print(f"  变异系数:   {(std_dev/mean*100):.1f}%")

    # 性能分析
    print(f"\n{'='*70}")
    print("🔍 性能瓶颈分析")
    print("=" * 70)

    # 估算各部分耗时
    estimated_startup = 150  # 子进程启动
    estimated_ipc = 75       # IPC通信
    estimated_api = median - estimated_startup - estimated_ipc

    print(f"\n耗时分解 (基于中位数 {median:.1f}ms):")
    print(f"  1. 子进程启动:      ~{estimated_startup}ms ({estimated_startup/median*100:.1f}%)")
    print(f"  2. IPC通信:         ~{estimated_ipc}ms ({estimated_ipc/median*100:.1f}%)")
    print(f"  3. API推理时间:     ~{estimated_api:.1f}ms ({estimated_api/median*100:.1f}%)")

    print(f"\n优化潜力:")
    print(f"  • 连接池优化:       可节省 ~{estimated_startup}ms (~{(estimated_startup/median*100):.0f}%)")
    print(f"     → 优化后预期:    ~{median - estimated_startup:.1f}ms")
    print(f"\n  • 服务器模式:       可节省 ~{estimated_startup + estimated_ipc}ms (~{((estimated_startup+estimated_ipc)/median*100):.0f}%)")
    print(f"     → 优化后预期:    ~{median - estimated_startup - estimated_ipc:.1f}ms")
    print(f"\n  • 直接HTTP API:     可节省 ~{estimated_startup + estimated_ipc}ms (~{((estimated_startup+estimated_ipc)/median*100):.0f}%)")
    print(f"     → 优化后预期:    ~{estimated_api:.1f}ms")

    # 与理论最优对比
    print(f"\n{'='*70}")
    print("📈 性能对比")
    print("=" * 70)

    print(f"\n当前性能:")
    print(f"  平均延迟:     {mean:.1f}ms")

    print(f"\n优化后性能预测:")
    print(f"  连接池模式:   {mean - estimated_startup:.1f}ms (提升 {(estimated_startup/mean*100):.0f}%)")
    print(f"  服务器模式:   {mean - estimated_startup - estimated_ipc:.1f}ms (提升 {((estimated_startup+estimated_ipc)/mean*100):.0f}%)")
    print(f"  直接API:      {estimated_api:.1f}ms (提升 {((estimated_startup+estimated_ipc)/mean*100):.0f}%)")

    # 建议
    print(f"\n{'='*70}")
    print("💡 优化建议")
    print("=" * 70)

    if median > 5000:
        print("\n🔴 优先级最高 - 实施连接池优化")
        print("   预期提升: 3-5倍")
        print("   实施难度: 中等")
        print("   时间投入: 1-2周")
    elif median > 1000:
        print("\n🟡 高优先级 - 实施连接池")
        print("   预期提升: 2-3倍")
        print("   同时考虑服务器模式")
    else:
        print("\n🟢 性能良好 - 持续优化")
        print("   考虑服务器模式获得更好性能")

    print(f"\n{'='*70}")

if __name__ == "__main__":
    run_test(iterations=5)
