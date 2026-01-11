---
name: graham-value-investing
description: 分析股票的Graham内在价值和安全边际。当用户询问股票价值、内在价值、安全边际、Graham分析或价值投资时使用。支持快速估值、详细分析和批量分析。
---

# Graham Value Investing

基于Benjamin Graham的价值投资法则计算股票内在价值和安全边际。

## 快速开始

分析单个股票：
```bash
cargo run --bin invest_cli -- analyze AAPL
```

批量分析多只股票：
```bash
cargo run --bin invest_cli -- batch-analyze AAPL MSFT GOOGL
```

## 分析类型

### 快速估值
使用Graham公式快速计算内在价值和安全边际。适用于：
- 初步筛选投资标的
- 快速判断股票是否被低估
- 获取Graham评分

### 详细分析
完整的Graham-Buffett价值分析，包括：
- Graham内在价值计算
- Buffett质量指标（ROIC、护城河）
- DCF估值模型
- 完整的投资建议

## 核心公式

**Graham内在价值**：
```
V = EPS × (8.5 + 2g)
```

**安全边际**：
```
Margin of Safety = (Intrinsic Value - Current Price) / Intrinsic Value
```

**买入标准**：安全边际 ≥ 30%

## 投资建议标准

- **强烈买入** (5/5): 安全边际 ≥ 50%
- **买入** (4/5): 安全边际 ≥ 30%
- **持有** (3/5): 安全边际 ≥ 15%
- **观望** (2/5): 安全边际 ≥ 0%
- **避免** (1/5): 安全边际 < 0%

## 支持功能

详见 [detailed-analysis.md](detailed-analysis.md) 了解完整分析框架
详见 [evaluation-criteria.md](evaluation-criteria.md) 了解评分标准
详见 [reference-implementation.md](reference-implementation.md) 了解Rust实现示例

## 使用技巧

1. **适用于稳定增长的成熟公司**
2. **对高增长公司可能低估内在价值**
3. **周期性行业需要调整增长率预期**
4. **亏损公司不适用Graham公式**

## 相关资料

- Benjamin Graham - 《智能投资者》
- [Graham Formula详解](https://www.grahamvalue.com/article/understanding-benjamin-graham-formula-correctly)
- [Investing.com: Benjamin Graham Formula](https://www.investing.com/academy/analysis/benjamin-graham-formula-definition/)
