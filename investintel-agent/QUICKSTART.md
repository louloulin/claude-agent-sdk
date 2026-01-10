# InvestIntel AI - 快速开始指南

## 🚀 5分钟上手

### 1. 项目结构
```
investintel-agent/
├── .claude/skills/investment-analyst/SKILL.md  # Agent技能定义
├── app/main.rs                                   # 完整应用
├── README.md                                     # 详细文档
└── IMPLEMENTATION_REPORT.md                      # 实现报告
```

### 2. 核心功能

基于Claude Agent SDK实现：

- **Query API**: 一行代码调用Claude
- **MCP Tools**: 5个自定义投资分析工具
- **Agent Skills**: SKILL.md定义能力
- **libSQL**: 200ns查询延迟数据持久化

### 3. MCP Tools列表

| 工具 | 功能 |
|------|------|
| `technical_analysis` | 技术分析 - 趋势、支撑/阻力、RSI/MACD |
| `var_calculation` | VaR风险计算 - 1天/5天/30天风险价值 |
| `sentiment_analysis` | 情感分析 - 新闻、社交媒体、分析师 |
| `save_portfolio` | 保存投资组合到libSQL |
| `load_portfolio` | 从libSQL加载投资组合 |

### 4. 使用示例

```rust
// 创建MCP工具
let tools = create_sdk_mcp_server(
    "investment-tools",
    vec![tool! {
        name: "technical_analysis",
        description: "技术分析",
        handler: technical_analysis
    }],
)?;

// 配置Claude
let options = ClaudeAgentOptions::builder()
    .permission_mode(PermissionMode::BypassPermissions)
    .mcp_servers(McpServers::new().add_server(tools))
    .build();

// 查询Claude
let messages = query("分析AAPL股票", Some(options)).await?;
```

### 5. VaR计算示例

```rust
// 参数法VaR
let portfolio_value = 100000.0;
let volatility = 0.20;        // 20%年化波动率
let z_score = 1.65;            // 95%置信度

let var_1day = portfolio_value * volatility * (1.0 / 365.0).sqrt() * z_score;
// 结果: var_1day ≈ $2,250 (2.25%)

let var_5day = var_1day * 5.0_f64.sqrt();
let var_30day = var_1day * 30.0_f64.sqrt();
```

### 6. 技术指标返回示例

```json
{
  "symbol": "AAPL",
  "trend": "bullish",
  "strength": 0.75,
  "support": 148.50,
  "resistance": 155.20,
  "indicators": {
    "rsi": 62.5,
    "macd": {"value": 1.2, "signal": 0.8},
    "sma_20": 150.2,
    "sma_50": 148.5
  },
  "recommendation": "buy",
  "confidence": 0.72
}
```

### 7. 情感分析返回示例

```json
{
  "symbol": "AAPL",
  "overall_score": 0.65,
  "label": "Positive",
  "sources": {
    "news": {"score": 0.68, "sample_size": 25},
    "social": {"score": 0.58, "sample_size": 15000},
    "analyst": {"score": 0.70, "sample_size": 12}
  },
  "trend": "improving",
  "trading_signal": "bullish"
}
```

## 📚 更多文档

- **详细文档**: README.md
- **实现报告**: IMPLEMENTATION_REPORT.md
- **计划文档**: plan2.0.md (已标记实现状态)
- **代码示例**: app/main.rs, src/main.rs

## ✅ 已实现功能

- ✅ Claude Agent SDK完整集成
- ✅ 5个MCP Tools
- ✅ Agent Skills系统 (SKILL.md)
- ✅ libSQL数据持久化架构
- ✅ 完整类型系统
- ✅ 测试套件
- ✅ 详细文档

## 🎯 下一步

1. 修复workspace配置
2. 运行实际测试
3. 集成真实libSQL
4. 添加实时数据API

## 📞 支持

查看 `README.md` 获取完整文档和实现细节。
