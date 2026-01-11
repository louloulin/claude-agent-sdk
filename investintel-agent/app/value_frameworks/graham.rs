//! Graham Framework - 深度价值投资
//!
//! 基于Benjamin Graham的价值投资法则实现
//! 包括Graham公式、Net-Net筛选器和安全边际计算

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Graham Formula - Graham公式
// ============================================================================

/// Graham公式计算器
///
/// V = EPS × (8.5 + 2g)
///
/// 其中:
/// - V = 内在价值
/// - EPS = 每股收益
/// - g = 预期增长率 (小数形式,如0.05表示5%)
/// - 8.5 = 基础倍数 (零增长公司的合理倍数)
/// - 2 = 增长倍数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrahamFormula {
    /// 基础倍数 (零增长公司)
    pub base_multiplier: f64,

    /// 增长倍数
    pub growth_multiplier: f64,
}

impl Default for GrahamFormula {
    fn default() -> Self {
        Self {
            base_multiplier: 8.5,
            growth_multiplier: 2.0,
        }
    }
}

impl GrahamFormula {
    /// 创建新的Graham公式计算器
    pub fn new() -> Self {
        Self::default()
    }

    /// 自定义参数创建
    pub fn with_params(base_multiplier: f64, growth_multiplier: f64) -> Self {
        Self {
            base_multiplier,
            growth_multiplier,
        }
    }

    /// 计算内在价值 (原始Graham公式)
    ///
    /// # 公式
    /// V = EPS × (8.5 + 2g)
    ///
    /// # 参数
    /// - `eps`: 每股收益 (EPS)
    /// - `growth_rate`: 预期增长率 (小数形式,如0.05表示5%)
    ///
    /// # 返回
    /// 内在价值
    ///
    /// # 示例
    /// ```
    /// use investintel_agent_app::value_frameworks::GrahamFormula;
    ///
    /// let formula = GrahamFormula::new();
    /// let intrinsic_value = formula.calculate(5.0, 0.08).unwrap();
    /// // V = 5.0 × (8.5 + 2×0.08) = 5.0 × 10.16 = 50.8
    /// ```
    pub fn calculate(&self, eps: f64, growth_rate: f64) -> Result<f64> {
        if eps < 0.0 {
            return Err(anyhow::anyhow!("EPS不能为负数"));
        }

        if growth_rate < 0.0 || growth_rate > 0.20 {
            return Err(anyhow::anyhow!("增长率应在0-20%之间"));
        }

        let multiplier = self.base_multiplier + self.growth_multiplier * growth_rate * 100.0;
        let intrinsic_value = eps * multiplier;

        Ok(intrinsic_value)
    }

    /// 计算现代化调整的内在价值
    ///
    /// 考虑利率环境的调整版本
    /// 当利率高于4.5%时,降低倍数
    ///
    /// # 参数
    /// - `eps`: 每股收益
    /// - `growth_rate`: 预期增长率
    /// - `risk_free_rate`: 无风险利率 (小数形式)
    ///
    /// # 示例
    /// ```
    /// # use investintel_agent_app::value_frameworks::GrahamFormula;
    /// let formula = GrahamFormula::new();
    /// let value = formula.calculate_adjusted(5.0, 0.08, 0.045).unwrap();
    /// ```
    pub fn calculate_adjusted(
        &self,
        eps: f64,
        growth_rate: f64,
        risk_free_rate: f64,
    ) -> Result<f64> {
        if eps < 0.0 {
            return Err(anyhow::anyhow!("EPS不能为负数"));
        }

        if growth_rate < 0.0 || growth_rate > 0.20 {
            return Err(anyhow::anyhow!("增长率应在0-20%之间"));
        }

        // 当利率高于4.5%时,降低倍数
        let rate_adjustment = if risk_free_rate > 0.045 {
            (0.045 - risk_free_rate) / 0.045
        } else {
            1.0
        };

        let multiplier = (self.base_multiplier + self.growth_multiplier * growth_rate * 100.0)
            * rate_adjustment;

        let intrinsic_value = eps * multiplier;

        Ok(intrinsic_value)
    }

