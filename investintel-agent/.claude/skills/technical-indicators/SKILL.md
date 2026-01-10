---
name: technical-indicators
description: |
  技术指标分析Skill,提供全面的技术分析工具:
  - 趋势指标(MA, EMA, MACD, Bollinger Bands)
  - 动量指标(RSI, Stochastic, Williams %R)
  - 成交量指标(OBV, VWAP, Volume Profile)
  - 波动率指标(ATR, Keltner Channels)
  - 图表模式识别

  基于Claude Agent SDK,提供专业的技术分析能力。

allowed-tools:
  - moving_averages
  - macd_indicator
  - rsi_indicator
  - bollinger_bands
  - stochastic_oscillator
  - atr_indicator
  - volume_profile
  - pattern_recognition

model: claude-sonnet-4-20250514
tags:
  - technical-analysis
  - indicators
  - chart-patterns
  - trading-signals

examples:
  - description: "均线分析"
    input: |
      分析AAPL的均线:
      - 50日简单移动平均(SMA50)
      - 200日简单移动平均(SMA200)
      - 20日指数移动平均(EMA20)
      - 判断金叉死叉信号

  - description: "MACD指标"
    input: |
      计算MACD指标:
      - 快线EMA: 12日
      - 慢线EMA: 26日
      - 信号线EMA: 9日
      - 识别柱状图变化

  - description: "RSI超买超卖"
    input: |
      分析RSI指标:
      - 周期: 14天
      - 超买阈值: 70
      - 超卖阈值: 30
      - 识别背离信号

  - description: "布林带突破"
    input: |
      分析布林带:
      - 周期: 20日
      - 标准差: 2
      - 识别突破和squeeze
---
# Technical Indicators Skill

## 概述

Technical Indicators Skill提供全面的技术分析工具,涵盖趋势、动量、成交量等各个维度。

## 核心功能

### 1. 趋势指标

- **moving_averages**: 移动平均线
  - SMA(简单移动平均)
  - EMA(指数移动平均)
  - WMA(加权移动平均)
  - 多重均线系统

- **macd_indicator**: MACD指标
  - MACD线
  - 信号线
  - 柱状图(Histogram)
  - 金叉死叉信号

### 2. 动量指标

- **rsi_indicator**: 相对强弱指标
  - RSI值
  - 超买超卖判断
  - 背离识别
  - 趋势确认

- **stochastic_oscillator**: 随机振荡器
  - %K线
  - %D线
  - 超买超卖
  - 信号交叉

### 3. 波动率指标

- **bollinger_bands**: 布林带
  - 上轨
  - 中轨(SMA)
  - 下轨
  - 带宽
  - %B指标

- **atr_indicator**: 平均真实波幅
  - ATR值
  - ATR止损位
  - 波动率趋势

### 4. 成交量指标

- **volume_profile**: 成交量分布
  - VWAP(成交量加权平均价)
  - OBV(能量潮)
  - 成交量分布图
  - 成交量确认

### 5. 模式识别

- **pattern_recognition**: 图表模式
  - 头肩顶/底
  - 双顶/双底
  - 三角形整理
  - 楔形
  - 旗形/三角旗

## 移动平均线

### 简单移动平均(SMA)

```python
def sma(prices, period):
    """
    简单移动平均

    SMA = (P₁ + P₂ + ... + Pₙ) / n
    """
    return prices.rolling(window=period).mean()
```

### 指数移动平均(EMA)

```python
def ema(prices, period):
    """
    指数移动平均

    EMAₜ = α × Pₜ + (1-α) × EMAₜ₋₁
    α = 2 / (period + 1)
    """
    return prices.ewm(span=period, adjust=False).mean()
```

### 均线系统信号

```python
def ma_crossover_signals(prices, fast_period=50, slow_period=200):
    """
    均线交叉信号

    金叉: 快线上穿慢线 → 买入信号
    死叉: 快线下穿慢线 → 卖出信号
    """
    fast_ma = ema(prices, fast_period)
    slow_ma = ema(prices, slow_period)

    # 交叉信号
    crossover = fast_ma > slow_ma
    signals = crossover.diff().fillna(0)

    return {
        'fast_ma': fast_ma,
        'slow_ma': slow_ma,
        'golden_cross': signals[signals > 0],
        'death_cross': signals[signals < 0]
    }
```

