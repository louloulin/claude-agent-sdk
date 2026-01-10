# InvestIntel AI - 技术演进计划 4.0 (Plan4)
# A股量化投资平台 + 多Agent智能交易系统

**版本**: 4.0
**日期**: 2026-01-10
**核心理念**: **A股本土化 + Multi-Agent架构 + AI投资决策**
**基于**: Plan3完整实现 + A股市场特性分析 + 2025-2026最佳实践
**技术栈**: Rust + Claude Agent SDK + Multi-Agent Orchestration + A股券商API

---

## 📋 执行摘要

### Plan4核心目标

**从加密货币交易平台进化为A股AI量化投资平台**

1. ✅ **A股市场适配** - 支持T+1、涨跌停、中国特色交易规则
2. ✅ **Multi-Agent架构** - 专业Agent分工协作的智能交易系统
3. ✅ **券商API集成** - 对接主流券商交易接口
4. ✅ **AI投资决策** - 从策略执行到AI自主决策
5. ✅ **本土化数据源** - A股实时行情、财务数据、研报等

### 市场机会分析

**2025-2026年A股量化投资趋势**:
- ✅ 中国AI投资达¥287亿元（2025年）
- ✅ 86%的中国CEO期望3年内看到AI投资回报
- ✅ 券商密集部署DeepSeek等AI技术
- ✅ 多家券商开始提供量化交易API
- ✅ BigQuant等AI量化平台快速发展

---

## 🎯 Phase实施规划

| Phase | 功能 | 优先级 | 预计周期 | 依赖 |
|-------|------|--------|----------|------|
| **Phase 1** | A股市场基础架构 | ⭐⭐⭐⭐⭐ | 4周 | Plan3 |
| **Phase 2** | Multi-Agent架构升级 | ⭐⭐⭐⭐⭐ | 6周 | Phase 1 |
| **Phase 3** | 券商API集成 | ⭐⭐⭐⭐⭐ | 4周 | Phase 1 |
| **Phase 4** | AI投资决策系统 | ⭐⭐⭐⭐ | 6周 | Phase 2 |
| **Phase 5** | 本土化数据源 | ⭐⭐⭐⭐ | 4周 | Phase 1 |
| **Phase 6** | 高级Agent扩展 | ⭐⭐⭐ | 6周 | Phase 2,4 |

---

## 📊 市场与竞品深度分析

### 国内AI量化平台对比

#### 1. BigQuant
**官网**: https://bigquant.com/
**特点**: AI人工智能量化投资交易平台

**核心优势**:
- ✅ 机器学习AI技术集成
- ✅ 海量因子库（1000+因子）
- ✅ 高速回测引擎
- ✅ 量化交易接口
- ✅ Python开发环境

**我们的借鉴**:
- ✅ 海量因子库管理
- ✅ AI模型训练平台
- ❌ 不采用云端架构（我们坚持本地优先）

#### 2. 聚宽 (JoinQuant)
**特点**: 社区活跃的量化平台

**核心优势**:
- ✅ 海量因子库
- ✅ 策略商城（可分享/售卖策略）
- ✅ 界面简洁，对新手友好
- ✅ Python开发环境

**我们的改进**:
- ✅ 更深度的Claude SDK集成
- ✅ Rust性能优势（10-100x）
- ✅ 本地数据存储（libSQL 200ns查询）

#### 3. 优矿 (Uqer)
**特点**: 通联数据支持的量化平台

**核心优势**:
- ✅ 丰富的数据资源（通联数据）
- ✅ Jupyter Notebook环境
- ✅ 免费基础数据
- ✅ 模拟交易支持

**我们的改进**:
- ✅ 真实券商API集成
- ✅ Multi-Agent架构
- ✅ 完整的AI决策系统

#### 4. 掘金 (GminD)
**特点**: 支持实盘交易的量化平台

**核心优势**:
- ✅ 支持实盘交易
- ✅ 本地运行，安全性高
- ✅ 适合专业交易者

**我们的改进**:
- ✅ 更开放的架构（支持多券商）
- ✅ AI Agent自主决策
- ✅ 更强大的风控系统

### 国际Multi-Agent交易系统