    /// 计算安全边际
    ///
    /// # 公式
    /// 安全边际 = (内在价值 - 当前价格) / 内在价值
    ///
    /// # 参数
    /// - `intrinsic_value`: 内在价值
    /// - `current_price`: 当前价格
    ///
    /// # 返回
    /// 安全边际 (0-1之间的小数)
    pub fn margin_of_safety(&self, intrinsic_value: f64, current_price: f64) -> Result<f64> {
        if intrinsic_value <= 0.0 {
            return Err(anyhow::anyhow!("内在价值必须大于0"));
        }

        if current_price <= 0.0 {
            return Err(anyhow::anyhow!("当前价格必须大于0"));
        }

        let margin = (intrinsic_value - current_price) / intrinsic_value;

        Ok(margin.max(0.0))
    }

    /// 评估投资吸引力
    ///
    /// 根据安全边际评估投资吸引力
    ///
    /// # 返回
    /// - "深度价值买入" - 安全边际 > 40%
    /// - "价值买入" - 安全边际 > 30%
    /// - "谨慎买入" - 安全边际 > 20%
    /// - "观察" - 安全边际 > 10%
    /// - "避免" - 安全边际 ≤ 10%
    pub fn evaluate attractiveness(&self, margin_of_safety: f64) -> &'static str {
        if margin_of_safety > 0.40 {
            "深度价值买入"
        } else if margin_of_safety > 0.30 {
            "价值买入"
        } else if margin_of_safety > 0.20 {
            "谨慎买入"
        } else if margin_of_safety > 0.10 {
            "观察"
        } else {
            "避免"
        }
    }
}

// ============================================================================
// Net-Net Screener - Net-Net筛选器
// ============================================================================

/// Net-Net筛选器
///
/// 实现Graham最保守的Net-Net策略
/// Net-Net = (流动资产 - 总负债) / 股数
///
/// 只有当价格低于Net-Net值的66%时才考虑买入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetNetScreener;

impl NetNetScreener {
    /// 创建新的Net-Net筛选器
    pub fn new() -> Self {
        Self
    }

    /// 计算NCAV (Net Current Asset Value)
    ///
    /// NCAV = (流动资产 - 总负债) / 股数
    ///
    /// # 参数
    /// - `current_assets`: 流动资产
    /// - `total_liabilities`: 总负债
    /// - `shares_outstanding`: 流通股数
    ///
    /// # 返回
    /// 每股NCAV值
    pub fn calculate_ncav(
        &self,
        current_assets: f64,
        total_liabilities: f64,
        shares_outstanding: f64,
    ) -> Result<f64> {
        if current_assets < 0.0 {
            return Err(anyhow::anyhow!("流动资产不能为负数"));
        }

        if total_liabilities < 0.0 {
            return Err(anyhow::anyhow!("总负债不能为负数"));
        }

        if shares_outstanding <= 0.0 {
            return Err(anyhow::anyhow!("流通股数必须大于0"));
        }

        let net_current_assets = current_assets - total_liabilities;

        if net_current_assets <= 0.0 {
            return Err(anyhow::anyhow!("净流动资产必须大于0"));
        }

        let ncav_per_share = net_current_assets / shares_outstanding;

        Ok(ncav_per_share)
    }

