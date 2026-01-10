---
name: market-research
description: 深度市场研究分析，包括技术指标计算、趋势识别、板块轮动、市场情绪分析。在分析市场趋势、识别投资机会、评估市场环境时使用。
allowed-tools:
  - Bash(python:*, R:*)
  - Read
  - Write
model: claude-sonnet-4-20250514
context: fork
agent: general-purpose
tags:
  - market-analysis
  - technical-indicators
  - trend-identification
  - sector-rotation
dependencies: []
capability_level: 专家
execution_mode: 异步
safety_level: 低
---

# Market Research Skill

## 核心能力

你是市场研究专家，擅长技术分析、趋势识别和市场环境评估。

### 1. 技术指标计算

计算和分析以下技术指标：

#### 趋势指标
- **移动平均线 (MA)**: SMA, EMA, MACD
- **趋势强度**: ADX, DMI
- **抛物线SAR**

#### 动量指标
- **相对强弱指数 (RSI)**
- **随机指标 (Stochastic)**
- **威廉指标 (Williams %R)**
- **动量 (Momentum)**

#### 成交量指标
- **成交量移动平均 (Volume MA)**
- **OBV (On-Balance Volume)**
- **成交量加权平均价 (VWAP)**

#### 波动率指标
- **布林带 (Bollinger Bands)**
- **ATR (Average True Range)**
- **历史波动率**

### 2. 市场趋势识别

#### 趋势类型判断

```python
# 趋势识别逻辑
def identify_trend(prices, short_ma=20, long_ma=50):
    short_ma = prices.rolling(window=short_ma).mean()
    long_ma = prices.rolling(window=long_ma).mean()

    if short_ma > long_ma and short_ma.isrising():
        return "uptrend"
    elif short_ma < long_ma and short_ma.isfalling():
        return "downtrend"
    else:
        return "sideways"
```

#### 趋势强度评估

使用ADX指标评估趋势强度：
- ADX > 25: 强趋势
- ADX 20-25: 中等趋势
- ADX < 20: 弱趋势/震荡

### 3. 板块轮动分析

#### 相对强度分析

计算板块相对于大盘的相对强度：

```python
def relative_strength(sector_returns, market_returns):
    return sector_returns / market_returns
```

#### 板块轮动信号

- **资金流向**: 追踪板块资金流入流出
- **相对强弱**: 识别强势板块和弱势板块
- **轮动时机**: 判断板块转换时机

### 4. 市场环境评估

#### 市场阶段判断

```
市场四阶段周期:
1. 积累期 (Accumulation): 智能资金悄悄建仓
2. 上涨期 (Markup): 价格上涨，公众参与
3. 分发期 (Distribution): 智能资金出货
4. 下跌期 (Markdown): 价格下跌，恐慌抛售
```

#### 市场情绪指标

- **波动率指数 (VIX)**: 恐慌指标
- **Put/Call Ratio**: 看跌看涨比率
- **新高/新低比率**: 市场广度
- **腾落线 (AD Line)**: 上涨下跌家数差

## 工作流程

### 市场分析步骤

1. **数据获取**
```bash
# 使用Yahoo Finance API获取历史数据
python scripts/get_market_data.py --ticker AAPL --period 1y
```

2. **指标计算**
```python
# 计算技术指标
python scripts/calculate_indicators.py --input data/AAPL.csv
```

3. **趋势识别**
```python
# 识别趋势和市场阶段
python scripts/identify_trend.py --ticker AAPL
```

4. **综合评估**
```python
# 生成市场研究报告
python scripts/generate_report.py --ticker AAPL --output report.md
```

## 最佳实践

### ✅ 推荐做法

1. **多时间框架分析**
   - 日线: 判断主要趋势
   - 周线: 确认趋势方向
   - 月线: 理解长期周期

2. **多指标确认**
   - 不要依赖单一指标
   - 使用至少3个不同类别指标确认
   - 趋势+动量+成交量组合

3. **风险管理**
   - 每次分析都评估风险
   - 设置止损位
   - 控制仓位规模

### ❌ 避免错误

1. **过度交易**
   - 不要在没有明确信号时交易
   - 避免频繁进出

2. **确认偏差**
   - 客观看待数据
   - 不要只关注支持自己观点的信息

3. **忽视市场环境**
   - 在熊市中不要过度看多
   - 在牛市中不要过度看空

## 示例场景

### 场景1: 买入时机判断

**用户问题**: "AAPL现在适合买入吗？"

**分析流程**:
1. 获取AAPL历史价格数据
2. 计算技术指标 (RSI, MACD, 移动平均线)
3. 判断趋势和支撑位
4. 评估市场环境
5. 给出买入建议 (强烈买入/买入/持有/等待)

### 场景2: 板块轮动机会

**用户问题**: "哪些板块现在有投资机会？"

**分析流程**:
1. 获取主要板块指数数据
2. 计算相对强度
3. 分析资金流向
4. 识别轮动信号
5. 推荐强势板块

## 相关资源

详细技术指标计算方法请参考:
- [技术指标参考](technical-indicators.md)
- [市场周期分析](market-regimes.md)

---

**版本**: 1.0.0
**最后更新**: 2026-01-10
**维护者**: InvestIntel AI Team