#### 1. TradingGroup (arXiv 2025)
**论文**: [TradingGroup: A Multi-Agent Trading System](https://arxiv.org/html/2508.17565v1)

**核心特点**:
- ✅ Self-reflective architecture（自反思架构）
- ✅ End-to-end解决方案
- ✅ 2025年8月最新研究

**我们的借鉴**:
- ✅ 引入Self-Reflection机制
- ✅ 多Agent协作决策
- ✅ 可解释的决策流程

#### 2. AutoTrader-AgentEdge
**GitHub**: [iAmGiG/AutoTrader-AgentEdge](https://github.com/iAmGiG/AutoTrader-AgentEdge)

**核心特点**:
- ✅ Microsoft AutoGen驱动
- ✅ Production-ready实现
- ✅ MACD+RSI指标验证
- ✅ Agent-based设计

**我们的改进**:
- ✅ 基于Claude Agent SDK（更统一）
- ✅ Rust高性能实现
- ✅ 更丰富的Agent生态

#### 3. Multi-Agent Framework for Quantitative Finance
**论文**: [ACL Anthology 2025](https://aclanthology.org/2025.emnlp-main.55.pdf)

**核心特点**:
- ✅ 复杂量化任务集成框架
- ✅ 10个学术引用
- ✅ 2025年最新研究

**我们的借鉴**:
- ✅ 任务分解与编排
- ✅ Agent间协作协议
- ✅ 决策可追溯性

### A股市场特殊性分析

#### 1. T+1交易制度

**规则**:
- T日买入股票，T日不可卖出
- T+1日及以后才能卖出
- 资金实行T+0制度（卖出后资金当天可用）

**对我们的影响**:
```rust
// ❌ 加密货币的T+0策略不适用
// 可以当天买入卖出

// ✅ A股需要T+1策略
pub struct ASharePosition {
    pub symbol: String,
    pub shares: u64,
    pub buy_date: Date,     // 买入日期
    pub can_sell: bool,     // 是否可卖出（T+1检查）
}

impl ASharePosition {
    pub fn check_sellable(&self, current_date: Date) -> bool {
        // T+1规则：买入次日才能卖出
        current_date > self.buy_date
    }
}
```

#### 2. 涨跌停限制

**规则**:
- 主板：±10%
- 创业板/科创板：±20%
- ST股票：±5%

**策略调整**:
```rust
pub enum AShareTradingRule {
    PriceLimit {
        limit: f64,  // 0.10 or 0.20
        is_st: bool,
    },
}

impl AShareTradingRule {
    pub fn check_order_price(&self, price: f64, limit_price: f64) -> bool {
        match self {
            AShareTradingRule::PriceLimit { limit, .. } => {
                let change = (price - limit_price) / limit_price;
                change.abs() <= *limit
            }
        }
    }
}
```

#### 3. 量化策略特殊性

**中低频策略为主**:
- ✅ 真正的高频策略不存在（受T+1限制）
- ✅ 持仓周期：数天到数周
- ✅ 因子投资、多因子模型
- ✅ 事件驱动策略

**不适合的策略**:
- ❌ 日内高频交易（受T+1限制）
- ❌ 做市策略（A股无做市商制度）
- ❌ 统计套利（机会较少）

**适合的策略**:
- ✅ 多因子选股
- ✅ 行业轮动
- ✅ 事件驱动（财报、公告）
- ✅ 量化打板（需谨慎）

---

## 🏗️ Plan4架构设计

### 总体架构

```
┌─────────────────────────────────────────────────────────┐
│         InvestIntel AI 4.0 - A股智能投资平台           │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────────────────────────────────────────────┐   │
│  │       Claude Agent SDK (核心大脑)               │   │
│  │  - Claude Sonnet 4.5 (默认模型)                 │   │
│  │  - Claude Opus 4 (复杂分析)                     │   │
│  │  - Claude Haiku (快速响应)                      │   │
│  └─────────────────────────────────────────────────┘   │
│                       ↓                                 │
│  ┌─────────────────────────────────────────────────┐   │
│  │    Multi-Agent Orchestration Layer (新增)       │   │
│  │  ┌────────────┬────────────┬──────────────┐    │   │
│  │  │ Sequential │  Parallel  │  Hierarchical │    │   │
│  │  │ Orch.      │  Orch.     │  Orch.        │    │   │
│  │  └────────────┴────────────┴──────────────┘    │   │
│  └─────────────────────────────────────────────────┘   │
│                       ↓                                 │
│  ┌─────────────────────────────────────────────────┐   │
│  │      Specialized Agent System (专业Agent)       │   │
│  │  ┌───────────┬──────────┬──────────┬────────┐  │   │
│  │  │   Data    │  Research │ Strategy  │ Trading│  │   │
│  │  │  Agent    │  Agent   │  Agent   │ Agent │  │   │
│  │  └───────────┴──────────┴──────────┴────────┘  │   │
│  │  ┌───────────┬──────────┬──────────┬────────┐  │   │
│  │  │   Risk    │ Portfolio │ Sentiment │Report │  │   │
│  │  │  Agent    │  Agent   │  Agent   │ Agent │  │   │
│  │  └───────────┴──────────┴──────────┴────────┘  │   │
│  └─────────────────────────────────────────────────┘   │
│                       ↓                                 │
│  ┌─────────────────────────────────────────────────┐   │
│  │         A股 Market Layer (A股市场层)            │   │
│  │  ┌──────────┬──────────┬──────────┬────────┐  │   │
│  │  │ 券商API  │ 行情数据 │ 财务数据 │ 研报   │  │   │
│  │  └──────────┴──────────┴──────────┴────────┘  │   │
│  └─────────────────────────────────────────────────┘   │
│                       ↓                                 │
│  ┌─────────────────────────────────────────────────┐   │
│  │       Execution & Risk Layer (执行与风控)       │   │
│  │  - T+1交易规则检查                              │   │
│  │  - 涨跌停价格限制                               │   │
│  │  - 多层风控引擎                                 │   │
│  │  - 紧急停止机制                                 │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### vs Plan3架构对比

**Plan3架构**:
```
InvestIntel AI 3.0
├─ Claude Agent SDK
├─ 25 Agent Skills (平面化)
├─ 135+ MCP工具
└─ 加密货币交易（Binance/OKX）
```

**Plan4架构升级**:
```
InvestIntel AI 4.0
├─ Claude Agent SDK (保持)
├─ Multi-Agent Orchestration (新增) ⭐
├─ 10+ 专业Agent (分层) ⭐
├─ 25+ Agent Skills (保留)
├─ 200+ MCP工具 (扩展) ⭐
└─ A股市场对接 (全新) ⭐
```

---

## 📦 Phase 1: A股市场基础架构 (4周)

### 目标

**建立A股市场的基础设施和交易规则引擎**

### 1.1 A股交易规则引擎

**实现**: `investintel-agent/ashare/trading_rules.rs`

```rust
/// A股交易规则
pub struct AShareTradingRules {
    /// T+1交易制度
    t_plus_one: TPlusOneRule,
    /// 涨跌停限制
    price_limits: HashMap<String, PriceLimit>,
    /// 交易时间
    trading_hours: TradingHours,
    /// 最小交易单位
    min_unit: MinTradingUnit,
}

#[derive(Debug, Clone)]
pub struct TPlusOneRule {
    /// 股票T+1
    stock_t_plus_one: bool,
    /// 资金T+0
    cash_t_plus_zero: bool,
}

impl AShareTradingRules {
    /// 检查是否可以卖出
    pub fn check_sellable(&self, position: &Position, trade_date: Date) -> Result<bool> {
        if self.t_plus_one.stock_t_plus_one {
            // T+1规则：买入次日才能卖出
            return Ok(trade_date > position.buy_date);
        }
        Ok(true)
    }

    /// 检查订单价格是否在涨跌停范围内
    pub fn check_price_limit(&self, symbol: &str, price: f64, limit_price: f64) -> Result<bool> {
        if let Some(limit) = self.price_limits.get(symbol) {
            let change = (price - limit_price) / limit_price;
            return Ok(change.abs() <= limit.limit);
        }
        Ok(true)
    }

    /// 计算涨停价
    pub fn calculate_upper_limit(&self, symbol: &str, limit_price: f64) -> f64 {
        if let Some(limit) = self.price_limits.get(symbol) {
            return limit_price * (1.0 + limit.limit);
        }
        limit_price * 1.10  // 默认10%
    }

    /// 计算跌停价
    pub fn calculate_lower_limit(&self, symbol: &str, limit_price: f64) -> f64 {
        if let Some(limit) = self.price_limits.get(symbol) {
            return limit_price * (1.0 - limit.limit);
        }
        limit_price * 0.90  // 默认10%
    }
}
```

### 1.2 A股数据模型

**实现**: `investintel-agent/ashare/models.rs`

```rust
/// A股股票信息
pub struct AShareStock {
    /// 股票代码 (如: 600519.SH)
    pub symbol: String,
    /// 股票名称
    pub name: String,
    /// 市场: SH/SZ
    pub market: Market,
    /// 板块: 主板/创业板/科创板/北交所
    pub board: Board,
    /// 是否ST股票
    pub is_st: bool,
    /// 涨跌停限制
    pub price_limit: f64,  // 0.10 or 0.20 or 0.05
}

#[derive(Debug, Clone, PartialEq)]
pub enum Market {
    SH,  // 上交所
    SZ,  // 深交所
    BJ,  // 北交所
}

#[derive(Debug, Clone, PartialEq)]
pub enum Board {
    Main,         // 主板
    ChiNext,      // 创业板
    STAR,         // 科创板
    BSE,          // 北交所
}

/// A股行情数据
pub struct AShareQuote {
    pub symbol: String,
    pub current_price: f64,
    pub upper_limit: f64,   // 涨停价
    pub lower_limit: f64,   // 跌停价
    pub volume: u64,
    pub amount: f64,
    pub bid_price: Vec<f64>,  // 买盘价格（5档）
    pub ask_price: Vec<f64>,  // 卖盘价格（5档）
    pub bid_volume: Vec<u64>,
    pub ask_volume: Vec<u64>,
    pub timestamp: DateTime<Utc>,
}
```

### 1.3 A股订单管理

**实现**: `investintel-agent/ashare/order_manager.rs`

```rust
/// A股订单管理器
pub struct AShareOrderManager {
    /// 券商客户端列表
    brokers: Vec<Box<dyn BrokerClient>>,
    /// 交易规则引擎
    rules: Arc<AShareTradingRules>,
    /// 持仓信息
    positions: Arc<RwLock<HashMap<String, ASharePosition>>>,
    /// 资金信息
    account: Arc<RwLock<AccountInfo>>,
    /// 风控引擎
    risk_engine: Arc<RiskEngine>,
}

impl AShareOrderManager {
    /// 下单（含T+1和涨跌停检查）
    pub async fn place_order(&self, request: AShareOrderRequest) -> Result<AShareOrderResponse> {
        // 1. T+1规则检查
        if request.side == OrderSide::Sell {
            let positions = self.positions.read().await;
            if let Some(position) = positions.get(&request.symbol) {
                let sellable = self.rules.check_sellable(
                    position,
                    Utc::now().date_naive()
                )?;
                if !sellable {
                    bail!("T+1规则限制：今日买入不可卖出");
                }
            }
        }

        // 2. 涨跌停价格检查
        let quote = self.get_quote(&request.symbol).await?;
        let price_valid = self.rules.check_price_limit(
            &request.symbol,
            request.price,
            quote.current_price
        )?;
        if !price_valid {
            bail!("订单价格超出涨跌停限制范围");
        }

        // 3. 风控检查
        self.risk_engine.pre_trade_check(&request).await?;

        // 4. 提交到券商
        let broker = self.select_broker(&request.symbol)?;
        let response = broker.place_order(request).await?;

        Ok(response)
    }

    /// 选择最优券商（根据佣金、滑点等）
    fn select_broker(&self, symbol: &str) -> Result<&Box<dyn BrokerClient>> {
        // TODO: 实现券商选择逻辑
        self.brokers.first()
            .ok_or_else(|| anyhow!("No available broker"))
    }
}
```

### 1.4 A股持仓管理

**实现**: `investintel-agent/ashare/position.rs`

```rust
/// A股持仓
pub struct ASharePosition {
    /// 股票代码
    pub symbol: String,
    /// 持仓数量
    pub shares: u64,
    /// 买入价格
    pub buy_price: f64,
    /// 买入日期
    pub buy_date: Date,
    /// 当前市值
    pub market_value: f64,
    /// 盈亏
    pub pnl: f64,
    /// 盈亏比例
    pub pnl_pct: f64,
    /// 是否可卖出（T+1检查）
    pub can_sell: bool,
}

impl ASharePosition {
    /// 更新市值
    pub fn update_market_value(&mut self, current_price: f64) {
        self.market_value = self.shares as f64 * current_price;
        self.pnl = self.market_value - (self.shares as f64 * self.buy_price);
        self.pnl_pct = (current_price - self.buy_price) / self.buy_price;
    }

    /// 检查T+1可卖状态
    pub fn update_sellable_status(&mut self, current_date: Date) {
        self.can_sell = current_date > self.buy_date;
    }
}
```

**关键功能**:
- ✅ T+1交易规则检查
- ✅ 涨跌停价格限制
- ✅ A股持仓管理
- ✅ 最小交易单位（100股=1手）
- ✅ 交易时间检查

---

## 🤖 Phase 2: Multi-Agent架构升级 (6周)

### 目标

**从Skills平面化架构升级为Multi-Agent分层架构**

### 2.1 Agent角色定义

**核心Agent** (10个):

#### 1. DataAgent (数据采集Agent)

**职责**: 负责所有市场数据采集

```rust
pub struct DataAgent {
    name: String,
    data_sources: Vec<Box<dyn DataSource>>,
}

#[async_trait]
impl Agent for DataAgent {
    fn name(&self) -> &str {
        "DataCollector"
    }

    fn description(&self) -> &str {
        "负责采集A股实时行情、财务数据、研报等"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        match input.task.as_str() {
            "collect_quotes" => {
                // 采集实时行情
                let symbols = input.params["symbols"].as_array()
                    .ok_or_else(|| anyhow!("Missing symbols"))?;

                let quotes = self.collect_quotes(symbols).await?;

                Ok(AgentOutput {
                    content: serde_json::to_string_pretty(&quotes)?,
                    data: serde_json::to_value(quotes)?,
                    confidence: 0.95,
                    metadata: self.default_metadata(),
                })
            }
            "collect_financials" => {
                // 采集财务数据
                let symbol = input.params["symbol"].as_str()
                    .ok_or_else(|| anyhow!("Missing symbol"))?;

                let financials = self.collect_financials(symbol).await?;

                Ok(AgentOutput {
                    content: serde_json::to_string_pretty(&financials)?,
                    data: serde_json::to_value(financials)?,
                    confidence: 0.90,
                    metadata: self.default_metadata(),
                })
            }
            _ => bail!("Unknown task: {}", input.task),
        }
    }
}
```

#### 2. ResearchAgent (研究分析Agent)

**职责**: 负责个股研究和行业分析

```rust
pub struct ResearchAgent {
    name: String,
    claude_client: Arc<ClaudeClient>,
}

#[async_trait]
impl Agent for ResearchAgent {
    fn name(&self) -> &str {
        "ResearchAnalyst"
    }

    fn description(&self) -> &str {
        "负责个股研究、行业分析、公司基本面分析"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        match input.task.as_str() {
            "analyze_stock" => {
                let symbol = input.params["symbol"].as_str()
                    .ok_or_else(|| anyhow!("Missing symbol"))?;

                // 使用Claude进行分析
                let prompt = format!(
                    "分析股票{}的基本面、财务状况、技术面和投资价值。\n\n请从以下几个维度分析：
                    1. 公司基本面（行业地位、竞争优势）
                    2. 财务状况（营收、利润、现金流）
                    3. 估值水平（PE、PB、PS等）
                    4. 技术面（趋势、支撑压力位）
                    5. 风险因素

                    请给出具体的投资建议（买入/持有/卖出）和理由。",
                    symbol
                );

                let messages = query(&prompt, None).await?;
                let analysis = self.extract_analysis(messages)?;

                Ok(AgentOutput {
                    content: analysis.clone(),
                    data: serde_json::json!({
                        "symbol": symbol,
                        "analysis": analysis,
                        "timestamp": Utc::now()
                    }),
                    confidence: 0.85,
                    metadata: self.default_metadata(),
                })
            }
            _ => bail!("Unknown task: {}", input.task),
        }
    }
}
```

#### 3. StrategyAgent (策略制定Agent)

**职责**: 负责交易策略制定和优化

```rust
pub struct StrategyAgent {
    name: String,
    models: HashMap<String, Box<dyn StrategyModel>>,
}

#[async_trait]
impl Agent for StrategyAgent {
    fn name(&self) -> &str {
        "StrategyMaker"
    }

    fn description(&self) -> &str {
        "负责制定交易策略、优化参数、回测验证"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        match input.task.as_str() {
            "generate_strategy" => {
                // 基于市场状态生成策略
                let market_state: MarketState = serde_json::from_value(
                    input.params["market_state"].clone()
                )?;

                let strategy = self.generate_strategy(&market_state).await?;

                Ok(AgentOutput {
                    content: format!("策略生成: {:?}", strategy),
                    data: serde_json::to_value(&strategy)?,
                    confidence: strategy.confidence,
                    metadata: self.default_metadata(),
                })
            }
            "backtest" => {
                // 回测策略
                let strategy: Strategy = serde_json::from_value(
                    input.params["strategy"].clone()
                )?;

                let backtest_result = self.backtest(&strategy).await?;

                Ok(AgentOutput {
                    content: serde_json::to_string_pretty(&backtest_result)?,
                    data: serde_json::to_value(&backtest_result)?,
                    confidence: 0.90,
                    metadata: self.default_metadata(),
                })
            }
            _ => bail!("Unknown task: {}", input.task),
        }
    }
}
```

#### 4. TradingAgent (交易执行Agent)

**职责**: 负责订单执行和交易管理

```rust
pub struct TradingAgent {
    name: String,
    order_manager: Arc<AShareOrderManager>,
}

#[async_trait]
impl Agent for TradingAgent {
    fn name(&self) -> &str {
        "TradeExecutor"
    }

    fn description(&self) -> &str {
        "负责订单执行、交易管理、仓位控制"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        match input.task.as_str() {
            "place_order" => {
                let order: AShareOrderRequest = serde_json::from_value(
                    input.params["order"].clone()
                )?;

                let response = self.order_manager.place_order(order).await?;

                Ok(AgentOutput {
                    content: format!("订单已提交: {:?}", response.order_id),
                    data: serde_json::to_value(&response)?,
                    confidence: 0.95,
                    metadata: self.default_metadata(),
                })
            }
            _ => bail!("Unknown task: {}", input.task),
        }
    }
}
```

#### 5. RiskAgent (风险控制Agent)

**职责**: 负责风险监控和控制

```rust
pub struct RiskAgent {
    name: String,
    risk_engine: Arc<RiskEngine>,
}

#[async_trait]
impl Agent for RiskAgent {
    fn name(&self) -> &str {
        "RiskController"
    }

    fn description(&self) -> &str {
        "负责风险监控、仓位控制、止盈止损"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        match input.task.as_str() {
            "check_risk" => {
                let portfolio: Portfolio = serde_json::from_value(
                    input.params["portfolio"].clone()
                )?;

                let risk_report = self.risk_engine.analyze(&portfolio).await?;

                Ok(AgentOutput {
                    content: serde_json::to_string_pretty(&risk_report)?,
                    data: serde_json::to_value(&risk_report)?,
                    confidence: 0.95,
                    metadata: self.default_metadata(),
                })
            }
            _ => bail!("Unknown task: {}", input.task),
        }
    }
}
```

#### 6. PortfolioAgent (组合管理Agent)

**职责**: 负责投资组合优化和管理

#### 7. SentimentAgent (情绪分析Agent)

**职责**: 负责市场情绪和新闻分析

#### 8. ReportAgent (报告生成Agent)

**职责**: 负责生成交易报告和分析报告

#### 9. MonitorAgent (监控Agent)

**职责**: 负责实时监控和告警

#### 10. SupervisorAgent (监督Agent) ⭐

**职责**: 负责协调所有Agent，做最终决策

```rust
pub struct SupervisorAgent {
    name: String,
    agents: HashMap<String, Box<dyn Agent>>,
    claude_client: Arc<ClaudeClient>,
}

#[async_trait]
impl Agent for SupervisorAgent {
    fn name(&self) -> &str {
        "Supervisor"
    }

    fn description(&self) -> &str {
        "负责协调所有专业Agent，做出最终投资决策"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        match input.task.as_str() {
            "make_decision" => {
                // 1. 收集各Agent的意见
                let data_output = self.agents.get("DataCollector")
                    .ok_or_else(|| anyhow!("DataCollector not found"))?
                    .execute(AgentInput {
                        task: "collect_market_data".to_string(),
                        params: input.params.clone(),
                    }).await?;

                let research_output = self.agents.get("ResearchAnalyst")
                    .ok_or_else(|| anyhow!("ResearchAnalyst not found"))?
                    .execute(AgentInput {
                        task: "analyze_market".to_string(),
                        params: input.params.clone(),
                    }).await?;

                let strategy_output = self.agents.get("StrategyMaker")
                    .ok_or_else(|| anyhow!("StrategyMaker not found"))?
                    .execute(AgentInput {
                        task: "generate_strategy".to_string(),
                        params: input.params.clone(),
                    }).await?;

                let risk_output = self.agents.get("RiskController")
                    .ok_or_else(|| anyhow!("RiskController not found"))?
                    .execute(AgentInput {
                        task: "assess_risk".to_string(),
                        params: input.params.clone(),
                    }).await?;

                // 2. 使用Claude综合各Agent意见，做出最终决策
                let prompt = format!(
                    "你是投资决策监督者。以下是各专业Agent的分析结果：

【数据采集Agent】
{}

【研究分析Agent】
{}

【策略制定Agent】
{}

【风险控制Agent】
{}

请综合以上信息，做出最终投资决策（买入/卖出/持有），并说明理由。",
                    data_output.content,
                    research_output.content,
                    strategy_output.content,
                    risk_output.content
                );

                let messages = query(&prompt, None).await?;
                let decision = self.extract_decision(messages)?;

                Ok(AgentOutput {
                    content: decision.clone(),
                    data: serde_json::json!({
                        "decision": decision,
                        "agent_outputs": [
                            data_output,
                            research_output,
                            strategy_output,
                            risk_output,
                        ],
                        "timestamp": Utc::now()
                    }),
                    confidence: 0.80,
                    metadata: self.default_metadata(),
                })
            }
            _ => bail!("Unknown task: {}", input.task),
        }
    }
}
```

### 2.2 Agent编排模式

#### SequentialOrchestration (顺序编排)

```rust
use claude_agent_sdk_rs::orchestration::SequentialOrchestrator;

// 示例：数据采集 -> 研究 -> 策略 -> 决策
let agents = vec![
    Box::new(DataAgent::new()),
    Box::new(ResearchAgent::new()),
    Box::new(StrategyAgent::new()),
];

let orchestrator = SequentialOrchestrator::new(agents);

let input = AgentInput {
    task: "full_analysis".to_string(),
    params: serde_json::json!({"symbol": "600519.SH"}),
};

let output = orchestrator.orchestrate(input).await?;
```

#### ParallelOrchestration (并行编排)

```rust
use claude_agent_sdk_rs::orchestration::ParallelOrchestrator;

// 示例：并行分析多个股票
let agents = vec![
    Box::new(ResearchAgent::new()),
    Box::new(ResearchAgent::new()),
    Box::new(ResearchAgent::new()),
];

let orchestrator = ParallelOrchestrator::new(agents);

// 并行分析多个股票
let tasks = vec![
    AgentInput {
        task: "analyze".to_string(),
        params: serde_json::json!({"symbol": "600519.SH"}),
    },
    AgentInput {
        task: "analyze".to_string(),
        params: serde_json::json!({"symbol": "000858.SZ"}),
    },
    AgentInput {
        task: "analyze".to_string(),
        params: serde_json::json!({"symbol": "300750.SZ"}),
    },
];

let outputs = orchestrator.orchestrate_parallel(tasks).await?;
```

#### HierarchicalOrrchestration (分层编排) ⭐

```rust
// 新增：分层编排模式
pub struct HierarchicalOrchestrator {
    supervisor: Box<dyn Agent>,
    sub_agents: HashMap<String, Vec<Box<dyn Agent>>>,
}

impl HierarchicalOrchestrator {
    pub fn new(supervisor: Box<dyn Agent>) -> Self {
        Self {
            supervisor,
            sub_agents: HashMap::new(),
        }
    }

    pub fn add_layer(&mut self, layer_name: &str, agents: Vec<Box<dyn Agent>>) {
        self.sub_agents.insert(layer_name.to_string(), agents);
    }

    pub async fn orchestrate(&self, input: AgentInput) -> Result<AgentOutput> {
        // 1. Supervisor向各层Agent分发任务
        let mut layer_outputs = HashMap::new();

        for (layer_name, agents) in &self.sub_agents {
            let mut layer_results = Vec::new();

            for agent in agents {
                let output = agent.execute(input.clone()).await?;
                layer_results.push(output);
            }

            layer_outputs.insert(layer_name.clone(), layer_results);
        }

        // 2. Supervisor综合各层结果，做出决策
        let supervisor_input = AgentInput {
            task: "synthesize".to_string(),
            params: serde_json::json!({
                "layer_outputs": layer_outputs,
                "original_input": input,
            }),
        };

        let final_output = self.supervisor.execute(supervisor_input).await?;

        Ok(final_output)
    }
}

// 使用示例
let mut orchestrator = HierarchicalOrchestrator::new(Box::new(SupervisorAgent::new()));

// 数据层
orchestrator.add_layer("data", vec![
    Box::new(DataAgent::new()),
    Box::new(RealtimeDataAgent::new()),
]);

// 分析层
orchestrator.add_layer("analysis", vec![
    Box::new(ResearchAgent::new()),
    Box::new(TechnicalAgent::new()),
    Box::new(FundamentalAgent::new()),
]);

// 决策层
orchestrator.add_layer("decision", vec![
    Box::new(StrategyAgent::new()),
    Box::new(RiskAgent::new()),
]);

let output = orchestrator.orchestrate(input).await?;
```

---

## 🏦 Phase 3: 券商API集成 (4周)

### 目标

**对接主流券商交易API，实现实盘交易能力**

### 3.1 券商API抽象层

**实现**: `investintel-agent/ashare/brokers/mod.rs`

```rust
/// 券商客户端Trait
#[async_trait]
pub trait BrokerClient: Send + Sync {
    /// 券商名称
    fn name(&self) -> &str;

    /// 下单
    async fn place_order(&self, order: AShareOrderRequest) -> Result<AShareOrderResponse>;

    /// 撤单
    async fn cancel_order(&self, order_id: &str) -> Result<()>;

    /// 查询订单
    async fn query_order(&self, order_id: &str) -> Result<OrderStatus>;

    /// 查询持仓
    async fn query_positions(&self) -> Result<Vec<ASharePosition>>;

    /// 查询资金
    async fn query_account(&self) -> Result<AccountInfo>;

    /// 查询行情
    async fn query_quotes(&self, symbols: &[String]) -> Result<Vec<AShareQuote>>;
}
```

### 3.2 具体券商实现

#### 3.2.1 国金证券 (QMT/PTrade)

**实现**: `investintel-agent/ashare/brokers/guojin.rs`

```rust
/// 国金证券QMT客户端
pub struct GuojinQmtClient {
    api_key: String,
    client: Client,
    base_url: String,
}

#[async_trait]
impl BrokerClient for GuojinQmtClient {
    fn name(&self) -> &str {
        "GuojinQMT"
    }

    async fn place_order(&self, order: AShareOrderRequest) -> Result<AShareOrderResponse> {
        let endpoint = "/api/v1/order";

        let params = serde_json::json!({
            "symbol": order.symbol,
            "side": order.side,
            "price": order.price,
            "quantity": order.quantity,
            "order_type": order.order_type,
        });

        let response = self.client
            .post(format!("{}{}", self.base_url, endpoint))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&params)
            .send()
            .await?
            .error_for_context()?
            .json::<serde_json::Value>()
            .await?;

        // 解析响应
        Ok(AShareOrderResponse {
            order_id: response["order_id"].as_str().unwrap().to_string(),
            status: OrderStatus::from_str(response["status"].as_str().unwrap())?,
            message: response["message"].as_str().unwrap().to_string(),
        })
    }

    // ... 其他方法实现
}
```

#### 3.2.2 华泰证券

**实现**: `investintel-agent/ashare/brokers/huatai.rs`

```rust
pub struct HuataiClient {
    client_id: String,
    client_secret: String,
    client: Client,
}

#[async_trait]
impl BrokerClient for HuataiClient {
    fn name(&self) -> &str {
        "Huatai"
    }

    // ... 实现类似
}
```

#### 3.2.3 东方财富

**实现**: `investintel-agent/ashare/brokers/eastmoney.rs`

```rust
pub struct EastmoneyClient {
    // ...
}
```

#### 3.2.4 通用接口

**实现**: `investintel-agent/ashare/brokers/generic.rs`

```rust
/// 模拟交易客户端（用于测试）
pub struct SimulatedBroker {
    initial_cash: f64,
    positions: Arc<RwLock<HashMap<String, ASharePosition>>>,
    cash: Arc<RwLock<f64>>,
}

#[async_trait]
impl BrokerClient for SimulatedBroker {
    fn name(&self) -> &str {
        "Simulated"
    }

    async fn place_order(&self, order: AShareOrderRequest) -> Result<AShareOrderResponse> {
        // 模拟撮合成交
        let order_id = Uuid::new_v4().to_string();

        match order.side {
            OrderSide::Buy => {
                let required_cash = order.price * order.quantity as f64;
                let mut cash = self.cash.write().await;

                if *cash < required_cash {
                    return Ok(AShareOrderResponse {
                        order_id: order_id.clone(),
                        status: OrderStatus::Rejected,
                        message: "资金不足".to_string(),
                    });
                }

                *cash -= required_cash;

                // 更新持仓
                let mut positions = self.positions.write().await;
                let position = positions.entry(order.symbol.clone())
                    .or_insert_with(|| ASharePosition {
                        symbol: order.symbol.clone(),
                        shares: 0,
                        buy_price: 0.0,
                        buy_date: Utc::now().date_naive(),
                        market_value: 0.0,
                        pnl: 0.0,
                        pnl_pct: 0.0,
                        can_sell: false,
                    });

                // 更新持仓成本
                let total_cost = position.buy_price * position.shares as f64 + required_cash;
                position.shares += order.quantity;
                position.buy_price = total_cost / position.shares as f64;
            }
            OrderSide::Sell => {
                // 卖出逻辑
                let mut positions = self.positions.write().await;

                if let Some(position) = positions.get_mut(&order.symbol) {
                    if position.shares < order.quantity {
                        return Ok(AShareOrderResponse {
                            order_id: order_id.clone(),
                            status: OrderStatus::Rejected,
                            message: "持仓不足".to_string(),
                        });
                    }

                    position.shares -= order.quantity;
                    let proceeds = order.price * order.quantity as f64;

                    let mut cash = self.cash.write().await;
                    *cash += proceeds;
                }
            }
        }

        Ok(AShareOrderResponse {
            order_id,
            status: OrderStatus::Filled,
            message: "成交".to_string(),
        })
    }

    // ... 其他方法
}
```

### 3.3 券商选择策略

**实现**: `investintel-agent/ashare/broker_selector.rs`

```rust
/// 券商选择器
pub struct BrokerSelector {
    brokers: Vec<Box<dyn BrokerClient>>,
    selection_strategy: SelectionStrategy,
}

pub enum SelectionStrategy {
    /// 佣金优先
    CommissionFirst,
    /// 滑点优先
    SlippageFirst,
    /// 成交率优先
    FillRateFirst,
    /// 负载均衡
    LoadBalancing,
    /// 自定义评分
    CustomScoring(Box<dyn Fn(&BrokerContext) -> f64 + Send + Sync>),
}

impl BrokerSelector {
    /// 选择最优券商
    pub fn select_broker(
        &self,
        order: &AShareOrderRequest,
    ) -> Result<&Box<dyn BrokerClient>> {
        match &self.selection_strategy {
            SelectionStrategy::CommissionFirst => {
                // 选择佣金最低的
                self.brokers.iter()
                    .min_by(|a, b| {
                        a.get_commission_rate().unwrap_or(0.0)
                            .partial_cmp(&b.get_commission_rate().unwrap_or(0.0))
                            .unwrap()
                    })
                    .ok_or_else(|| anyhow!("No available broker"))
            }
            SelectionStrategy::LoadBalancing => {
                // 负载均衡
                self.brokers.iter()
                    .min_by_key(|b| b.get_active_orders().unwrap_or(0))
                    .ok_or_else(|| anyhow!("No available broker"))
            }
            SelectionStrategy::CustomScoring(scoring_fn) => {
                // 自定义评分
                self.brokers.iter()
                    .max_by(|a, b| {
                        scoring_fn(&a.get_context().unwrap())
                            .partial_cmp(&scoring_fn(&b.get_context().unwrap()))
                            .unwrap()
                    })
                    .ok_or_else(|| anyhow!("No available broker"))
            }
            _ => {
                // 默认第一个
                self.brokers.first()
                    .ok_or_else(|| anyhow!("No available broker"))
            }
        }
    }
}
```

---

## 🧠 Phase 4: AI投资决策系统 (6周)

### 目标

**从策略执行升级为AI自主决策**

### 4.1 Self-Reflection机制 ⭐

**基于**: TradingGroup论文（arXiv 2508.17565v1）

**实现**: `investintel-agent/ai/self_reflection.rs`

```rust
/// 自反思机制
pub struct SelfReflection {
    decision_history: Arc<RwLock<Vec<DecisionRecord>>>,
    performance_tracker: Arc<PerformanceTracker>,
}

#[derive(Debug, Clone)]
pub struct DecisionRecord {
    pub timestamp: DateTime<Utc>,
    pub decision: TradingDecision,
    pub confidence: f64,
    pub reasoning: String,
    pub outcome: Option<DecisionOutcome>,
    pub reflection: Option<Reflection>,
}

#[derive(Debug, Clone)]
pub struct Reflection {
    pub what_went_right: Vec<String>,
    pub what_went_wrong: Vec<String>,
    pub what_to_improve: Vec<String>,
    pub adjusted_confidence: f64,
}

impl SelfReflection {
    /// 记录决策
    pub async fn record_decision(&self, decision: TradingDecision, reasoning: String) {
        let record = DecisionRecord {
            timestamp: Utc::now(),
            decision,
            confidence: 0.0,
            reasoning,
            outcome: None,
            reflection: None,
        };

        let mut history = self.decision_history.write().await;
        history.push(record);
    }

    /// 更新决策结果
    pub async fn update_outcome(&self, decision_id: &str, outcome: DecisionOutcome) {
        let mut history = self.decision_history.write().await;

        if let Some(record) = history.iter_mut().find(|r| r.decision.id == decision_id) {
            record.outcome = Some(outcome.clone());

            // 生成反思
            let reflection = self.generate_reflection(record, &outcome);
            record.reflection = Some(reflection);

            // 调整后续决策的置信度
            self.adjust_future_confidence(&reflection).await;
        }
    }

    /// 生成反思
    fn generate_reflection(&self, record: &DecisionRecord, outcome: &DecisionOutcome) -> Reflection {
        let mut what_went_right = Vec::new();
        let mut what_went_wrong = Vec::new();
        let mut what_to_improve = Vec::new();

        // 分析决策的正确性
        let correct = match outcome {
            DecisionOutcome::Profit { pnl } if *pnl > 0.0 => true,
            DecisionOutcome::Loss { pnl } if *pnl < 0.0 => false,
            _ => false,
        };

        if correct {
            what_went_right.push("预测准确".to_string());
            what_went_right.push(format!("置信度{}合理", record.confidence));
        } else {
            what_went_wrong.push("预测失误".to_string());
            what_to_improve.push("需要重新评估决策模型".to_string());
        }

        // 分析推理过程
        if record.reasoning.contains("技术分析") {
            if correct {
                what_went_right.push("技术分析方法有效".to_string());
            } else {
                what_went_wrong.push("技术分析可能失效".to_string());
                what_to_improve.push("结合基本面分析".to_string());
            }
        }

        let adjusted_confidence = if correct {
            (record.confidence + 0.05).min(1.0)
        } else {
            (record.confidence - 0.1).max(0.0)
        };

        Reflection {
            what_went_right,
            what_went_wrong,
            what_to_improve,
            adjusted_confidence,
        }
    }

    /// 调整未来决策的置信度
    async fn adjust_future_confidence(&self, reflection: &Reflection) {
        // 更新全局置信度调整
        // TODO: 实现更复杂的置信度调整逻辑
    }
}
```

### 4.2 AI决策引擎

**实现**: `investintel-agent/ai/decision_engine.rs`

```rust
/// AI决策引擎
pub struct AIDecisionEngine {
    claude_client: Arc<ClaudeClient>,
    agents: HashMap<String, Box<dyn Agent>>,
    self_reflection: Arc<SelfReflection>,
    knowledge_base: Arc<RwLock<KnowledgeBase>>,
}

#[derive(Debug, Clone)]
pub struct TradingDecision {
    pub id: String,
    pub action: TradingAction,
    pub symbol: String,
    pub quantity: u64,
    pub price: Option<f64>,
    pub confidence: f64,
    pub reasoning: String,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone)]
pub enum TradingAction {
    Buy,
    Sell,
    Hold,
    ReducePosition,
    IncreasePosition,
}

#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Extreme,
}

