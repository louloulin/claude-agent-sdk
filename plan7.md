# Plan7 - AI投资智能助手：基于Claude Agent SDK的最小化改造实现

**版本**: 7.0
**日期**: 2026-01-11
**状态**: 🚀 **实施中**
**核心理念**: **充分复用现有Claude Agent SDK能力，最小改造实现投资智能助手**
**基于**: Plan6愿景 + 现有代码深度分析 + 高内聚低耦合设计

---

## 🎯 Plan7核心目标

**基于Plan6愿景，充分复用Claude Agent SDK现有能力，以最小改造方式实现投资智能助手**

### 核心原则

1. **充分复用现有代码** - 最大化利用已有的25+ Skills、MCP工具、Subagents
2. **高内聚低耦合** - 保持代码模块化，便于扩展和维护
3. **增量式改进** - 不重写，只添加必要的投资助手功能
4. **保持向后兼容** - 不破坏现有功能
5. **实用主义** - 专注核心价值，避免过度设计

---

## 📊 现有代码资产分析

### 已实现的核心能力

基于对现有代码库的分析：

#### 1. Agent Skills系统 (25+ Skills)
```
investintel-agent/.claude/skills/
├── fundamental-analysis/     # 基本面分析
├── technical-analysis/       # 技术分析
├── portfolio-management/     # 投资组合管理
├── risk-analysis/           # 风险分析
├── market-research/         # 市场研究
├── trading-execution/       # 交易执行
├── backtesting/            # 回测引擎
├── sentiment-analysis/      # 情绪分析
├── momentum-trading/        # 动量交易
├── options-trading/         # 期权交易
├── lstm-prediction/         # LSTM预测
├── reinforcement-learning/  # 强化学习
├── realtime-monitor/        # 实时监控
├── data-fusion/            # 数据融合
├── yahoo-finance/          # Yahoo Finance
├── alpha-vantage/          # Alpha Vantage
├── portfolio-optimization/ # 组合优化
└── ... (25+ total)
```

#### 2. Orchestration系统 (Claude Agent SDK)
```
src/orchestration/
├── agent.rs                # Agent trait定义
├── orchestrator.rs         # Orchestrator trait
├── patterns/              # 编排模式
│   ├── sequential.rs      # 顺序执行
│   ├── parallel.rs        # 并行执行
│   └── ... (更多模式)
├── context.rs             # 执行上下文
└── registry.rs            # Agent注册表
```

#### 3. 数据层
```
investintel-agent/data/
├── yahoo.rs               # Yahoo Finance API
├── alpha_vantage.rs       # Alpha Vantage API
├── websocket.rs           # WebSocket实时数据
├── websocket_enhanced.rs  # 增强WebSocket
├── fusion.rs              # 数据融合引擎
└── quality.rs             # 数据质量验证
```

#### 4. 交易层
```
investintel-agent/trading/
├── binance.rs             # Binance期货API
├── okx.rs                 # OKX API
├── order_manager.rs       # 订单管理
└── emergency_stop.rs      # 紧急停止
```

#### 5. 策略层
```
investintel-agent/strategies/
├── lstm_predictor.rs      # LSTM价格预测
└── dqn_agent.rs          # DQN强化学习
```

---

## 🏗️ Plan7架构设计：最小改造方案

### 核心理念：**Add, Don't Replace**

**不要重写现有代码，而是添加投资助手所需的协调层**

### 架构图

```
┌─────────────────────────────────────────────────────────────┐
│              InvestIntel AI Investment Assistant             │
│                  (投资智能助手 - 新增层)                      │
└─────────────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┼────────────────────────────────┐
         │                    │                                │
    ┌────▼────┐         ┌────▼────┐                    ┌─────▼─────┐
    │ Advisor │         │ Planner │                    │ Executor │
    │  Agent  │         │  Agent  │                    │  Agent   │
    └─────────┘         └─────────┘                    └───────────┘
         │                    │                                │
         └────────────────────┼────────────────────────────────┘
                              │
         ┌────────────────────▼────────────────────────────────┐
         │       Claude Agent SDK Orchestration Layer          │
         │          (复用现有orchestration系统)                  │
         │  - Sequential, Parallel, Hierarchical patterns      │
         │  - 25+ existing Agent Skills                        │
         │  - MCP tools integration                           │
         └─────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┼────────────────────────────────┐
         │                    │                                │
    ┌────▼─────┐        ┌────▼─────┐                    ┌─────▼─────┐
    │   Data   │        │ Trading  │                    │  Skills   │
    │  Layer   │        │  Layer   │                    │  System   │
    └──────────┘        └──────────┘                    └───────────┘
    (复用现有)          (复用现有)                      (复用现有)
```

