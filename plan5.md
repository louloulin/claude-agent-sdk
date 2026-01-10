# Plan5 - 个性化多市场智能价值投资平台

**版本**: 5.0
**日期**: 2026-01-11
**状态**: 🚀 **规划中**
**核心**: 个性化Agent + 多市场 + 多投资理念 + AI驱动构建

---

## 🎯 Plan5核心愿景

**从固定Agent系统进化到个性化可组合智能投资平台**

### 核心理念

> **"每个投资者都值得拥有自己专属的AI投资助手"**

Plan5不是一个固定的投资平台，而是一个**可动态组合的投资Agent构建系统**：
- 用户可以基于自己的投资理念构建个性化Agent
- Agent = 投资理念 + 技能组合 + 市场偏好 + 风险约束
- Claude AI辅助用户设计和优化Agent
- 支持多市场(A股/美股/加密货币)
- 支持多投资理念(价值/成长/动量/质量)

---

## 📊 Plan5 vs Plan3/Plan4对比

### Plan3/Plan4架构回顾

**Plan3 (加密货币交易平台)**:
```
Claude Agent SDK
├─ 25 Agent Skills (平面化)
├─ 135+ MCP工具
├─ 9个固定Subagents
└─ Binance/OKX交易
```

**Plan4 (A股AI量化平台)**:
```
Claude Agent SDK
├─ 10个固定专业Agents (分层)
│  ├─ DataAgent, ResearchAgent, ...
│  └─ SupervisorAgent
├─ 25 Agent Skills
├─ 200+ MCP工具
└─ A股券商API
```

**Plan5架构升级** (本计划):
```
Claude Agent SDK
├─ 🆕 AgentBuilder (动态Agent构建系统)
│  ├─ 投资理念选择 (价值/成长/动量/质量)
│  ├─ 技能动态组合 (25+ Skills)
│  ├─ AI智能推荐 (Claude驱动)
│  └─ 配置热加载
├─ 🆕 MarketAdapter (多市场适配层)
│  ├─ AShareMarket (A股, T+1, ±10%)
│  ├─ USMarket (美股, T+0, 无限制)
│  ├─ CryptoMarket (加密货币, 24/7)
│  └─ MultiMarket (跨市场)
├─ 🆕 StrategyEngine (多策略引擎)
│  ├─ ValueStrategy (深度价值)
│  ├─ GrowthStrategy (成长精选)
│  ├─ MomentumStrategy (动量跟踪)
│  ├─ QualityStrategy (质量因子)
│  └─ MultiStyle (多风格组合)
├─ 🆕 IntelligentBuilder (AI驱动构建)
│  ├─ 需求智能分析
│  ├─ 技能智能推荐
│  ├─ 配置智能生成
│  └─ Self-Optimization
├─ 保留25 Skills (可复用)
├─ 保留200+ MCP工具 (可复用)
└─ 保留券商API (可复用)
```

### 关键差异对比

| 维度 | Plan3 | Plan4 | Plan5 |
|------|-------|-------|-------|
| **Agent数量** | 9个固定 | 10个固定 | **无限动态** |
| **Agent构建** | 静态配置 | 静态配置 | **AI动态构建** |
| **投资理念** | 通用量化 | 通用量化 | **个性化选择** |
| **市场支持** | 加密货币 | A股 | **多市场统一** |
| **策略类型** | 技术分析 | AI决策 | **多策略组合** |
| **用户定制** | ❌ | ❌ | **✅ 完全定制** |
| **AI参与** | 策略执行 | 决策层 | **全流程AI** |
| **价值投资** | ❌ | 部分 | **深度整合** |

---

## 🏗️ Plan5架构设计

### 1. AgentBuilder - 动态Agent构建系统

#### 核心概念

**Agent = 投资理念 + 技能组合 + 市场偏好 + 风险约束**

```rust
/// 个性化投资Agent
pub struct PersonalizedAgent {
    // 基础信息
    pub id: String,
    pub name: String,
    pub description: String,

    // 投资理念
    pub philosophy: InvestmentPhilosophy,
    pub philosophy_params: PhilosophyParams,

    // 市场偏好
    pub markets: Vec<Market>,
    pub market_constraints: HashMap<Market, MarketConstraints>,

    // 技能组合
    pub skills: Vec<Box<dyn Skill>>,
    pub skill_config: SkillConfig,

    // 风险管理
    pub risk_tolerance: RiskTolerance,
    pub risk_limits: RiskLimits,

    // Claude配置
    pub claude_model: String,
    pub system_prompt: String,

    // 性能追踪
    pub performance_history: PerformanceHistory,
}

/// 投资理念枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvestmentPhilosophy {
    /// 价值投资 (Graham-Buffett)
    Value {
        // 安全边际要求
        margin_of_safety: f64,  // 默认0.3 (30%)
        // 估值方法
        valuation_methods: Vec<ValuationMethod>,
        // 持有期限
        holding_period: Duration,
    },

    /// 成长投资
    Growth {
        // 最低增长率
        min_growth_rate: f64,  // 默认0.3 (30%)
        // 增长类型 (营收/利润/双重)
        growth_type: GrowthType,
        // 估值容忍度
        valuation_tolerance: f64,  // PEG可接受范围
    },

    /// 动量投资
    Momentum {
        // 动量周期
        lookback_period: Duration,  // 默认3-12个月
        // 相对强度阈值
        relative_strength_threshold: f64,
        // 趋势确认指标
        trend_confirmation: Vec<TrendIndicator>,
    },

    /// 质量投资
    Quality {
        // ROE要求
        min_roe: f64,  // 默认0.15 (15%)
        // 盈利质量评分
        earnings_quality_threshold: f64,
        // 财务健康度
        financial_health_threshold: f64,
    },

    /// 多风格组合
    MultiStyle {
        // 各风格权重
        style_weights: HashMap<String, f64>,
        // 再平衡频率
        rebalance_frequency: Duration,
        // 相关性监控
        correlation_monitoring: bool,
    },
}

/// 风险偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskTolerance {
    Conservative {
        max_volatility: f64,        // < 15%
        max_drawdown: f64,          // < 10%
        asset_allocation: String,   // "60/40股票/债券"
    },
    Moderate {
        max_volatility: f64,        // < 20%
        max_drawdown: f64,          // < 20%
        asset_allocation: String,   // "80/20股票/债券"
    },
    Aggressive {
        max_volatility: f64,        // 可接受 > 25%
        max_drawdown: f64,          // 可接受 > 30%
        asset_allocation: String,   // "100%股票"
    },
}
```