    /// 计算NNWC (Net Net Working Capital)
    ///
    /// NNWC是对NCAV的更保守版本
    /// NNWC = (现金 + 应收账款×0.75 + 存货×0.5 - 总负债) / 股数
    ///
    /// # 参数
    /// - `cash`: 现金及现金等价物
    /// - `receivables`: 应收账款
    /// - `inventory`: 存货
    /// - `total_liabilities`: 总负债
    /// - `shares_outstanding`: 流通股数
    ///
    /// # 返回
    /// 每股NNWC值
    pub fn calculate_nnwc(
        &self,
        cash: f64,
        receivables: f64,
        inventory: f64,
        total_liabilities: f64,
        shares_outstanding: f64,
    ) -> Result<f64> {
        if cash < 0.0 || receivables < 0.0 || inventory < 0.0 {
            return Err(anyhow::anyhow!("资产项目不能为负数"));
        }

        if total_liabilities < 0.0 {
            return Err(anyhow::anyhow!("总负债不能为负数"));
        }

        if shares_outstanding <= 0.0 {
            return Err(anyhow::anyhow!("流通股数必须大于0"));
        }

        // NNWC计算:现金按100%,应收账款按75%,存货按50%
        let adjusted_current_assets =
            cash + (receivables * 0.75) + (inventory * 0.50);

        let net_working_capital = adjusted_current_assets - total_liabilities;

        if net_working_capital <= 0.0 {
            return Err(anyhow::anyhow!("净营运资本必须大于0"));
        }

        let nnwc_per_share = net_working_capital / shares_outstanding;

        Ok(nnwc_per_share)
    }

    /// 检查是否为Net-Net机会
    ///
    /// Graham的Net-Net策略要求:
    /// 价格 ≤ Net-Net值的66%
    ///
    /// # 参数
    /// - `current_price`: 当前价格
    /// - `ncav_value`: Net-Net值 (NCAV或NNWC)
    ///
    /// # 返回
    /// - true: 是Net-Net机会
    /// - false: 不是Net-Net机会
    pub fn is_net_net_opportunity(&self, current_price: f64, ncav_value: f64) -> bool {
        if current_price <= 0.0 || ncav_value <= 0.0 {
            return false;
        }

        // Graham要求价格 ≤ Net-Net的66%
        let threshold = ncav_value * 0.66;
        current_price <= threshold
    }

    /// 计算Net-Net折扣率
    ///
    /// # 参数
    /// - `current_price`: 当前价格
    /// - `ncav_value`: Net-Net值
    ///
    /// # 返回
    /// 折扣率 (0-1之间的小数)
    pub fn discount_to_netnet(&self, current_price: f64, ncav_value: f64) -> Result<f64> {
        if current_price <= 0.0 || ncav_value <= 0.0 {
            return Err(anyhow::anyhow!("价格和Net-Net值必须大于0"));
        }

        let discount = 1.0 - (current_price / ncav_value);
        Ok(discount.max(0.0))
    }
}

// ============================================================================
// Graham Analysis - Graham分析结果
// ============================================================================

/// Graham分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrahamAnalysis {
    /// 股票代码
    pub symbol: String,

    /// 每股收益
    pub eps: f64,

    /// 预期增长率
    pub growth_rate: f64,

    /// Graham公式计算的内在价值
    pub intrinsic_value: f64,

    /// 当前价格
    pub current_price: f64,

    /// 安全边际
    pub margin_of_safety: f64,

    /// Net-Net值 (如果适用)
    pub net_net_value: Option<f64>,

    /// 是否为Net-Net机会
    pub is_net_net_opportunity: bool,

    /// 投资建议
    pub recommendation: String,

    /// 是否符合Graham标准
    pub meets_graham_criteria: bool,

    /// 综合评分 (0-100)
    pub score: f64,
}

impl GrahamAnalysis {
    /// 创建Graham分析结果
    pub fn new(
        symbol: String,
        eps: f64,
        growth_rate: f64,
        intrinsic_value: f64,
        current_price: f64,
        margin_of_safety: f64,
    ) -> Self {
        // 计算综合评分
        let score = Self::calculate_score(margin_of_safety, eps, growth_rate);

        // 判断是否符合Graham标准
        let meets_graham_criteria = margin_of_safety >= 0.30 && eps > 0.0;

        // 生成建议
        let recommendation = if margin_of_safety > 0.40 {
            "强烈买入 - 深度价值机会".to_string()
        } else if margin_of_safety > 0.30 {
            "买入 - 符合Graham安全边际标准".to_string()
        } else if margin_of_safety > 0.20 {
            "考虑买入 - 适度安全边际".to_string()
        } else if margin_of_safety > 0.10 {
            "观察 - 安全边际不足".to_string()
        } else {
            "避免 - 无安全边际".to_string()
        };

        Self {
            symbol,
            eps,
            growth_rate,
            intrinsic_value,
            current_price,
            margin_of_safety,
            net_net_value: None,
            is_net_net_opportunity: false,
            recommendation,
            meets_graham_criteria,
            score,
        }
    }

