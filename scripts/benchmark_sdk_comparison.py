#!/usr/bin/env python3
"""
Claude Agent SDK跨语言性能对比测试工具

支持测试Rust、Python和Node.js SDK的性能，并生成对比报告。
"""

import asyncio
import json
import subprocess
import time
from dataclasses import dataclass
from typing import Dict, List, Optional
from pathlib import Path
import statistics


@dataclass
class BenchmarkResult:
    """基准测试结果"""
    name: str
    mean_ms: float
    median_ms: float
    p95_ms: float
    p99_ms: float
    min_ms: float
    max_ms: float
    std_dev_ms: float
    samples: int


class SDKBenchmark:
    """SDK基准测试器"""

    def __init__(self, iterations: int = 50, timeout: int = 30):
        self.iterations = iterations
        self.timeout = timeout

    def _run_rust_example(self, example_name: str, prompt: str) -> float:
        """运行Rust示例并测量时间"""
        start = time.perf_counter()
        try:
            result = subprocess.run(
                ["cargo", "run", "--release", "--example", example_name],
                input=prompt.encode(),
                capture_output=True,
                timeout=self.timeout,
                cwd=Path(__file__).parent.parent
            )
            if result.returncode != 0:
                print(f"Rust error: {result.stderr.decode()}")
                return -1
        except subprocess.TimeoutExpired:
            return -1
        return (time.perf_counter() - start) * 1000  # 转换为毫秒

    def _run_python_sdk(self, prompt: str) -> float:
        """运行Python SDK并测量时间"""
        try:
            from anthropic import Anthropic
            client = Anthropic()

            start = time.perf_counter()
            client.messages.create(
                model="claude-sonnet-4-5",
                max_tokens=1024,
                messages=[{"role": "user", "content": prompt}]
            )
            return (time.perf_counter() - start) * 1000
        except Exception as e:
            print(f"Python error: {e}")
            return -1

    def _run_nodejs_sdk(self, prompt: str) -> float:
        """运行Node.js SDK并测量时间"""
        script = """
const anthropic = require('@anthropic-ai/sdk');
const client = new anthropic.Anthropic();

async function query(prompt) {
    const start = Date.now();
    await client.messages.create({
        model: 'claude-sonnet-4-5',
        max_tokens: 1024,
        messages: [{ role: 'user', content: prompt }]
    });
    return Date.now() - start;
}

query(process.argv[2]).then(time => console.log(time));
"""
        start = time.perf_counter()
        try:
            result = subprocess.run(
                ["node", "-e", script, prompt],
                capture_output=True,
                timeout=self.timeout
            )
            if result.returncode != 0:
                print(f"Node.js error: {result.stderr.decode()}")
                return -1
            return float(result.stdout.decode().strip())
        except subprocess.TimeoutExpired:
            return -1
        except ValueError:
            return -1
        return (time.perf_counter() - start) * 1000

    def benchmark_rust(self, prompt: str, example: str = "01_hello_world") -> BenchmarkResult:
        """运行Rust SDK基准测试"""
        print(f"运行Rust SDK测试 ({self.iterations}次迭代)...")
        times: List[float] = []

        for i in range(self.iterations):
            elapsed = self._run_rust_example(example, prompt)
            if elapsed > 0:
                times.append(elapsed)
            print(f"  迭代 {i+1}/{self.iterations}: {elapsed:.1f}ms", end='\r')

        if not times:
            raise RuntimeError("Rust SDK测试失败: 所有迭代都超时或出错")

        return self._calculate_statistics("Rust SDK", times)

    def benchmark_python(self, prompt: str) -> BenchmarkResult:
        """运行Python SDK基准测试"""
        print(f"运行Python SDK测试 ({self.iterations}次迭代)...")
        times: List[float] = []

        for i in range(self.iterations):
            elapsed = self._run_python_sdk(prompt)
            if elapsed > 0:
                times.append(elapsed)
            print(f"  迭代 {i+1}/{self.iterations}: {elapsed:.1f}ms", end='\r')

        if not times:
            raise RuntimeError("Python SDK测试失败: 所有迭代都超时或出错")

        return self._calculate_statistics("Python SDK", times)

    def benchmark_nodejs(self, prompt: str) -> BenchmarkResult:
        """运行Node.js SDK基准测试"""
        print(f"运行Node.js SDK测试 ({self.iterations}次迭代)...")
        times: List[float] = []

        for i in range(self.iterations):
            elapsed = self._run_nodejs_sdk(prompt)
            if elapsed > 0:
                times.append(elapsed)
            print(f"  迭代 {i+1}/{self.iterations}: {elapsed:.1f}ms", end='\r')

        if not times:
            raise RuntimeError("Node.js SDK测试失败: 所有迭代都超时或出错")

        return self._calculate_statistics("Node.js SDK", times)

    def _calculate_statistics(self, name: str, times: List[float]) -> BenchmarkResult:
        """计算统计数据"""
        sorted_times = sorted(times)
        n = len(times)

        return BenchmarkResult(
            name=name,
            mean_ms=statistics.mean(times),
            median_ms=statistics.median(times),
            p95_ms=sorted_times[int(n * 0.95)] if n >= 20 else max(times),
            p99_ms=sorted_times[int(n * 0.99)] if n >= 100 else max(times),
            min_ms=min(times),
            max_ms=max(times),
            std_dev_ms=statistics.stdev(times) if n > 1 else 0,
            samples=n
        )

    def print_comparison_table(self, results: Dict[str, BenchmarkResult]):
        """打印对比表格"""
        print("\n" + "="*100)
        print("性能对比结果")
        print("="*100)

        # 打印表头
        print(f"{'场景':<20} {'SDK':<15} {'平均':<10} {'中位数':<10} {'P95':<10} {'P99':<10} {'标准差':<10}")
        print("-" * 100)

        # 打印每个场景的结果
        for scenario, results_dict in results.items():
            for sdk_name, result in results_dict.items():
                print(f"{scenario:<20} {sdk_name:<15} "
                      f"{result.mean_ms:<10.1f} "
                      f"{result.median_ms:<10.1f} "
                      f"{result.p95_ms:<10.1f} "
                      f"{result.p99_ms:<10.1f} "
                      f"{result.std_dev_ms:<10.1f}")
            print()

    def generate_markdown_report(self, results: Dict[str, BenchmarkResult], output_path: str = "benchmark_results.md"):
        """生成Markdown格式的报告"""
        report = []
        report.append("# Claude Agent SDK 性能对比报告\n")
        report.append(f"**生成时间**: {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
        report.append(f"**测试配置**: 每个SDK {self.iterations} 次迭代\n")

        # 概览表格
        report.append("## 性能概览\n")
        report.append("| 场景 | SDK | 平均 (ms) | 中位数 (ms) | P95 (ms) | P99 (ms) | 标准差 (ms) |")
        report.append("|------|-----|-----------|-------------|----------|----------|-------------|")

        for scenario, results_dict in results.items():
            for sdk_name, result in results_dict.items():
                report.append(
                    f"| {scenario} | {sdk_name} | "
                    f"{result.mean_ms:.1f} | "
                    f"{result.median_ms:.1f} | "
                    f"{result.p95_ms:.1f} | "
                    f"{result.p99_ms:.1f} | "
                    f"{result.std_dev_ms:.1f} |"
                )

        # 详细分析
        report.append("\n## 详细分析\n")

        for scenario, results_dict in results.items():
            report.append(f"### {scenario}\n")

            # 找出最快的SDK
            fastest = min(results_dict.values(), key=lambda r: r.mean_ms)

            for sdk_name, result in results_dict.items():
                speedup = result.mean_ms / fastest.mean_ms
                report.append(f"#### {sdk_name}\n")
                report.append(f"- 平均延迟: **{result.mean_ms:.1f}ms**")
                report.append(f"- 相对性能: {speedup:.2f}x " +
                             ("(最快) 🚀" if speedup == 1.0 else f"({speedup:.2f}x 慢)"))
                report.append(f"- 延迟范围: {result.min_ms:.1f}ms - {result.max_ms:.1f}ms")
                report.append(f"- 标准差: {result.std_dev_ms:.1f}ms ({(result.std_dev_ms/result.mean_ms*100):.1f}% 变异系数)")
                report.append("")

        # 建议
        report.append("## 性能建议\n")

        for scenario, results_dict in results.items():
            report.append(f"### {scenario}\n")
            fastest_sdk = min(results_dict.items(), key=lambda x: x[1].mean_ms)
            report.append(f"- **推荐**: {fastest_sdk[0]} ({fastest_sdk[1].mean_ms:.1f}ms 平均延迟)")
            report.append(f"- **最慢**: {max(results_dict.items(), key=lambda x: x[1].mean_ms)[0]}")

            # 性能差异分析
            speeds = [r.mean_ms for r in results_dict.values()]
            variation = (max(speeds) - min(speeds)) / min(speeds) * 100
            report.append(f"- **性能差异**: {variation:.1f}%")
            report.append("")

        # 写入文件
        output_file = Path(output_path)
        output_file.write_text("\n".join(report), encoding='utf-8')
        print(f"\n报告已生成: {output_file.absolute()}")


async def main():
    """主函数"""
    print("Claude Agent SDK 性能对比测试")
    print("=" * 50)

    # 测试场景
    test_scenarios = {
        "简单查询": "What is 2 + 2?",
        "中等复杂度": "Explain the concept of recursion in programming",
        "代码生成": "Write a function to calculate fibonacci numbers in Python",
    }

    benchmark = SDKBenchmark(iterations=30, timeout=60)
    all_results = {}

    for scenario_name, prompt in test_scenarios.items():
        print(f"\n{'='*50}")
        print(f"测试场景: {scenario_name}")
        print(f"Prompt: {prompt[:50]}...")
        print(f"{'='*50}\n")

        scenario_results = {}

        # 测试每个SDK
        try:
            scenario_results["Rust"] = benchmark.benchmark_rust(prompt, "01_hello_world")
        except Exception as e:
            print(f"Rust SDK测试失败: {e}")

        try:
            scenario_results["Python"] = benchmark.benchmark_python(prompt)
        except Exception as e:
            print(f"Python SDK测试失败: {e}")

        try:
            scenario_results["Node.js"] = benchmark.benchmark_nodejs(prompt)
        except Exception as e:
            print(f"Node.js SDK测试失败: {e}")

        all_results[scenario_name] = scenario_results

    # 打印结果
    benchmark.print_comparison_table(all_results)

    # 生成报告
    benchmark.generate_markdown_report(all_results)


if __name__ == "__main__":
    asyncio.run(main())