impl AIDecisionEngine {
    /// 做出交易决策
    pub async fn make_decision(&self, context: &MarketContext) -> Result<TradingDecision> {
        // 1. 收集各Agent的分析
        let agent_outputs = self.collect_agent_opinions(context).await?;

        // 2. 查询知识库（类似决策）
        let historical_decisions = self.query_knowledge_base(context).await?;

        // 3. 使用Claude综合分析
        let prompt = self.build_decision_prompt(context, &agent_outputs, &historical_decisions);

        let messages = query(&prompt, None).await?;

        // 4. 解析Claude的决策
        let decision = self.parse_decision(&messages, context)?;

        // 5. 应用Self-Reflection调整
        let adjusted_decision = self.apply_reflection_adjustment(decision).await?;

        // 6. 记录决策
        self.self_reflection.record_decision(
            adjusted_decision.clone(),
            adjusted_decision.reasoning.clone()
        ).await;

        Ok(adjusted_decision)
    }

    /// 构建决策提示词
    fn build_decision_prompt(
        &self,
        context: &MarketContext,
        agent_outputs: &HashMap<String, AgentOutput>,
        historical_decisions: &[DecisionRecord],
    ) -> String {
        format!(
            "你是AI投资决策引擎。请基于以下信息做出投资决策：

=== 当前市场状态 ===
市场: {}
时间: {}
指数: {}

=== 各专业Agent分析 ===

【数据采集Agent】
{}

【研究分析Agent】
{}

【策略制定Agent】
{}

【风险控制Agent】
{}

=== 历史类似决策 ===
{}

=== 决策要求 ===
请做出以下决策：
1. 交易动作（买入/卖出/持有）
2. 目标股票
3. 交易数量
4. 目标价格（可选）
5. 置信度（0.0-1.0）
6. 风险等级（低/中/高/极高）
7. 决策理由（详细说明）

请以JSON格式返回：
{{
    \"action\": \"Buy\",
    \"symbol\": \"600519.SH\",
    \"quantity\": 100,
    \"price\": 1800.0,
    \"confidence\": 0.75,
    \"risk_level\": \"Medium\",
    \"reasoning\": \"详细的理由\"
}}
",
            context.market,
            context.timestamp,
            context.index,

            agent_outputs.get("DataCollector")
                .map(|o| o.content.as_str())
                .unwrap_or("无"),

            agent_outputs.get("ResearchAnalyst")
                .map(|o| o.content.as_str())
                .unwrap_or("无"),

            agent_outputs.get("StrategyMaker")
                .map(|o| o.content.as_str())
                .unwrap_or("无"),

            agent_outputs.get("RiskController")
                .map(|o| o.content.as_str())
                .unwrap_or("无"),

            historical_decisions.iter()
                .take(3)
                .map(|r| format!("决策: {}, 结果: {:?}", r.decision.action, r.outcome))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// 解析Claude的决策
    fn parse_decision(&self, messages: &[Message], context: &MarketContext) -> Result<TradingDecision> {
        for message in messages {
            if let Message::Assistant(msg) = message {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        // 提取JSON
                        if let Some(json_start) = text.text.find('{') {
                            if let Some(json_end) = text.text.rfind('}') {
                                let json_str = &text.text[json_start..=json_end];
                                let decision: TradingDecisionInput = serde_json::from_str(json_str)?;

                                return Ok(TradingDecision {
                                    id: Uuid::new_v4().to_string(),
                                    action: decision.action,
                                    symbol: decision.symbol,
                                    quantity: decision.quantity,
                                    price: decision.price,
                                    confidence: decision.confidence,
                                    reasoning: decision.reasoning,
                                    risk_level: decision.risk_level,
                                });
                            }
                        }
                    }
                }
            }
        }

        bail!("Failed to parse decision from Claude response")
    }

