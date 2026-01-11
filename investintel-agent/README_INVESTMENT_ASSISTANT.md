# InvestIntel AI - 投资智能助手 🤖

> **基于Claude Agent SDK的AI价值投资助手 - 让普通人也能通过价值投资赚钱**

**版本**: 1.0.0 MVP
**日期**: 2026-01-11
**核心理念**: Graham-Buffett-Munger价值投资三位一体

---

## 🎯 核心特性

### 1. 价值投资分析 ⭐⭐⭐⭐⭐

基于Benjamin Graham和Warren Buffett的经典价值投资方法：

- **Graham安全边际** (30%+要求)
- **Buffett质量标准** (ROIC > 10%)
- **DCF估值模型**
- **护城河评分系统**
- **综合投资建议**

### 2. 投资组合管理 📊

- 资产配置分析
- 再平衡建议
- 绩效评估
- 风险监控

### 3. 交易建议 💡

- 入场时机分析
- 科学仓位管理
- 止损止盈建议
- 风险等级评估

---

## 🚀 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/your-repo/claude-agent-sdk.git
cd claude-agent-sdk

# 构建
cargo build --release
```

### CLI使用

```bash
# 启动交互式投资助手
cargo run --bin invest_cli

# 或者直接分析股票
echo "分析AAPL" | cargo run --bin invest_cli
```

### 编程使用

```rust
use investintel_agent::InvestmentAssistant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建投资助手
    let assistant = InvestmentAssistant::new();

    // 分析股票
    let analysis = assistant.analyze_stock("AAPL").await?;
    println!("📊 价值分析:\n{}", analysis.value_analysis);
    println!("💡 交易建议:\n{}", analysis.trading_advice);

    // 交互式咨询
    let response = assistant.chat("MSFT值得投资吗").await?;
    println!("{}", response);

    Ok(())
}
```

---

## 📊 使用示例

### 示例1: 分析苹果公司

```bash
💬 您: 分析AAPL

🤖 助手:
📊 AAPL 价值投资分析报告

💡 投资建议: 买入
✅ 置信度: 75.0%
📈 目标价位: $185.20
💰 当前价格: $150.00
🛡️ 安全边际: 23.5%
⚠️ 风险等级: 2/3

📝 分析理由:
Graham安全边际23.5% (良好); Buffett ROIC 15.0% (优秀);
护城河评分 3/3 (宽护城河)

📊 详细分析:
- Graham内在价值: $195.00
- Buffett内在价值: $175.40
- ROIC: 15.0%
- 护城河: 3/3
```

### 示例2: 投资组合分析

```rust
use investintel_agent::agents::{Portfolio, Holding, PortfolioManagerAgent};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = PortfolioManagerAgent::new();

    // 定义投资组合
    let mut target_weights = HashMap::new();
    target_weights.insert("AAPL".to_string(), 0.4);
    target_weights.insert("MSFT".to_string(), 0.3);
    target_weights.insert("GOOGL".to_string(), 0.3);

    let portfolio = Portfolio {
        holdings: vec![
            Holding {
                symbol: "AAPL".to_string(),
                shares: 100,
                value: 15000.0,
                cost_basis: 12000.0,
            },
            // ... 其他持仓
        ],
        target_weights,
    };

    // 分析组合
    let result = agent.analyze_portfolio(&portfolio).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}
```

---

## 🏗️ 架构设计

### 核心组件

```
InvestIntel AI
├── InvestmentAssistant (主协调Agent)
│   ├── ValueInvestmentAgent (Graham-Buffett价值分析)
│   ├── PortfolioManagerAgent (组合管理)
│   └── TradingAdvisorAgent (交易建议)
└── Claude Agent SDK (复用)
    ├── 25+ Agent Skills
    ├── Orchestration系统
    ├── 数据层 (Yahoo/AlphaVantage)
    └── 交易层 (Binance/OKX)
```

### 设计原则

1. **充分复用** - 100%基于Claude Agent SDK
2. **最小改造** - 只添加4个Agent文件
3. **高内聚低耦合** - 每个Agent职责单一
4. **可扩展** - 预留接口，易于扩展

---

## 📚 价值投资框架

### Graham深度价值

- **公式**: V = EPS × (8.5 + 2g)
- **安全边际**: ≥30%
- **应用**: 深度折价股票

### Buffett质量价值

- **ROIC标准**: >10%
- **护城河**: 宽/窄/无
- **应用**: 优质企业

### 综合评分

- Graham得分: 0-40分
- Buffett得分: 0-40分
- 护城河得分: 0-20分
- 总分: 0-100分

---

## 📖 参考资料

### 价值投资经典

- [Graham Formula详解](https://www.grahamvalue.com/article/understanding-benjamin-graham-formula-correctly)
- [巴菲特价值投资实践](https://finance.sina.com.cn/money/fund/jjzl/2025-11-15/doc-infxmuwe3681007.shtml)
- [Charlie Munger Mental Models](https://moserendipity.com/2025/11/30/charlie-munger-mental-models-wealth-lollapalooza/)

### Claude Agent SDK

- [Building agents with Claude Agent SDK](https://www.anthropic.com/engineering/building-agents-with-the-claude-agent-sdk)
- [Agent SDK overview](https://platform.claude.com/docs/en/agent-sdk/overview)

---

## 🧪 测试

```bash
# 运行所有测试
cargo test --package investintel-agent

# 运行集成测试
cargo test --test investment_assistant_test

# 查看测试覆盖率
cargo tarpaulin --out Html
```

---

## 📝 待办事项

### Phase 2: 数据集成
- [ ] 连接Yahoo Finance API
- [ ] 连接Alpha Vantage API
- [ ] 实时价格获取

### Phase 3: Munger框架
- [ ] Lollapalooza效应检测
- [ ] Mental Models应用
- [ ] 能力圈评估

### Phase 4: 高级功能
- [ ] Kelly仓位管理
- [ ] MCP Gateway
- [ ] 回测系统

---

## 🤝 贡献

欢迎贡献！请查看:
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [代码规范](.rustfmt.toml)
- [测试指南](tests/)

---

## 📄 许可证

MIT License - 详见 [LICENSE.md](LICENSE.md)

---

## 🙏 致谢

- Anthropic - Claude Agent SDK
- Benjamin Graham - 价值投资理论
- Warren Buffett - 质量价值投资
- Charlie Munger - 多元思维模型

---

**InvestIntel AI - 让每个人都能通过价值投资赚钱** 💰🚀

**Sources**:
- [Graham Formula - Understanding the Benjamin Graham Formula](https://www.grahamvalue.com/article/understanding-benjamin-graham-formula-correctly)
- [Investing.com - Benjamin Graham Formula](https://www.investing.com/academy/analysis/benjamin-graham-formula-definition/)
- [新浪财经 - 巴菲特投资传奇](https://finance.sina.com.cn/money/fund/jjzl/2025-11-15/doc-infxmuwe3681007.shtml)
- [Charlie Munger Mental Models](https://moserendipity.com/2025/11/30/charlie-munger-mental-models-wealth-lollapalooza/)
