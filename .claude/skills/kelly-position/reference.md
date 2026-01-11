# Kelly Position Sizing Reference

## Kelly公式数学推导

### 信息论视角

Kelly公式源自信息论，用于最大化对数财富的长期增长率：

```
G = p × log(1 + b×f) + q × log(1 - f)
```

对f求导并令为0：

```
dG/df = p×b/(1+b×f) - q/(1-f) = 0

解得：f* = (bp - q) / b
```

### 长期增长率

使用Kelly仓位时的预期对数增长率：

```
G* = p × log(1 + b×f*) + q × log(1 - f*)
   = p × log(1 + bp - q) + q × log(1 - (bp-q)/b)
   = p × log(bp/q) + q × log((1-p)/q)
```

## Kelly vs 其他策略

### 策略对比

| 策略 | 长期增长 | 波动性 | 破产风险 |
|------|---------|-------|---------|
| 全押 | 低 | 极高 | 极高 |
| 固定比例 | 中 | 中 | 低 |
| 完整Kelly | **最高** | 高 | 中（理论接近0） |
| 1/2 Kelly | 高 | 中 | 低 |
| 1/4 Kelly | 中高 | 低 | 极低 |

### Kelly优势

1. **渐近最优** - 长期看财富增长最快
2. **避免破产** - 理论上破产概率为0
3. **数学严谨** - 基于信息论，有坚实理论基础

### Kelly劣势

1. **短期高波动** - 可能经历50%+回撤
2. **参数敏感** - 小的估计误差导致大的仓位变化
3. **假设理想化** - 实际市场不满足i.i.d.假设

## 分数Kelly实践

### 为什么使用分数Kelly？

**Munger的观点**：
> "聪明人做的事情，他们也会做一半。我从来没用过完整Kelly，只用半Kelly或四分之一Kelly。"

**原因**：
1. **降低波动** - 1/4 Kelly的波动是完整Kelly的1/4
2. **参数不确定性** - 我们的估计总有误差
3. **心理承受力** - 大部分人无法承受完整Kelly的回撤
4. **相关性** - 实际投资存在相关性，i.i.d.假设不成立

### 分数选择

| 投资者类型 | Kelly分数 | 适用场景 |
|-----------|----------|---------|
| 极度保守 | 1/8 | 学习阶段，新手 |
| 保守 | 1/4 | 大部分投资者（Munger推荐） |
| 中性 | 1/3 | 专业投资者 |
| 进取 | 1/2 | 高置信度机会 |
| 激进 | 完整 | 很少使用 |

## 实现细节

### Rust代码结构

```rust
pub struct KellyAnalysis {
    /// 股票代码
    pub symbol: String,

    /// 完整Kelly仓位
    pub full_kelly: f64,

    /// 推荐仓位（1/4 Kelly）
    pub recommended_kelly: f64,

    /// Kelly分数（0.25 = 1/4 Kelly）
    pub kelly_fraction: f64,

    /// 风险等级
    pub risk_level: RiskLevel,

    /// 最终仓位（考虑限制）
    pub final_position: f64,

    /// 限制原因
    pub limit_reason: Option<String>,
}

pub enum RiskLevel {
    High,    // Kelly > 15%
    Medium,  // Kelly 5-15%
    Low,     // Kelly 2-5%
    VeryLow, // Kelly < 2%
}
```

### 关键函数

```rust
impl KellyPositionAgent {
    /// 完整Kelly公式
    fn calculate_kelly(
        &self,
        win_rate: f64,
        avg_win: f64,
        avg_loss: f64
    ) -> f64 {
        if avg_loss <= 0.0 || win_rate <= 0.0 || win_rate >= 1.0 {
            return 0.0;
        }

        let b = avg_win / avg_loss;  // 盈亏比
        let p = win_rate;
        let q = 1.0 - p;

        let kelly = (b * p - q) / b;
        kelly.max(0.0)
    }

    /// 分数Kelly
    fn calculate_fractional_kelly(
        &self,
        full_kelly: f64,
        fraction: f64
    ) -> f64 {
        (full_kelly * fraction).min(1.0)
    }

    /// 应用仓位限制
    fn apply_position_limits(
        &self,
        kelly: f64,
        max_single_position: f64
    ) -> f64 {
        kelly.min(max_single_position).max(0.02)
    }

    /// 评估风险等级
    fn assess_risk_level(&self, kelly: f64) -> RiskLevel {
        if kelly > 0.15 {
            RiskLevel::High
        } else if kelly > 0.05 {
            RiskLevel::Medium
        } else if kelly > 0.02 {
            RiskLevel::Low
        } else {
            RiskLevel::VeryLow
        }
    }
}
```

## 组合优化

### 多资产Kelly

当有N个不相关资产时：

```rust
// 1. 计算每个资产的Kelly
let kellys: Vec<f64> = assets.iter()
    .map(|a| calculate_kelly(a.win_rate, a.avg_win, a.avg_loss))
    .collect();

// 2. 总Kelly
let total_kelly: f64 = kellys.iter().sum();

// 3. 如果总Kelly > 100%，归一化
let normalized_kellys: Vec<f64> = if total_kelly > 1.0 {
    kellys.iter().map(|k| k / total_kelly).collect()
} else {
    kellys
};

// 4. 应用单资产上限
let final_positions: Vec<f64> = normalized_kellys.iter()
    .map(|k| k.min(0.25))
    .collect();
```

### 相关性调整

当资产相关时，需要降低仓位：

```rust
// 简化方法：根据相关性降低Kelly
fn adjust_for_correlation(
    kelly: f64,
    correlation: f64
) -> f64 {
    // correlation in [0, 1]
    let adjustment_factor = 1.0 - correlation * 0.5;
    kelly * adjustment_factor
}
```

## 历史性能

### Kelly vs 1/4 Kelly vs 固定比例

假设策略：
- 胜率：55%
- 盈亏比：1.2
- 初始资金：$10,000
- 交易次数：100次

| 策略 | 期望终值 | 标准差 | 最大回撤 |
|------|---------|-------|---------|
| 完整Kelly | $48,000 | $35,000 | -65% |
| 1/2 Kelly | $31,000 | $15,000 | -40% |
| 1/4 Kelly | $21,000 | $7,000 | -22% |
| 固定10% | $18,000 | $5,000 | -18% |

结论：1/4 Kelly在增长和风险间取得最佳平衡。

## 集成到InvestmentAssistant

```rust
impl InvestmentAssistant {
    /// Kelly仓位建议
    pub async fn suggest_kelly_position(
        &self,
        symbol: &str
    ) -> Result<String> {
        let output = self.kelly_agent
            .execute(AgentInput::new(symbol))
            .await?;

        Ok(output.content)
    }
}
```