## MACD指标

### 计算公式

```python
def macd(prices, fast=12, slow=26, signal=9):
    """
    MACD指标

    MACD线 = EMA(12) - EMA(26)
    信号线 = EMA(MACD, 9)
    柱状图 = MACD - 信号线
    """
    ema_fast = ema(prices, fast)
    ema_slow = ema(prices, slow)

    macd_line = ema_fast - ema_slow
    signal_line = ema(macd_line, signal)
    histogram = macd_line - signal_line

    return {
        'macd': macd_line,
        'signal': signal_line,
        'histogram': histogram
    }
```

### 信号解读

1. **金叉**: MACD线上穿信号线 → 买入
2. **死叉**: MACD线下穿信号线 → 卖出
3. **柱状图扩张**: 动量增强
4. **柱状图收缩**: 动量减弱
5. **顶背离**: 价格新高但MACD不创新高 → 卖出
6. **底背离**: 价格新低但MACD不创新低 → 买入

## RSI指标

### 计算公式

```python
def rsi(prices, period=14):
    """
    相对强弱指标

    RSI = 100 - (100 / (1 + RS))
    RS = 平均涨幅 / 平均跌幅
    """
    delta = prices.diff()

    gains = delta.where(delta > 0, 0)
    losses = -delta.where(delta < 0, 0)

    avg_gains = gains.rolling(period).mean()
    avg_losses = losses.rolling(period).mean()

    rs = avg_gains / avg_losses
    rsi = 100 - (100 / (1 + rs))

    return rsi
```

### 信号解读

- **RSI > 70**: 超买,考虑卖出
- **RSI < 30**: 超卖,考虑买入
- **RSI > 80**: 严重超买
- **RSI < 20**: 严重超卖
- **RSI = 50**: 中性

### 背离识别

```python
def detect_divergence(prices, rsi_values, lookback=20):
    """
    检测RSI背离

    顶背离: 价格新高但RSI不创新高
    底背离: 价格新低但RSI不创新低
    """
    # 寻找价格高点
    price_peaks = find_peaks(prices, lookback)

    # 顶背离
    bullish_divergence = []
    for peak in price_peaks:
        if rsi_values[peak] < max(rsi_values[peak-lookback:peak]):
            bullish_divergence.append(peak)

    # 寻找价格低点
    price_troughs = find_troughs(prices, lookback)

    # 底背离
    bearish_divergence = []
    for trough in price_troughs:
        if rsi_values[trough] > min(rsi_values[trough-lookback:trough]):
            bearish_divergence.append(trough)

    return {
        'bullish_divergence': bullish_divergence,
        'bearish_divergence': bearish_divergence
    }
```

## 布林带

### 计算公式

```python
def bollinger_bands(prices, period=20, num_std=2):
    """
    布林带

    中轨 = SMA(period)
    上轨 = 中轨 + num_std × 标准差
    下轨 = 中轨 - num_std × 标准差
    """
    middle_band = sma(prices, period)
    std = prices.rolling(period).std()

    upper_band = middle_band + (num_std * std)
    lower_band = middle_band - (num_std * std)

    # 带宽
    bandwidth = (upper_band - lower_band) / middle_band

    # %B指标
    percent_b = (prices - lower_band) / (upper_band - lower_band)

    return {
        'middle': middle_band,
        'upper': upper_band,
        'lower': lower_band,
        'bandwidth': bandwidth,
        'percent_b': percent_b
    }
```

### 信号解读

1. **突破上轨**: 强势,可能持续
2. **突破下轨**: 弱势,可能持续
3. **回归中轨**: 均值回归
4. **带宽收缩(Squeeze)**: 突破前兆
5. **%B > 1**: 价格在上轨之上
6. **%B < 0**: 价格在下轨之下

### Squeeze识别