#### Agent配置文件格式 (.agent.md)

```markdown
---
name: my-value-agent
description: 我的深度价值投资Agent
philosophy: value
philosophy_params:
  margin_of_safety: 0.4
  valuation_methods: [dcf, pe, pb, ev_ebitda]
  holding_period: 3y
markets: [ashare, us]
risk_tolerance: moderate
skills:
  - fundamental-analysis
  - valuation-models
  - quality-screening
  - portfolio-optimization
model: claude-opus-4-20250514
---

# My Value Agent

## 投资理念

深度价值投资，寻找被低估的优质公司。

## 核心策略

1. **估值筛选**: P/B < 1, PEG < 1, 高股息率
2. **质量检查**: ROE > 15%, 现金流充裕
3. **安全边际**: 内在价值30%以上折扣
4. **长期持有**: 3-5年持有期

## 市场偏好

- **A股**: 关注银行、地产、制造业低估值标的
- **美股**: 关注 Berkshire、苹果等高质量公司

## 风险控制

- 单只股票 < 10%
- 行业集中度 < 40%
- 最大回撤 < 20%
```

#### AgentBuilder API

```rust
pub struct AgentBuilder {
    skill_registry: Arc<SkillRegistry>,
    claude_client: Arc<ClaudeClient>,
    config_store: Arc<ConfigStore>,
}

impl AgentBuilder {
    /// 创建新的Agent配置
    pub async fn create_agent(&self, config: AgentConfig) -> Result<PersonalizedAgent> {
        // 1. 验证配置
        self.validate_config(&config)?;

        // 2. 加载Skills
        let skills = self.load_skills(&config.skills).await?;

        // 3. 初始化市场适配器
        let markets = self.load_markets(&config.markets).await?;

        // 4. 构建Agent
        Ok(PersonalizedAgent {
            id: uuid::Uuid::new_v4().to_string(),
            skills,
            markets,
            ..
config
        })
    }

    /// 从文件加载Agent
    pub async fn load_from_file(&self, path: &Path) -> Result<PersonalizedAgent> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: AgentConfig = serde_yaml::from_str(&content)?;
        self.create_agent(config).await
    }

    /// 热重载Agent配置
    pub async fn reload_agent(&self, agent_id: &str) -> Result<()> {
        // 动态更新Agent配置，无需重启
    }

    /// 克隆Agent
    pub async fn clone_agent(&self, agent_id: &str, new_name: String) -> Result<PersonalizedAgent> {
        // 基于现有Agent快速创建新Agent
    }
}
```

---

### 2. MarketAdapter - 多市场适配层

#### 市场抽象

```rust
/// 市场适配器trait
#[async_trait]
pub trait MarketAdapter: Send + Sync {
    /// 市场标识
    fn market_id(&self) -> &str;

    /// 交易规则
    fn trading_rules(&self) -> &TradingRules;

    /// 数据源
    fn data_source(&self) -> Arc<dyn DataSource>;

    /// 券商API
    fn broker_api(&self) -> Arc<dyn BrokerApi>;

    /// 获取市场日历
    async fn get_calendar(&self, start: Date, end: Date) -> Result<Vec<TradingDay>>;

    /// 调整交易规则检查
    async fn validate_order(&self, order: &OrderRequest) -> Result<OrderValidation>;
}

/// 交易规则
#[derive(Debug, Clone)]
pub struct TradingRules {
    /// T+1交易 (true) vs T+0 (false)
    pub t_plus_one: bool,

    /// 价格限制 (None = 无限制, Some(0.10) = ±10%)
    pub price_limit: Option<f64>,

    /// 交易时间
    pub trading_hours: TradingHours,

    /// 最小交易单位
    pub min_unit: u64,

    /// 货币
    pub currency: Currency,

    /// 交易费用
    pub commission: CommissionStructure,

    /// 结算周期
    pub settlement_period: Duration,
}

/// A股市场实现
pub struct AShareMarket {
    rules: TradingRules,
    data_source: Arc<TushareDataSource>,
    broker_api: Arc<dyn BrokerApi>,  // QMT/PTrade/华泰/东方财富
}

impl AShareMarket {
    pub fn new(api_key: String, broker: String) -> Result<Self> {
        Ok(Self {
            rules: TradingRules {
                t_plus_one: true,
                price_limit: Some(0.10),  // 主板±10%, 创业板±20%
                trading_hours: TradingHours::china(),
                min_unit: 100,  // 100股=1手
                currency: Currency::CNY,
                commission: CommissionStructure::china(),
                settlement_period: Duration::days(1),  // T+1结算
            },
            data_source: Arc::new(TushareDataSource::new(api_key)?),
            broker_api: load_broker_api(broker)?,
        })
    }
}

#[async_trait]
impl MarketAdapter for AShareMarket {
    fn market_id(&self) -> &str {
        "ashare"
    }

    fn trading_rules(&self) -> &TradingRules {
        &self.rules
    }

    fn data_source(&self) -> Arc<dyn DataSource> {
        self.data_source.clone()
    }

    fn broker_api(&self) -> Arc<dyn BrokerApi> {
        self.broker_api.clone()
    }

    async fn validate_order(&self, order: &OrderRequest) -> Result<OrderValidation> {
        // 1. 检查T+1规则
        if order.side == OrderSide::Sell {
            let position = self.broker_api().get_position(order.symbol).await?;
            let buy_date = position.buy_date;
            let today = Local::now().date_naive();

            if buy_date >= today {
                return Ok(OrderValidation {
                    valid: false,
                    reason: "T+1规则：当日买入次日才能卖".to_string(),
                });
            }
        }

        // 2. 检查涨跌停
        if let Some(limit) = self.rules.price_limit {
            let current_price = self.data_source().get_quote(order.symbol).await?.price;
            let limit_up = current_price * (1.0 + limit);
            let limit_down = current_price * (1.0 - limit);

            if order.price > limit_up || order.price < limit_down {
                return Ok(OrderValidation {
                    valid: false,
                    reason: format!("价格超出涨跌停范围 [{:.2}, {:.2}]", limit_down, limit_up),
                });
            }
        }

        // 3. 检查最小单位
        if order.quantity % self.rules.min_unit != 0 {
            return Ok(OrderValidation {
                valid: false,
                reason: format!("数量必须是{}的整数倍", self.rules.min_unit),
            });
        }

        Ok(OrderValidation {
            valid: true,
            reason: "订单有效".to_string(),
        })
    }
}

/// 美股市场实现
pub struct USMarket {
    rules: TradingRules,
    data_source: Arc<YahooFinanceDataSource>,
    broker_api: Arc<dyn BrokerApi>,  // Interactive Brokers
}

impl USMarket {
    pub fn new() -> Result<Self> {
        Ok(Self {
            rules: TradingRules {
                t_plus_one: false,  // T+0
                price_limit: None,
                trading_hours: TradingHours::us(),
                min_unit: 1,
                currency: Currency::USD,
                commission: CommissionStructure::us(),
                settlement_period: Duration::days(2),  // T+2结算
            },
            data_source: Arc::new(YahooFinanceDataSource::new()?),
            broker_api: Arc::new(InteractiveBrokersApi::new()?),
        })
    }
}

/// 加密货币市场实现
pub struct CryptoMarket {
    rules: TradingRules,
    data_source: Arc<WebSocketDataSource>,  // Binance/OKX WebSocket
    broker_api: Arc<dyn BrokerApi>,  // Binance/OKX API
}

impl CryptoMarket {
    pub fn new(exchange: String, api_key: String, secret: String) -> Result<Self> {
        Ok(Self {
            rules: TradingRules {
                t_plus_one: false,  // 24/7交易
                price_limit: None,
                trading_hours: TradingHours::crypto_24_7(),
                min_unit: 1,
                currency: Currency::USD,
                commission: CommissionStructure::crypto(),
                settlement_period: Duration::zero(),
            },
            data_source: Arc::new(WebSocketDataSource::new(exchange.clone())?),
            broker_api: load_exchange_api(exchange, api_key, secret)?,
        })
    }
}
```

