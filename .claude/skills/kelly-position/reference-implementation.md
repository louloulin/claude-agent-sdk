# Kelly仓位计算 - Rust实现参考

## 核心数据结构

```rust
use serde::{Deserialize, Serialize};

/// Kelly计算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KellyResult {
    /// 完整Kelly结果
    pub kelly_fraction: f64,

    /// 推荐仓位(分数Kelly后)
    pub recommended_position: f64,

    /// 风险等级
    pub risk_level: RiskLevel,

    /// 建议说明
    pub reasoning: String,

    /// 是否被仓位限制
    pub is_limited: bool,
}

/// 风险等级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    High,
    Medium,
    Low,
    VeryLow,
}

/// 交易统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingStats {
    /// 胜率
    pub win_rate: f64,

    /// 平均盈利
    pub avg_win: f64,

    /// 平均亏损
    pub avg_loss: f64,

    /// 总交易次数
    pub total_trades: usize,
}
```

## 核心计算实现

```rust
impl KellyResult {
    /// 使用完整Kelly公式计算
    pub fn from_trading_stats(stats: &TradingStats) -> Self {
        // 计算盈亏比
        let b = stats.avg_win / stats.avg_loss;

        // 胜率和败率
        let p = stats.win_rate;
        let q = 1.0 - p;

        // 完整Kelly公式
        let kelly = (b * p - q) / b;

        Self::from_kelly(kelly)
    }

    /// 使用简化Kelly公式(基于收益率)
    pub fn from_returns(returns: &[f64]) -> Self {
        // 计算平均收益
        let mu = returns.iter().sum::<f64>() / returns.len() as f64;

        // 计算方差
        let variance = returns.iter()
            .map(|r| (r - mu).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;

        // 简化Kelly
        let kelly = mu / variance;

        Self::from_kelly(kelly)
    }

    /// 从Kelly值创建最终结果
    fn from_kelly(kelly: f64) -> Self {
        // 应用1/4分数Kelly
        let fractional_kelly = kelly * 0.25;

        // 应用仓位限制
        let (recommended, is_limited) = Self::apply_limits(fractional_kelly);

        // 评估风险等级
        let risk_level = Self::assess_risk(fractional_kelly);

        // 生成建议
        let reasoning = Self::generate_reasoning(
            kelly,
            fractional_kelly,
            recommended,
            is_limited,
        );

        Self {
            kelly_fraction: kelly,
            recommended_position: recommended,
            risk_level,
            reasoning,
            is_limited,
        }
    }

    /// 应用仓位限制
    fn apply_limits(kelly: f64) -> (f64, bool) {
        const MAX_POSITION: f64 = 0.25;  // 单只股票最大25%
        const MIN_POSITION: f64 = 0.02;  // 最小2%

        if kelly < MIN_POSITION {
            (0.0, true)  // 信号太弱
        } else if kelly > MAX_POSITION {
            (MAX_POSITION, true)  // 触发上限
        } else {
            (kelly, false)
        }
    }

    /// 评估风险等级
    fn assess_risk(kelly: f64) -> RiskLevel {
        if kelly > 0.20 {
            RiskLevel::High
        } else if kelly > 0.10 {
            RiskLevel::Medium
        } else if kelly > 0.05 {
            RiskLevel::Low
        } else {
            RiskLevel::VeryLow
        }
    }

    /// 生成建议说明
    fn generate_reasoning(
        raw_kelly: f64,
        fractional_kelly: f64,
        final_position: f64,
        is_limited: bool,
    ) -> String {
        let mut reasons = vec![
            format!("完整Kelly: {:.2}%", raw_kelly * 100.0),
            format!("1/4分数Kelly: {:.2}%", fractional_kelly * 100.0),
        ];

        if is_limited {
            if final_position == 0.0 {
                reasons.push("Kelly信号不足(<2%),不建议建仓".to_string());
            } else if final_position >= 0.25 {
                reasons.push("触发单只股票上限(25%)".to_string());
            }
        }

        reasons.join("\n")
    }
}
```

## 组合级别Kelly

```rust
/// 组合Kelly优化
pub struct PortfolioKelly {
    /// 单只股票Kelly结果
    pub positions: Vec<KellyResult>,

    /// 总仓位
    pub total_exposure: f64,
}

impl PortfolioKelly {
    /// 为多只股票计算Kelly
    pub fn optimize_portfolio(stats: Vec<TradingStats>) -> Self {
        // 1. 计算每只股票的Kelly
        let mut kellys: Vec<f64> = stats.iter()
            .map(|s| {
                let result = KellyResult::from_trading_stats(s);
                result.kelly_fraction
            })
            .collect();

        // 2. 过滤低Kelly
        kellys.retain(|k| *k >= 0.02);

        // 3. 归一化
        let total: f64 = kellys.iter().sum();
        if total > 1.0 {
            kellys = kellys.iter().map(|k| k / total).collect();
        }

        // 4. 应用分数和限制
        let positions: Vec<KellyResult> = kellys.iter()
            .map(|k| KellyResult::from_kelly(*k))
            .collect();

        // 5. 计算总仓位
        let total_exposure: f64 = positions.iter()
            .map(|p| p.recommended_position)
            .sum();

        Self {
            positions,
            total_exposure,
        }
    }

    /// 生成组合报告
    pub fn generate_report(&self) -> String {
        let mut report = String::from("## Kelly组合优化\n\n");

        report.push_str(&format!("总仓位: {:.2}%\n\n", self.total_exposure * 100.0));

        for (i, pos) in self.positions.iter().enumerate() {
            report.push_str(&format!(
                "股票 #{}:\n\
                - 完整Kelly: {:.2}%\n\
                - 推荐仓位: {:.2}%\n\
                - 风险等级: {:?}\n\
                - 理由: {}\n\n",
                i + 1,
                pos.kelly_fraction * 100.0,
                pos.recommended_position * 100.0,
                pos.risk_level,
                pos.reasoning
            ));
        }

        report
    }
}
```

## 使用示例

### 单只股票分析

```rust
use investintel_agent::agents::KellyPositionAgent;
use investintel_agent::agents::{Agent, AgentInput};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = KellyPositionAgent::new();

    // 提供交易统计数据
    let input = AgentInput::new("AAPL")
        .with_context(serde_json::json!({
            "win_rate": 0.55,
            "avg_win": 120.0,
            "avg_loss": 80.0,
            "total_trades": 150
        }));

    let output = agent.execute(input).await?;
    println!("{}", output.content);

    Ok(())
}
```

### 组合优化

```rust
use investintel_agent::kelly::{PortfolioKelly, TradingStats};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 准备多只股票的统计数据
    let stats = vec![
        TradingStats {
            win_rate: 0.55,
            avg_win: 120.0,
            avg_loss: 80.0,
            total_trades: 150,
        },
        TradingStats {
            win_rate: 0.48,
            avg_win: 150.0,
            avg_loss: 100.0,
            total_trades: 200,
        },
        TradingStats {
            win_rate: 0.60,
            avg_win: 90.0,
            avg_loss: 70.0,
            total_trades: 180,
        },
    ];

    // 优化组合
    let portfolio = PortfolioKelly::optimize_portfolio(stats);
    println!("{}", portfolio.generate_report());

    Ok(())
}
```
