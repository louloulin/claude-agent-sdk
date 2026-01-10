# Plan4 总览 - A股AI量化投资平台

**版本**: 4.0
**日期**: 2026-01-10
**状态**: ✅ **规划完成**

---

## 🎯 Plan4核心目标

**从加密货币交易平台进化为A股AI量化投资平台**

### 四大核心升级

1. ✅ **A股市场适配** - T+1、涨跌停、本土化数据
2. ✅ **Multi-Agent架构** - 10个专业Agent分层协作
3. ✅ **券商API集成** - 支持实盘交易
4. ✅ **AI投资决策** - 从策略执行到AI自主决策

---

## 📊 市场研究总结

### 国内AI量化平台对比

| 平台 | 优势 | 我们的优势 |
|------|------|-----------|
| **BigQuant** | AI技术、海量因子 | Rust性能(10-100x)、本地优先 |
| **聚宽** | 社区活跃、策略商城 | Claude SDK深度集成 |
| **优矿** | 数据丰富、Notebook | 真实券商API、Multi-Agent |
| **掘金** | 实盘交易、本地运行 | 更开放架构、AI自主决策 |

### 国际Multi-Agent交易系统

**2025年三大突破**:

1. **TradingGroup** (arXiv 2508.17565v1) - Self-reflective架构
2. **AutoTrader-AgentEdge** (GitHub) - Microsoft AutoGen，Production-ready
3. **Multi-Agent Framework** (ACL 2025) - 复杂量化任务集成

**我们的优势**: 基于Claude Agent SDK的统一架构

### A股市场特殊性

| 特性 | 规则 | 影响 |
|------|------|------|
| **T+1交易** | 当日买入次日才能卖 | 无真正高频策略 |
| **涨跌停限制** | 主板±10%、创业板±20% | 价格限制策略 |
| **交易时间** | 9:30-15:00 | 非24/7市场 |
| **最小单位** | 100股=1手 | 订单管理调整 |

**结论**: A股需要专门的量化策略架构

---

## 🏗️ Plan4架构设计

### vs Plan3架构对比

**Plan3架构**:
```
Claude Agent SDK
├─ 25 Agent Skills (平面化)
├─ 135+ MCP工具
└─ Binance/OKX交易
```

**Plan4架构升级**:
```
Claude Agent SDK
├─ Multi-Agent Orchestration (新增) ⭐
│  ├─ Sequential Orchestration
│  ├─ Parallel Orchestration
│  └─ Hierarchical Orchestration (新增) ⭐
├─ 10 Professional Agents (分层) ⭐
│  ├─ DataAgent (数据采集)
│  ├─ ResearchAgent (研究分析)
│  ├─ StrategyAgent (策略制定)
│  ├─ TradingAgent (交易执行)
│  ├─ RiskAgent (风险控制)
│  ├─ PortfolioAgent (组合管理)
│  ├─ SentimentAgent (情绪分析)
│  ├─ ReportAgent (报告生成)
│  ├─ MonitorAgent (实时监控)
│  └─ SupervisorAgent (总监督) ⭐
├─ 25 Agent Skills (保留)
├─ 200+ MCP工具 (扩展) ⭐
└─ A股券商API (全新) ⭐
```

### 关键创新

#### 1. Multi-Agent分层架构 ⭐

**从Skills平面化到Agent分层**:

- **DataAgent**: 负责所有数据采集
- **ResearchAgent**: 负责个股和行业研究
- **StrategyAgent**: 负责策略制定和回测
- **TradingAgent**: 负责订单执行
- **RiskAgent**: 负责风险控制
- **PortfolioAgent**: 负责组合优化
- **SentimentAgent**: 负责情绪分析
- **ReportAgent**: 负责报告生成
- **MonitorAgent**: 负责实时监控
- **SupervisorAgent**: 负责协调所有Agent，做最终决策 ⭐

#### 2. Hierarchical Orchestration (新增) ⭐

**三层编排模式**:

```rust
HierarchicalOrchestrator
├─ 数据层 (Data Layer)
│  ├─ DataAgent
│  └─ RealtimeDataAgent
├─ 分析层 (Analysis Layer)
│  ├─ ResearchAgent
│  ├─ TechnicalAgent
│  └─ FundamentalAgent
└─ 决策层 (Decision Layer)
   ├─ StrategyAgent
   └─ RiskAgent
```

#### 3. Self-Reflection机制 (新增) ⭐

**基于TradingGroup论文 (arXiv 2508.17565v1)**:

```rust
pub struct SelfReflection {
    decision_history: Vec<DecisionRecord>,
    performance_tracker: PerformanceTracker,
}

// 记录决策
record_decision(decision, reasoning)

// 更新结果
update_outcome(decision_id, outcome)

// 生成反思
generate_reflection(record, outcome)

// 调整未来置信度
adjust_future_confidence(reflection)
```

#### 4. AI决策引擎 (新增) ⭐

**从策略执行到AI自主决策**:

