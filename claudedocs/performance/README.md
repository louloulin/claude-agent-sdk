# Performance Analysis & Testing

性能测试和分析文档。

## 📊 报告文档

- **FINAL_PERFORMANCE_REPORT.md** - 完整性能测试报告（推荐首先阅读）
- **bench.md** - 详细的技术分析和优化计划
- **PERFORMANCE_TEST_SUMMARY.md** - 测试执行总结
- **benchmark_results.md** - 实测数据报告

## 🧪 测试工具

### 基准测试
- `../../benches/query_performance.rs` - Criterion 基准测试

### 性能测试脚本
- `../../scripts/benchmark_sdk_comparison.py` - 跨语言性能对比
- `../../scripts/detailed_benchmark.py` - 详细统计分析
- `../../scripts/quick_benchmark.py` - 快速性能测试
- `../../scripts/simple_test.py` - 单次测试
- `../../scripts/quick_test.sh` - Bash 版本快速测试

## 🚀 快速开始

```bash
# 运行快速测试
python3 scripts/simple_test.py

# 运行详细基准测试（5次迭代）
python3 scripts/detailed_benchmark.py

# 运行 Criterion 基准测试
cargo bench --bench query_performance
```

## 📈 主要发现

1. **API推理是主要瓶颈** (99% 延迟)
2. **SDK优化空间有限** (仅1%)
3. **简单查询优化收益大**，复杂查询不需要优化

详细分析请参考 `FINAL_PERFORMANCE_REPORT.md`。

## 📅 测试信息

- **测试日期**: 2026-01-15
- **SDK版本**: v0.1.5
- **测试环境**: macOS, Rust 1.85+, Claude CLI 2.0+