#### 多市场组合

```rust
/// 多市场投资组合
pub struct MultiMarketPortfolio {
    markets: HashMap<String, Arc<dyn MarketAdapter>>,
    allocations: HashMap<String, f64>,  // 各市场权重
    currency_converter: Arc<CurrencyConverter>,
}

impl MultiMarketPortfolio {
    /// 跨市场资产配置
    pub async fn allocate(&self, total_capital: f64) -> Result<Vec<Allocation>> {
        let mut allocations = Vec::new();

        for (market_id, weight) in &self.allocations {
            let market = self.markets.get(market_id).unwrap();
            let capital = total_capital * weight;

            // 转换为市场本地货币
            let local_capital = self.currency_converter
                .convert(capital, Currency::USD, market.trading_rules().currency)
                .await?;

            allocations.push(Allocation {
                market: market_id.clone(),
                capital: local_capital,
                currency: market.trading_rules().currency,
                weight: *weight,
            });
        }

        Ok(allocations)
    }

    /// 跨市场风险归因
    pub async fn risk_attribution(&self) -> Result<RiskAttribution> {
        // 计算各市场的风险贡献
        // 考虑汇率风险
    }
}
```

---

### 3. StrategyEngine - 多策略引擎

#### 策略抽象

```rust
/// 投资策略trait
#[async_trait]
pub trait InvestmentStrategy: Send + Sync {
    /// 策略名称
    fn name(&self) -> &str;

    /// 投资理念
    fn philosophy(&self) -> InvestmentPhilosophy;

    /// 适用市场
    fn market(&self) -> Market;

    /// 生成交易信号
    async fn generate_signals(&self, context: &MarketContext) -> Result<Vec<Signal>>;

    /// 评估策略表现
    fn evaluate(&self, metrics: &PerformanceMetrics) -> StrategyScore;

    /// 策略参数
    fn params(&self) -> &StrategyParams;
}

/// 交易信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub symbol: String,
    pub action: SignalAction,  // Buy/Sell/Hold
    pub confidence: f64,        // 0-1
    pub reasoning: String,
    pub expected_return: Option<f64>,
    pub risk_level: f64,
    pub time_horizon: Duration,
    pub position_size: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalAction {
    Buy,
    Sell,
    Hold,
    ReducePosition { percent: f64 },
    AddPosition { percent: f64 },
}
```

#### 价值投资策略

```rust
/// 价值投资策略
pub struct ValueStrategy {
    /// 估值模型
    valuation_models: Vec<Box<dyn ValuationModel>>,

    /// 筛选器
    screeners: Vec<Box<dyn Screener>>,

    /// 安全边际
    margin_of_safety: f64,

    /// 持有期限
    holding_period: Duration,
}

impl ValueStrategy {
    pub fn new() -> Self {
        Self {
            valuation_models: vec![
                Box::new(DCFModel::new()),
                Box::new(DDMModel::new()),
                Box::new(PEModel::new()),
                Box::new(PBModel::new()),
                Box::new(EVEBITDAModel::new()),
            ],
            screeners: vec![
                Box::new(PBScreener::new(1.0)),          // P/B < 1
                Box::new(PEGScreener::new(1.0)),         // PEG < 1
                Box::new(DividendScreener::new(0.03)),   // 股息率 > 3%
                Box::new(ROScreener::new(0.15)),         // ROE > 15%
            ],
            margin_of_safety: 0.3,  // 30%安全边际
            holding_period: Duration::days(365 * 3),  // 3年
        }
    }
}

#[async_trait]
impl InvestmentStrategy for ValueStrategy {
    fn name(&self) -> &str {
        "Deep Value Strategy"
    }

    fn philosophy(&self) -> InvestmentPhilosophy {
        InvestmentPhilosophy:: Value {
            margin_of_safety: self.margin_of_safety,
            valuation_methods: self.valuation_models.iter().map(|m| m.method()).collect(),
            holding_period: self.holding_period,
        }
    }

    fn market(&self) -> Market {
        Market::MultiMarket
    }

    async fn generate_signals(&self, context: &MarketContext) -> Result<Vec<Signal>> {
        let mut signals = Vec::new();

        // 1. 筛选候选股票
        let candidates = self.screen_candidates(context).await?;

        // 2. 估值分析
        for symbol in candidates {
            let intrinsic_value = self.calculate_intrinsic_value(&symbol, context).await?;
            let current_price = context.get_price(&symbol).await?;

            // 3. 安全边际检查
            let margin = (intrinsic_value - current_price) / intrinsic_value;

            if margin > self.margin_of_safety {
                signals.push(Signal {
                    symbol,
                    action: SignalAction::Buy,
                    confidence: (margin / self.margin_of_safety).min(1.0),
                    reasoning: format!(
                        "内在价值 {:.2} > 市价 {:.2}, 安全边际 {:.1}%",
                        intrinsic_value, current_price, margin * 100.0
                    ),
                    expected_return: Some((intrinsic_value / current_price) - 1.0),
                    risk_level: 0.3,  // 低风险
                    time_horizon: self.holding_period,
                    position_size: Some(0.05),  // 默认5%仓位
                });
            }
        }

        Ok(signals)
    }

    fn evaluate(&self, metrics: &PerformanceMetrics) -> StrategyScore {
        // 价值策略评估标准:
        // 1. 长期收益率 > 15%
        // 2. 最大回撤 < 25%
        // 3. 夏普比率 > 1.0
        // 4. 胜率 > 55%

        let mut score = 0.0;

        if metrics.annual_return > 0.15 {
            score += 30.0;
        }

        if metrics.max_drawdown < 0.25 {
            score += 25.0;
        }

        if metrics.sharpe_ratio > 1.0 {
            score += 25.0;
        }

        if metrics.win_rate > 0.55 {
            score += 20.0;
        }

        StrategyScore {
            total: score,
            pass: score > 60.0,
            details: vec![],
        }
    }

    fn params(&self) -> &StrategyParams {
        // 返回策略参数
    }
}

impl ValueStrategy {
    /// 筛选价值股
    async fn screen_candidates(&self, context: &MarketContext) -> Result<Vec<String>> {
        let mut candidates = Vec::new();

        for screener in &self.screeners {
            let screened = screener.screen(context).await?;
            candidates.extend(screened);
        }

        // 去重
        candidates.sort();
        candidates.dedup();

        Ok(candidates)
    }

    /// 计算内在价值
    async fn calculate_intrinsic_value(&self, symbol: &str, context: &MarketContext) -> Result<f64> {
        let mut values = Vec::new();

        for model in &self.valuation_models {
            let value = model.value(symbol, context).await?;
            values.push(value);
        }

        // 取中位数 (更稳健)
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = values[values.len() / 2];

        Ok(median)
    }
}
```

