---
name: technical-analysis
description: 技术分析专家，包括图表形态识别、指标信号、时机选择策略。在进行技术分析、识别买卖信号、选择交易时机时使用。
allowed-tools:
  - Bash(python:*, R:*)
  - Read
  - Write
model: claude-sonnet-4-20250514
tags:
  - technical-analysis
  - chart-patterns
  - trading-signals
  - timing
dependencies: []
---

# Technical Analysis Skill

## 核心能力

你是技术分析专家，专注于图表形态识别和交易时机选择。

### 1. 图表形态识别

#### 反转形态

**头肩顶/头肩底** (Head and Shoulders)
```
特征:
- 三个峰/谷，中间最高/最低
- 颈线突破确认形态
- 成交量在突破时放大

交易信号:
- 头肩顶: 突破颈线做空
- 头肩底: 突破颈线做多
```

**双顶/双底** (Double Top/Bottom)
```
特征:
- 两个相近的高点/低点
- 颈线支撑/阻力
- 成交量递减

交易信号:
- 双顶: 突破颈线做空
- 双底: 突破颈线做多
```

**三重顶/三重底** (Triple Top/Bottom)
```
特征:
- 三个相近的高点/低点
- 更强的反转信号
- 需要更大成交量确认
```

#### 持续形态

**三角形** (Triangles)
- 上升三角形: 看涨突破
- 下降三角形: 看跌突破
- 对称三角形: 方向不确定

**旗形/三角旗** (Flags/Pennants)
- 牛旗/熊旗: 强趋势后的整理
- 突破方向与趋势相同

**楔形** (Wedges)
- 上升楔形: 看跌
- 下降楔形: 看涨

### 2. 指标信号系统

#### 趋势指标信号

**移动平均线交叉**
```python
def ma_cross_signal(price, short_ma=20, long_ma=50):
    """
    金叉: 短期MA上穿长期MA -> 买入信号
    死叉: 短期MA下穿长期MA -> 卖出信号
    """
    if short_ma > long_ma and short_ma_was_below:
        return "golden_cross - BUY"
    elif short_ma < long_ma and short_ma_was_above:
        return "death_cross - SELL"
    return "HOLD"
```

**MACD信号**
```python
def macd_signal(macd_line, signal_line, histogram):
    """
    MACD金叉: MACD线上穿信号线 -> 买入
    MACD死叉: MACD线下穿信号线 -> 卖出
    柱状图: 正值看多，负值看空
    """
    if macd_line > signal_line and macd_was_below:
        return "bullish crossover - BUY"
    elif macd_line < signal_line and macd_was_above:
        return "bearish crossover - SELL"
```

#### 动量指标信号

**RSI信号**
```python
def rsi_signal(rsi, period=14):
    """
    RSI > 70: 超买，考虑卖出
    RSI < 30: 超卖，考虑买入
    RSI 40-60: 中性区域
    """
    if rsi > 70:
        return "overbought - consider SELL"
    elif rsi < 30:
        return "oversold - consider BUY"
    elif rsi > 50:
        return "bullish zone"
    else:
        return "bearish zone"
```

**随机指标 (Stochastic)**
```python
def stochastic_signal(k, d, period=14):
    """
    %K > %D: 金叉买入
    %K < %D: 死叉卖出
    %K > 80: 超买
    %K < 20: 超卖
    """
    if k > 80:
        return "overbought"
    elif k < 20:
        return "oversold"
    elif k > d and k_was_below:
        return "bullish crossover"
    elif k < d and k_was_above:
        return "bearish crossover"
```

### 3. 交易时机选择

#### 入场时机

**突破买入**
```python
def breakout_entry(price, resistance, volume_confirm=True):
    """
    价格突破阻力位 + 成交量放大 = 买入信号
    """
    if price > resistance and volume_confirm:
        return "ENTRY - Breakout confirmed"
    return "WAIT"
```