    /// 计算Graham评分
    ///
    /// 综合考虑安全边际、盈利能力和增长潜力
    fn calculate_score(margin_of_safety: f64, eps: f64, growth_rate: f64) -> f64 {
        let margin_score = (margin_of_safety * 100.0).min(50.0);

        let eps_score = if eps > 5.0 {
            20.0
        } else if eps > 2.0 {
            15.0
        } else if eps > 0.0 {
            10.0
        } else {
            0.0
        };

        let growth_score = (growth_rate * 100.0).min(30.0);

        margin_score + eps_score + growth_score
    }
}

// ============================================================================
// Graham Framework - Graham框架整合
// ============================================================================

/// Graham框架
///
/// 整合Graham公式、Net-Net筛选和安全边际分析的完整框架
#[derive(Debug, Clone)]
pub struct GrahamFramework {
    /// Graham公式计算器
    pub formula: GrahamFormula,

    /// Net-Net筛选器
    pub netnet_screener: NetNetScreener,

    /// 安全边际要求 (默认30%)
    pub margin_requirement: f64,
}

impl Default for GrahamFramework {
    fn default() -> Self {
        Self {
            formula: GrahamFormula::default(),
            netnet_screener: NetNetScreener::new(),
            margin_requirement: 0.30,
        }
    }
}

impl GrahamFramework {
    /// 创建新的Graham框架
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置安全边际要求
    pub fn with_margin_requirement(mut self, margin: f64) -> Self {
        self.margin_requirement = margin;
        self
    }

    /// 使用自定义公式
    pub fn with_formula(mut self, formula: GrahamFormula) -> Self {
        self.formula = formula;
        self
    }

    /// 执行完整的Graham分析
    ///
    /// # 参数
    /// - `symbol`: 股票代码
    /// - `eps`: 每股收益
    /// - `growth_rate`: 预期增长率
    /// - `current_price`: 当前价格
    /// - `current_assets`: 流动资产 (可选,用于Net-Net计算)
    /// - `total_liabilities`: 总负债 (可选,用于Net-Net计算)
    /// - `shares_outstanding`: 流通股数 (可选,用于Net-Net计算)
    ///
    /// # 返回
    /// Graham分析结果
    pub async fn analyze(
        &self,
        symbol: String,
        eps: f64,
        growth_rate: f64,
        current_price: f64,
        current_assets: Option<f64>,
        total_liabilities: Option<f64>,
        shares_outstanding: Option<f64>,
    ) -> Result<GrahamAnalysis> {
        // 1. 使用Graham公式计算内在价值
        let intrinsic_value = self.formula.calculate(eps, growth_rate)?;

        // 2. 计算安全边际
        let margin_of_safety =
            self.formula.margin_of_safety(intrinsic_value, current_price)?;

        // 3. 创建基础分析结果
        let mut analysis = GrahamAnalysis::new(
            symbol,
            eps,
            growth_rate,
            intrinsic_value,
            current_price,
            margin_of_safety,
        );

        // 4. 如果提供了Net-Net数据,计算Net-Net值
        if let (Some(ca), Some(tl), Some(shares)) =
            (current_assets, total_liabilities, shares_outstanding)
        {
            if let Ok(ncav) = self.netnet_screener.calculate_ncav(ca, tl, shares) {
                analysis.net_net_value = Some(ncav);
                analysis.is_net_net_opportunity =
                    self.netnet_screener.is_net_net_opportunity(current_price, ncav);
            }
        }

        // 5. 更新评分和建议 (考虑Net-Net)
        if analysis.is_net_net_opportunity {
            analysis.score += 10.0; // Net-Net机会额外加分
            analysis.recommendation =
                format!("{} (Net-Net机会)", analysis.recommendation);
        }

        Ok(analysis)
    }