#### 成长投资策略

```rust
/// 成长投资策略
pub struct GrowthStrategy {
    /// 增长率要求
    min_revenue_growth: f64,
    min_earnings_growth: f64,

    /// 增长持续性
    growth_consistency_years: usize,

    /// 估值容忍度
    max_pe_ratio: Option<f64>,
    max_peg_ratio: f64,
}

impl GrowthStrategy {
    pub fn new() -> Self {
        Self {
            min_revenue_growth: 0.30,  // 30%
            min_earnings_growth: 0.25,  // 25%
            growth_consistency_years: 3,
            max_pe_ratio: None,  // 不设上限
            max_peg_ratio: 2.0,
        }
    }
}

#[async_trait]
impl InvestmentStrategy for GrowthStrategy {
    fn name(&self) -> &str {
        "Growth at Reasonable Price (GARP)"
    }

    async fn generate_signals(&self, context: &MarketContext) -> Result<Vec<Signal>> {
        let mut signals = Vec::new();

        // 1. 筛选高增长股票
        let candidates = context
            .screener()
            .revenue_growth(self.min_revenue_growth)
            .earnings_growth(self.min_earnings_growth)
            .consistency(self.growth_consistency_years)
            .run()
            .await?;

        // 2. 估值合理性检查
        for symbol in candidates {
            let pe = context.get_pe_ratio(&symbol).await?;
            let eps_growth = context.get_eps_growth(&symbol).await?;

            let peg = pe / eps_growth;

            if peg < self.max_peg_ratio {
                signals.push(Signal {
                    symbol,
                    action: SignalAction::Buy,
                    confidence: (2.0 - peg) / 2.0,
                    reasoning: format!(
                        "营收增长 {:.1}%, 利润增长 {:.1}%, PEG {:.2}",
                        self.min_revenue_growth * 100.0,
                        self.min_earnings_growth * 100.0,
                        peg
                    ),
                    expected_return: Some(eps_growth * 0.8),
                    risk_level: 0.6,
                    time_horizon: Duration::days(365),
                    position_size: Some(0.08),
                });
            }
        }

        Ok(signals)
    }

    fn evaluate(&self, metrics: &PerformanceMetrics) -> StrategyScore {
        // 成长策略评估:
        // 1. 高收益容忍 (可接受较高波动)
        // 2. 重点关注长期复合增长率

        let mut score = 0.0;

        if metrics.annual_return > 0.20 {
            score += 40.0;
        }

        if metrics.max_drawdown < 0.35 {
            score += 20.0;
        }

        if metrics.sharpe_ratio > 0.8 {
            score += 20.0;
        }

        if metrics.win_rate > 0.50 {
            score += 20.0;
        }

        StrategyScore {
            total: score,
            pass: score > 60.0,
            details: vec![],
        }
    }

    // ... 其他方法
}
```

#### 动量投资策略

```rust
/// 动量投资策略
pub struct MomentumStrategy {
    /// 动量周期
    lookback_period: Duration,

    /// 相对强度阈值
    relative_strength_threshold: f64,

    /// 趋势确认
    trend_indicators: Vec<String>,
}

impl MomentumStrategy {
    pub fn new() -> Self {
        Self {
            lookback_period: Duration::days(252),  // 12个月
            relative_strength_threshold: 0.0,
            trend_indicators: vec!["SMA50".to_string(), "SMA200".to_string()],
        }
    }
}

#[async_trait]
impl InvestmentStrategy for MomentumStrategy {
    fn name(&self) -> &str {
        "Momentum Strategy"
    }

    fn philosophy(&self) -> InvestmentPhilosophy {
        InvestmentPhilosophy::Momentum {
            lookback_period: self.lookback_period,
            relative_strength_threshold: self.relative_strength_threshold,
            trend_confirmation: vec![],
        }
    }

    async fn generate_signals(&self, context: &MarketContext) -> Result<Vec<Signal>> {
        let mut signals = Vec::new();

        // 1. 计算动量得分
        let symbols = context.get_universe().await?;

        for symbol in symbols {
            // 价格动量
            let price_momentum = context.calculate_price_momentum(
                &symbol,
                self.lookback_period
            ).await?;

            // 相对强度
            let relative_strength = context.calculate_relative_strength(
                &symbol,
                "SPY"  // 相对S&P 500
            ).await?;

            // 趋势确认
            let sma50 = context.get_sma(&symbol, 50).await?;
            let sma200 = context.get_sma(&symbol, 200).await?;
            let current_price = context.get_price(&symbol).await?;

            let trend_confirmed = current_price > sma50 && sma50 > sma200;

            // 综合判断
            if price_momentum > 0.2  // 12个月涨幅 > 20%
                && relative_strength > self.relative_strength_threshold
                && trend_confirmed
            {
                signals.push(Signal {
                    symbol,
                    action: SignalAction::Buy,
                    confidence: 0.7,
                    reasoning: format!(
                        "价格动量 {:.1}%, 相对强度 {:.2}, 趋势向上",
                        price_momentum * 100.0,
                        relative_strength
                    ),
                    expected_return: Some(price_momentum * 0.5),
                    risk_level: 0.7,
                    time_horizon: Duration::days(90),
                    position_size: Some(0.06),
                });
            }
        }

        Ok(signals)
    }

    // ... 其他方法
}
```