---

## 📋 Plan7实施计划（最小改造）

### Phase 1: 投资助手Agent实现 (2周)

#### 目标
基于现有Orchestration系统，实现3个核心协调Agent

#### Week 1: 核心Agent实现

**1.1 InvestmentAdvisor Agent**
```rust
// investintel-agent/agents/advisor.rs
use claude_agent_sdk_rs::orchestration::{Agent, AgentInput, AgentOutput};
use async_trait::async_trait;

/// 投资顾问Agent - 协调所有Skills提供投资建议
pub struct InvestmentAdvisor {
    skill_registry: Arc<SkillRegistry>,
}

#[async_trait]
impl Agent for InvestmentAdvisor {
    fn name(&self) -> &str {
        "InvestmentAdvisor"
    }

    fn description(&self) -> &str {
        "投资顾问，协调分析技能提供投资建议"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 1. 解析用户输入
        let query = self.parse_query(&input)?;

        // 2. 选择相关Skills
        let skills = self.select_relevant_skills(&query)?;

        // 3. 并行执行Skills
        let results = self.execute_skills_parallel(skills, &query).await?;

        // 4. 综合分析
        let recommendation = self.synthesize_recommendation(results)?;

        Ok(AgentOutput {
            content: recommendation.text,
            data: serde_json::to_value(recommendation)?,
            confidence: recommendation.confidence,
            metadata: recommendation.metadata,
        })
    }
}
```

**1.2 InvestmentPlanner Agent**
```rust
// investintel-agent/agents/planner.rs
/// 投资规划Agent - 制定投资计划
pub struct InvestmentPlanner {
    advisor: Arc<InvestmentAdvisor>,
}

#[async_trait]
impl Agent for InvestmentPlanner {
    fn name(&self) -> &str {
        "InvestmentPlanner"
    }

    fn description(&self) -> &str {
        "投资规划师，基于用户目标制定投资计划"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 1. 分析用户目标和约束
        let goals = self.analyze_goals(&input)?;

        // 2. 调用Advisor获取市场分析
        let market_analysis = self.advisor.execute(
            AgentInput::from("分析当前市场状况")
        ).await?;

        // 3. 制定投资计划
        let plan = self.create_investment_plan(goals, market_analysis)?;

        Ok(AgentOutput {
            content: plan.description,
            data: serde_json::to_value(plan)?,
            confidence: 0.85,
            metadata: Default::default(),
        })
    }
}
```

**1.3 InvestmentExecutor Agent**
```rust
// investintel-agent/agents/executor.rs
/// 投资执行Agent - 执行投资决策
pub struct InvestmentExecutor {
    trading: Arc<trading::OrderManager>,
}

#[async_trait]
impl Agent for InvestmentExecutor {
    fn name(&self) -> &str {
        "InvestmentExecutor"
    }

    fn description(&self) -> &str {
        "投资执行者，执行交易决策"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        // 1. 解析交易指令
        let order: trading::OrderRequest = serde_json::from_value(input.data)?;

        // 2. 风险检查
        self.risk_check(&order)?;

        // 3. 执行交易
        let result = self.trading.place_order(order).await?;

        Ok(AgentOutput {
            content: format!("交易执行成功: {:?}", result),
            data: serde_json::to_value(result)?,
            confidence: 0.95,
            metadata: Default::default(),
        })
    }
}
```

#### Week 2: Agent注册和编排

**1.4 注册Agents到现有Registry**
```rust
// investintel-agent/agents/mod.rs
pub mod advisor;
pub mod planner;
pub mod executor;

use claude_agent_sdk_rs::orchestration::AgentRegistry;

/// 注册所有投资助手Agents
pub fn register_investment_agents(registry: &mut AgentRegistry) {
    registry.register(Box::new(advisor::InvestmentAdvisor::new()));
    registry.register(Box::new(planner::InvestmentPlanner::new()));
    registry.register(Box::new(executor::InvestmentExecutor::new()));
}
```

