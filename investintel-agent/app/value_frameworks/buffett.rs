//! Buffett Framework - 质量价值投资
//!
//! 基于Warren Buffett的质量价值投资法则实现
//! 包括护城河分析、管理层评估和DCF估值模型

use anyhow::Result;
use serde::{Deserialize, Serialize};

// ============================================================================
// Moat Score - 护城河评分
// ============================================================================

/// 护城河评分
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MoatScore {
    /// 无护城河
    None = 0,
    /// 窄护城河
    Narrow = 1,
    /// 宽护城河
    Wide = 2,
    /// 非常宽的护城河
    VeryWide = 3,
}

impl MoatScore {
    /// 获取数值
    pub fn as_value(&self) -> u8 {
        *self as u8
    }

    /// 从数值创建
    pub fn from_value(value: u8) -> Self {
        match value {
            0 => MoatScore::None,
            1 => MoatScore::Narrow,
            2 => MoatScore::Wide,
            _ => MoatScore::VeryWide,
        }
    }

    /// 获取描述
    pub fn description(&self) -> &'static str {
        match self {
            MoatScore::None => "无护城河",
            MoatScore::Narrow => "窄护城河",
            MoatScore::Wide => "宽护城河",
            MoatScore::VeryWide => "非常宽的护城河",
        }
    }
}

// ============================================================================
// Moat Analyzer - 护城河分析器
// ============================================================================

/// 护城河分析器
///
/// 评估企业的竞争优势和护城河深度
/// 基于Buffett的四大护城河来源:
/// 1. 品牌价值
/// 2. 成本优势
/// 3. 转换成本
/// 4. 网络效应 (最强大)
#[derive(Debug, Clone)]
pub struct MoatAnalyzer {
    /// ROIC最低要求 (默认10%)
    pub min_roic: f64,
}

impl Default for MoatAnalyzer {
    fn default() -> Self {
        Self { min_roic: 0.10 }
    }
}

impl MoatAnalyzer {
    /// 创建新的护城河分析器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置ROIC最低要求
    pub fn with_min_roic(mut self, min_roic: f64) -> Self {
        self.min_roic = min_roic;
        self
    }

    /// 评估护城河
    ///
    /// 综合评估企业的各种竞争优势
    ///
    /// # 参数
    /// - `has_strong_brand`: 是否有强大品牌
    /// - `has_cost_advantage`: 是否有成本优势
    /// - `has_high_switching_cost`: 是否有高转换成本
    /// - `has_network_effects`: 是否有网络效应
    /// - `roic`: 投资资本回报率
    ///
    /// # 返回
    /// 护城河评分
    pub fn evaluate_moat(
        &self,
        has_strong_brand: bool,
        has_cost_advantage: bool,
        has_high_switching_cost: bool,
        has_network_effects: bool,
        roic: f64,
    ) -> MoatScore {
        // Buffett最低要求: ROIC > 10%
        if roic < self.min_roic {
            return MoatScore::None;
        }

        let mut score = 0;

        // 1. 品牌价值 (+1分)
        if has_strong_brand {
            score += 1;
        }

        // 2. 成本优势 (+1分)
        if has_cost_advantage {
            score += 1;
        }

        // 3. 转换成本 (+1分)
        if has_high_switching_cost {
            score += 1;
        }

        // 4. 网络效应 (+2分,最强大)
        if has_network_effects {
            score += 2;
        }

        // ROIC额外加成
        if roic > 0.20 {
            score += 1;
        } else if roic > 0.15 {
            score += 0;
        }

        match score {
            0..=1 => MoatScore::None,
            2 => MoatScore::Narrow,
            3..=4 => MoatScore::Wide,
            _ => MoatScore::VeryWide,
        }
    }

    /// 分析护城河可持续性
    ///
    /// 评估护城河是否能够长期维持
    ///
    /// # 参数
    /// - `moat_score`: 当前护城河评分
    /// - `industry_trend`: 行业趋势 (1=增长, 0=稳定, -1=衰退)
    /// - `competitive_pressure`: 竞争压力 (1=低, 2=中, 3=高)
    ///
    /// # 返回
    /// 可持续性评分 (0-100)
    pub fn assess_sustainability(
        &self,
        moat_score: MoatScore,
        industry_trend: i8,
        competitive_pressure: u8,
    ) -> f64 {
        let base_score = match moat_score {
            MoatScore::None => 0.0,
            MoatScore::Narrow => 40.0,
            MoatScore::Wide => 70.0,
            MoatScore::VeryWide => 90.0,
        };

        // 行业趋势调整
        let trend_adjustment = match industry_trend {
            1 => 10.0,  // 增长行业
            0 => 0.0,   // 稳定行业
            -1 => -20.0, // 衰退行业
            _ => 0.0,
        };

        // 竞争压力调整
        let pressure_adjustment = match competitive_pressure {
            1 => 10.0,  // 低压力
            2 => 0.0,   // 中等压力
            3 => -15.0, // 高压力
            _ => 0.0,
        };

        let sustainability = base_score + trend_adjustment + pressure_adjustment;
        sustainability.max(0.0).min(100.0)
    }
}

