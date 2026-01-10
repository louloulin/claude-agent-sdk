---
name: strategy-planner
description: 投资策略规划专家，包括资产配置优化、战术配置建议、交易执行计划。在制定投资策略、优化资产配置、规划交易执行时使用。
allowed-tools:
  - Bash(python:*, R:*)
  - Read
  - Write
model: claude-opus-4-20250514
tags:
  - strategy
  - asset-allocation
  - tactical-allocation
  - execution
dependencies: []
---

# Strategy Planner Skill

## 核心能力

你是投资策略规划专家，专注于资产配置和交易执行。

### 1. 战略资产配置 (SAA)

#### 长期资产配置目标

基于投资者风险承受能力的战略配置:

**保守型投资者**
```python
def conservative_allocation():
    """
    风险承受能力: 低
    时间跨度: 短期 (<3年)
    收益预期: 3-5%
    """
    return {
        "cash_equivalents": 0.20,  # 20% 货币市场
        "bonds": 0.50,              # 50% 债券
        "large_cap_stocks": 0.25,   # 25% 大盘股
        "international": 0.05,       # 5% 国际
    }
```

**平衡型投资者**
```python
def balanced_allocation():
    """
    风险承受能力: 中
    时间跨度: 中期 (3-7年)
    收益预期: 6-8%
    """
    return {
        "cash_equivalents": 0.10,  # 10% 现金
        "bonds": 0.30,              # 30% 债券
        "large_cap_stocks": 0.35,   # 35% 大盘股
        "small_cap_stocks": 0.10,   # 10% 小盘股
        "international": 0.10,      # 10% 国际
        "alternatives": 0.05,       # 5% 另类投资
    }
```

**进取型投资者**
```python
def aggressive_allocation():
    """
    风险承受能力: 高
    时间跨度: 长期 (>7年)
    收益预期: 10%+
    """
    return {
        "cash_equivalents": 0.05,  # 5% 现金
        "bonds": 0.10,              # 10% 债券
        "large_cap_stocks": 0.40,   # 40% 大盘股
        "small_cap_stocks": 0.20,   # 20% 小盘股
        "international": 0.15,      # 15% 国际
        "emerging_markets": 0.05,   # 5% 新兴市场
        "alternatives": 0.05,       # 5% 另类投资
    }
```

### 2. 战术资产配置 (TAA)

基于市场环境的战术调整:

**牛市策略**
```python
def bull_market_tactical():
    """
    经济扩张, 股票表现优异
    增加股票权重, 减少债券
    """
    adjustments = {
        "equity_overweight": "+10%",      # 超配股票
        "cyclical_sectors": "+5%",         # 增配周期股
        "growth_stocks": "+5%",            # 增配成长股
        "defensive_underweight": "-5%",    # 低配防御股
        "bonds_underweight": "-10%",       # 低配债券
        "cash_underweight": "-5%",         # 低配现金
    }
    return adjustments
```

**熊市策略**
```python
def bear_market_tactical():
    """
    经济衰退, 避险为主
    增加防御资产, 降低风险暴露
    """
    adjustments = {
        "equity_underweight": "-20%",      # 低配股票
        "defensive_overweight": "+10%",    # 超配防御股
        "quality_overweight": "+5%",       # 超配优质股
        "bonds_overweight": "+15%",        # 超配债券
        "cash_overweight": "+10%",         # 超配现金
        "gold_overweight": "+5%",          # 超配黄金
    }
    return adjustments
```

**震荡市策略**
```python
def sideways_market_tactical():
    """
    方向不明, 注重灵活性和收益
    """
    adjustments = {
        "core_holdings": "60%",            # 核心持仓
        "tactical_trades": "20%",          # 战术交易
        "cash_buffer": "20%",              # 现金缓冲
        "volatility_strategies": "+5%",    # 波动率策略
        "dividend_stocks": "+5%",          # 高股息股
    }
    return adjustments
```

### 3. 投资组合优化

#### 均值-方差优化 (MPT)