#### 质量投资策略

```rust
/// 质量投资策略
pub struct QualityStrategy {
    /// 质量因子
    quality_factors: Vec<Box<dyn QualityFactor>>,

    /// 最低ROE要求
    min_roe: f64,

    /// 盈利质量阈值
    earnings_quality_threshold: f64,
}

impl QualityStrategy {
    pub fn new() -> Self {
        Self {
            quality_factors: vec![
                Box::new(ROEFactor::new()),
                Box::new(ROAFactor::new()),
                Box::new(CashFlowFactor::new()),
                Box::new(DebtToEquityFactor::new()),
                Box::new(GrossMarginFactor::new()),
            ],
            min_roe: 0.15,
            earnings_quality_threshold: 0.8,
        }
    }
}

#[async_trait]
impl InvestmentStrategy for QualityStrategy {
    fn name(&self) -> &str {
        "Quality Factor Strategy"
    }

    fn philosophy(&self) -> InvestmentPhilosophy {
        InvestmentPhilosophy::Quality {
            min_roe: self.min_roe,
            earnings_quality_threshold: self.earnings_quality_threshold,
            financial_health_threshold: 0.7,
        }
    }

    async fn generate_signals(&self, context: &MarketContext) -> Result<Vec<Signal>> {
        let mut signals = Vec::new();

        // 1. 质量评分
        let symbols = context.get_universe().await?;

        for symbol in symbols {
            let mut quality_score = 0.0;

            // ROE评分
            let roe = context.get_roe(&symbol).await?;
            if roe > self.min_roe {
                quality_score += 25.0;
            }

            // 盈利质量评分
            let ocf = context.get_operating_cash_flow(&symbol).await?;
            let net_income = context.get_net_income(&symbol).await?;
            let cash_conversion = ocf / net_income;

            if cash_conversion > self.earnings_quality_threshold {
                quality_score += 25.0;
            }

            // 财务健康评分
            let debt_to_equity = context.get_debt_to_equity(&symbol).await?;
            if debt_to_equity < 0.5 {
                quality_score += 25.0;
            }

            // 毛利率评分
            let gross_margin = context.get_gross_margin(&symbol).await?;
            if gross_margin > 0.4 {
                quality_score += 25.0;
            }

            // 高质量公司
            if quality_score >= 75.0 {
                signals.push(Signal {
                    symbol,
                    action: SignalAction::Buy,
                    confidence: quality_score / 100.0,
                    reasoning: format!(
                        "质量评分 {:.0}, ROE {:.1}%, 现金转换 {:.2}",
                        quality_score, roe * 100.0, cash_conversion
                    ),
                    expected_return: Some(0.15),
                    risk_level: 0.2,
                    time_horizon: Duration::days(365 * 2),
                    position_size: Some(0.07),
                });
            }
        }

        Ok(signals)
    }

    // ... 其他方法
}
```

#### 多风格组合策略

```rust
/// 多风格组合策略
pub struct MultiStyleStrategy {
    /// 子策略
    strategies: Vec<Box<dyn InvestmentStrategy>>,

    /// 权重分配
    weights: HashMap<String, f64>,

    /// 再平衡频率
    rebalance_frequency: Duration,

    /// 相关性监控
    correlation_monitoring: bool,
}

impl MultiStyleStrategy {
    pub fn new() -> Self {
        let value_strategy = Box::new(ValueStrategy::new()) as Box<dyn InvestmentStrategy>;
        let growth_strategy = Box::new(GrowthStrategy::new()) as Box<dyn InvestmentStrategy>;
        let momentum_strategy = Box::new(MomentumStrategy::new()) as Box<dyn InvestmentStrategy>;
        let quality_strategy = Box::new(QualityStrategy::new()) as Box<dyn InvestmentStrategy>;

        let mut weights = HashMap::new();
        weights.insert(value_strategy.name().to_string(), 0.40);  // 价值 40%
        weights.insert(growth_strategy.name().to_string(), 0.20);  // 成长 20%
        weights.insert(momentum_strategy.name().to_string(), 0.20);  // 动量 20%
        weights.insert(quality_strategy.name().to_string(), 0.20);  // 质量 20%

        Self {
            strategies: vec![value_strategy, growth_strategy, momentum_strategy, quality_strategy],
            weights,
            rebalance_frequency: Duration::days(30),  // 月度再平衡
            correlation_monitoring: true,
        }
    }

    /// 动态权重调整 (基于市场环境)
    pub async fn adjust_weights(&mut self, market_regime: MarketRegime) {
        match market_regime {
            MarketRegime::Bull => {
                // 牛市: 增加动量和成长
                self.weights.insert("Momentum Strategy".to_string(), 0.30);
                self.weights.insert("Growth Strategy".to_string(), 0.30);
                self.weights.insert("Deep Value Strategy".to_string(), 0.20);
                self.weights.insert("Quality Factor Strategy".to_string(), 0.20);
            }

            MarketRegime::Bear => {
                // 熊市: 增加价值和质量
                self.weights.insert("Deep Value Strategy".to_string(), 0.50);
                self.weights.insert("Quality Factor Strategy".to_string(), 0.30);
                self.weights.insert("Growth Strategy".to_string(), 0.10);
                self.weights.insert("Momentum Strategy".to_string(), 0.10);
            }

            MarketRegime::Volatile => {
                // 震荡市: 均衡配置
                self.weights.insert("Quality Factor Strategy".to_string(), 0.40);
                self.weights.insert("Deep Value Strategy".to_string(), 0.30);
                self.weights.insert("Momentum Strategy".to_string(), 0.15);
                self.weights.insert("Growth Strategy".to_string(), 0.15);
            }
        }
    }
}

#[async_trait]
impl InvestmentStrategy for MultiStyleStrategy {
    fn name(&self) -> &str {
        "Multi-Style Portfolio"
    }

    async fn generate_signals(&self, context: &MarketContext) -> Result<Vec<Signal>> {
        let mut all_signals = Vec::new();

        // 1. 收集各策略信号
        for strategy in &self.strategies {
            let signals = strategy.generate_signals(context).await?;

            // 应用权重
            let weight = self.weights.get(strategy.name()).unwrap();

            for mut signal in signals {
                signal.position_size = Some(signal.position_size.unwrap() * weight);
                all_signals.push(signal);
            }
        }

        // 2. 相关性检查 (避免过度集中)
        if self.correlation_monitoring {
            all_signals = self.deduplicate_correlated_signals(all_signals, context).await?;
        }

        Ok(all_signals)
    }

    // ... 其他方法
}
```