**1.5 创建投资助手Orchestrator**
```rust
// investintel-agent/orchestration/investment_orchestrator.rs
use claude_agent_sdk_rs::orchestration::{
    Orchestrator, OrchestratorInput, OrchestratorOutput,
    SequentialOrchestrator, ParallelOrchestrator,
};

/// 投资助手编排器
pub struct InvestmentAssistantOrchestrator {
    advisor_agent: Box<dyn Agent>,
    planner_agent: Box<dyn Agent>,
    executor_agent: Box<dyn Agent>,
}

impl InvestmentAssistantOrchestrator {
    pub fn new() -> Self {
        Self {
            advisor_agent: Box::new(advisor::InvestmentAdvisor::new()),
            planner_agent: Box::new(planner::InvestmentPlanner::new()),
            executor_agent: Box::new(executor::InvestmentExecutor::new()),
        }
    }

    /// 完整投资流程：咨询 -> 规划 -> 执行
    pub async fn assist_investment(
        &self,
        user_query: &str,
    ) -> Result<InvestmentRecommendation> {
        // Phase 1: 咨询分析
        let advice = self.advisor_agent.execute(
            AgentInput::from(user_query)
        ).await?;

        // Phase 2: 制定计划
        let plan = self.planner_agent.execute(
            AgentInput {
                task: "制定投资计划".to_string(),
                context: advice.data,
            }
        ).await?;

        // Phase 3: (可选) 执行交易
        // 根据用户决定是否执行

        Ok(InvestmentRecommendation {
            advice: advice.content,
            plan: plan.data,
            confidence: (advice.confidence + plan.confidence) / 2.0,
        })
    }
}
```

**验收标准**:
- ✅ 3个核心Agent实现完成
- ✅ Agent可以独立执行
- ✅ Agent可以编排组合执行
- ✅ 通过单元测试

---

### Phase 2: 投资助手CLI工具 (1周)

#### 目标
提供用户友好的命令行界面

#### Week 1: CLI实现

**2.1 投资助手命令**
```rust
// investintel-agent/cli/invest_assistant.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "invest-assistant")]
#[command(about = "AI投资智能助手", long_about = None)]
struct InvestAssistant {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 咨询投资建议
    Advise {
        /// 投资标的
        #[arg(short, long)]
        symbol: String,

        /// 投资期限 (天)
        #[arg(short, long, default_value_t = 365)]
        horizon: u32,

        /// 风险偏好 (conservative/moderate/aggressive)
        #[arg(short, long, default_value = "moderate")]
        risk: String,
    },

    /// 制定投资计划
    Plan {
        /// 投资金额
        #[arg(short, long)]
        amount: f64,

        /// 目标收益率
        #[arg(short, long)]
        target_return: f64,

        /// 投资目标描述
        #[arg(short, long)]
        goals: String,
    },

    /// 交互式咨询
    Chat {
        /// 开启聊天模式
        #[arg(short, long)]
        interactive: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = InvestAssistant::parse();
    let orchestrator = InvestmentAssistantOrchestrator::new();

    match cli.command {
        Commands::Advise { symbol, horizon, risk } => {
            let query = format!(
                "分析{}的投资价值，期限{}天，风险偏好{}",
                symbol, horizon, risk
            );

            let recommendation = orchestrator.assist_investment(&query).await?;

            println!("📊 投资建议");
            println!("{}\n", recommendation.advice);
            println!("📈 投资计划");
            println!("{}", serde_json::to_string_pretty(&recommendation.plan)?);
            println!("\n✅ 置信度: {:.1}%", recommendation.confidence * 100.0);
        }

        Commands::Plan { amount, target_return, goals } => {
            let query = format!(
                "规划投资方案，金额{}，目标收益{}，{}",
                amount, target_return, goals
            );

            let plan = orchestrator.assist_investment(&query).await?;

            println!("📋 投资计划");
            println!("{}", serde_json::to_string_pretty(&plan.plan)?);
        }

        Commands::Chat { interactive } => {
            if interactive {
                println!("💬 进入交互式投资咨询模式 (输入'quit'退出)");
                loop {
                    print!("> ");
                    use std::io::Write;
                    std::io::stdout().flush()?;

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;

                    if input.trim() == "quit" {
                        println!("👋 再见！");
                        break;
                    }

                    let response = orchestrator.assist_investment(&input).await?;
                    println!("🤖 {}", response.advice);
                }
            }
        }
    }

    Ok(())
}
```

**验收标准**:
- ✅ CLI工具可用
- ✅ 支持咨询、规划、聊天三种模式
- ✅ 输出格式友好
- ✅ 错误处理完善

---