**回撤买入**
```python
def pullback_entry(price, support, trend_is_up=True):
    """
    上升趋势中价格回撤至支撑位 = 买入机会
    """
    if trend_is_up and price_near_support:
        return "ENTRY - Pullback to support"
    return "WAIT"
```

#### 出场时机

**阻力位卖出**
```python
def resistance_exit(price, resistance, target_profit=0.05):
    """
    价格接近阻力位或达到目标利润 = 卖出信号
    """
    if price >= resistance * (1 - 0.01):
        return "EXIT - Near resistance"
    if price >= entry_price * (1 + target_profit):
        return "EXIT - Target reached"
    return "HOLD"
```

**止损退出**
```python
def stop_loss_exit(price, stop_loss, trailing_stop=False):
    """
    价格触及止损位 = 强制平仓
    """
    if price <= stop_loss:
        return "EXIT - Stop loss triggered"
    if trailing_stop:
        # 更新止损位
        update_trailing_stop()
    return "HOLD"
```

### 4. 多时间框架分析

**时间框架同步**
```python
def multi_timeframe_signal(daily, weekly, monthly):
    """
    日线、周线、月线信号一致 = 高置信度
    """
    signals = [daily, weekly, monthly]
    bullish_count = sum(s == "bullish" for s in signals)
    bearish_count = sum(s == "bearish" for s in signals)

    if bullish_count == 3:
        return "Strong BUY - All timeframes aligned"
    elif bearish_count == 3:
        return "Strong SELL - All timeframes aligned"
    else:
        return "WAIT - Timeframes not aligned"
```

## 交易策略示例

### 策略1: MA交叉 + RSI确认

```python
def ma_rsi_strategy(price, short_ma, long_ma, rsi):
    # MA交叉信号
    ma_signal = ma_cross_signal(short_ma, long_ma)

    # RSI确认
    rsi_confirm = (30 < rsi < 70)  # RSI不在极值区

    if ma_signal == "golden_cross" and rsi_confirm:
        return "BUY - Golden cross with RSI confirmation"
    elif ma_signal == "death_cross" and rsi_confirm:
        return "SELL - Death cross with RSI confirmation"
    else:
        return "HOLD"
```

### 策略2: 突破 + 成交量

```python
def breakout_volume_strategy(price, resistance, volume, avg_volume):
    # 价格突破
    breakout = price > resistance

    # 成交量确认 (至少高于平均50%)
    volume_confirm = volume > avg_volume * 1.5

    if breakout and volume_confirm:
        return "BUY - Breakout with volume confirmation"
    else:
        return "WAIT"
```

### 策略3: 背离交易

```python
def divergence_strategy(price_rsi, price_trend, rsi_trend):
    """
    价格创新高但RSI不创新高 = 看跌背离
    价格创新低但RSI不创新低 = 看涨背离
    """
    if price_trend == "up" and rsi_trend == "down":
        return "bearish divergence - SELL signal"
    elif price_trend == "down" and rsi_trend == "up":
        return "bullish divergence - BUY signal"
    else:
        return "No divergence"
```

## 最佳实践

### ✅ 推荐做法

1. **多指标确认**
   - 至少3个指标确认
   - 不同类型指标组合 (趋势+动量+成交量)

2. **多时间框架分析**
   - 月线看大趋势
   - 周线看中期趋势
   - 日线选入场点

3. **严格止损**
   - 每笔交易设止损
   - 止损位基于技术位
   - 不移动止损(除非追踪止损)

4. **风险管理**
   - 单笔风险<2%
   - 总仓位控制
   - 分批建仓/平仓

### ❌ 避免错误

1. **过度交易**
   - 不要追求每个信号
   - 等待高置信度机会

2. **忽视止损**
   - 严格止损是生存基础
   - 不要让亏损扩大

3. **追涨杀跌**
   - 等待回调入场
   - 不要在突破后追高

## 相关资源

- [图表形态详解](chart-patterns.md)
- [指标信号参考](indicator-signals.md)
- [时机选择策略](timing-strategies.md)

---

**版本**: 1.0.0
**最后更新**: 2026-01-10