// ============================================================================
// Management Score - 管理层评分
// ============================================================================

/// 管理层评分
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ManagementScore {
    /// 差
    Poor = 0,
    /// 一般
    Average = 1,
    /// 良好
    Good = 2,
    /// 优秀
    Excellent = 3,
}

impl ManagementScore {
    /// 获取数值
    pub fn as_value(&self) -> u8 {
        *self as u8
    }

    /// 获取描述
    pub fn description(&self) -> &'static str {
        match self {
            ManagementScore::Poor => "差",
            ManagementScore::Average => "一般",
            ManagementScore::Good => "良好",
            ManagementScore::Excellent => "优秀",
        }
    }
}

// ============================================================================
// Management Evaluator - 管理层评估器
// ============================================================================

/// 管理层评估器
///
/// 基于Buff菲特的管理层评估标准:
/// 1. 资本配置历史
/// 2. 股东回报记录
/// 3. 透明度和诚信
#[derive(Debug, Clone)]
pub struct ManagementEvaluator;

impl ManagementEvaluator {
    /// 创建新的管理层评估器
    pub fn new() -> Self {
        Self
    }

    /// 评估管理层质量
    ///
    /// # 参数
    /// - `capital_allocation_score`: 资本配置评分 (0-100)
    /// - `shareholder_returns`: 股东回报率 (过去5年)
    /// - `transparency_score`: 透明度评分 (0-100)
    /// - `insider_ownership_ratio`: 内部人持股比例
    ///
    /// # 返回
    /// 管理层评分
    pub fn evaluate_management(
        &self,
        capital_allocation_score: f64,
        shareholder_returns: f64,
        transparency_score: f64,
        insider_ownership_ratio: f64,
    ) -> ManagementScore {
        let mut total_score = 0.0;

        // 1. 资本配置历史 (权重40%)
        let allocation_weight = 0.4;
        let allocation_score = (capital_allocation_score / 100.0) * 40.0;
        total_score += allocation_score;

        // 2. 股东回报记录 (权重30%)
        let returns_weight = 0.3;
        let returns_score = if shareholder_returns > 0.20 {
            30.0
        } else if shareholder_returns > 0.10 {
            20.0
        } else if shareholder_returns > 0.0 {
            10.0
        } else {
            0.0
        };
        total_score += returns_score;

        // 3. 透明度和诚信 (权重20%)
        let transparency_weight = 0.2;
        let transparency_score = (transparency_score / 100.0) * 20.0;
        total_score += transparency_score;

        // 4. 内部人持股 (权重10%)
        let ownership_weight = 0.1;
        let ownership_score = if insider_ownership_ratio > 0.10 {
            10.0
        } else if insider_ownership_ratio > 0.05 {
            5.0
        } else {
            0.0
        };
        total_score += ownership_score;

        // 综合评分转换为等级
        match total_score {
            score if score >= 80.0 => ManagementScore::Excellent,
            score if score >= 60.0 => ManagementScore::Good,
            score if score >= 40.0 => ManagementScore::Average,
            _ => ManagementScore::Poor,
        }
    }