### Phase 3: 知识库和上下文管理 (1周)

#### 目标
实现简单的记忆和上下文管理

#### Week 1: 实现

**3.1 用户配置存储**
```rust
// investintel-agent/storage/user_profile.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub name: String,
    pub risk_tolerance: RiskTolerance,
    pub investment_goals: Vec<InvestmentGoal>,
    pub preferred_markets: Vec<String>,
    pub capital: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskTolerance {
    Conservative,
    Moderate,
    Aggressive,
}

impl UserProfile {
    pub fn save(&self, path: PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load(path: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let profile: UserProfile = toml::from_str(&content)?;
        Ok(profile)
    }
}
```

**3.2 会话历史管理**
```rust
// investintel-agent/storage/conversation_history.rs
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct ConversationHistory {
    messages: VecDeque<ChatMessage>,
    max_size: usize,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum ChatRole {
    User,
    Assistant,
}

impl ConversationHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn add(&mut self, role: ChatRole, content: String) {
        let message = ChatMessage {
            role,
            content,
            timestamp: Utc::now(),
        };

        self.messages.push_back(message);

        if self.messages.len() > self.max_size {
            self.messages.pop_front();
        }
    }

    pub fn get_context(&self, n: usize) -> Vec<&ChatMessage> {
        self.messages.iter().rev().take(n).rev().collect()
    }
}
```

**验收标准**:
- ✅ 用户配置可以保存和加载
- ✅ 会话历史可以记录
- ✅ 上下文可以传递给Agent
- ✅ 测试通过

---

### Phase 4: 简单的价值投资框架 (2周)

#### 目标
实现简化的Graham-Buffett价值分析

#### Week 1: Graham框架

**4.1 Graham公式实现**
```rust
// investintel-agent/value/graham.rs
/// Graham框架 - 深度价值投资
pub struct GrahamFramework {
    pub base_multiplier: f64,  // 8.5
    pub growth_multiplier: f64, // 2.0
    pub margin_requirement: f64, // 0.3 (30%)
}

impl GrahamFramework {
    pub fn new() -> Self {
        Self {
            base_multiplier: 8.5,
            growth_multiplier: 2.0,
            margin_requirement: 0.3,
        }
    }

    /// Graham公式: V = EPS × (8.5 + 2g)
    pub fn calculate_intrinsic_value(
        &self,
        eps: f64,
        growth_rate: f64,
    ) -> f64 {
        let multiplier = self.base_multiplier + self.growth_multiplier * growth_rate * 100.0;
        eps * multiplier
    }

    /// 计算安全边际
    pub fn calculate_margin_of_safety(
        &self,
        intrinsic_value: f64,
        current_price: f64,
    ) -> f64 {
        (intrinsic_value - current_price) / intrinsic_value
    }

    /// 分析股票
    pub async fn analyze(&self, symbol: &str) -> Result<GrahamAnalysis> {
        // 从data层获取数据
        let eps = self.get_eps(symbol).await?;
        let growth_rate = self.get_growth_rate(symbol).await?;
        let current_price = self.get_price(symbol).await?;

        let intrinsic_value = self.calculate_intrinsic_value(eps, growth_rate);
        let margin = self.calculate_margin_of_safety(intrinsic_value, current_price);

        Ok(GrahamAnalysis {
            symbol: symbol.to_string(),
            eps,
            growth_rate,
            intrinsic_value,
            current_price,
            margin_of_safety: margin,
            recommendation: if margin > self.margin_requirement {
                "深度价值买入".to_string()
            } else if margin > 0.15 {
                "价值买入".to_string()
            } else {
                "等待更好价格".to_string()
            },
        })
    }
}

#[derive(Debug, Clone)]
pub struct GrahamAnalysis {
    pub symbol: String,
    pub eps: f64,
    pub growth_rate: f64,
    pub intrinsic_value: f64,
    pub current_price: f64,
    pub margin_of:Safety: f64,
    pub recommendation: String,
}
```

#### Week 2: Buffett框架

