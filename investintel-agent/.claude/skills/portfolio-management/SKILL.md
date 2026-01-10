---
name: portfolio-management
description: 投资组合管理，包括资产配置、仓位管理、再平衡策略、绩效评估。在管理投资组合、优化资产配置、评估组合表现时使用。
allowed-tools:
  - Bash(python:*, R:*)
  - Read
  - Write
model: claude-sonnet-4-20250514
tags:
  - portfolio
  - asset-allocation
  - rebalancing
  - performance
dependencies: []
---

# Portfolio Management Skill

## 核心能力

### 1. 投资组合分析

#### 收益率计算

```python
# 时间加权收益率
def time_weighted_return(portfolio_values):
    returns = []
    for i in range(1, len(portfolio_values)):
        r = (portfolio_values[i] - portfolio_values[i-1]) / portfolio_values[i-1]
        returns.append(r)

    twr = 1
    for r in returns:
        twr *= (1 + r)
    return twr - 1

# 资金加权收益率 (XIRR)
def money_weighted_return(cash_flows, dates):
    # 使用numpy的irr函数计算
    return np.irr(cash_flows)
```

#### 风险指标

```python
# 夏普比率
def sharpe_ratio(returns, risk_free_rate):
    excess_returns = returns - risk_free_rate
    return excess_returns.mean() / excess_returns.std()

# 索提诺比率
def sortino_ratio(returns, risk_free_rate):
    excess_returns = returns - risk_free_rate
    downside_returns = excess_returns[excess_returns < 0]
    downside_deviation = downside_returns.std()
    return excess_returns.mean() / downside_deviation

# 最大回撤
def max_drawdown(portfolio_values):
    rolling_max = portfolio_values.expanding().max()
    drawdown = (portfolio_values - rolling_max) / rolling_max
    return drawdown.min()
```

### 2. 资产配置策略

#### 现代投资组合理论 (MPT)

```python
# 均值-方差优化
from scipy.optimize import minimize

def optimize_portfolio(returns):
    # 目标函数: 最小化组合方差
    def portfolio_variance(weights, cov_matrix):
        return np.dot(weights.T, np.dot(cov_matrix, weights))

    # 约束条件
    constraints = ({'type': 'eq', 'fun': lambda w: np.sum(w) - 1})
    bounds = tuple((0, 1) for _ in range(len(returns.columns)))

    # 优化
    result = minimize(portfolio_variance,
                     initial_weights,
                     args=(returns.cov(),),
                     method='SLSQP',
                     bounds=bounds,
                     constraints=constraints)

    return result.x
```

#### Black-Litterman模型

```python
# 结合投资者观点和市场均衡
def black_litterman(market_caps, equilibrium_returns,
                    investor_views, view_confidence):
    # 实现Black-Litterman模型
    # ...
    return bl_returns
```

#### 风险平价 (Risk Parity)

```python
def risk_parity_allocation(returns):
    # 使各资产对组合风险贡献相等
    # ...
    return weights
```

### 3. 组合再平衡

#### 再平衡触发条件

1. **时间触发**: 每月/每季度/每年
2. **偏差触发**: 权重偏离目标超过阈值 (如5%)
3. **波动触发**: 市场大幅波动后

#### 再平衡策略

```python
def rebalance_portfolio(current_weights, target_weights, threshold=0.05):
    """
    再平衡投资组合

    Args:
        current_weights: 当前权重
        target_weights: 目标权重
        threshold: 触发再平衡的偏差阈值

    Returns:
        需要调整的仓位
    """
    deviation = current_weights - target_weights
    rebalance_needed = np.abs(deviation) > threshold

    if rebalance_needed.any():
        adjustments = {}
        for asset, dev in zip(target_weights.index, deviation):
            if rebalance_needed[asset]:
                adjustments[asset] = -dev  # 反向调整

        return adjustments
    return None
```

### 4. 绩效归因

#### Brinson归因模型

```python
def brinson_attribution(portfolio_returns, benchmark_returns,
                      portfolio_weights, benchmark_weights):
    """
    分解超额收益来源:
    1. 配置效应 (Allocation): 资产配置差异
    2. 选择效应 (Selection): 个股选择差异
    3. 交互效应 (Interaction)
    """
    # 实现Brinson归因
    # ...
    return {
        'allocation': allocation_effect,
        'selection': selection_effect,
        'interaction': interaction_effect
    }
```

## 工作流程

### 投资组合管理流程

1. **数据收集**
```bash
# 获取持仓数据
python scripts/get_holdings.py --portfolio my_portfolio

# 获取价格数据
python scripts/get_prices.py --tickers AAPL MSFT GOOGL
```

2. **绩效分析**
```python
# 计算收益率和风险指标
python scripts/portfolio_performance.py --portfolio my_portfolio
```

3. **风险评估**
```python
# 计算VaR和压力测试
python scripts/portfolio_risk.py --portfolio my_portfolio
```

4. **再平衡建议**
```python
# 生成再平衡建议
python scripts/rebalance_suggestions.py --portfolio my_portfolio
```

## 相关资源

- [资产配置策略](allocation-strategies.md)
- [再平衡规则](rebalancing-rules.md)

---

**版本**: 1.0.0
**最后更新**: 2026-01-10