    /// 批量分析多只股票
    ///
    /// # 参数
    /// - `stocks`: 股票数据列表
    ///
    /// # 返回
    /// 分析结果列表,按评分排序
    pub async fn batch_analyze(
        &self,
        stocks: Vec<GrahamStockData>,
    ) -> Result<Vec<GrahamAnalysis>> {
        let mut results = Vec::new();

        for stock in stocks {
            let analysis = self
                .analyze(
                    stock.symbol,
                    stock.eps,
                    stock.growth_rate,
                    stock.current_price,
                    stock.current_assets,
                    stock.total_liabilities,
                    stock.shares_outstanding,
                )
                .await?;

            results.push(analysis);
        }

        // 按评分降序排序
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        Ok(results)
    }

    /// 筛选符合Graham标准的股票
    ///
    /// # 参数
    /// - `analyses`: Graham分析结果列表
    ///
    /// # 返回
    /// 符合标准的股票列表
    pub fn filter_graham_stocks(
        &self,
        analyses: &[GrahamAnalysis],
    ) -> Vec<&GrahamAnalysis> {
        analyses
            .iter()
            .filter(|a| a.meets_graham_criteria)
            .collect()
    }
}

/// Graham股票数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrahamStockData {
    pub symbol: String,
    pub eps: f64,
    pub growth_rate: f64,
    pub current_price: f64,
    pub current_assets: Option<f64>,
    pub total_liabilities: Option<f64>,
    pub shares_outstanding: Option<f64>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graham_formula() {
        let formula = GrahamFormula::new();

        // 测试基本计算
        let result = formula.calculate(5.0, 0.08).unwrap();
        // V = 5.0 × (8.5 + 2×0.08) = 5.0 × 10.16 = 50.8
        assert!((result - 50.8).abs() < 0.1);

        // 测试安全边际计算
        let margin = formula.margin_of_safety(50.0, 35.0).unwrap();
        // (50 - 35) / 50 = 0.30 = 30%
        assert!((margin - 0.30).abs() < 0.01);
    }

    #[test]
    fn test_netnet_screener() {
        let screener = NetNetScreener::new();

        // 测试NCAV计算
        let ncav = screener
            .calculate_ncav(100_000_000.0, 30_000_000.0, 10_000_000.0)
            .unwrap();
        // (100M - 30M) / 10M = 7.0
        assert!((ncav - 7.0).abs() < 0.1);

        // 测试Net-Net机会判断
        let is_opportunity = screener.is_net_net_opportunity(4.0, 7.0);
        // 4.0 ≤ 7.0 × 0.66 = 4.62, 所以是机会
        assert!(is_opportunity);
    }

    #[test]
    fn test_graham_framework() {
        let framework = GrahamFramework::new();

        // 测试完整分析
        let analysis = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                framework
                    .analyze(
                        "TEST".to_string(),
                        5.0,
                        0.08,
                        35.0,
                        Some(100_000_000.0),
                        Some(30_000_000.0),
                        Some(10_000_000.0),
                    )
                    .await
            })
            .unwrap();

        assert_eq!(analysis.symbol, "TEST");
        assert!(analysis.intrinsic_value > 0.0);
        assert!(analysis.margin_of_safety > 0.0);
        assert!(analysis.score > 0.0);
    }

    #[test]
    fn test_graham_scoring() {
        let framework = GrahamFramework::new();

        let analysis = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                framework
                    .analyze("HIGH".to_string(), 8.0, 0.12, 30.0, None, None, None)
                    .await
            })
            .unwrap();

        // 高EPS、高增长、高安全边际应该得到高分
        assert!(analysis.score > 60.0);
        assert!(analysis.meets_graham_criteria);
    }
}