```rust
pub struct AIDecisionEngine {
    claude_client: ClaudeClient,
    agents: HashMap<String, Agent>,
    self_reflection: SelfReflection,
    knowledge_base: KnowledgeBase,
}

// AI自主决策
make_decision(context)
  ├─ collect_agent_opinions()  // 收集各Agent意见
  ├─ query_knowledge_base()    // 查询历史类似决策
  ├─ build_decision_prompt()   // 构建决策提示词
  ├─ parse_decision()          // 解析Claude决策
  └─ apply_reflection_adjustment()  // 应用自反思调整
```

---

## 📦 Phase实施规划

### 6个Phase, 30周, 约7个月

| Phase | 功能 | 周期 | 优先级 | 关键产出 |
|-------|------|------|--------|----------|
| **Phase 1** | A股市场基础架构 | 4周 | ⭐⭐⭐⭐⭐ | T+1规则、涨跌停、持仓管理 |
| **Phase 2** | Multi-Agent架构升级 | 6周 | ⭐⭐⭐⭐⭐ | 10个专业Agent、3种编排模式 |
| **Phase 3** | 券商API集成 | 4周 | ⭐⭐⭐⭐⭐ | QMT/PTrade、华泰、东方财富 |
| **Phase 4** | AI投资决策系统 | 6周 | ⭐⭐⭐⭐ | Self-Reflection、AI决策引擎 |
| **Phase 5** | 本土化数据源 | 4周 | ⭐⭐⭐⭐ | Tushare、Wind、研报、宏观数据 |
| **Phase 6** | 高级Agent扩展 | 6周 | ⭐⭐⭐ | 5个新增Agent、高级能力 |

### 里程碑

- **Week 4**: 可以模拟A股交易
- **Week 10**: Multi-Agent可以协作
- **Week 14**: 可以实盘交易
- **Week 20**: AI可以自主决策
- **Week 24**: 数据全面完整
- **Week 30**: 系统功能完整

---

## 🚀 核心技术亮点

### 1. A股交易规则引擎

```rust
pub struct AShareTradingRules {
    t_plus_one: TPlusOneRule,        // T+1规则
    price_limits: HashMap<...>,      // 涨跌停限制
    trading_hours: TradingHours,     // 交易时间
    min_unit: MinTradingUnit,        // 100股=1手
}

// 检查T+1可卖状态
check_sellable(position, trade_date)

// 检查涨跌停价格
check_price_limit(symbol, price, limit_price)

// 计算涨停/跌停价
calculate_upper_limit(symbol, limit_price)
calculate_lower_limit(symbol, limit_price)
```

### 2. 10个专业Agent

**核心Agent**:
1. DataAgent - 数据采集
2. ResearchAgent - 研究分析
3. StrategyAgent - 策略制定
4. TradingAgent - 交易执行
5. RiskAgent - 风险控制
6. PortfolioAgent - 组合管理
7. SentimentAgent - 情绪分析
8. ReportAgent - 报告生成
9. MonitorAgent - 实时监控
10. SupervisorAgent - 总监督 ⭐

### 3. 券商API集成

**支持的券商**:
- 国金证券 (QMT/PTrade)
- 华泰证券
- 东方财富
- 模拟交易 (Simulated)

**统一的BrokerClient接口**:
```rust
#[async_trait]
pub trait BrokerClient {
    fn name(&self) -> &str;
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse>;
    async fn cancel_order(&self, order_id: &str) -> Result<()>;
    async fn query_order(&self, order_id: &str) -> Result<OrderStatus>;
    async fn query_positions(&self) -> Result<Vec<Position>>;
    async fn query_account(&self) -> Result<AccountInfo>;
    async fn query_quotes(&self, symbols: &[String]) -> Result<Vec<Quote>>;
}
```

### 4. AI决策引擎 + Self-Reflection

**AI决策流程**:
1. 收集各Agent意见
2. 查询知识库（类似决策）
3. Claude综合分析
4. 生成决策
5. Self-Reflection调整
6. 记录决策供学习

**Self-Reflection机制**:
- 记录决策历史
- 分析决策结果
- 生成反思（正确/错误/改进）
- 调整未来置信度

### 5. 本土化数据源

**数据源**:
- Tushare (实时行情、财务数据) ⭐
- Wind (专业金融终端)
- 同花顺iFinD
- 东方财富 (研报)
- 新浪财经 (宏观数据)

---

## 📈 预期成果

### 技术指标

| 指标 | Plan3 | Plan4 | 提升 |
|------|-------|-------|------|
| **Claude SDK集成** | 100% | 100% | 保持 |
| **Agent数量** | 25 Skills | 10 Agents + 25 Skills | 135% |
| **MCP工具** | 135+ | 200+ | 48% ↑ |
| **支持市场** | 加密货币 | A股 | 新增 |
| **券商集成** | 0 | 3+ | 新增 |
| **决策自动化** | 策略执行 | AI自主决策 | 质的飞跃 |
| **自反思能力** | 无 | 有 | 新增 |

### 性能目标

| 指标 | 目标 |
|------|------|
| **决策延迟** | <500ms |
| **数据延迟** | <100ms |
| **订单延迟** | <300ms |
| **AI决策准确率** | >60% |
| **回测覆盖率** | 100% |

### 业务目标

