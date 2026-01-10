---
name: momentum-trading
description: |
  动量交易策略Skill,提供完整的动量因子分析:
  - 价格动量计算
  - 相对强度指标(RSI)
  - 多因子动量模型
  - 动量回测系统
  - 因子暴露分析

  基于Claude Agent SDK,提供科学的动量交易策略开发能力。

allowed-tools:
  - calculate_momentum
  - relative_strength_index
  - multi_factor_momentum
  - backtest_momentum
  - factor_attribution
  - momentum_screening

model: claude-sonnet-4-20250514
tags:
  - momentum
  - factors
  - strategy
  - backtesting

examples:
  - description: "计算股票动量"
    input: |
      计算AAPL的动量因子:
      - 12个月动量(扣除最近1月)
      - 相对强弱vs S&P 500
      - 动量强度评分(0-100)

  - description: "多因子动量策略"
    input: |
      构建多因子动量策略:
      因子1: 12个月价格动量 (权重40%)
      因子2: 盈利动量 (权重30%)
      因子3: 分析师评级调整 (权重30%)

      股票池: Russell 1000
      再平衡: 月度

  - description: "回测动量策略"
    input: |
      回测简单动量策略:
      - 选择过去12个月收益最高的前50只股票
      - 扣除最近1个月收益(避免短期反转)
      - 月度再平衡
      - 时间: 2010-2024

  - description: "因子归因分析"
    input: |
      分析组合的因子暴露:
      组合: 科技股精选50
      因子: 市值、动量、价值、质量、低波动
      基准: S&P 500
---
# Momentum Trading Skill

## 概述

Momentum Trading Skill提供完整的动量因子分析和交易策略开发能力,基于学术研究和业界实践。

## 核心功能

### 1. 动量计算

- **calculate_momentum**: 计算各种动量指标
  - 价格动量 (1/3/6/12个月)
  - 相对强度 (vs基准/行业)
  - 动量强度评分
  - 动量加速度

### 2. 相对强弱指标

- **relative_strength_index**: RSI计算
  - 14天RSI
  - 超买超卖判断
  - 背离识别
  - 趋势确认

### 3. 多因子模型

- **multi_factor_momentum**: 综合动量评分
  - 价格动量
  - 盈利动量
  - 估值动量
  - 分析师情绪动量

### 4. 回测系统

- **backtest_momentum**: 完整的回测框架
  - 交易成本
  - 滑点模拟
  - 再平衡频率
  - 性能指标计算

### 5. 因子归因

- **factor_attribution**: 因子暴露分析
  - 因子载荷计算
  - 因子收益贡献
  - 特质收益分析
  - 基准对比

### 6. 动量筛选

- **momentum_screening**: 动量选股
  - 多维度筛选
  - 排名评分
  - 行业中性
  - 流动性过滤

## 使用示例

### 示例1: 基础动量计算

```
计算AAPL的动量指标:
1. 3个月动量
2. 6个月动量
3. 12个月动量(扣除最近1月)
4. 相对于S&P 500的相对强度
5. 动量趋势(上升/下降/横盘)
```

### 示例2: 多因子动量策略

```
构建动量选股策略:
股票池: Russell 2000

因子:
- 价格动量: 过去12个月收益(排除最近1月) - 权重50%
- 盈利动量: 过去2季度盈利增速 - 权重30%
- 分析师调升: 过去3个月上调评级比例 - 权重20%

筛选:
- 排除流动性差的股票(日均成交额<$5M)
- 排除价格<$5的股票
- 每个行业最多20只
- 选前100只

回测: 2015-2024,月度再平衡
```

### 示例3: 动量回测

```
回测经典动量策略:
- 12-1月动量(过去12个月收益,扣除最近1月)
- 持仓前20只股票
- 月度再平衡
- 交易成本: 20bp单边
- 时间范围: 2010-2024
- 基准: S&P 500

输出:
- 年化收益
- 波动率
- 夏普比率
- 最大回撤
- 换手率
```

### 示例4: 因子归因

```
分析组合的因子暴露:
组合: 我的动量策略组合(50只股票)
基准: Russell 1000
因子:
1. Size (市值)
2. Value (估值)
3. Momentum (动量)
4. Quality (质量)
5. Low Volatility (低波)
6. Profitability (盈利能力)

计算:
- 因子载荷(暴露度)
- 因子收益贡献
- 特质收益
- R²
```