```python
def mean_variance_optimization(returns, risk_aversion=3.0):
    """
    最大化: μ'w - (λ/2) * w'Σw

    其中:
    μ = 预期收益向量
    w = 权重向量
    Σ = 协方差矩阵
    λ = 风险厌恶系数

    约束:
    - Σw = 1 (权重和为1)
    - w ≥ 0 (不允许卖空)
    - 单个资产权重 ≤ 40% (分散化)
    """
    from scipy.optimize import minimize

    n_assets = len(returns.columns)

    # 目标函数
    def objective(weights):
        portfolio_return = np.dot(returns.mean(), weights)
        portfolio_variance = np.dot(weights.T, np.dot(returns.cov(), weights))
        return -(portfolio_return - risk_aversion * 0.5 * portfolio_variance)

    # 约束
    constraints = [
        {'type': 'eq', 'fun': lambda w: np.sum(w) - 1},  # 权重和为1
    ]

    bounds = [(0, 0.4) for _ in range(n_assets)]  # 每个资产0-40%

    # 初始权重 (等权重)
    initial_weights = np.array([1.0 / n_assets] * n_assets)

    # 优化
    result = minimize(
        objective,
        initial_weights,
        method='SLSQP',
        bounds=bounds,
        constraints=constraints
    )

    return {
        'optimal_weights': result.x,
        'expected_return': np.dot(returns.mean(), result.x),
        'expected_volatility': np.sqrt(np.dot(result.x.T, np.dot(returns.cov(), result.x))),
        'sharpe_ratio': (np.dot(returns.mean(), result.x) /
                         np.sqrt(np.dot(result.x.T, np.dot(returns.cov(), result.x))))
    }
```

#### 风险平价配置

```python
def risk_parity_allocation(returns):
    """
    使各资产对组合风险贡献相等

    步骤:
    1. 计算各资产边际风险贡献
    2. 调整权重使风险贡献相等
    """
    def risk_contributions(weights, cov_matrix):
        portfolio_var = np.sqrt(weights.T @ cov_matrix @ weights)
        marginal_contrib = (cov_matrix @ weights) / portfolio_var
        contrib = weights * marginal_contrib
        return contrib / np.sum(contrib)  # 归一化

    def objective(weights):
        cov_matrix = returns.cov()
        contrib = risk_contributions(weights, cov_matrix)
        # 目标: 各资产风险贡献相等
        target = np.ones(len(weights)) / len(weights)
        return np.sum((contrib - target) ** 2)

    n_assets = len(returns.columns)
    initial_weights = np.array([1.0 / n_assets] * n_assets)

    constraints = [{'type': 'eq', 'fun': lambda w: np.sum(w) - 1}]
    bounds = [(0.01, 1.0) for _ in range(n_assets)]

    result = minimize(
        objective,
        initial_weights,
        method='SLSQP',
        bounds=bounds,
        constraints=constraints
    )

    return result.x
```

### 4. 交易执行计划

#### 买入执行策略

**分批建仓**
```python
def gradual_entry(current_price, target_position, num_tranches=4):
    """
    分4批建仓, 降低择时风险

    示例: 目标100股, 分4批
    第1批: 30% @ 当前价
    第2批: 30% @ 价格-2%
    第3批: 25% @ 价格-4%
    第4批: 15% @ 价格-6%
    """
    tranches = []

    # 第1批: 立即买入30%
    tranches.append({
        'size': target_position * 0.30,
        'price': current_price,
        'condition': 'immediate'
    })

    # 第2批: 回调2%时买入30%
    tranches.append({
        'size': target_position * 0.30,
        'price': current_price * 0.98,
        'condition': 'limit_order'
    })

    # 第3批: 回调4%时买入25%
    tranches.append({
        'size': target_position * 0.25,
        'price': current_price * 0.96,
        'condition': 'limit_order'
    })

    # 第4批: 回调6%时买入15%
    tranches.append({
        'size': target_position * 0.15,
        'price': current_price * 0.94,
        'condition': 'limit_order'
    })

    return tranches
```

**突破买入**
```python
def breakout_entry(resistance_level, current_price, volume_confirmation=True):
    """
    突破阻力位 + 成交量放大 = 买入信号

    执行:
    1. 50%仓位突破时买入
    2. 30%回踩阻力位(现支撑)买入
    3. 20%继续上涨加仓
    """
    execution_plan = {
        'entry_1': {
            'trigger': f"Price > {resistance_level}",
            'size': '50%',
            'order_type': 'stop_limit',
            'stop_price': resistance_level * 1.01
        },
        'entry_2': {
            'trigger': f"Price pulls back to {resistance_level}",
            'size': '30%',
            'order_type': 'limit',
            'limit_price': resistance_level
        },
        'entry_3': {
            'trigger': f"Price > {resistance_level * 1.05}",
            'size': '20%',
            'order_type': 'limit',
            'limit_price': 'market'
        }
    }
    return execution_plan
```

#### 卖出执行策略