| 指标 | 目标 |
|------|------|
| **年化收益率** | >15% |
| **最大回撤** | <20% |
| **夏普比率** | >1.5 |
| **胜率** | >55% |

---

## 💡 实施建议

### 1. 渐进式实施 ⭐

**推荐的实施顺序**:

1. ✅ **Phase 1优先** (4周) - A股基础架构
2. ✅ **Phase 3第二** (4周) - 券商API集成，实现实盘
3. ✅ **Phase 2第三** (6周) - Multi-Agent升级
4. ✅ **Phase 4第四** (6周) - AI决策系统
5. ✅ **Phase 5最后** (4周) - 本土化数据
6. ✅ **Phase 6可选** (6周) - 高级功能

**理由**: 快速建立A股交易能力，然后逐步智能化

### 2. 保持Plan3优势

**Plan4不是替代，而是增强Plan3**:

- ✅ 保留Plan3的25个Skills
- ✅ 保留Plan3的135+个MCP工具
- ✅ 保留Plan3的LSTM/DQN模型
- ✅ 保留加密货币交易能力

**Plan4 = Plan3 + A股 + Multi-Agent + AI决策**

### 3. 专注A股特殊性

**我们的核心竞争力**:

1. ✅ **T+1规则引擎** - 国内平台少有
2. ✅ **涨跌停限制** - 精确风控
3. ✅ **本土化数据** - Tushare等
4. ✅ **券商API** - 实盘交易
5. ✅ **Multi-Agent** - 2025年趋势
6. ✅ **Self-Reflection** - 持续学习

### 4. 技术选型

**核心技术栈**:

```toml
# 核心
claude-agent-sdk-rs = { path = "..", features = ["orchestration"] }

# 运行时
tokio = { version = "1.48", features = ["full"] }

# 网络
reqwest = { version = "0.12", features = ["json"] }

# 存储
libsql = "0.5"      # 200ns查询
duckdb = "0.11"     # 分析查询

# AI/ML
tch = "0.15"       # PyTorch (LSTM/DQN)

# 工具
serde = "1.0"
anyhow = "1.0"
chrono = "0.4"
```

---

## 📚 参考资源

### Multi-Agent系统

- [TradingGroup: A Multi-Agent Trading System (arXiv 2508.17565v1)](https://arxiv.org/html/2508.17565v1) - Self-reflective架构
- [AutoTrader-AgentEdge (GitHub)](https://github.com/iAmGiG/AutoTrader-AgentEdge) - Microsoft AutoGen，Production-ready
- [Multi-Agent Framework for Quantitative Finance (ACL 2025)](https://aclanthology.org/2025.emnlp-main.55.pdf) - 量化框架集成

### A股量化平台

- [BigQuant - AI量化交易平台](https://bigquant.com/) - AI技术、海量因子
- [聚宽量化平台](https://www.joinquant.com/) - 社区活跃、策略商城
- [优矿量化平台](https://uqer.datayes.com/) - 数据丰富、Notebook
- [掘金量化平台](https://www.myquant.cn/) - 实盘交易、本地运行

### A股数据API

- [Tushare数据](https://tushare.pro/) - 免费额度+完整API ⭐
- [Wind金融终端](https://www.wind.com.cn/) - 专业金融数据
- [同花顺iFinD](http://data.10jqka.com.cn/) - 实时行情数据

### A股交易规则

- [A股T+1交易规则详解](https://www.9db.com/user/blog/A305AFD6-8937-46B5-8781-9C7AD7DC30E0/)
- [涨跌停限制规则](https://finance.sina.com.cn/money/future/roll/2020-07-24/doc-iivhuipn4771923.shtml)

### Claude Agent SDK

- [Building agents with Claude Agent SDK](https://www.anthropic.com/engineering/building-agents-with-the-claude-agent-sdk)
- [Agent SDK overview](https://platform.claude.com/docs/en/agent-sdk/overview)

---

## 🎊 总结

**Plan4 = A股本土化 + Multi-Agent + AI投资决策**

### 核心价值主张

**打造国内首个基于Claude Agent SDK的A股AI量化投资平台**

1. ✅ **技术领先**: 唯一100% Claude SDK + Rust平台
2. ✅ **架构先进**: Multi-Agent + Self-Reflection
3. ✅ **市场专注**: 深度适配A股特性
4. ✅ **生产就绪**: 券商API + 完整风控
5. ✅ **持续进化**: AI自主决策 + 自我学习

### 关键差异 vs Plan3

| 维度 | Plan3 | Plan4 |
|------|-------|-------|
| **市场** | 加密货币 | A股 |
| **架构** | Skills平面 | Agent分层 |
| **决策** | 策略执行 | AI自主决策 |
| **自反思** | 无 | 有 |
| **编排** | 2种模式 | 3种模式 |
| **交易** | 2个交易所 | 3+券商 |
| **数据** | WebSocket | A股本土化 |

### 最终目标

**成为A股AI量化投资领域的技术领导者** 🚀

---

**文档版本**: 4.0
**创建日期**: 2026-01-10
**维护者**: InvestIntel AI Team

**END OF OVERVIEW**