---

### 4. IntelligentBuilder - AI驱动的Agent构建

#### 核心理念

**让Claude AI帮助用户构建最适合的个性化Agent**

```rust
/// 智能Agent构建器
pub struct IntelligentAgentBuilder {
    claude_client: Arc<ClaudeClient>,
    skill_registry: Arc<SkillRegistry>,
    agent_registry: Arc<AgentRegistry>,
    market_registry: Arc<MarketRegistry>,
}

impl IntelligentAgentBuilder {
    /// 从用户描述分析需求
    pub async fn analyze_requirements(&self, user_description: &str) -> Result<AgentRequirements> {
        let prompt = format!(
            r#"你是专业的投资顾问。请分析用户的投资需求，提取关键信息。

用户描述:
{}

请提取以下信息（JSON格式）:
{{
  "investment_philosophy": "价值投资/成长投资/动量投资/质量投资/多风格组合",
  "risk_tolerance": "保守/稳健/激进",
  "preferred_markets": ["A股", "美股", "加密货币"],
  "investment_horizon": "短期(<1年)/中期(1-3年)/长期(>3年)",
  "capital_amount": 数字,
  "special_requirements": ["需求1", "需求2"],
  "constraints": ["约束1", "约束2"]
}}

请只返回JSON，不要其他内容。"#,
            user_description
        );

        let response = self.claude_client.query(&prompt).await?;
        let requirements: AgentRequirements = serde_json::from_str(&response)?;

        Ok(requirements)
    }

    /// AI推荐技能组合
    pub async fn recommend_skills(&self, requirements: &AgentRequirements) -> Result<Vec<SkillRecommendation>> {
        let available_skills = self.skill_registry.list_all()?;

        let prompt = format!(
            r#"用户投资需求: {:?}

可用Skills:
{}

请推荐最合适的Skills组合（5-10个），说明理由。

返回格式(JSON):
[
  {{
    "skill_name": "skill_name",
    "reason": "推荐理由",
    "priority": "high/medium/low",
    "config": {{ "skill特定配置": "值" }}
  }}
]

请只返回JSON，不要其他内容。"#,
            requirements,
            available_skills.iter()
                .map(|s| format!("- {}: {}", s.name(), s.description()))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let response = self.claude_client.query(&prompt).await?;
        let recommendations: Vec<SkillRecommendation> = serde_json::from_str(&response)?;

        Ok(recommendations)
    }

    /// AI生成Agent配置
    pub async fn generate_agent_config(
        &self,
        requirements: &AgentRequirements,
        skill_recommendations: &[SkillRecommendation],
    ) -> Result<AgentConfig> {
        let prompt = format!(
            r#"基于以下信息，生成完整的Agent配置文件 (.agent.md格式):

投资需求: {:?}
推荐Skills: {:?}

请生成完整的.agent.md配置文件，包括:
1. YAML frontmatter (name, description, philosophy, markets, skills等)
2. 详细的Agent描述
3. 投资理念说明
4. 核心策略
5. 风险控制

配置文件应该:
- 专业且完整
- 符合用户需求
- 可直接使用
- 包含详细注释"#,
            requirements,
            skill_recommendations
        );

        let config_content = self.claude_client.query(&prompt).await?;

        // 解析生成的配置
        let config: AgentConfig = parse_agent_config(&config_content)?;

        Ok(config)
    }

    /// AI优化现有Agent
    pub async fn optimize_agent(
        &self,
        agent: &PersonalizedAgent,
        performance: &PerformanceMetrics,
    ) -> Result<OptimizationSuggestions> {
        let prompt = format!(
            r#"当前Agent配置:
{}

近期表现:
{{
  "annual_return": {:.1}%",
  "max_drawdown": {:.1}%",
  "sharpe_ratio": {:.2},
  "win_rate": {:.1}%
}}

请分析表现并提供优化建议:

1. Skills调整 (添加/删除/修改)
2. 权重优化
3. 风险参数调整
4. 策略改进

返回格式(JSON):
{{
  "performance_analysis": "表现分析",
  "suggestions": [
    {{
      "type": "add_skill/remove_skill/adjust_param",
      "description": "建议描述",
      "priority": "high/medium/low",
      "expected_impact": "预期影响"
    }}
  ]
}}"#,
            serde_yaml::to_string(agent)?,
            performance.annual_return * 100.0,
            performance.max_drawdown * 100.0,
            performance.sharpe_ratio,
            performance.win_rate * 100.0
        );

        let response = self.claude_client.query(&prompt).await?;
        let suggestions: OptimizationSuggestions = serde_json::from_str(&response)?;

        Ok(suggestions)
    }

    /// 一键创建Agent
    pub async fn create_agent_from_description(&self, description: &str) -> Result<PersonalizedAgent> {
        // 1. 分析需求
        let requirements = self.analyze_requirements(description).await?;

        // 2. 推荐Skills
        let skill_recs = self.recommend_skills(&requirements).await?;

        // 3. 生成配置
        let config = self.generate_agent_config(&requirements, &skill_recs).await?;

        // 4. 创建Agent
        let agent = AgentBuilder::new()
            .with_skill_registry(self.skill_registry.clone())
            .create_agent(config)
            .await?;

        Ok(agent)
    }
}
```

#### 使用示例