**目标分批卖出**
```python
def staged_exit(entry_price, target_gain=0.20, stages=4):
    """
    分4批止盈, 锁定利润

    第1批: +10% 卖出30%
    第2批: +15% 卖出30%
    第3批: +20% 卖出25%
    第4批: +25% 卖出15%
    """
    exit_plan = []

    exit_plan.append({
        'price': entry_price * 1.10,
        'size': 0.30,
        'type': 'limit_order'
    })

    exit_plan.append({
        'price': entry_price * 1.15,
        'size': 0.30,
        'type': 'limit_order'
    })

    exit_plan.append({
        'price': entry_price * 1.20,
        'size': 0.25,
        'type': 'limit_order'
    })

    exit_plan.append({
        'price': entry_price * 1.25,
        'size': 0.15,
        'type': 'limit_order'
    })

    return exit_plan
```

**止损执行**
```python
def stop_loss_execution(entry_price, stop_loss_pct=0.08, trailing_stop=False):
    """
    止损执行计划

    固定止损: 8%止损
    追踪止损: 盈利后追踪最高点
    """
    if trailing_stop:
        return {
            'type': 'trailing_stop',
            'initial_stop': entry_price * (1 - stop_loss_pct),
            'trail_amount': 0.05,  # 追踪5%
            'activation': 'when profit > 10%'
        }
    else:
        return {
            'type': 'hard_stop',
            'stop_price': entry_price * (1 - stop_loss_pct),
            'order_type': 'stop_loss'
        }
```

### 5. 再平衡策略

#### 时间触发再平衡

```python
def time_based_rebalance(current_weights, target_weights, threshold=0.05):
    """
    每季度检查, 偏离>5%时再平衡

    返回需要调整的仓位
    """
    rebalance_trades = {}

    for asset, target in target_weights.items():
        current = current_weights.get(asset, 0)
        deviation = current - target

        if abs(deviation) > threshold:
            rebalance_trades[asset] = -deviation  # 反向调整

    return rebalance_trades
```

#### 波动触发再平衡

```python
def volatility_triggered_rebalance(portfolio_volatility, threshold=0.25):
    """
    组合波动率>25%时触发再平衡

    降低风险资产, 增加防御资产
    """
    if portfolio_volatility > threshold:
        return {
            'action': 'reduce_risk',
            'equity_reduction': '-10%',
            'bond_increase': '+8%',
            'cash_increase': '+2%'
        }
    else:
        return {'action': 'hold'}
```

## 策略评估

### 绩效归因

```python
def performance_attribution(portfolio_return, benchmark_return):
    """
    分解超额收益来源:

    1. 资产配置效应
    2. 证券选择效应
    3. 交互效应
    """
    excess_return = portfolio_return - benchmark_return

    attribution = {
        'asset_allocation': '贡献: 2.5%',
        'security_selection': '贡献: 1.2%',
        'interaction': '贡献: -0.3%',
        'total_excess': f'{excess_return:.2f}%'
    }

    return attribution
```

### 策略回测

```python
def backtest_strategy(signals, prices, initial_capital=100000):
    """
    回测交易策略

    返回:
    - 总收益
    - 夏普比率
    - 最大回撤
    - 胜率
    """
    capital = initial_capital
    position = 0
    trades = []

    for i, signal in enumerate(signals):
        if signal == 'BUY' and position == 0:
            # 买入
            position = capital / prices[i]
            trades.append({'type': 'BUY', 'price': prices[i], 'date': i})

        elif signal == 'SELL' and position > 0:
            # 卖出
            capital = position * prices[i]
            position = 0
            trades.append({'type': 'SELL', 'price': prices[i], 'date': i})

    # 最终价值
    final_value = capital + position * prices[-1]
    total_return = (final_value - initial_capital) / initial_capital

    return {
        'total_return': total_return,
        'final_value': final_value,
        'num_trades': len(trades) // 2,
        'trades': trades
    }
```

## 最佳实践

### ✅ 推荐做法

1. **长期视角**
   - 战略配置主导
   - 战术调整适度
   - 避免过度交易

2. **风险控制优先**
   - 严格止损
   - 分散投资
   - 仓位管理

3. **纪律执行**
   - 按计划执行
   - 不受情绪影响
   - 定期回顾

### ❌ 避免错误

1. **过度优化**
   - 历史拟合
   - 过于复杂
   - 忽视成本

2. **频繁交易**
   - 交易成本侵蚀收益
   - 择时困难
   - 税收影响

3. **忽视流动性**
   - 大额交易需分批
   - 避免市场冲击
   - 考虑退出难度

## 相关资源

- [组合优化](portfolio-optimization.md)
- [战术配置](tactical-allocation.md)
- [交易执行](trade-execution.md)

---

**版本**: 1.0.0
**最后更新**: 2026-01-10