```python
def detect_squeeze(bandwidth, threshold=0.05, lookback=20):
    """
    检测布林带收缩

    Squeeze: 带宽降至最低20%分位数以下
    """
    squeeze_threshold = bandwidth.rolling(lookback).quantile(threshold)
    is_squeeze = bandwidth < squeeze_threshold

    return is_squeeze
```

## 随机振荡器

### 计算公式

```python
def stochastic_oscillator(high, low, close, k_period=14, d_period=3):
    """
    随机振荡器

    %K = 100 × (Close - Low14) / (High14 - Low14)
    %D = SMA(%K, 3)
    """
    low14 = low.rolling(k_period).min()
    high14 = high.rolling(k_period).max()

    k_percent = 100 * (close - low14) / (high14 - low14)
    d_percent = k_percent.rolling(d_period).mean()

    return {
        'k': k_percent,
        'd': d_percent
    }
```

### 信号解读

- **%K > 80**: 超买
- **%K < 20**: 超卖
- **%K上穿%D**: 买入信号
- **%K下穿%D**: 卖出信号

## ATR(平均真实波幅)

### 计算公式

```python
def atr(high, low, close, period=14):
    """
    平均真实波幅

    TR = max(High-Low, |High-Close_prev|, |Low-Close_prev|)
    ATR = SMA(TR, 14)
    """
    prev_close = close.shift(1)

    tr1 = high - low
    tr2 = (high - prev_close).abs()
    tr3 = (low - prev_close).abs()

    true_range = pd.concat([tr1, tr2, tr3], axis=1).max(axis=1)
    average_true_range = sma(true_range, period)

    return average_true_range
```

### 应用场景

1. **止损位**: ATR倍数止损
2. **仓位管理**: 根据波动率调整仓位
3. **波动率趋势**: ATR上升/下降

```python
def atr_stop_loss(prices, atr_values, multiplier=2):
    """
    ATR止损

    止损价 = 入场价 - multiplier × ATR
    """
    long_stop = prices - (multiplier * atr_values)
    short_stop = prices + (multiplier * atr_values)

    return {
        'long_stop': long_stop,
        'short_stop': short_stop
    }
```

## 成交量指标

### OBV(能量潮)

```python
def obv(prices, volume):
    """
    能量潮

    如果今日收盘价 > 昨日收盘价: OBV += 成交量
    如果今日收盘价 < 昨日收盘价: OBV -= 成交量
    """
    obv_values = pd.Series(index=prices.index, dtype=float)
    obv_values.iloc[0] = volume.iloc[0]

    for i in range(1, len(prices)):
        if prices.iloc[i] > prices.iloc[i-1]:
            obv_values.iloc[i] = obv_values.iloc[i-1] + volume.iloc[i]
        elif prices.iloc[i] < prices.iloc[i-1]:
            obv_values.iloc[i] = obv_values.iloc[i-1] - volume.iloc[i]
        else:
            obv_values.iloc[i] = obv_values.iloc[i-1]

    return obv_values
```

### VWAP

```python
def vwap(prices, volumes, period=20):
    """
    成交量加权平均价

    VWAP = Σ(Price × Volume) / ΣVolume
    """
    typical_price = (prices['high'] + prices['low'] + prices['close']) / 3
    tp_volume = typical_price * volumes['volume']

    cum_tp_volume = tp_volume.rolling(period).sum()
    cum_volume = volumes['volume'].rolling(period).sum()

    vwap = cum_tp_volume / cum_volume

    return vwap
```

## 图表模式

### 头肩顶/底