**4.2 Buffett框架实现**
```rust
// investintel-agent/value/buffett.rs
/// Buffett框架 - 质量价值投资
pub struct BuffettFramework {
    pub min_roic: f64,  // 0.10 (10%)
}

impl BuffettFramework {
    pub fn new() -> Self {
        Self {
            min_roic: 0.10,
        }
    }

    /// 分析股票
    pub async fn analyze(&self, symbol: &str) -> Result<BuffettAnalysis> {
        let roic = self.get_roic(symbol).await?;
        let current_price = self.get_price(symbol).await?;
        let intrinsic_value = self.calculate_dcf(symbol).await?;

        Ok(BuffettAnalysis {
            symbol: symbol.to_string(),
            roic,
            intrinsic_value,
            current_price,
            fair_price: intrinsic_value * 0.9,
            recommendation: if roic > self.min_roic {
                "买入优质企业".to_string()
            } else {
                "继续观察".to_string()
            },
        })
    }

    /// 简化DCF估值
    async fn calculate_dcf(&self, symbol: &str) -> Result<f64> {
        let fcf = self.get_free_cash_flow(symbol).await?;
        let growth_rate = 0.10; // 假设10%增长
        let discount_rate = 0.10; // 10%折现率

        // 简化：永续增长模型
        let terminal_value = fcf * (1.0 + growth_rate) / (discount_rate - growth_rate);
        Ok(terminal_value)
    }
}

#[derive(Debug, Clone)]
pub struct BuffettAnalysis {
    pub symbol: String,
    pub roic: f64,
    pub intrinsic_value: f64,
    pub current_price: f64,
    pub fair_price: f64,
    pub recommendation: String,
}
```

**4.3 价值分析Agent**
```rust
// investintel-agent/agents/value_analyst.rs
/// 价值分析Agent - 整合Graham和Buffett框架
pub struct ValueAnalystAgent {
    graham: GrahamFramework,
    buffett: BuffettFramework,
}

#[async_trait]
impl Agent for ValueAnalystAgent {
    fn name(&self) -> &str {
        "ValueAnalyst"
    }

    fn description(&self) -> &str {
        "价值分析师，使用Graham-Buffett框架分析股票"
    }

    async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
        let symbol = input.task;

        // 并行执行Graham和Buffett分析
        let (graham_result, buffett_result) = tokio::try_join!(
            self.graham.analyze(&symbol),
            self.buffett.analyze(&symbol)
        )?;

        // 综合判断
        let recommendation = if graham_result.margin_of_safety > 0.30
            && buffett_result.roic > 0.10
        {
            "强推荐：Graham安全边际30%+，Buffett ROIC>10%"
        } else if graham_result.margin_of_safety > 0.20 {
            "推荐：Graham安全边际充足"
        } else if buffett_result.roic > 0.10 {
            "关注：Buffett质量优秀，等待更好价格"
        } else {
            "观望：不符合价值投资标准"
        };

        Ok(AgentOutput {
            content: format!(
                "📊 {} 价值分析\n\nGraham分析:\n{}\n\nBuffett分析:\n{}\n\n综合建议: {}",
                symbol, graham_result, buffett_result, recommendation
            ),
            data: serde_json::json!({
                "symbol": symbol,
                "graham": graham_result,
                "buffett": buffett_result,
                "recommendation": recommendation,
            }),
            confidence: 0.80,
            metadata: Default::default(),
        })
    }
}
```

**验收标准**:
- ✅ Graham框架实现
- ✅ Buffett框架实现
- ✅ 价值分析Agent可用
- ✅ 可以分析真实股票

---

### Phase 5: 测试和文档 (1周)

#### 目标
完善测试和文档

#### Week 1: 测试和文档

**5.1 集成测试**
```rust
// investintel-agent/tests/investment_assistant_test.rs
#[tokio::test]
async fn test_investment_assistant_flow() {
    let orchestrator = InvestmentAssistantOrchestrator::new();

    // 测试咨询流程
    let advice = orchestrator.assist_investment(
        "分析AAPL的投资价值"
    ).await.unwrap();

    assert!(advice.confidence > 0.5);
    assert!(!advice.advice.is_empty());
}

#[tokio::test]
async fn test_value_analysis() {
    let analyst = ValueAnalystAgent::new();

    let result = analyst.execute(AgentInput {
        task: "AAPL".to_string(),
        context: serde_json::json!({}),
    }).await.unwrap();

    assert!(result.confidence > 0.7);
}
```

**5.2 文档更新**
- 更新README.md
- 添加QUICKSTART.md
- 添加API文档
- 添加使用示例

**验收标准**:
- ✅ 测试覆盖率>70%
- ✅ 文档完善
- ✅ 示例可运行

---

## 📦 Phase实施时间表