    /// 评估资本配置历史
    ///
    /// 分析企业在以下方面的资本配置决策:
    /// - 内部增长投资
    /// - 收购
    /// - 分红和股票回购
    ///
    /// # 参数
    /// - `roi_on_invested_capital`: 投资资本回报率
    /// - `dividend_growth_rate`: 分红增长率
    /// - `buyback_effectiveness`: 回购有效性
    ///
    /// # 返回
    /// 资本配置评分 (0-100)
    pub fn evaluate_capital_allocation(
        &self,
        roi_on_invested_capital: f64,
        dividend_growth_rate: f64,
        buyback_effectiveness: f64,
    ) -> f64 {
        let mut score = 0.0;

        // ROIC评分 (权重50%)
        let roic_score = if roi_on_invested_capital > 0.15 {
            50.0
        } else if roi_on_invested_capital > 0.10 {
            35.0
        } else if roi_on_invested_capital > 0.05 {
            20.0
        } else {
            0.0
        };
        score += roic_score;

        // 分红增长评分 (权重25%)
        let dividend_score = if dividend_growth_rate > 0.10 {
            25.0
        } else if dividend_growth_rate > 0.05 {
            15.0
        } else if dividend_growth_rate > 0.0 {
            5.0
        } else {
            0.0
        };
        score += dividend_score;

        // 回购有效性评分 (权重25%)
        let buyback_score = (buyback_effectiveness.min(1.0) * 25.0);
        score += buyback_score;

        score
    }
}

// ============================================================================
// DCF Calculator - DCF估值计算器
// ============================================================================

/// DCF估值计算器
///
/// 实现Buffett使用的自由现金流折现模型
#[derive(Debug, Clone)]
pub struct DcfCalculator {
    /// 预测年数 (默认10年)
    pub projection_years: usize,

    /// 终值增长率 (默认2.5%)
    pub terminal_growth_rate: f64,
}

impl Default for DcfCalculator {
    fn default() -> Self {
        Self {
            projection_years: 10,
            terminal_growth_rate: 0.025,
        }
    }
}

impl DcfCalculator {
    /// 创建新的DCF计算器
    pub fn new() -> Self {
        Self::default()
    }

    /// 自定义参数
    pub fn with_params(projection_years: usize, terminal_growth_rate: f64) -> Self {
        Self {
            projection_years,
            terminal_growth_rate,
        }
    }

    /// 计算企业内在价值
    ///
    /// # 参数
    /// - `free_cash_flow`: 当前自由现金流
    /// - `growth_rate`: 预期增长率 (前N年)
    /// - `discount_rate`: 折现率 (WACC)
    ///
    /// # 返回
    /// 企业内在价值
    ///
    /// # 公式
    /// 1. 预测未来N年现金流
    /// 2. 计算终值
    /// 3. 折现到现值
    /// 4. 加总求和
    pub fn calculate_intrinsic_value(
        &self,
        free_cash_flow: f64,
        growth_rate: f64,
        discount_rate: f64,
    ) -> Result<f64> {
        if free_cash_flow <= 0.0 {
            return Err(anyhow::anyhow!("自由现金流必须大于0"));
        }

        if discount_rate <= self.terminal_growth_rate {
            return Err(anyhow::anyhow!(
                "折现率必须大于终值增长率 ({} > {})",
                discount_rate,
                self.terminal_growth_rate
            ));
        }

        let mut present_value = 0.0;

        // 1. 计算预测期内现金流的现值
        for year in 1..=self.projection_years {
            let projected_fcf = free_cash_flow * (1.0 + growth_rate).powi(year as i32);
            let pv = projected_fcf / (1.0 + discount_rate).powi(year as i32);
            present_value += pv;
        }

        // 2. 计算终值
        let terminal_fcf = free_cash_flow
            * (1.0 + growth_rate).powi(self.projection_years as i32)
            * (1.0 + self.terminal_growth_rate);
        let terminal_value =
            terminal_fcf / (discount_rate - self.terminal_growth_rate);

        // 3. 终值的现值
        let pv_terminal = terminal_value / (1.0 + discount_rate).powi(self.projection_years as i32);

        present_value += pv_terminal;

        Ok(present_value)
    }

    /// 计算每股内在价值
    ///
    /// # 参数
    /// - `enterprise_value`: 企业价值
    /// - `net_debt`: 净债务
    /// - `shares_outstanding`: 流通股数
    ///
    /// # 返回
    /// 每股内在价值
    pub fn calculate_per_share_value(
        &self,
        enterprise_value: f64,
        net_debt: f64,
        shares_outstanding: f64,
    ) -> Result<f64> {
        if shares_outstanding <= 0.0 {
            return Err(anyhow::anyhow!("流通股数必须大于0"));
        }

        let equity_value = enterprise_value - net_debt;
        let per_share_value = equity_value / shares_outstanding;

        Ok(per_share_value)
    }
}

// ============================================================================
// Buffett Analysis - Buffett分析结果
// ============================================================================

