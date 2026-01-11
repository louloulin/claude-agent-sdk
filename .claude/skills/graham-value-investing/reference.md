# Graham Value Investing Reference

## Benjamin Graham核心思想

### 1. 价值投资三原则

1. **安全边际** - 以低于内在价值的价格买入
2. **内在价值** - 基于商业事实的客观价值
3. **市场波动** - 市场先生理论，利用波动而非被其左右

### 2. Graham公式推导

**基础版本**：
```
V = EPS × (8.5 + 2g)
```

**调整版本**（考虑债券收益率）：
```
V = EPS × (8.5 + 2g) × (4.4 / Y)
```
其中Y = 当前AAA企业债券收益率

### 3. 防御型投资者标准

| 标准 | 要求 |
|------|------|
| PE倍数 | < 25（理想< 15） |
| PB倍数 | < 1.5 |
| 股息率 | > 2% |
| 负债率 | < 50% |
| 流动比率 | > 2 |
| 营收增长 | 过去10年每年增长 |

### 4. 进取型投资者标准

- 满足防御型所有标准
- PE < 10
- 股息率 > 3%
- 盈利增长连续5年以上

## 计算示例

### 示例1：可口可乐 (KO)

假设数据：
- EPS = $2.50
- 当前价格 = $60
- 预期增长率 = 5%

计算：
```
内在价值 = 2.50 × (8.5 + 2×0.05)
         = 2.50 × 9.5
         = $23.75

安全边际 = (23.75 - 60) / 23.75
         = -152.6%
```

结论：高估，避免

### 示例2：某被低估公司

假设数据：
- EPS = $5.00
- 当前价格 = $50
- 预期增长率 = 8%

计算：
```
内在价值 = 5.00 × (8.5 + 2×0.08)
         = 5.00 × 10.16
         = $50.80

安全边际 = (50.80 - 50) / 50.80
         = 1.6%
```

结论：略微低估，但不满足30%安全边际标准，观望

## 实现细节

### Rust代码结构

```rust
pub struct GrahamAnalysis {
    /// 股票代码
    pub symbol: String,

    /// 内在价值
    pub intrinsic_value: f64,

    /// 当前价格
    pub current_price: f64,

    /// 安全边际（百分比）
    pub margin_of_safety: f64,

    /// 买入建议
    pub recommendation: String,

    /// Graham评分 (0-40)
    pub graham_score: u8,

    /// 分析详情
    pub details: GrahamDetails,
}

pub struct GrahamDetails {
    /// 每股收益
    pub eps: f64,

    /// 预期增长率
    pub growth_rate: f64,

    /// 估值折扣评分
    pub valuation_score: u8,

    /// 盈利质量评分
    pub earnings_quality_score: u8,

    /// 财务健康评分
    pub financial_health_score: u8,
}
```

### 关键函数

```rust
impl ValueInvestmentAgent {
    /// Graham内在价值计算
    fn calculate_graham_intrinsic_value(
        &self,
        eps: f64,
        growth_rate: f64
    ) -> f64 {
        let multiplier = 8.5 + 2.0 * growth_rate * 100.0;
        eps * multiplier
    }

    /// 安全边际计算
    fn calculate_margin_of_safety(
        &self,
        intrinsic_value: f64,
        current_price: f64
    ) -> f64 {
        (intrinsic_value - current_price) / intrinsic_value
    }

    /// Graham评分 (0-40)
    fn calculate_graham_score(
        &self,
        margin_of_safety: f64,
        eps_stability: f64,
        debt_ratio: f64
    ) -> u8 {
        let mut score = 0u8;

        // 估值折扣评分 (0-20分)
        if margin_of_safety >= 0.50 {
            score += 20;
        } else if margin_of_safety >= 0.30 {
            score += 16;
        } else if margin_of_safety >= 0.15 {
            score += 12;
        } else if margin_of_safety >= 0.0 {
            score += 8;
        }

        // 盈利质量评分 (0-10分)
        if eps_stability >= 0.8 {
            score += 10;
        } else if eps_stability >= 0.6 {
            score += 7;
        } else if eps_stability >= 0.4 {
            score += 4;
        }

        // 财务健康评分 (0-10分)
        if debt_ratio < 0.3 {
            score += 10;
        } else if debt_ratio < 0.5 {
            score += 7;
        } else if debt_ratio < 0.7 {
            score += 4;
        }

        score
    }
}
```

## 集成到InvestmentAssistant

```rust
impl InvestmentAssistant {
    /// Graham价值投资分析
    pub async fn analyze_graham(&self, symbol: &str) -> Result<String> {
        let output = self.value_agent
            .execute(AgentInput::new(symbol))
            .await?;

        Ok(output.content)
    }
}
```
