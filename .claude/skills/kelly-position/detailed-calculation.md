# Kelly仓位计算 - 详细方法

## Kelly公式推导

### 信息理论基础

Kelly公式源于信息论,用于最大化长期资本增长率:

```
G = p·log(1 + b·f) + q·log(1 - f)
```

对f求导并令其为0:

```
dG/df = p·b/(1 + b·f) - q/(1 - f) = 0
f* = (bp - q) / b
```

## 两种计算方式

### 1. 基于交易统计的完整Kelly

当您有以下数据时:
- 胜率 p (盈利交易占比)
- 平均盈利金额
- 平均亏损金额

**步骤**:

```rust
// 计算盈亏比
let b = avg_win / avg_loss;

// 计算败率
let q = 1.0 - p;

// 完整Kelly
let kelly = (b * p - q) / b;
```

**示例**:

```
胜率: 60%
平均盈利: $120
平均亏损: $80

b = 120 / 80 = 1.5
p = 0.6, q = 0.4
kelly = (1.5 × 0.6 - 0.4) / 1.5 = 0.333

推荐仓位 = 1/4 Kelly = 8.25%
```

### 2. 基于收益率的简化Kelly

当您只有历史收益率序列时:

```rust
// 计算平均收益率
let mu = returns.iter().sum::<f64>() / returns.len() as f64;

// 计算方差
let variance = returns.iter()
    .map(|r| (r - mu).powi(2))
    .sum::<f64>() / (returns.len() - 1) as f64;

// 简化Kelly
let kelly = mu / variance;
```

## 分数Kelly策略

### 为什么使用分数Kelly?

完整Kelly理论最优但实际应用问题:

1. **参数估计误差**: 真实胜率和盈亏比难以精确估计
2. **市场非平稳**: 历史数据不能保证未来表现
3. **高波动性**: 完整Kelly可能导致巨大回撤
4. **心理压力**: 大幅波动影响执行纪律

### 常用分数策略

| 分数 | 波动性 | 适用场景 |
|------|--------|----------|
| **1/2 Kelly** | 中等 | Kelly本人推荐,平衡增长与波动 |
| **1/4 Kelly** | 低 | Buffett-Munger推荐,保守策略 |
| **1/8 Kelly** | 极低 | 极度厌恶风险者 |

**建议**: 个人投资者使用1/4 Kelly,专业投资者可用1/2 Kelly

## 仓位限制规则

### 单只股票限制

基于Munger的风险管理原则:

```rust
fn apply_position_limits(kelly: f64) -> f64 {
    // 应用1/4分数
    let fractional_kelly = kelly * 0.25;

    // 单只股票最大25%
    let max_position = 0.25;

    // 最小2%,信号太弱不建仓
    let min_position = 0.02;

    if fractional_kelly < min_position {
        0.0  // 不建仓
    } else {
        fractional_kelly.min(max_position)
    }
}
```

### 组合级别管理

当持有多只股票时:

```rust
// 1. 计算每只股票的Kelly
let kellys: Vec<f64> = stocks.iter()
    .map(|s| calculate_kelly(s))
    .collect();

// 2. 归一化(确保总仓位≤100%)
let total: f64 = kellys.iter().sum();
let normalized: Vec<f64> = if total > 1.0 {
    kellys.iter().map(|k| k / total).collect()
} else {
    kellys
};

// 3. 应用单只股票上限
let final_positions: Vec<f64> = normalized.iter()
    .map(|k| apply_position_limits(*k))
    .collect();
```

## 风险评估

### Kelly结果解释

| Kelly范围 | 解释 | 推荐操作 |
|-----------|------|----------|
| > 25% | 极强信号 | 1/4 Kelly后6.25%仓位 |
| 15-25% | 强信号 | 1/4 Kelly后3.75-6.25% |
| 5-15% | 良好信号 | 1/4 Kelly后1.25-3.75% |
| 2-5% | 弱信号 | 1/4 Kelly后0.5-1.25% |
| < 2% | 信号不足 | 不建仓 |

### 负Kelly的处理

如果Kelly计算结果为负:
- 意味着期望收益为负
- **不应该建立仓位**
- 考虑反向操作(如果允许做空)

## 特殊场景

### 趋势跟踪策略

趋势策略通常:
- 胜率低(35-40%)
- 盈亏比高(2-3:1)

Kelly计算:
```
b = 2.5, p = 0.38
kelly = (2.5 × 0.38 - 0.62) / 2.5 = 0.128
推荐 = 1/4 × 12.8% = 3.2%
```

### 均值回归策略

均值回归策略通常:
- 胜率高(55-65%)
- 盈亏比低(0.8-1.2:1)

Kelly计算:
```
b = 1.0, p = 0.60
kelly = (1.0 × 0.60 - 0.40) / 1.0 = 0.20
推荐 = 1/4 × 20% = 5.0%
```

## 实际应用建议

### 数据要求

1. **最少交易次数**: ≥30笔(统计显著性)
2. **时间周期**: ≥6个月历史数据
3. **一致性**: 策略执行前后一致

### 参数估计技巧

1. **胜率估计**: 使用置信区间而非点估计
2. **盈亏比**: 使用中位数而非平均数(减少极端值影响)
3. **定期更新**: 每季度重新计算Kelly参数

### 组合构建流程

```
1. 筛选: Kelly > 2%的股票
2. 计算: 每只股票的1/4 Kelly
3. 归一化: 确保总仓位 ≤ 100%
4. 限制: 单只 ≤ 25%
5. 再平衡: 每月或信号显著变化时
```