/// Buffett分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffettAnalysis {
    /// 股票代码
    pub symbol: String,

    /// ROIC (投资资本回报率)
    pub roic: f64,

    /// 护城河评分
    pub moat_score: MoatScore,

    /// 护城河可持续性评分
    pub moat_sustainability: f64,

    /// 管理层评分
    pub management_score: ManagementScore,

    /// DCF内在价值
    pub intrinsic_value: f64,

    /// 公允价格 (内在价值的90%)
    pub fair_price: f64,

    /// 当前价格
    pub current_price: f64,

    /// 安全边际
    pub margin_of_safety: f64,

    /// 投资建议
    pub recommendation: String,

    /// 是否符合Buffett标准
    pub meets_buffett_criteria: bool,

    /// 综合评分 (0-100)
    pub score: f64,
}

impl BuffettAnalysis {
    /// 创建Buffett分析结果
    pub fn new(
        symbol: String,
        roic: f64,
        moat_score: MoatScore,
        moat_sustainability: f64,
        management_score: ManagementScore,
        intrinsic_value: f64,
        current_price: f64,
    ) -> Self {
        // Buffett愿意支付合理价格,所以公允价格为内在价值的90%
        let fair_price = intrinsic_value * 0.90;

        // 计算安全边际
        let margin_of_safety = if current_price > 0.0 {
            ((intrinsic_value - current_price) / intrinsic_value).max(0.0)
        } else {
            0.0
        };

        // 判断是否符合Buffett标准
        let meets_buffett_criteria = roic > 0.10
            && moat_score >= MoatScore::Wide
            && management_score >= ManagementScore::Good;

        // 生成建议
        let recommendation = if meets_buffett_criteria && current_price <= fair_price {
            "买入优质企业".to_string()
        } else if roic > 0.08 && moat_score >= MoatScore::Narrow {
            "考虑买入".to_string()
        } else {
            "继续观察".to_string()
        };

        // 计算综合评分
        let score = Self::calculate_score(
            roic,
            moat_score,
            management_score,
            margin_of_safety,
        );

        Self {
            symbol,
            roic,
            moat_score,
            moat_sustainability,
            management_score,
            intrinsic_value,
            fair_price,
            current_price,
            margin_of_safety,
            recommendation,
            meets_buffett_criteria,
            score,
        }
    }

    /// 计算Buffett评分
    fn calculate_score(
        roic: f64,
        moat_score: MoatScore,
        management_score: ManagementScore,
        margin_of_safety: f64,
    ) -> f64 {
        // ROIC评分 (40分)
        let roic_score = if roic > 0.20 {
            40.0
        } else if roic > 0.15 {
            30.0
        } else if roic > 0.10 {
            20.0
        } else {
            (roic * 100.0).min(10.0)
        };

        // 护城河评分 (30分)
        let moat_score = match moat_score {
            MoatScore::VeryWide => 30.0,
            MoatScore::Wide => 25.0,
            MoatScore::Narrow => 15.0,
            MoatScore::None => 0.0,
        };

        // 管理层评分 (20分)
        let mgmt_score = match management_score {
            ManagementScore::Excellent => 20.0,
            ManagementScore::Good => 15.0,
            ManagementScore::Average => 8.0,
            ManagementScore::Poor => 0.0,
        };

        // 安全边际评分 (10分)
        let margin_score = (margin_of_safety * 50.0).min(10.0);

        roic_score + moat_score + mgmt_score + margin_score
    }
}

// ============================================================================
// Buffett Framework - Buffett框架整合
// ============================================================================

/// Buffett框架
///
/// 整合护城河分析、管理层评估和DCF估值的完整框架
#[derive(Debug, Clone)]
pub struct BuffettFramework {
    /// 护城河分析器
    pub moat_analyzer: MoatAnalyzer,

    /// 管理层评估器
    pub management_evaluator: ManagementEvaluator,

    /// DCF计算器
    pub dcf_calculator: DcfCalculator,

    /// 最低ROIC要求 (默认10%)
    pub min_roic: f64,
}

impl Default for BuffettFramework {
    fn default() -> Self {
        Self {
            moat_analyzer: MoatAnalyzer::new(),
            management_evaluator: ManagementEvaluator::new(),
            dcf_calculator: DcfCalculator::new(),
            min_roic: 0.10,
        }
    }
}

impl BuffettFramework {
    /// 创建新的Buffett框架
    pub fn new() -> Self {
        Self::default()
    }

