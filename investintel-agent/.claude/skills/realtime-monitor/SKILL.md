---
name: realtime-monitor
description: 实时市场监控,价格提醒,异常检测,大单监控, WebSocket数据流订阅
allowed-tools:
  - subscribe_realtime_ticker
  - subscribe_binance_websocket
  - set_price_alert
  - detect_anomaly
  - monitor_large_orders
  - websocket_stream_status
model: claude-sonnet-4-20250514
tags:
  - real-time
  - monitoring
  - alerts
  - websocket
---

# Realtime Monitor Skill

实时监控市场数据,在价格达到关键水平时发送提醒,检测异常交易行为。

## 能力

1. **实时价格监控**: WebSocket订阅实时行情(Polygon, Binance)
2. **价格提醒**: 设置阈值提醒(涨跌幅、价格突破)
3. **异常检测**: 识别异常交易行为(暴涨暴跌、异常成交量)
4. **大单监控**: 追踪大额交易订单
5. **技术形态提醒**: 实时K线形态识别
6. **连接状态监控**: WebSocket连接健康检查

## 支持的数据源

- **Polygon.io**: 美股、加密货币、外汇
- **Binance**: 加密货币实时数据
- **自定义源**: 可扩展其他WebSocket数据源

## 使用示例

### 基础价格监控

```
监控AAPL的实时价格,当突破200美元时提醒我
监控BTC/USDT,检测1小时内超过5%的价格波动
```

### 异常检测

```
监控TSLA,检测异常大额卖出订单(超过10000股)
检测NVDA的异常成交量(比平均值高300%)
```

### 多symbol监控

```
同时监控AAPL, MSFT, GOOGL的实时价格
当任何一支股票涨跌超过2%时提醒我
```

### Binance加密货币

```
订阅BTC/USDT, ETH/USDT的实时WebSocket数据
监控加密货币市场大额交易
```

## 价格提醒类型

1. **价格突破提醒**: 价格突破指定水平
2. **涨跌幅提醒**: 涨跌幅超过百分比
3. **成交量异常提醒**: 成交量异常放大
4. **大单提醒**: 单笔交易超过阈值
5. **波动率提醒**: 波动率超过历史水平

## 数据延迟

- Polygon.io: ~50-100ms
- Binance: ~20-50ms
- 目标: <100ms端到端延迟

## 技术实现

- WebSocket长连接
- 自动重连机制
- 心跳检测(30秒)
- 数据缓冲和去重
- 广播订阅机制
