# Graham Value Investing - Rust实现参考

## 核心数据结构

```rust
use serde::{Deserialize, Serialize};

/// Graham分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrahamAnalysis {
    /// 股票代码
    pub symbol: String,

    /// 内在价值
    pub intrinsic_value: f64,

    /// 当前价格
    pub current_price: f64,

    /// 安全边际
    pub margin_of_safety: f64,

    /// 每股收益
    pub eps: f64,

    /// 预期增长率
    pub growth_rate: f64,

    /// Graham评分
    pub graham_score: u8,

    /// Buffett加分
    pub buffett_bonus: u8,

    /// 总分
    pub total_score: u8,

    /// 投资建议
    pub recommendation: InvestmentAction,

    /// 详细分析
    pub details: AnalysisDetails,
}

/// 分析详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisDetails {
    /// 估值分析
    pub valuation: ValuationAnalysis,

    /// 盈利质量
    pub earnings_quality: EarningsQuality,

    /// 财务健康
    pub financial_health: FinancialHealth,

    /// Buffett质量指标
    pub buffett_quality: Option<BuffettQuality>,
}

/// 估值分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationAnalysis {
    /// Graham估值
    pub graham_valuation: f64,

    /// DCF估值
    pub dcf_valuation: Option<f64>,

    /// 估值折扣
    pub discount: f64,

    /// 估值得分
    pub score: u8,
}

/// 盈利质量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsQuality {
    /// EPS增长率
    pub eps_growth: f64,

    /// 增长稳定性
    pub stability: bool,

    /// 得分
    pub score: u8,
}

/// 财务健康
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialHealth {
    /// 负债率
    pub debt_ratio: f64,

    /// 流动比率
    pub current_ratio: f64,

    /// 利息保障倍数
    pub interest_coverage: Option<f64>,

    /// 得分
    pub score: u8,
}

/// Buffett质量指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffettQuality {
    /// ROIC
    pub roic: f64,

    /// ROE
    pub roe: f64,

    /// 护城河评分
    pub moat_score: u8,

    /// 总加分
    pub total_bonus: u8,
}
```

## Graham计算实现

```rust
impl GrahamAnalysis {
    /// 计算Graham内在价值
    pub fn calculate_intrinsic_value(eps: f64, growth_rate: f64) -> f64 {
        eps * (8.5 + 2.0 * growth_rate)
    }

    /// 计算安全边际
    pub fn calculate_margin_of_safety(
        intrinsic_value: f64,
        current_price: f64,
    ) -> f64 {
        (intrinsic_value - current_price) / intrinsic_value
    }

    /// 计算CAGR (复合年均增长率)
    pub fn calculate_cagr(values: &[f64], years: usize) -> f64 {
        if values.len() < 2 || years == 0 {
            return 0.0;
        }

        let start_value = values.first().unwrap();
        let end_value = values.last().unwrap();

        if *start_value <= 0.0 || *end_value <= 0.0 {
            return 0.0;
        }

        (end_value / start_value).powf(1.0 / years as f64) - 1.0
    }

    /// 计算估值得分
    pub fn calculate_valuation_score(margin_of_safety: f64) -> u8 {
        match margin_of_safety {
            m if m >= 0.50 => 20,
            m if m >= 0.40 => 16,
            m if m >= 0.30 => 12,
            m if m >= 0.20 => 8,
            m if m >= 0.10 => 4,
            _ => 0,
        }
    }

    /// 计算盈利质量得分
    pub fn calculate_quality_score(
        eps_growth: f64,
        is_stable: bool,
    ) -> u8 {
        if is_stable && eps_growth > 0.10 {
            10
        } else if eps_growth > 0.05 {
            8
        } else if eps_growth > 0.0 {
            6
        } else if eps_growth > -0.05 {
            3
        } else {
            0
        }
    }

    /// 计算财务健康得分
    pub fn calculate_health_score(
        debt_ratio: f64,
        current_ratio: f64,
    ) -> u8 {
        match (debt_ratio, current_ratio) {
            (dr, cr) if dr < 0.30 && cr > 2.0 => 10,
            (dr, cr) if dr < 0.50 && cr > 1.5 => 7,
            (dr, cr) if dr < 0.70 && cr > 1.0 => 4,
            _ => 0,
        }
    }
}
```

## Buffett质量加分实现

```rust
impl BuffettQuality {
    /// 计算ROIC得分
    pub fn calculate_roic_bonus(roic: f64) -> u8 {
        match roic {
            r if r > 0.20 => 2,
            r if r > 0.15 => 2,
            r if r > 0.10 => 1,
            _ => 0,
        }
    }

    /// 计算ROE得分
    pub fn calculate_roe_bonus(roe: f64) -> u8 {
        match roe {
            r if r > 0.20 => 2,
            r if r > 0.15 => 2,
            r if r > 0.12 => 1,
            _ => 0,
        }
    }

    /// 计算护城河得分
    pub fn calculate_moat_bonus(
        has_brand: bool,
        has_cost_advantage: bool,
        has_network_effect: bool,
    ) -> u8 {
        let mut score = 0;
        if has_brand { score += 1; }
        if has_cost_advantage { score += 1; }
        if has_network_effect { score += 1; }
        score
    }

    /// 计算总Buffett加分
    pub fn calculate_total_bonus(&self) -> u8 {
        self.calculate_roic_bonus(self.roic)
            + self.calculate_roe_bonus(self.roe)
            + self.moat_score
    }
}
```