```rust
// 用户描述自己的投资需求
let user_description = r#"
我是一个长期价值投资者，专注于A股和美股市场。
我希望寻找被低估的优质公司，持有3-5年。
我的风险偏好是稳健型，可以接受20%以内的回撤。
我希望重点关注公司的基本面和现金流。
我倾向于巴菲特式的投资理念。
"#;

// AI自动创建Agent
let builder = IntelligentAgentBuilder::new(
    claude_client,
    skill_registry,
    agent_registry,
    market_registry,
);

let agent = builder.create_agent_from_description(user_description).await?;

println!("AI已为你创建个性化Agent: {}", agent.name);
println!("投资理念: {:?}", agent.philosophy);
println!("技能组合: {}", agent.skills.iter().map(|s| s.name()).join(", "));
```

---

## 📦 Phase实施规划

### Phase 1: 动态Agent构建系统 (4周)

**目标**: 实现AgentBuilder核心框架

**Week 1-2: 基础架构**
- [ ] `PersonalizedAgent`结构体
- [ ] `InvestmentPhilosophy`枚举
- [ ] `RiskTolerance`枚举
- [ ] `.agent.md`配置文件格式
- [ ] 配置解析器

**Week 3: AgentBuilder**
- [ ] `AgentBuilder`实现
- [ ] Skill加载系统
- [ ] 配置验证
- [ ] Agent序列化/反序列化

**Week 4: CLI工具**
- [ ] `invest-cli agent create` - 创建Agent
- [ ] `invest-cli agent list` - 列出所有Agent
- [ ] `invest-cli agent show <id>` - 显示Agent详情
- [ ] `invest-cli agent edit <id>` - 编辑Agent
- [ ] `invest-cli agent clone <id>` - 克隆Agent

**验收标准**:
- ✅ 可以通过配置文件创建Agent
- ✅ 可以通过CLI创建Agent
- ✅ Agent可以序列化保存和加载
- ✅ 配置验证工作正常

---

### Phase 2: 多市场适配层 (4周)

**目标**: 实现MarketAdapter抽象和市场实现

**Week 1: MarketAdapter抽象**
- [ ] `MarketAdapter` trait定义
- [ ] `TradingRules`结构体
- [ ] `DataSource` trait统一接口
- [ ] `BrokerApi` trait统一接口
- [ ] `OrderValidation`系统

**Week 2: A股市场**
- [ ] `AShareMarket`实现
- [ ] T+1规则检查
- [ ] 涨跌停价格检查
- [ ] 最小交易单位验证
- [ ] Tushare数据源集成
- [ ] QMT/PTrade券商API集成 (复用Plan4)

**Week 3: 美股市场**
- [ ] `USMarket`实现
- [ ] Yahoo Finance数据源
- [ ] Interactive Brokers API集成
- [ ] 美股交易规则 (T+0, 无涨跌停)

**Week 4: 加密货币市场**
- [ ] `CryptoMarket`实现
- [ ] Binance/OKX WebSocket数据源 (复用Plan3)
- [ ] Binance/OKX交易API (复用Plan3)
- [ ] 24/7交易支持

**验收标准**:
- ✅ 三大市场都可以正常交易
- ✅ 交易规则自动验证
- ✅ 统一的订单接口
- ✅ 跨市场数据查询

---

### Phase 3: 投资策略引擎 (4周)

**目标**: 实现4大核心投资策略

**Week 1: 价值投资策略**
- [ ] `ValueStrategy`实现
- [ ] DCF估值模型
- [ ] 相对估值模型 (P/E, P/B, EV/EBITDA)
- [ ] 安全边际计算
- [ ] 价值股筛选器
- [ ] 信号生成逻辑

**Week 2: 成长投资策略**
- [ ] `GrowthStrategy`实现
- [ ] 增长率筛选 (营收/利润)
- [ ] 增长持续性检查
- [ ] PEG估值
- [ ] 信号生成逻辑

**Week 3: 动量投资策略**
- [ ] `MomentumStrategy`实现
- [ ] 价格动量计算
- [ ] 相对强度计算
- [ ] 趋势确认指标
- [ ] 信号生成逻辑

**Week 4: 质量投资策略**
- [ ] `QualityStrategy`实现
- [ ] ROE/ROA筛选
- [ ] 现金流质量评分
- [ ] 财务健康度检查
- [ ] 信号生成逻辑

**验收标准**:
- ✅ 4个策略都能生成交易信号
- ✅ 信号包含置信度和推理过程
- ✅ 策略可以回测评估
- ✅ 策略参数可配置

---

### Phase 4: 智能Agent构建 (4周)

**目标**: 实现AI驱动的Agent构建系统

**Week 1: 需求分析**
- [ ] `IntelligentAgentBuilder`框架
- [ ] `analyze_requirements()` 实现
- [ ] Prompt工程优化
- [ ] 需求解析和验证

**Week 2: 技能推荐**
- [ ] `recommend_skills()` 实现
- [ ] Skill-Skill关联分析
- [ ] 技能组合优化算法
- [ ] 推荐理由生成

**Week 3: 配置生成**
- [ ] `generate_agent_config()` 实现
- [ ] 配置文件模板
- [ ] AI生成内容验证
- [ ] 配置优化建议

**Week 4: Agent优化**
- [ ] `optimize_agent()` 实现
- [ ] 性能分析Prompt
- [ ] 优化建议生成
- [ ] 一键优化功能

**验收标准**:
- ✅ AI可以准确理解用户需求
- ✅ AI推荐的Skills合理
- ✅ 生成的配置文件可用
- ✅ 优化建议有价值

---

### Phase 5: 多策略组合系统 (4周)

**目标**: 实现策略组合和动态权重调整

**Week 1: 多策略组合**
- [ ] `MultiStyleStrategy`实现
- [ ] 策略信号合并
- [ ] 权重分配算法
- [ ] 信号去重和过滤

**Week 2: 动态权重调整**
- [ ] `MarketRegime`识别 (牛市/熊市/震荡)
- [ ] 基于市场环境的权重调整
- [ ] 风险平价权重
- [ ] 动态再平衡

**Week 3: 相关性分析**
- [ ] 策略相关性计算
- [ ] 信号相关性检查
- [ ] 多样化优化
- [ ] 风险分散算法

**Week 4: 回测验证**
- [ ] 策略组合回测
- [ ] 多策略对比分析
- [ ] 最优权重搜索
- [ ] 参数敏感性分析

**验收标准**:
- ✅ 多策略可以协同工作
- ✅ 权重可以根据市场环境调整
- ✅ 相关性分析准确
- ✅ 回测结果可信

---

### Phase 6: 高级特性 (4周)

**目标**: 实现Self-Optimization和生产就绪功能