```python
def detect_head_shoulders(prices, lookback=100):
    """
    检测头肩顶/底模式

    头肩顶: 左峰 < 头部 > 右峰
    头肩底: 左谷 > 头谷 < 右谷
    """
    peaks = find_peaks(prices, lookback)
    troughs = find_troughs(prices, lookback)

    head_shoulders_top = []
    head_shoulders_bottom = []

    # 检测头肩顶
    for i in range(1, len(peaks)-1):
        left_shoulder = prices[peaks[i-1]]
        head = prices[peaks[i]]
        right_shoulder = prices[peaks[i+1]]

        if (left_shoulder < head > right_shoulder and
            abs(left_shoulder - right_shoulder) / head < 0.05):
            head_shoulders_top.append(peaks[i])

    # 检测头肩底
    for i in range(1, len(troughs)-1):
        left_shoulder = prices[troughs[i-1]]
        head = prices[troughs[i]]
        right_shoulder = prices[troughs[i+1]]

        if (left_shoulder > head < right_shoulder and
            abs(left_shoulder - right_shoulder) / head < 0.05):
            head_shoulders_bottom.append(troughs[i])

    return {
        'head_shoulders_top': head_shoulders_top,
        'head_shoulders_bottom': head_shoulders_bottom
    }
```

### 双顶/双底

```python
def detect_double_tops_bottoms(prices, tolerance=0.02):
    """
    检测双顶/双底

    双顶: 两个相近高点
    双底: 两个相近低点
    """
    peaks = find_peaks(prices, lookback=20)
    troughs = find_troughs(prices, lookback=20)

    double_tops = []
    double_bottoms = []

    # 检测双顶
    for i in range(len(peaks)-1):
        price1 = prices[peaks[i]]
        price2 = prices[peaks[i+1]]

        if abs(price1 - price2) / price1 < tolerance:
            double_tops.append((peaks[i], peaks[i+1]))

    # 检测双底
    for i in range(len(troughs)-1):
        price1 = prices[troughs[i]]
        price2 = prices[troughs[i+1]]

        if abs(price1 - price2) / price1 < tolerance:
            double_bottoms.append((troughs[i], troughs[i+1]))

    return {
        'double_tops': double_tops,
        'double_bottoms': double_bottoms
    }
```

## 多指标确认

```python
def multi_indicator_confirmation(prices, high, low, close, volume):
    """
    多指标确认

    当多个指标同时给出信号时,信号更可靠
    """
    # 计算各指标
    macd_data = macd(close)
    rsi_values = rsi(close)
    bb_data = bollinger_bands(close)
    stoch_data = stochastic_oscillator(high, low, close)

    # 买入信号
    buy_signals = []

    # 1. MACD金叉
    if macd_data['macd'].iloc[-1] > macd_data['signal'].iloc[-1]:
        buy_signals.append('MACD金叉')

    # 2. RSI超卖回升
    if 30 < rsi_values.iloc[-1] < 50:
        if rsi_values.iloc[-1] > rsi_values.iloc[-2]:
            buy_signals.append('RSI超卖回升')

    # 3. 价格触及下轨
    if close.iloc[-1] < bb_data['lower'].iloc[-1]:
        buy_signals.append('触及布林带下轨')

    # 4. 随机指标超卖
    if stoch_data['k'].iloc[-1] < 20:
        buy_signals.append('随机指标超卖')

    # 5. 成交量确认
    if volume.iloc[-1] > volume.iloc[-20:].mean():
        buy_signals.append('成交量放大')

    return {
        'buy_signals': buy_signals,
        'signal_strength': len(buy_signals) / 5  # 信号强度
    }
```

## 最佳实践

1. **多时间框架**: 结合日、周、月线
2. **多指标确认**: 不要依赖单一指标
3. **趋势为王**: 顺势交易,逆势谨慎
4. **成交量验证**: 价格变化需要成交量确认
5. **支撑阻力**: 结合历史价格水平
6. **风险管理**: 始终设置止损

## 技术实现

- 数据分析: pandas, numpy
- 技术指标: TA-Lib, pandas-ta
- 可视化: matplotlib, plotly
- 模式识别: scipy.signal

## 相关Skill

- momentum-trading: 动量与趋势
- backtesting-engine: 指标回测
- sentiment-analysis: 情绪与技术面结合

## 参考资料

- Murphy, J. J. "Technical Analysis of the Financial Markets"
- Kirkpatrick, C. D., & Dahlquist, J. R. "Technical Analysis"
- Pring, M. J. "Technical Analysis Explained"
