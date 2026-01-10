---
name: options-trading
description: |
  期权交易策略Skill,提供完整的期权分析工具:
  - Black-Scholes期权定价
  - Greeks计算(Delta, Gamma, Theta, Vega, Rho)
  - 隐含波动率计算
  - 波动率曲面分析
  - 期权策略构建

  基于Claude Agent SDK,提供专业的期权交易和风险管理能力。

allowed-tools:
  - black_scholes_price
  - calculate_greeks
  - implied_volatility
  - volatility_surface
  - options_strategy_builder
  - options_pnl_analyzer

model: claude-sonnet-4-20250514
tags:
  - options
  - derivatives
  - volatility
  - greeks

examples:
  - description: "期权定价"
    input: |
      计算AAPL看涨期权价格:
      - 标的价: $175
      - 行权价: $180
      - 到期: 30天
      - 无风险利率: 5%
      - 波动率: 25%

      使用Black-Scholes模型。

  - description: "Greeks计算"
    input: |
      计算期权的所有Greeks:
      - Delta
      - Gamma
      - Theta
      - Vega
      - Rho

      期权: SPY 12月到期的看涨期权

  - description: "隐含波动率"
    input: |
      计算期权的隐含波动率:
      期权: TSLA 看涨期权
      行权价: $250
      到期: 60天
      市场价格: $15.50
      标的价: $245
      无风险利率: 4.5%

  - description: "构建期权策略"
    input: |
      构建铁秃鹰(Iron Condor)策略:
      标的: SPY
      到期: 45天

      看涨期权 Credit Spread:
      - 卖出 $460 Call
      - 买入 $470 Call

      看跌期权 Credit Spread:
      - 卖出 $440 Put
      - 买入 $430 Put

      分析最大收益、最大损失、盈亏平衡点。
---
# Options Trading Skill

## 概述

Options Trading Skill提供完整的期权分析和交易策略开发能力,包括定价、Greeks、波动率分析和策略构建。

## 核心功能

### 1. 期权定价

- **black_scholes_price**: Black-Scholes-Merton模型
  - 欧式看涨/看跌期权
  - 支付股利版本
  - 美式期权近似

### 2. Greeks计算

- **calculate_greeks**: 所有主要Greeks
  - Delta: 对标的价格敏感度
  - Gamma: Delta敏感度
  - Theta: 时间衰减
  - Vega: 波动率敏感度
  - Rho: 利率敏感度

### 3. 隐含波动率

- **implied_volatility**: 反推隐含波动率
  - Newton-Raphson方法
  - 二分法
  - 波动率曲面构建

### 4. 波动率分析

- **volatility_surface**: 波动率曲面
  - 隐含波动率vs行权价
  - 隐含波动率vs到期时间
  - 偏度(Skew)和期限结构

### 5. 策略构建

- **options_strategy_builder**: 常用期权策略
  - Covered Call
  - Protective Put
  - Iron Condor
  - Butterfly Spread
  - Calendar Spread
  - Straddle/Strangle

### 6. 损益分析

- **options_pnl_analyzer**: 策略损益分析
  - 到期损益图
  - 盈亏平衡点
  - 最大收益/损失
  - 概率分析

## Black-Scholes模型

### 看涨期权定价公式

```
C = S₀N(d₁) - Ke^(-rT)N(d₂)

其中:
d₁ = [ln(S₀/K) + (r + σ²/2)T] / (σ√T)
d₂ = d₁ - σ√T
```

### 看跌期权定价公式

```
P = Ke^(-rT)N(-d₂) - S₀N(-d₁)
```

### 参数说明

- S₀: 标的当前价格
- K: 行权价格
- r: 无风险利率(连续复利)
- σ: 波动率
- T: 到期时间(年)
- N(x): 标准正态CDF

## Greeks公式

### Delta (Δ)

**看涨期权:**
```
Δ = N(d₁)
```

**看跌期权:**
```
Δ = N(d₁) - 1
```

### Gamma (Γ)

```
Γ = N'(d₁) / (S₀σ√T)
```

### Theta (Θ)

**看涨期权:**
```
Θ = -[S₀N'(d₁)σ / (2√T) + rKe^(-rT)N(d₂)] / 365
```

**看跌期权:**
```
Θ = -[S₀N'(d₁)σ / (2√T) - rKe^(-rT)N(-d₂)] / 365
```

### Vega (ν)

```
ν = S₀N'(d₁)√T / 100
```

### Rho (ρ)

**看涨期权:**
```
ρ = KTe^(-rT)N(d₂) / 100
```

**看跌期权:**
```
ρ = -KTe^(-rT)N(-d₂) / 100
```

