---
name: trading-execution
description: |
  实时交易执行系统,提供完整的交易功能包括:
  - Binance/OKX交易所集成
  - 订单管理和监控
  - 风险控制和紧急停止
  - 账户和持仓查询

  基于Claude Agent SDK架构,提供安全可靠的交易执行能力。

allowed-tools:
  - place_order
  - cancel_order
  - get_account_info
  - get_positions
  - get_order_status
  - cancel_all_orders
  - set_leverage
  - emergency_stop
  - get_order_statistics

model: claude-sonnet-4-20250514
tags:
  - trading
  - execution
  - binance
  - okx
  - risk-management

examples:
  - description: "在Binance上下单买入BTC"
    input: |
      使用Binance交易所,以市价单买入0.001个BTC
      交易对: BTCUSDT

  - description: "查询账户余额和持仓"
    input: |
      查询我的Binance期货账户余额
      显示所有持仓的详细信息

  - description: "取消所有未成交订单"
    input: |
      取消所有BTCUSDT的未成交订单

  - description: "设置杠杆倍数"
    input: |
      在Binance上将BTCUSDT的杠杆设置为10倍

  - description: "触发紧急停止"
    input: |
      触发紧急停止,取消所有订单并平仓所有持仓

  - description: "查询订单状态"
    input: |
      查询订单123456的状态
      交易所: Binance, 交易对: BTCUSDT
---

# Trading Execution Skill

## 概述

Trading Execution Skill提供完整的实时交易执行功能,集成Binance和OKX交易所,支持订单管理、风险控制和紧急停止机制。

## 核心功能

### 1. 订单管理

- **place_order**: 下单(市价单/限价单/止损单)
- **cancel_order**: 取消指定订单
- **cancel_all_orders**: 取消所有未成交订单
- **get_order_status**: 查询订单状态

### 2. 账户查询

- **get_account_info**: 获取账户余额和资产信息
- **get_positions**: 获取当前持仓
- **get_order_statistics**: 获取订单统计信息

### 3. 风险管理

- **set_leverage**: 设置杠杆倍数
- **emergency_stop**: 触发紧急停止
- **风险预检查**: 自动验证订单大小、仓位限制、每日亏损

## 交易所支持

### Binance Futures
- ✅ 完整支持
- ✅ 市价单/限价单/止损单
- ✅ 双向持仓模式
- ✅ 杠杆调整

### OKX Futures
- ✅ 基础支持
- ⏳ 高级功能开发中

## 风控机制

### 交易前检查
- ✅ 交易对白名单
- ✅ 订单大小限制
- ✅ 每日亏损限制
- ✅ 仓位大小限制

### 紧急停止触发条件
- 每日亏损达到上限
- 仓位超过限制
- 技术错误
- 手动触发
- 保证金不足
- 网络问题

## 使用示例

### 示例1: 下单买入

```
使用Binance交易所,以市价单买入0.001个BTC
交易对: BTCUSDT
```

### 示例2: 限价单卖出

```
在Binance上设置限价单卖出0.001个BTC
交易对: BTCUSDT
价格: 50000 USDT
```

### 示例3: 查询账户

```
查询我的Binance期货账户余额
显示可用余额、总资产、未实现盈亏
```

### 示例4: 风险控制

```
检查当前风险状态
如果每日亏损超过500 USDT,触发紧急停止
```

## MCP工具说明

### place_order
在指定交易所下单

**参数:**
- exchange: 交易所 (binance/okx)
- symbol: 交易对 (如BTCUSDT)
- side: 方向 (buy/sell)
- quantity: 数量
- price: 价格 (可选,限价单)
- type: 订单类型 (market/limit/stop)

**返回:** 订单ID和状态

### cancel_order
取消指定订单

**参数:**
- exchange: 交易所
- symbol: 交易对
- order_id: 订单ID

**返回:** 取消结果

### get_account_info
获取账户信息

**参数:**
- exchange: 交易所

**返回:**
- 总资产
- 可用余额
- 未实现盈亏
- 持仓列表

### get_positions
获取持仓列表

**参数:**
- exchange: 交易所
- symbol: 交易对 (可选)

**返回:** 持仓详情

### emergency_stop
触发紧急停止

**参数:**
- reason: 停止原因
- close_positions: 是否平仓 (默认false)

**返回:** 停止报告

## 注意事项

⚠️ **重要提示:**

1. **测试环境**: 建议先在testnet环境测试
2. **风险控制**: 始终设置合理的止损和仓位限制
3. **API密钥**: 妥善保管API密钥,设置IP白名单
4. **权限管理**: API密钥只需交易权限,不要开启提现权限
5. **监控**: 定期检查订单状态和持仓情况

## 技术实现

- 100% 基于Claude Agent SDK
- 异步Rust实现(tokio)
- HMAC-SHA256签名
- 完整的错误处理
- 自动重试机制

## 相关文件

- `investintel-agent/trading/binance.rs` - Binance客户端
- `investintel-agent/trading/okx.rs` - OKX客户端
- `investintel-agent/trading/order_manager.rs` - 订单管理
- `investintel-agent/trading/emergency_stop.rs` - 紧急停止