**Week 1: Self-Optimization**
- [ ] Agent自我评估框架
- [ ] 表现追踪系统
- [ ] 自动优化触发
- [ ] 优化效果验证

**Week 2: 风格轮动**
- [ ] `StyleRotationAgent`实现
- [ ] 经济周期识别
- [ ] 风格轮动信号
- [ ] 动态风格配置

**Week 3: 性能追踪**
- [ ] 实时性能监控
- [ ] 策略归因分析
- [ ] 风险指标计算
- [ ] 性能报告生成

**Week 4: 生产就绪**
- [ ] 配置热加载
- [ ] 优雅关闭
- [ ] 日志和监控
- [ ] 错误处理和重试
- [ ] 文档完善

**验收标准**:
- ✅ Agent可以自我优化
- ✅ 风格轮动工作正常
- ✅ 性能追踪准确
- ✅ 系统稳定可靠

---

## 🚀 核心创新点

### 1. 真正的个性化

**传统平台**: 固定Agent，用户只能配置参数
**Plan5**: 用户完全自定义Agent = 理念 + 技能 + 市场 + 风险

### 2. AI驱动的构建

**传统平台**: 用户手动配置
**Plan5**: Claude AI理解需求，推荐配置，生成Agent

### 3. 多市场统一

**传统平台**: 单一市场或多个独立系统
**Plan5**: 统一接口，A港美加密货币一体化

### 4. 价值投资深度整合

**传统平台**: 技术分析和量化策略为主
**Plan5**: 深度整合Graham-Buffett价值投资理念

### 5. 多策略动态组合

**传统平台**: 单一策略或简单策略切换
**Plan5**: 多策略协同，动态权重，风格轮动

---

## 📈 预期成果

### 技术指标

| 指标 | Plan4 | Plan5 | 提升 |
|------|-------|-------|------|
| **Agent类型** | 10个固定 | 无限动态 | ∞ |
| **市场支持** | 1个 (A股) | 3+ (A/美/加密) | 300% ↑ |
| **投资策略** | AI决策 | 4+专业策略 | 400% ↑ |
| **个性化** | ❌ | ✅ 完全定制 | 新增 |
| **AI参与度** | 决策层 | 全流程 | 质的飞跃 |
| **价值投资** | 部分 | 深度整合 | 显著提升 |

### 业务价值

1. **用户体验**
   - 每个用户都有专属AI投资助手
   - 投资理念和技术实现完美匹配
   - AI降低使用门槛

2. **投资效果**
   - 多策略组合降低风险
   - 动态权重调整提升收益
   - 价值投资降低回撤

3. **市场竞争力**
   - 业界首个个性化Agent平台
   - 唯一深度整合价值投资
   - 技术架构领先

---

## 💡 实施建议

### 1. 渐进式实施 ⭐

**推荐顺序**:
1. ✅ Phase 1优先 - AgentBuilder框架
2. ✅ Phase 2第二 - 多市场支持
3. ✅ Phase 3第三 - 核心策略
4. ✅ Phase 4第四 - AI构建
5. ✅ Phase 5第五 - 多策略组合
6. ✅ Phase 6最后 - 高级特性

### 2. 充分复用Plan3/Plan4

- ✅ 复用Plan3的加密货币交易 (Binance/OKX)
- ✅ 复用Plan4的A股交易规则和券商API
- ✅ 复用25个Agent Skills
- ✅ 复用200+个MCP工具

**Plan5 = Plan3 + Plan4 + AgentBuilder + 多策略 + AI构建**

### 3. 价值投资特色 ⭐

**核心竞争力**:
1. ✅ Graham-Buffett理念深度整合
2. ✅ 安全边际计算
3. ✅ 内在价值评估
4. ✅ 长期持有策略
5. ✅ 质量因子筛选

**这是Plan5的最大特色！**

---

## 📚 参考资源

### 价值投资

- [本杰明·格雷厄姆 - 聪明的投资者](https://www.amazon.com/Intelligent-Investor-Definitive-Investing-Essentials/dp/0060555661)
- [沃伦·巴菲特 - 致股东信](https://www.berkshirehathaway.com/letters/)
- [价值投资理念详解](https://www.investopedia.com/terms/v/valueinvesting.asp)

### 多因子模型

- [Fama-French五因子模型](https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/Data_Library/f-f_5_factors_2x3.html)
- [价值+动量黄金组合](https://www.aqr.com/Insights/Research/Journal-Article/Value-and-Momentum-Everywhere)
- [质量因子投资](https://research.af.msci.com/www-research-publications/working-papers/Quality-Investing.pdf)

### Multi-Agent系统

- [TradingGroup论文 (arXiv 2508.17565v1)](https://arxiv.org/html/2508.17565v1)
- [AutoTrader-AgentEdge](https://github.com/iAmGiG/AutoTrader-AgentEdge)

### Claude Agent SDK

- [Building agents with Claude Agent SDK](https://www.anthropic.com/engineering/building-agents-with-the-claude-agent-sdk)
- [Agent SDK overview](https://platform.claude.com/docs/en/agent-sdk/overview)

---

## 🎊 总结

**Plan5 = 个性化Agent + 多市场 + 多策略 + AI驱动 + 价值投资**

### 核心价值主张

**打造业界首个个性化、AI驱动、深度整合价值投资的多市场智能投资平台**

1. ✅ **真正个性化**: 用户自定义Agent，不是固定产品
2. ✅ **AI全流程参与**: 从需求分析到Agent优化
3. ✅ **多市场统一**: A股、美股、加密货币一体化
4. ✅ **多策略协同**: 价值、成长、动量、质量动态组合
5. ✅ **价值投资导向**: Graham-Buffett理念深度整合
6. ✅ **Claude Agent SDK**: 100%基于SDK，技术先进

### 与Plan3/Plan4的定位

| Plan | 市场定位 | 技术特点 | 目标用户 |
|------|---------|---------|---------|
| **Plan3** | 加密货币 | 固定Agent + 技术分析 | 加密货币投资者 |
| **Plan4** | A股 | Multi-Agent + AI决策 | A股量化投资者 |
| **Plan5** | 多市场 | 个性化Agent + 价值投资 | 所有价值投资者 |

**Plan5是最灵活、最智能、最个性化的投资平台！**

---

**文档版本**: 5.0
**创建日期**: 2026-01-11
**维护者**: InvestIntel AI Team
**状态**: 🚀 **规划完成，等待实施**

---

**END OF PLAN5**