## 完整分析流程

```rust
use anyhow::Result;

pub async fn analyze_graham(
    symbol: &str,
    market_data: &MarketDataProvider,
) -> Result<GrahamAnalysis> {
    // 1. 获取市场数据
    let quote = market_data.get_quote(symbol).await?;
    let fundamental = market_data.get_fundamental(symbol).await?;

    // 2. 计算内在价值
    let eps = fundamental.eps.unwrap_or(0.0);
    let growth_rate = fundamental.earnings_growth.unwrap_or(0.05);
    let intrinsic_value = GrahamAnalysis::calculate_intrinsic_value(eps, growth_rate);

    // 3. 计算安全边际
    let current_price = quote.current_price;
    let margin_of_safety = GrahamAnalysis::calculate_margin_of_safety(
        intrinsic_value,
        current_price,
    );

    // 4. 计算估值得分
    let valuation_score = GrahamAnalysis::calculate_valuation_score(margin_of_safety);

    // 5. 计算盈利质量得分
    let eps_growth = growth_rate;
    let quality_score = GrahamAnalysis::calculate_quality_score(eps_growth, true);

    // 6. 计算财务健康得分
    let debt_ratio = fundamental.debt_ratio.unwrap_or(0.5);
    let current_ratio = fundamental.current_ratio.unwrap_or(1.5);
    let health_score = GrahamAnalysis::calculate_health_score(debt_ratio, current_ratio);

    // 7. Graham基础分
    let graham_score = valuation_score + quality_score + health_score;

    // 8. Buffett加分
    let roic = fundamental.roic.unwrap_or(0.0);
    let roe = fundamental.roe.unwrap_or(0.0);
    let buffett_bonus = BuffettQuality::calculate_roic_bonus(roic)
        + BuffettQuality::calculate_roe_bonus(roe);

    // 9. 总分
    let total_score = graham_score + buffett_bonus;

    // 10. 投资建议
    let recommendation = if margin_of_safety >= 0.50 {
        InvestmentAction::StrongBuy
    } else if margin_of_safety >= 0.30 {
        InvestmentAction::Buy
    } else if margin_of_safety >= 0.15 {
        InvestmentAction::Hold
    } else if margin_of_safety >= 0.0 {
        InvestmentAction::Sell
    } else {
        InvestmentAction::StrongSell
    };

    Ok(GrahamAnalysis {
        symbol: symbol.to_string(),
        intrinsic_value,
        current_price,
        margin_of_safety,
        eps,
        growth_rate,
        graham_score,
        buffett_bonus,
        total_score,
        recommendation,
        details: AnalysisDetails {
            valuation: ValuationAnalysis {
                graham_valuation: intrinsic_value,
                dcf_valuation: None, // 可选DCF计算
                discount: margin_of_safety,
                score: valuation_score,
            },
            earnings_quality: EarningsQuality {
                eps_growth,
                stability: true,
                score: quality_score,
            },
            financial_health: FinancialHealth {
                debt_ratio,
                current_ratio,
                interest_coverage: None,
                score: health_score,
            },
            buffett_quality: if roic > 0.0 || roe > 0.0 {
                Some(BuffettQuality {
                    roic,
                    roe,
                    moat_score: 0,
                    total_bonus: buffett_bonus,
                })
            } else {
                None
            },
        },
    })
}
```

## 使用示例

### 单股分析

```rust
use investintel_agent::agents::ValueInvestmentAgent;
use investintel_agent::agents::{Agent, AgentInput};

#[tokio::main]
async fn main() -> Result<()> {
    let agent = ValueInvestmentAgent::new();
    let input = AgentInput::new("AAPL");
    let output = agent.execute(input).await?;

    println!("{}", output.content);

    Ok(())
}
```

### 批量分析

```rust
use investintel_agent::orchestration::InvestmentOrchestrator;
use investintel_agent::orchestration::{AnalysisType, OrchestrationConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let symbols = vec!["AAPL", "MSFT", "GOOGL", "JNJ", "KO"];
    let orchestrator = InvestmentOrchestrator::new();

    for symbol in symbols {
        let result = orchestrator.analyze(
            symbol,
            AnalysisType::QuickValue,
            OrchestrationConfig::default(),
        ).await?;

        println!("{}: {} (Score: {}/47)",
            symbol,
            result.recommendation,
            result.confidence * 47.0
        );
    }

    Ok(())
}
```