    /// 执行完整的Buffett分析
    ///
    /// # 参数
    /// - `symbol`: 股票代码
    /// - `roic`: 投资资本回报率
    /// - `free_cash_flow`: 自由现金流
    /// - `growth_rate`: 预期增长率
    /// - `discount_rate`: 折现率 (WACC)
    /// - `current_price`: 当前价格
    /// - `has_strong_brand`: 是否有强大品牌
    /// - `has_cost_advantage`: 是否有成本优势
    /// - `has_high_switching_cost`: 是否有高转换成本
    /// - `has_network_effects`: 是否有网络效应
    /// - `capital_allocation_score`: 资本配置评分
    /// - `shareholder_returns`: 股东回报率
    /// - `transparency_score`: 透明度评分
    /// - `insider_ownership`: 内部人持股比例
    ///
    /// # 返回
    /// Buffett分析结果
    pub async fn analyze(
        &self,
        symbol: String,
        roic: f64,
        free_cash_flow: f64,
        growth_rate: f64,
        discount_rate: f64,
        current_price: f64,
        has_strong_brand: bool,
        has_cost_advantage: bool,
        has_high_switching_cost: bool,
        has_network_effects: bool,
        capital_allocation_score: f64,
        shareholder_returns: f64,
        transparency_score: f64,
        insider_ownership: f64,
    ) -> Result<BuffettAnalysis> {
        // 1. 评估护城河
        let moat_score = self.moat_analyzer.evaluate_moat(
            has_strong_brand,
            has_cost_advantage,
            has_high_switching_cost,
            has_network_effects,
            roic,
        );

        let moat_sustainability = self
            .moat_analyzer
            .assess_sustainability(moat_score, 0, 2); // 假设行业稳定,竞争中等

        // 2. 评估管理层
        let management_score = self.management_evaluator.evaluate_management(
            capital_allocation_score,
            shareholder_returns,
            transparency_score,
            insider_ownership,
        );

        // 3. DCF估值
        let intrinsic_value = self
            .dcf_calculator
            .calculate_intrinsic_value(free_cash_flow, growth_rate, discount_rate)?;

        // 4. 创建分析结果
        let analysis = BuffettAnalysis::new(
            symbol,
            roic,
            moat_score,
            moat_sustainability,
            management_score,
            intrinsic_value,
            current_price,
        );

        Ok(analysis)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moat_analyzer() {
        let analyzer = MoatAnalyzer::new();

        // 测试有护城河的公司
        let moat = analyzer.evaluate_moat(true, true, false, true, 0.15);
        assert_eq!(moat, MoatScore::VeryWide);

        // 测试无护城河的公司
        let moat = analyzer.evaluate_moat(false, false, false, false, 0.08);
        assert_eq!(moat, MoatScore::None);
    }

    #[test]
    fn test_management_evaluator() {
        let evaluator = ManagementEvaluator::new();

        // 测试优秀管理层
        let score = evaluator.evaluate_management(85.0, 0.18, 90.0, 0.12);
        assert_eq!(score, ManagementScore::Excellent);

        // 测试一般管理层
        let score = evaluator.evaluate_management(60.0, 0.08, 70.0, 0.03);
        assert_eq!(score, ManagementScore::Average);
    }

    #[test]
    fn test_dcf_calculator() {
        let calculator = DcfCalculator::new();

        // 测试DCF计算
        let fcf = 1_000_000_000.0; // 10亿
        let growth = 0.10;         // 10%增长
        let discount = 0.10;       // 10%折现率

        let value = calculator
            .calculate_intrinsic_value(fcf, growth, discount)
            .unwrap();

        // 应该大于0
        assert!(value > 0.0);
    }

    #[test]
    fn test_buffett_framework() {
        let framework = BuffettFramework::new();

        let analysis = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                framework
                    .analyze(
                        "TEST".to_string(),
                        0.15, // ROIC 15%
                        1_000_000_000.0, // FCF 10亿
                        0.10,             // 10%增长
                        0.10,             // 10%折现率
                        100.0,            // 当前价格
                        true,  // 有品牌
                        true,  // 有成本优势
                        false, // 无转换成本
                        true,  // 有网络效应
                        80.0,  // 资本配置评分
                        0.15,  // 股东回报15%
                        85.0,  // 透明度
                        0.08,  // 内部人持股8%
                    )
                    .await
            })
            .unwrap();

        assert_eq!(analysis.symbol, "TEST");
        assert!(analysis.intrinsic_value > 0.0);
        assert!(analysis.score > 0.0);
    }
}