    /// 应用Self-Reflection调整
    async fn apply_reflection_adjustment(&self, decision: TradingDecision) -> Result<TradingDecision> {
        let history = self.decision_history.read().await;

        // 计算最近的平均表现
        let recent_decisions: Vec<_> = history.iter()
            .rev()
            .take(10)
            .collect();

        if recent_decisions.is_empty() {
            return Ok(decision);
        }

        let success_rate = recent_decisions.iter()
            .filter(|r| {
                r.outcome.as_ref()
                    .map(|o| matches!(o, DecisionOutcome::Profit { pnl: _ }))
                    .unwrap_or(false)
            })
            .count() as f64 / recent_decisions.len() as f64;

        // 根据成功率调整置信度
        let adjusted_confidence = if success_rate < 0.5 {
            // 成功率低，降低置信度
            decision.confidence * 0.8
        } else if success_rate > 0.7 {
            // 成功率高，提高置信度
            (decision.confidence * 1.1).min(1.0)
        } else {
            decision.confidence
        };

        Ok(TradingDecision {
            confidence: adjusted_confidence,
            ..decision
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct TradingDecisionInput {
    action: TradingAction,
    symbol: String,
    quantity: u64,
    price: Option<f64>,
    confidence: f64,
    risk_level: RiskLevel,
    reasoning: String,
}
```

### 4.3 知识库

**实现**: `investintel-agent/ai/knowledge_base.rs`

```rust
/// 知识库
pub struct KnowledgeBase {
    decisions: Arc<RwLock<Vec<DecisionRecord>>>,
    market_patterns: Arc<RwLock<Vec<MarketPattern>>>,
    strategies: Arc<RwLock<HashMap<String, StrategyPerformance>>>,
}

#[derive(Debug, Clone)]
pub struct MarketPattern {
    pub name: String,
    pub description: String,
    pub conditions: serde_json::Value,
    pub historically_success_rate: f64,
    pub sample_size: usize,
}

impl KnowledgeBase {
    /// 查询相似的历史决策
    pub async fn query_similar_decisions(
        &self,
        context: &MarketContext,
        limit: usize,
    ) -> Vec<DecisionRecord> {
        let decisions = self.decisions.read().await;

        // TODO: 实现相似度计算
        // 简化版本：返回最近的决策
        decisions.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// 记录市场模式
    pub async fn record_pattern(&self, pattern: MarketPattern) {
        let mut patterns = self.market_patterns.write().await;
        patterns.push(pattern);
    }

    /// 更新策略性能
    pub async fn update_strategy_performance(
        &self,
        strategy_name: &str,
        outcome: &DecisionOutcome,
    ) {
        let mut strategies = self.strategies.write().await;

        let performance = strategies.entry(strategy_name.to_string())
            .or_insert_with(|| StrategyPerformance {
                name: strategy_name.to_string(),
                total_decisions: 0,
                successful_decisions: 0,
                success_rate: 0.0,
                total_pnl: 0.0,
            });

        performance.total_decisions += 1;
        match outcome {
            DecisionOutcome::Profit { pnl } => {
                performance.successful_decisions += 1;
                performance.total_pnl += pnl;
            }
            DecisionOutcome::Loss { pnl } => {
                performance.total_pnl += pnl;
            }
        }

        performance.success_rate = performance.successful_decisions as f64
            / performance.total_decisions as f64;
    }
}
```

---

## 📊 Phase 5: 本土化数据源 (4周)

### 目标

**建立A股市场完整的数据基础设施**

### 5.1 实时行情数据

**数据源对比**:

| 数据源 | 延迟 | 覆盖 | 费用 | 推荐度 |
|--------|------|------|------|--------|
| **Tushare** | 低 | 全市场 | 免费+付费 | ⭐⭐⭐⭐⭐ |
| **Wind** | 极低 | 全市场 | 高 | ⭐⭐⭐⭐ |
| **同花顺iFinD** | 低 | 全市场 | 中 | ⭐⭐⭐⭐ |
| **东方财富** | 中 | 全市场 | 免费 | ⭐⭐⭐ |
| **新浪财经API** | 中 | 全市场 | 免费 | ⭐⭐⭐ |
| **腾讯财经API** | 中 | 全市场 | 免费 | ⭐⭐⭐ |

**推荐**: Tushare (免费额度+完整API)

**实现**: `investintel-agent/ashare/data/tushare.rs`

```rust
/// Tushare数据客户端
pub struct TushareClient {
    api_token: String,
    client: Client,
    base_url: String,
}

impl TushareClient {
    pub fn new(api_token: String) -> Self {
        Self {
            api_token,
            client: Client::new(),
            base_url: "http://api.tushare.pro".to_string(),
        }
    }

    /// 获取实时行情
    pub async fn get_quotes(&self, symbols: &[String]) -> Result<Vec<AShareQuote>> {
        let symbols_str = symbols.join(",");

        let params = serde_json::json!({
            "api_name": "realtime",
            "token": self.api_token,
            "params": {
                "list": symbols_str,
            },
            "fields": "symbol,name,last_price,upper_limit,lower_limit,volume,amount,bid1,bid2,bid3,bid4,bid5,ask1,ask2,ask3,ask4,ask5"
        });

        let response = self.client
            .post(&self.base_url)
            .json(&params)
            .send()
            .await?
            .error_for_context()?
            .json::<serde_json::Value>()
            .await?;

        let items = response["data"]["items"].as_array()
            .ok_or_else(|| anyhow!("Invalid response format"))?;

        let mut quotes = Vec::new();
        for item in items {
            let quote = self.parse_quote_item(item)?;
            quotes.push(quote);
        }

        Ok(quotes)
    }

    /// 获取K线数据
    pub async fn get_bars(
        &self,
        symbol: &str,
        start_date: &str,
        end_date: &str,
        freq: &str,  // 1min/5min/15min/30min/60min/daily/weekly/monthly
    ) -> Result<Vec<BarData>> {
        let params = serde_json::json!({
            "api_name": "bars",
            "token": self.api_token,
            "params": {
                "symbol": symbol,
                "start_date": start_date,
                "end_date": end_date,
                "freq": freq,
            },
            "fields": "trade_date,open,high,low,close,volume,amount"
        });

        // ... 类似实现
        todo!()
    }

    fn parse_quote_item(&self, item: &serde_json::Value) -> Result<AShareQuote> {
        Ok(AShareQuote {
            symbol: item["symbol"].as_str().unwrap().to_string(),
            current_price: item["last_price"].as_f64().unwrap(),
            upper_limit: item["upper_limit"].as_f64().unwrap(),
            lower_limit: item["lower_limit"].as_f64().unwrap(),
            volume: item["volume"].as_u64().unwrap(),
            amount: item["amount"].as_f64().unwrap(),
            bid_price: vec![
                item["bid1"].as_f64().unwrap_or(0.0),
                item["bid2"].as_f64().unwrap_or(0.0),
                item["bid3"].as_f64().unwrap_or(0.0),
                item["bid4"].as_f64().unwrap_or(0.0),
                item["bid5"].as_f64().unwrap_or(0.0),
            ],
            ask_price: vec![
                item["ask1"].as_f64().unwrap_or(0.0),
                item["ask2"].as_f64().unwrap_or(0.0),
                item["ask3"].as_f64().unwrap_or(0.0),
                item["ask4"].as_f64().unwrap_or(0.0),
                item["ask5"].as_f64().unwrap_or(0.0),
            ],
            bid_volume: vec![
                item["bid1_volume"].as_u64().unwrap_or(0),
                item["bid2_volume"].as_u64().unwrap_or(0),
                item["bid3_volume"].as_u64().unwrap_or(0),
                item["bid4_volume"].as_u64().unwrap_or(0),
                item["bid5_volume"].as_u64().unwrap_or(0),
            ],
            ask_volume: vec![
                item["ask1_volume"].as_u64().unwrap_or(0),
                item["ask2_volume"].as_u64().unwrap_or(0),
                item["ask3_volume"].as_u64().unwrap_or(0),
                item["ask4_volume"].as_u64().unwrap_or(0),
                item["ask5_volume"].as_u64().unwrap_or(0),
            ],
            timestamp: Utc::now(),
        })
    }
}
```

### 5.2 财务数据

**Tushare财务数据API**:

```rust
impl TushareClient {
    /// 获取财务数据
    pub async fn get_financials(
        &self,
        symbol: &str,
        period: &str,  // 如：20241231
    ) -> Result<FinancialData> {
        let params = serde_json::json!({
            "api_name": "fina_indicator",
            "token": self.api_token,
            "params": {
                "ts_code": symbol,
                "period": period,
            },
            "fields": "ts_code,ann_date,f_end_date,report_type,update_flag,roe_a,roe_waa,roe_dt,netprofit_margin,grossprofit_margin,cost_to_income,npta_to_or")
        });

        // ... 实现
        todo!()
    }

    /// 获取资产负债表
    pub async fn get_balance_sheet(&self, symbol: &str, period: &str) -> Result<BalanceSheet> {
        // ... 实现
        todo!()
    }

    /// 获取现金流量表
    pub async fn get_cash_flow(&self, symbol: &str, period: &str) -> Result<CashFlow> {
        // ... 实现
        todo!()
    }
}
```

### 5.3 研报数据

**东方财富研报API**:

```rust
/// 东方财富研报客户端
pub struct EastmoneyReportClient {
    client: Client,
}

impl EastmoneyReportClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// 搜索研报
    pub async fn search_reports(
        &self,
        keyword: &str,
        page: usize,
    ) -> Result<Vec<ResearchReport>> {
        let url = format!(
            "https://datacenter-web.eastmoney.com/api/data/v1/get?report=ResearchReport&rptDataListSearch&pageNo={}&keyword={}",
            page, keyword
        );

        let response = self.client
            .get(&url)
            .send()
            .await?
            .error_for_context()?
            .json::<serde_json::Value>()
            .await?;

        let reports = response["data"]
            .as_array()
            .ok_or_else(|| anyhow!("Invalid response"))?
            .iter()
            .map(|item| self.parse_report(item))
            .collect::<Result<Vec<_>>>()?;

        Ok(reports)
    }

    fn parse_report(&self, item: &serde_json::Value) -> Result<ResearchReport> {
        Ok(ResearchReport {
            title: item["title_cn"].as_str().unwrap().to_string(),
            author: item["author"].as_str().unwrap().to_string(),
            publish_date: item["publish_date"].as_str().unwrap().to_string(),
            org: item["org"].as_str().unwrap().to_string(),
            rating: item["rating"].as_str().unwrap().to_string(),
            target_price: item["target_price"].as_f64().ok(),
            summary: item["summary"].as_str().map(|s| s.to_string()),
            url: item["url"].as_str().unwrap().to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ResearchReport {
    pub title: String,
    pub author: String,
    pub publish_date: String,
    pub org: String,
    pub rating: String,
    pub target_price: Option<f64>,
    pub summary: Option<String>,
    pub url: String,
}
```

### 5.4 宏观经济数据

```rust
/// 宏观经济数据
pub struct MacroDataClient {
    tushare: Arc<TushareClient>,
}

impl MacroDataClient {
    /// 获取GDP数据
    pub async fn get_gdp(&self, year: i32) -> Result<GDPData> {
        // ... 实现
        todo!()
    }

    /// 获取CPI数据
    pub async fn get_cpi(&self, start_date: &str, end_date: &str) -> Result<Vec<CPIData>> {
        // ... 实现
        todo!()
    }

    /// 获取PMI数据
    pub async fn get_pmi(&self, start_date: &str, end_date: &str) -> Result<Vec<PMIData>> {
        // ... 实现
        todo!()
    }
}
```

---

## 🚀 Phase 6: 高级Agent扩展 (6周)

### 目标

**扩展专业Agent，覆盖更多投资场景**

### 6.1 新增Agent列表

#### 1. TechnicalAnalysisAgent (技术分析Agent)

```rust
pub struct TechnicalAnalysisAgent {
    indicators: HashMap<String, Box<dyn TechnicalIndicator>>,
}

#[async_trait]
impl Agent for TechnicalAnalysisAgent {
    fn name(&self) -> &str {
        "TechnicalAnalyst"
    }

    fn description(&self) -> &str {
        "负责技术分析、指标计算、形态识别"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 计算MACD、RSI、布林带等指标
    }
}
```

#### 2. FundamentalAnalysisAgent (基本面分析Agent)

```rust
pub struct FundamentalAnalysisAgent {
    claude_client: Arc<ClaudeClient>,
}

#[async_trait]
impl Agent for FundamentalAnalysisAgent {
    fn name(&self) -> &str {
        "FundamentalAnalyst"
    }

    fn description(&self) -> &str {
        "负责基本面分析、财务分析、估值分析"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 分析PE、PB、ROE、现金流等
    }
}
```

#### 3. NewsAnalysisAgent (新闻分析Agent)

```rust
pub struct NewsAnalysisAgent {
    claude_client: Arc<ClaudeClient>,
    news_sources: Vec<Box<dyn NewsSource>>,
}

#[async_trait]
impl Agent for NewsAnalysisAgent {
    fn name(&self) -> &str {
        "NewsAnalyst"
    }

    fn description(&self) -> &str {
        "负责新闻分析、事件跟踪、影响评估"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 分析新闻对股价的影响
    }
}
```

#### 4. SectorRotationAgent (行业轮动Agent)

```rust
pub struct SectorRotationAgent {
    sector_data: HashMap<String, SectorData>,
}

#[async_trait]
impl Agent for SectorRotationAgent {
    fn name(&self) -> &str {
        "SectorRotator"
    }

    fn description(&self) -> &str {
        "负责行业轮动策略、板块选择"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 分析行业轮动机会
    }
}
```

#### 5. EventDrivenAgent (事件驱动Agent)

```rust
pub struct EventDrivenAgent {
    event_calendar: Arc<RwLock<Vec<MarketEvent>>>,
}

#[async_trait]
impl Agent for EventDrivenAgent {
    fn name(&self) -> &str {
        "EventDriven"
    }

    fn description(&self) -> &str {
        "负责事件驱动策略、财报日历跟踪"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 分析财报、公告等事件
    }
}
```

### 6.2 Agent能力扩展

**每个Agent增加以下能力**:

1. **Learning Capability (学习能力)** - 从历史决策中学习
2. **Adaptation Capability (适应能力)** - 根据市场变化调整策略
3. **Collaboration Capability (协作能力)** - 与其他Agent协同工作
4. **Self-Monitoring Capability (自监控能力)** - 监控自身性能

```rust
#[async_trait]
pub trait AdvancedAgent: Agent {
    /// 学习能力
    async fn learn_from_history(&self, history: &[DecisionRecord]) -> Result<LearningOutcome>;

    /// 适应能力
    async fn adapt_to_market(&self, market_state: &MarketState) -> Result<AdaptationResult>;

    /// 协作能力
    async fn collaborate(&self, other_agent: &dyn Agent, task: &str) -> Result<AgentOutput>;

    /// 自监控能力
    async fn monitor_performance(&self) -> Result<PerformanceMetrics>;
}
```

---

## 📈 成功指标

### 技术指标

| 指标 | Plan3 | Plan4目标 | 提升 |
|------|-------|----------|------|
| **Claude SDK集成** | 100% | 100% | 保持 |
| **Agent数量** | 25 Skills | 10 Agents + 25 Skills | 135% |
| **MCP工具** | 135+ | 200+ | 48% ↑ |
| **支持市场** | 加密货币 | A股 | 新增 |
| **券商集成** | 0 | 3+ | 新增 |
| **决策自动化** | 策略执行 | AI自主决策 | 质的飞跃 |
| **自反思能力** | 无 | 有 | 新增 |

### 性能指标

| 指标 | 目标 |
|------|------|
| **决策延迟** | <500ms |
| **数据延迟** | <100ms (A股实时行情) |
| **订单延迟** | <300ms (A股) |
| **AI决策准确率** | >60% |
| **回测覆盖率** | 100% |

### 业务指标

| 指标 | 目标 |
|------|------|
| **年化收益率** | >15% |
| **最大回撤** | <20% |
| **夏普比率** | >1.5 |
| **胜率** | >55% |

---

## 🔧 技术栈

### 核心依赖

```toml
[dependencies]
# Claude Agent SDK (核心)
claude-agent-sdk-rs = { path = "..", features = ["orchestration"] }

# 异步运行时
tokio = { version = "1.48", features = ["full"] }

# 网络
reqwest = { version = "0.12", features = ["json"] }

# 数据存储
libsql = "0.5"      # 本地数据库 (200ns查询)
duckdb = "0.11"     # 分析查询

# 机器学习
tch = "0.15"       # PyTorch绑定 (LSTM, DQN)

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 错误处理
anyhow = "1.0"
async-trait = "0.1"

# 时间
chrono = { version = "0.4", features = ["serde"] }

# 加密
hmac = "0.12"
sha2 = "0.10"

# 工具
uuid = { version = "1.6", features = ["v4", "serde"] }
```

---

## 📚 参考资源

### Claude Agent SDK
- [Building agents with Claude Agent SDK](https://www.anthropic.com/engineering/building-agents-with-the-claude-agent-sdk)
- [Agent SDK overview](https://platform.claude.com/docs/en/agent-sdk/overview)
- [Claude Agent Skills Deep Dive](https://leehanchung.github.io/blogs/2025/10/26/claude-skills-deep-dive/)

### Multi-Agent系统
- [TradingGroup: A Multi-Agent Trading System (arXiv 2508.17565v1)](https://arxiv.org/html/2508.17565v1)
- [AutoTrader-AgentEdge (GitHub)](https://github.com/iAmGiG/AutoTrader-AgentEdge)
- [Multi-Agent Framework for Quantitative Finance (ACL 2025)](https://aclanthology.org/2025.emnlp-main.55.pdf)

### A股量化平台
- [BigQuant - AI量化交易平台](https://bigquant.com/)
- [聚宽量化平台](https://www.joinquant.com/)
- [优矿量化平台](https://uqer.datayes.com/)
- [掘金量化平台](https://www.myquant.cn/)

### A股数据API
- [Tushare数据](https://tushare.pro/)
- [Wind金融终端](https://www.wind.com.cn/)
- [同花顺iFinD](http://data.10jqka.com.cn/)

### A股交易规则
- [A股T+1交易规则详解](https://www.9db.com/user/blog/A305AFD6-8937-46B5-8781-9C7AD7DC30E0/)
- [涨跌停限制规则](https://finance.sina.com.cn/money/future/roll/2020-07-24/doc-iivhuipn4771923.shtml)

---

## 🎯 关键差异 vs Plan3

### 架构差异

| 维度 | Plan3 | Plan4 |
|------|-------|-------|
| **市场** | 加密货币 (24/7) | A股 (T+1, 涨跌停) |
| **Agent** | 25 Skills (平面) | 10 Agents + 25 Skills (分层) |
| **决策** | 策略执行 | AI自主决策 |
| **自反思** | 无 | 有 ⭐ |
| **编排** | Sequential/Parallel | + Hierarchical ⭐ |
| **交易** | Binance/OKX API | 券商API (QMT/PTrade等) |
| **数据** | WebSocket | Tushare/Wind等 |

### 技术升级

1. ✅ **Multi-Agent分层架构** - 从Skills平面化到Agent分层
2. ✅ **Hierarchical编排** - 新增分层编排模式
3. ✅ **Self-Reflection机制** - AI自主学习和改进
4. ✅ **A股交易规则引擎** - T+1、涨跌停等
5. ✅ **券商API集成** - 支持实盘交易

---

## 📝 实施时间表

### 总体时间表: 28周 (约7个月)

| Phase | 周期 | 起始日期 | 完成日期 |
|-------|------|----------|----------|
| **Phase 1** | 4周 | Week 1 | Week 4 |
| **Phase 2** | 6周 | Week 5 | Week 10 |
| **Phase 3** | 4周 | Week 11 | Week 14 |
| **Phase 4** | 6周 | Week 15 | Week 20 |
| **Phase 5** | 4周 | Week 21 | Week 24 |
| **Phase 6** | 6周 | Week 25 | Week 30 |

### 里程碑

- **Week 4**: A股基础架构完成，可以模拟A股交易
- **Week 10**: Multi-Agent架构完成，Agent可以协作
- **Week 14**: 券商API集成完成，可以实盘交易
- **Week 20**: AI决策系统完成，可以自主决策
- **Week 24**: 本土化数据源完成，数据全面
- **Week 30**: 高级Agent完成，系统功能完整

---

## 💡 最佳实践建议

### 1. 渐进式实施

**不要试图一次性完成所有Phase**:

1. ✅ **优先实施Phase 1** - 建立A股基础架构
2. ✅ **然后Phase 3** - 对接券商API，实现实盘
3. ✅ **然后Phase 2** - 升级Multi-Agent架构
4. ✅ **然后Phase 4** - AI决策系统
5. ✅ **最后Phase 5,6** - 完善数据和高级功能

### 2. 保持Plan3优势

**Plan4不是替代Plan3，而是增强Plan3**:

- ✅ **保留Plan3的所有功能**
- ✅ **复用Plan3的25个Skills**
- ✅ **复用Plan3的135+个MCP工具**
- ✅ **复用Plan3的LSTM/DQN模型**

### 3. 专注A股特殊性

**A股的特殊性是我们的核心优势**:

- ✅ **T+1规则引擎** - 国内平台少有
- ✅ **涨跌停限制** - 风控更精确
- ✅ **本土化数据** - Tushare等
- ✅ **券商API** - 实盘交易能力

### 4. Multi-Agent是核心竞争力

**2025年的趋势是Multi-Agent**:

- ✅ **10个专业Agent** - 分工明确
- ✅ **Hierarchical编排** - 三层架构
- ✅ **Self-Reflection** - 持续学习
- ✅ **AI自主决策** - 不是简单执行策略

---

## 🎊 总结

**Plan4 = Plan3 + A股 + Multi-Agent + AI决策**

### 核心升级

1. ✅ **市场**: 加密货币 → A股
2. ✅ **架构**: Skills平面 → Agent分层
3. ✅ **决策**: 策略执行 → AI自主决策
4. ✅ **能力**: 交易工具 → 智能投资系统
5. ✅ **数据**: WebSocket → A股本土化数据

### 最终目标

**打造国内首个基于Claude Agent SDK的A股AI量化投资平台**

- ✅ **技术领先**: 唯一100% Claude SDK + Rust平台
- ✅ **架构先进**: Multi-Agent + Self-Reflection
- ✅ **市场专注**: 深度适配A股特性
- ✅ **生产就绪**: 券商API + 风控完整

---

**文档版本**: 4.0
**创建日期**: 2026-01-10
**维护者**: InvestIntel AI Team

**立即开始Plan4实施，打造A股AI量化投资平台！** 🚀

---

**END OF PLAN4**
