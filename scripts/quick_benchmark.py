#!/usr/bin/env python3
"""
快速性能测试脚本 - 测试实际查询性能
"""

import subprocess
import time
import statistics
from pathlib import Path


def time_rust_query(prompt: str, iterations: int = 10) -> dict:
    """测试Rust SDK查询性能"""
    print(f"\n{'='*60}")
    print(f"测试Rust SDK性能 - {iterations}次迭代")
    print(f"Prompt: {prompt[:50]}...")
    print(f"{'='*60}\n")

    times = []

    for i in range(iterations):
        start = time.perf_counter()

        try:
            result = subprocess.run(
                ["cargo", "run", "--release", "--example", "01_hello_world"],
                input=prompt.encode(),
                capture_output=True,
                timeout=30,
                cwd=Path(__file__).parent.parent
            )

            if result.returncode == 0:
                elapsed = (time.perf_counter() - start) * 1000
                times.append(elapsed)
                print(f"  迭代 {i+1}/{iterations}: {elapsed:.1f}ms")
            else:
                print(f"  迭代 {i+1}/{iterations}: 失败")
                if result.stderr:
                    print(f"    错误: {result.stderr.decode()[:200]}")
        except subprocess.TimeoutExpired:
            print(f"  迭代 {i+1}/{iterations}: 超时 (>30s)")

    if not times:
        print("\n❌ 所有测试都失败了！")
        return None

    # 计算统计数据
    sorted_times = sorted(times)
    n = len(times)

    stats = {
        'mean': statistics.mean(times),
        'median': statistics.median(times),
        'min': min(times),
        'max': max(times),
        'p95': sorted_times[int(n * 0.95)] if n >= 20 else max(times),
        'p99': sorted_times[int(n * 0.99)] if n >= 100 else max(times),
        'std_dev': statistics.stdev(times) if n > 1 else 0,
        'samples': n,
        'all_times': times
    }

    # 打印统计结果
    print(f"\n📊 统计结果:")
    print(f"  平均延迟:     {stats['mean']:.1f}ms")
    print(f"  中位数:       {stats['median']:.1f}ms")
    print(f"  最小值:       {stats['min']:.1f}ms")
    print(f"  最大值:       {stats['max']:.1f}ms")
    print(f"  P95:          {stats['p95']:.1f}ms")
    print(f"  P99:          {stats['p99']:.1f}ms")
    print(f"  标准差:       {stats['std_dev']:.1f}ms")
    print(f"  变异系数:     {(stats['std_dev']/stats['mean']*100):.1f}%")

    return stats


def main():
    print("🚀 Claude Agent SDK - Rust性能测试")
    print("="*60)

    # 测试场景
    test_cases = [
        ("简单查询", "What is 2 + 2?", 10),
        ("中等复杂度", "Explain recursion in programming briefly", 5),
        ("代码生成", "Write a hello world in Python", 3),
    ]

    all_results = {}

    for name, prompt, iterations in test_cases:
        print(f"\n{'#'*60}")
        print(f"场景: {name}")
        print(f"{'#'*60}")

        result = time_rust_query(prompt, iterations)
        if result:
            all_results[name] = result

    # 生成总结报告
    if all_results:
        print(f"\n\n{'='*80}")
        print("📈 性能测试总结报告")
        print(f"{'='*80}\n")

        print(f"{'场景':<20} {'平均':<12} {'中位数':<12} {'最小':<12} {'最大':<12} {'P95':<12}")
        print("-" * 80)

        for scenario, stats in all_results.items():
            print(f"{scenario:<20} "
                  f"{stats['mean']:<12.1f} "
                  f"{stats['median']:<12.1f} "
                  f"{stats['min']:<12.1f} "
                  f"{stats['max']:<12.1f} "
                  f"{stats['p95']:<12.1f}")

        # 分析瓶颈
        print(f"\n🔍 性能分析:")
        print("-" * 80)

        for scenario, stats in all_results.items():
            print(f"\n{scenario}:")
            print(f"  • 平均延迟: {stats['mean']:.1f}ms")
            print(f"  • 延迟波动: {stats['max'] - stats['min']:.1f}ms (范围)")
            print(f"  • 稳定性: {'优秀' if stats['std_dev']/stats['mean'] < 0.2 else '良好' if stats['std_dev']/stats['mean'] < 0.4 else '需改进'}")

            # 估算瓶颈
            if stats['mean'] > 1000:
                print(f"  • 主要瓶颈: 可能是子进程启动或网络延迟")
                print(f"  • 建议: 实施连接池优化")
            elif stats['mean'] > 500:
                print(f"  • 主要瓶颈: 可能是IPC通信开销")
                print(f"  • 建议: 考虑服务器模式或直接API")
            else:
                print(f"  • 性能: 良好！")


if __name__ == "__main__":
    main()