| Phase | 功能 | 周期 | 累计时间 |
|-------|------|------|----------|
| **Phase 1** | 投资助手Agent实现 | 2周 | 2周 |
| **Phase 2** | CLI工具 | 1周 | 3周 |
| **Phase 3** | 知识库和上下文 | 1周 | 4周 |
| **Phase 4** | 价值投资框架 | 2周 | 6周 |
| **Phase 5** | 测试和文档 | 1周 | 7周 |

**总计**: 7周 (约2个月)

---

## 🚀 核心优势

### 1. 最小改造 ⭐

**不重写，只添加**:
- ✅ 复用现有25+ Skills
- ✅ 复用现有Orchestration系统
- ✅ 复用现有数据层和交易层
- ✅ 只添加3个协调Agent

### 2. 高内聚低耦合 ⭐

**模块化设计**:
- ✅ Agents独立可测
- ✅ 通过trait松耦合
- ✅ 易于扩展和维护

### 3. 渐进式实现 ⭐

**分阶段交付**:
- ✅ 每个Phase独立可用
- ✅ 逐步增加功能
- ✅ 持续交付价值

### 4. 实用主义 ⭐

**专注核心价值**:
- ✅ 不追求完美架构
- ✅ 先解决核心问题
- ✅ 快速验证反馈

---

## 💡 与Plan6的关系

### Plan6愿景 → Plan7实现

| Plan6愿景 | Plan7实现 | 策略 |
|-----------|----------|------|
| AI投资合伙人 | InvestmentAdvisor/Planner/Executor Agents | 简化为3个协调Agent |
| Subagent专业团队 | 复用现有25+ Skills | 不新建Subagents，复用Skills |
| MCP统一架构 | 复用现有MCP工具 | 不新建MCP，复用现有 |
| Kelly+MPT+Munger仓位 | 暂不实现 | 简化版本先不做 |
| Graham-Buffett-Munger | 简化Graham-Buffett | 先实现核心，Munger暂缓 |
| 混合编排模式 | 复用Sequential/Parallel | 不新增编排模式 |

### 未来演进路径

**Plan7 → Plan6的演进**:
1. ✅ Phase 1-5实现基础投资助手
2. ⏸️ 后续可逐步添加：
   - Kelly仓位管理
   - Munger框架
   - 更复杂的编排模式
   - 专业Subagent团队

---

## 📚 参考资源

### Claude Agent SDK
- [Building agents with the Claude Agent SDK](https://www.anthropic.com/engineering/building-agents-with-the-claude-agent-sdk)
- [Agent SDK overview](https://platform.claude.com/docs/en/agent-sdk/overview)
- 现有代码: `src/orchestration/`

### 价值投资
- [Graham Formula](https://www.grahamvalue.com/article/understanding-benjamin-graham-formula-correctly)
- [Buffett's Value Investing](https://www.investopedia.com/warren-buffett-s-value-investing-strategy-11840085)

### 现有代码
- Skills: `investintel-agent/.claude/skills/`
- Orchestration: `src/orchestration/`
- Data: `investintel-agent/data/`
- Trading: `investintel-agent/trading/`

---

## 🎊 总结

**Plan7 = 最小化改造实现投资智能助手**

### 核心公式

```
Plan7 = 现有Claude Agent SDK
     + 25+ Skills (复用)
     + Orchestration系统 (复用)
     + 数据/交易层 (复用)
     + 3个协调Agent (新增)
     + 简化价值框架 (新增)
```

### 4大实施原则

1. ✅ **充分复用** - 不重写现有代码
2. ✅ **最小改造** - 只添加必要功能
3. ✅ **高内聚低耦合** - 保持模块化
4. ✅ **实用主义** - 快速交付价值

### 预期成果

**7周后，你将拥有**:
- ✅ 可用的AI投资智能助手
- ✅ CLI工具支持
- ✅ Graham-Buffett价值分析
- ✅ 用户配置和会话管理
- ✅ 完善的测试和文档

### 与Plan6的定位

| Plan | 定位 | 目标用户 | 实现周期 |
|------|------|---------|---------|
| **Plan6** | 完整AI投资合伙公司 | 专业价值投资者 | 28周 (7个月) |
| **Plan7** | 实用投资智能助手 | 普通投资者 | 7周 (2个月) |

**Plan7是Plan6的MVP版本** - 快速验证核心价值，为后续演进奠定基础。

---

**文档版本**: 7.0
**创建日期**: 2026-01-11
**维护者**: InvestIntel AI Team
**状态**: 🚀 **实施中**

---

**END OF PLAN7**