## 常用期权策略

### 1. Covered Call (备兑看涨)

**构建:**
- 持有标的
- 卖出看涨期权

**适用场景:**
- 温震看涨
- 收益增强
- 持仓保护

### 2. Protective Put (保护性看跌)

**构建:**
- 持有标的
- 买入看跌期权

**适用场景:**
- 灾害保险
- 下跌保护

### 3. Iron Condor (铁秃鹰)

**构建:**
- 卖出虚值看涨期权
- 买入更虚值看涨期权
- 卖出虚值看跌期权
- 买入更虚值看跌期权

**适用场景:**
- 横盘市场
- 收取权利金

### 4. Long Straddle (买入跨式)

**构建:**
- 买入看涨期权
- 买入看跌期权(相同行权价和到期日)

**适用场景:**
- 预期大幅波动
- 方向不确定

### 5. Butterfly Spread (蝶式价差)

**构建:**
- 买入1个低行权价看涨期权
- 卖出2个中间行权价看涨期权
- 买入1个高行权价看涨期权

**适用场景:**
- 预期小幅波动
- 精确目标价位

## MCP工具说明

### black_scholes_price

期权定价

**参数:**
- underlying_price: 标的价
- strike: 行权价
- time_to_expiry: 到期时间(年)
- risk_free_rate: 无风险利率
- volatility: 波动率
- option_type: Call/Put
- dividend_yield: 股息率(可选)

**返回:**
- 期权理论价格
- 各Greeks
- 时间价值
- 内在价值

### calculate_greeks

Greeks计算

**参数:**
- option: 期权定义
- underlying_price: 标的价
- volatility: 波动率
- model: 定价模型

**返回:**
- Delta
- Gamma
- Theta
- Vega
- Rho
- 希腊字母解释

### implied_volatility

隐含波动率

**参数:**
- option: 期权定义
- market_price: 市场价格
- underlying_price: 标的价
- method: 求解方法

**返回:**
- 隐含波动率
- 迭代次数
- 收敛状态

### volatility_surface

波动率曲面

**参数:**
- underlying: 标的
- expiry_dates: 到期日列表
- strikes: 行权价列表

**返回:**
- 波动率矩阵
- 偏度数据
- 期限结构
- 可视化数据

### options_strategy_builder

策略构建器

**参数:**
- strategy_type: 策略类型
- underlying: 标的
- strikes: 行权价
- expiries: 到期日
- quantities: 数量

**返回:**
- 策略组成
- 净权利金
- 最大收益
- 最大损失
- 盈亏平衡点
- 损益图数据

### options_pnl_analyzer

损益分析

**参数:**
- strategy: 期权策略
- underlying_range: 标的价范围
- expiry_date: 分析日期

**返回:**
- 损益数据
- 盈亏平衡点
- 收益概率
- 风险指标

## 波动率类型

### 1. 历史波动率

基于过去价格波动计算

### 2. 隐含波动率

从期权价格反推

### 3. 远期波动率

两个时点之间的隐含波动率

### 4. 局部波动率

局部波动率模型(Dupire)

## 高级主题

### 1. 波动率微笑

同一到期日,不同行权价的隐含波动率差异

### 2. 期限结构

同一行权价,不同到期日的隐含波动率差异

### 3. 偏度(Skew)

虚值看跌期权相对虚值看涨期权的隐含波动率溢价

### 4. Delta对冲

动态对冲策略,保持市场中性

## 风险管理

### 1. Greeks对冲

- Delta对冲: 标的或期权
- Gamma对冲: 期权组合
- Vega对冲: 不同行权价期权

### 2. 仓位限制

- 单个期权最大仓位
- 总Greeks暴露限制
- 行业集中度限制

### 3. 压力测试

- 极端波动率情景
- 黑天鹅事件
- 流动性危机

## 最佳实践

1. **理解风险**: 期权具有非线性风险特征
2. **波动率意识**: 买入低波动率,卖出高波动率
3. **时间衰减**: 期权卖方受益于时间衰减
4. **分散化**: 避免过度集中
5. **对冲策略**: 使用Greeks对冲
6. **流动性**: 选择流动性好的合约

## 技术实现

- 数值计算: scipy.optimize
- 可视化: Plotly交互式图表
- 数据源: CBOE, IEX, Yahoo Finance

## 相关Skill

- risk-analytics: 风险管理
- portfolio-optimization: 组合优化
- sentiment-analysis: 市场情绪

## 参考资料

- Hull, J. "Options, Futures, and Other Derivatives"
- Natenberg, S. "Option Volatility and Pricing"
- Black, F., & Scholes, M. (1973). "The Pricing of Options and Corporate Liabilities"