## 动量因子类型

### 1. 价格动量

**公式:**
```
Momentum(t) = (P(t) / P(t-n)) - 1
```

常见参数:
- 短期: 1个月
- 中期: 3/6个月
- 长期: 12个月
- 调整: 扣除最近1月(12-1)

### 2. 相对强度

**公式:**
```
Relative_Strength = Return_stock / Return_index
```

或者:
```
Relative_Strength = Return_stock - Return_index
```

### 3. 盈利动量

**定义:**
- EPS增速
- 营收增速
- 盈利惊喜
- 分析师预期上调

### 4. 分析师情绪动量

**指标:**
- 评级变化
- 目标价变化
- 调研热度
- 媒体关注度

## MCP工具说明

### calculate_momentum

计算动量指标

**参数:**
- symbol: 股票代码
- lookback_period: 回看期(1/3/6/12个月)
- adjust_period: 调整期(默认1个月)
- benchmark: 基准指数
- method: 计算方法

**返回:**
- 动量值
- 动量百分位
- 相对强度
- 动量趋势
- 信号(买入/持有/卖出)

### relative_strength_index

计算RSI

**参数:**
- symbol: 股票代码
- period: 周期(默认14天)
- overbought: 超买阈值(默认70)
- oversold: 超卖阈值(默认30)

**返回:**
- RSI值
- 超买超卖信号
- 背离信息
- 趋势确认

### multi_factor_momentum

多因子动量评分

**参数:**
- universe: 股票池
- factors: 因子定义(权重、参数)
- scoring_method: 评分方法
- normalization: 归一化方法
- industry_neutral: 行业中性化

**返回:**
- 综合动量评分
- 因子得分
- 排名
- 推荐股票列表

### backtest_momentum

回测动量策略

**参数:**
- strategy: 策略定义
- universe: 股票池
- start_date: 开始日期
- end_date: 结束日期
- rebalance_freq: 再平衡频率
- transaction_cost: 交易成本
- max_position: 最大持仓数

**返回:**
- 累计收益曲线
- 年化收益率
- 波动率
- 夏普比率
- 最大回撤
- 换手率
- 胜率

### factor_attribution

因子归因分析

**参数:**
- portfolio: 组合持仓
- benchmark: 基准
- factor_model: 因子模型
- period: 分析周期

**返回:**
- 因子暴露
- 因子收益贡献
- 特质收益
- R²
- 因子重要性排序

### momentum_screening

动量选股

**参数:**
- universe: 股票池
- criteria: 筛选条件
- filters: 过滤条件
- ranking: 排名规则
- top_n: 选择前N只

**返回:**
- 符合条件的股票列表
- 动量评分
- 风险指标
- 推荐理由

## 学术研究基础

### Jegadeesh & Titman (1993)

"Returns to Buying Winners and Selling Losers"
- 3-12个月动量持续
- 中期动量有效

### Fama-French动量因子(2015)

五因子模型包含动量因子
- RMW (盈利能力)
- CMA (投资)
- Momentum (动量)

### Asness (2015)

"Quality Minus Junk"
- 质量因子
- 与动量结合

## 最佳实践

1. **避免短期反转**: 使用12-1月动量
2. **控制换手率**: 设置合理的再平衡频率
3. **行业分散**: 行业中性化或权重限制
4. **风险控制**: 结合波动率调整
5. **交易成本**: 考虑实际交易成本
6. **市场环境**: 牛市中动量更强

## 策略变体

### 1. 相对强度动量

选择相对强度最高的股票

### 2. 行业动量

选择动量最强的行业

### 3. 因子动量

选择因子动量强的股票

### 4. 基本面动量

结合盈利动量和分析师预期

### 5. 风险调整动量

信息比率(IR)最高的股票

## 技术实现

- 数据源: CRSP, Compustat, IBES
- 回测框架: 自建或Quantopian/Zipline
- 统计分析: statsmodels, scipy
- 可视化: Matplotlib, Plotly

## 相关Skill

- technical-indicators: 技术指标
- risk-analytics: 风险分析
- backtesting-engine: 回测引擎
