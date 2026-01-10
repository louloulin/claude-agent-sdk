---
name: backtesting
description: 回测引擎专家，包括策略回测、参数优化、绩效指标计算、回测报告生成。在测试交易策略、优化参数、评估历史表现时使用。
allowed-tools:
  - Bash(python:*, R:*)
  - Read
  - Write
model: claude-opus-4-20250514
tags:
  - backtesting
  - performance-analysis
  - parameter-optimization
  - strategy-validation
dependencies: []
---

# Backtesting Skill

## 核心能力

你是回测专家，专注于策略验证和绩效分析。

### 1. 回测框架

#### 向量化回测

```python
def vectorized_backtest(signals, prices, initial_capital=100000):
    """
    向量化回测 - 快速处理大量数据

    Args:
        signals: 信号序列 (1=做多, 0=空仓, -1=做空)
        prices: 价格序列
        initial_capital: 初始资金

    Returns:
        回测结果字典
    """
    import numpy as np
    import pandas as pd

    # 计算收益率
    returns = prices.pct_change()

    # 计算策略收益
    strategy_returns = signals.shift(1) * returns  # 使用前一日的信号

    # 累计收益
    cumulative_returns = (1 + strategy_returns).cumprod()

    # 计算权益曲线
    equity_curve = initial_capital * cumulative_returns

    # 计算回撤
    rolling_max = equity_curve.expanding().max()
    drawdown = (equity_curve - rolling_max) / rolling_max

    return {
        'equity_curve': equity_curve,
        'total_return': (equity_curve.iloc[-1] - initial_capital) / initial_capital,
        'max_drawdown': drawdown.min(),
        'final_value': equity_curve.iloc[-1],
        'returns': strategy_returns
    }
```

#### 事件驱动回测

```python
class EventDrivenBacktest:
    """
    事件驱动回测 - 更接近实盘交易
    """

    def __init__(self, initial_capital=100000, commission=0.001):
        self.capital = initial_capital
        self.commission = commission
        self.position = 0
        self.cash = initial_capital
        self.trades = []
        self.equity_curve = []

    def on_signal(self, signal, price, timestamp):
        """
        处理交易信号

        signal: 'BUY' | 'SELL' | 'HOLD'
        price: 当前价格
        timestamp: 时间戳
        """
        if signal == 'BUY' and self.position == 0:
            self._buy(price, timestamp)
        elif signal == 'SELL' and self.position > 0:
            self._sell(price, timestamp)
        # 更新权益
        self._update_equity(price, timestamp)

    def _buy(self, price, timestamp):
        """买入执行"""
        # 考虑交易成本
        buy_price = price * (1 + self.commission)
        shares = int(self.cash / buy_price)

        if shares > 0:
            self.position = shares
            self.cash -= shares * buy_price

            self.trades.append({
                'type': 'BUY',
                'shares': shares,
                'price': price,
                'timestamp': timestamp,
                'cost': shares * buy_price
            })

    def _sell(self, price, timestamp):
        """卖出执行"""
        # 考虑交易成本
        sell_price = price * (1 - self.commission)
        proceeds = self.position * sell_price

        self.cash += proceeds
        self.trades.append({
            'type': 'SELL',
            'shares': self.position,
            'price': price,
            'timestamp': timestamp,
            'proceeds': proceeds
        })
        self.position = 0

    def _update_equity(self, price, timestamp):
        """更新权益"""
        equity = self.cash + self.position * price
        self.equity_curve.append({
            'timestamp': timestamp,
            'equity': equity
        })

    def get_results(self):
        """获取回测结果"""
        equity_values = [e['equity'] for e in self.equity_curve]
        initial = equity_values[0]
        final = equity_values[-1]

        # 计算收益率序列
        returns = pd.Series(equity_values).pct_change().dropna()

        # 最大回撤
        rolling_max = pd.Series(equity_values).expanding().max()
        drawdown = (pd.Series(equity_values) - rolling_max) / rolling_max

        return {
            'total_return': (final - initial) / initial,
            'final_value': final,
            'max_drawdown': drawdown.min(),
            'num_trades': len([t for t in self.trades if t['type'] in ['BUY', 'SELL']]),
            'trades': self.trades,
            'equity_curve': self.equity_curve,
            'returns': returns
        }
```

### 2. 绩效指标计算

#### 收益率指标

```python
def calculate_return_metrics(equity_curve):
    """
    计算各种收益率指标
    """
    returns = equity_curve.pct_change().dropna()

    metrics = {}

    # 总收益率
    total_return = (equity_curve.iloc[-1] - equity_curve.iloc[0]) / equity_curve.iloc[0]
    metrics['total_return'] = total_return

    # 年化收益率
    n_days = len(equity_curve)
    annual_return = (1 + total_return) ** (252 / n_days) - 1
    metrics['annual_return'] = annual_return

    # 月度收益率
    monthly_returns = equity_curve.resample('M').last().pct_change().dropna()
    metrics['avg_monthly_return'] = monthly_returns.mean()

    # 波动率
    metrics['daily_volatility'] = returns.std()
    metrics['annual_volatility'] = returns.std() * np.sqrt(252)

    return metrics
```

#### 风险调整收益指标

```python
def calculate_risk_adjusted_metrics(returns, risk_free_rate=0.02):
    """
    计算风险调整后的收益指标
    """
    import numpy as np

    metrics = {}

    # 夏普比率 (年化)
    excess_returns = returns - risk_free_rate / 252
    sharpe = excess_returns.mean() / returns.std() * np.sqrt(252)
    metrics['sharpe_ratio'] = sharpe

    # 索提诺比率
    downside_returns = returns[returns < 0]
    if len(downside_returns) > 0:
        sortino = excess_returns.mean() / downside_returns.std() * np.sqrt(252)
    else:
        sortino = np.inf
    metrics['sortino_ratio'] = sortino

    # 卡尔玛比率
    max_drawdown = calculate_max_drawdown(returns)
    if max_drawdown != 0:
        calmar = returns.mean() * 252 / abs(max_drawdown)
    else:
        calmar = np.inf
    metrics['calmar_ratio'] = calmar

    return metrics
```

#### 回撤指标

```python
def calculate_drawdown_metrics(equity_curve):
    """
    计算回撤相关指标
    """
    # 计算滚动最高点
    rolling_max = equity_curve.expanding().max()

    # 计算回撤
    drawdown = (equity_curve - rolling_max) / rolling_max

    metrics = {
        'max_drawdown': drawdown.min(),
        'avg_drawdown': drawdown[drawdown < 0].mean(),
        'max_drawdown_duration': (drawdown.idxmin() - drawdown[drawdown == 0].last_valid_index()).days,
    }

    return metrics
```

#### 交易统计

```python
def calculate_trade_statistics(trades):
    """
    计算交易相关统计
    """
    if not trades:
        return {}

    buy_trades = [t for t in trades if t['type'] == 'BUY']
    sell_trades = [t for t in trades if t['type'] == 'SELL']

    statistics = {
        'total_trades': len(buy_trades),
        'winning_trades': 0,
        'losing_trades': 0,
        'avg_trade_return': 0,
        'win_rate': 0
    }

    # 配对买卖交易
    paired_trades = []
    for sell in sell_trades:
        # 找到对应的买入
        for buy in reversed(buy_trades):
            if buy['timestamp'] < sell['timestamp']:
                trade_return = (sell['proceeds'] - buy['cost']) / buy['cost']
                paired_trades.append(trade_return)
                buy_trades.remove(buy)
                break

    if paired_trades:
        statistics['winning_trades'] = sum(1 for r in paired_trades if r > 0)
        statistics['losing_trades'] = sum(1 for r in paired_trades if r <= 0)
        statistics['avg_trade_return'] = np.mean(paired_trades)
        statistics['win_rate'] = statistics['winning_trades'] / len(paired_trades)
        statistics['best_trade'] = max(paired_trades)
        statistics['worst_trade'] = min(paired_trades)

    return statistics
```

### 3. 参数优化

#### 网格搜索

```python
def grid_search_optimization(strategy_func, param_grid, data):
    """
    网格搜索参数优化

    Args:
        strategy_func: 策略函数, 返回信号序列
        param_grid: 参数网格
        data: 价格数据

    Returns:
        最优参数组合
    """
    from itertools import product

    best_sharpe = -np.inf
    best_params = None
    results = []

    # 生成所有参数组合
    param_combinations = product(*param_grid.values())

    for params in param_combinations:
        param_dict = dict(zip(param_grid.keys(), params))

        # 运行回测
        signals = strategy_func(data, **param_dict)
        backtest_result = vectorized_backtest(signals, data)

        # 计算夏普比率
        returns = backtest_result['returns']
        sharpe = returns.mean() / returns.std() * np.sqrt(252)

        results.append({
            'params': param_dict,
            'sharpe': sharpe,
            'total_return': backtest_result['total_return'],
            'max_drawdown': backtest_result['max_drawdown']
        })

        if sharpe > best_sharpe:
            best_sharpe = sharpe
            best_params = param_dict

    return {
        'best_params': best_params,
        'best_sharpe': best_sharpe,
        'all_results': results
    }
```

#### 遗传算法优化

```python
from scipy.optimize import differential_evolution

def genetic_optimization(strategy_func, param_bounds, data):
    """
    使用遗传算法优化参数

    Args:
        strategy_func: 策略函数
        param_bounds: 参数边界 [(min, max), ...]
        data: 价格数据

    Returns:
        最优参数
    """

    def objective(params):
        # 将参数传递给策略
        signals = strategy_func(data, *params)
        result = vectorized_backtest(signals, data)

        # 目标: 最大化夏普比率
        returns = result['returns']
        if returns.std() == 0:
            return 1e6  # 惩罚

        sharpe = returns.mean() / returns.std()
        return -sharpe  # 最小化负夏普 = 最大化夏普

    # 遗传算法优化
    result = differential_evolution(
        objective,
        bounds=param_bounds,
        maxiter=100,
        popsize=15
    )

    return {
        'optimal_params': result.x,
        'optimal_sharpe': -result.fun
    }
```

### 4. 前瞻性测试

#### Walk-Forward分析

```python
def walk_forward_analysis(strategy_func, data, train_size=252, test_size=63):
    """
    Walk-Forward分析 - 模拟实盘交易

    过程:
    1. 使用训练期优化参数
    2. 在测试期使用优化参数交易
    3. 滚动窗口重复
    """
    results = []

    for i in range(train_size, len(data) - test_size, test_size):
        # 训练期
        train_data = data[i - train_size:i]

        # 测试期
        test_data = data[i:i + test_size]

        # 在训练期优化参数
        optimized_params = optimize_params(train_data)

        # 在测试期使用优化参数
        signals = strategy_func(test_data, **optimized_params)
        test_result = vectorized_backtest(signals, test_data)

        results.append({
            'train_period': (i - train_size, i),
            'test_period': (i, i + test_size),
            'params': optimized_params,
            'test_return': test_result['total_return'],
            'test_sharpe': test_result['returns'].mean() / test_result['returns'].std()
        })

    return results
```

### 5. 滑点和交易成本模拟

```python
def backtest_with_costs(signals, prices, commission=0.001, slippage_bps=5):
    """
    包含交易成本和滑点的回测

    Args:
        signals: 交易信号
        prices: 价格
        commission: 佣金率 (0.1%)
        slippage_bps: 滑点 (基点, 5bps = 0.05%)
    """
    slippage = slippage_bps / 10000  # 转换为小数

    # 调整买入价格 (上滑)
    buy_prices = prices * (1 + slippage)

    # 调整卖出价格 (下滑)
    sell_prices = prices * (1 - slippage)

    # 计算交易成本
    buy_cost = buy_prices * (1 + commission)
    sell_proceeds = sell_prices * (1 - commission)

    # 执行回测
    capital = 100000
    position = 0
    equity_curve = []

    for i in range(len(signals)):
        if signals[i] == 1 and position == 0:  # 买入
            shares = int(capital / buy_cost.iloc[i])
            position = shares
            capital -= shares * buy_cost.iloc[i]

        elif signals[i] == -1 and position > 0:  # 卖出
            capital += position * sell_proceeds.iloc[i]
            position = 0

        equity = capital + position * prices.iloc[i]
        equity_curve.append(equity)

    return {
        'equity_curve': pd.Series(equity_curve),
        'total_return': (equity_curve[-1] - 100000) / 100000,
        'total_trades': signals.abs().sum() / 2
    }
```

### 6. 回测报告生成

```python
def generate_backtest_report(results, strategy_name):
    """
    生成详细的回测报告
    """
    report = f"""
# {strategy_name} 回测报告

## 策略概览
- 初始资金: $100,000
- 最终价值: ${results['final_value']:,.2f}
- 总收益率: {results['total_return']:.2%}
- 年化收益率: {results.get('annual_return', 0):.2%}

## 风险指标
- 最大回撤: {results['max_drawdown']:.2%}
- 年化波动率: {results.get('annual_volatility', 0):.2%}
- 夏普比率: {results.get('sharpe_ratio', 0):.2f}
- 索提诺比率: {results.get('sortino_ratio', 0):.2f}

## 交易统计
- 总交易次数: {results.get('num_trades', 0)}
- 盈利交易: {results.get('winning_trades', 0)}
- 亏损交易: {results.get('losing_trades', 0)}
- 胜率: {results.get('win_rate', 0):.2%}
- 平均交易收益: {results.get('avg_trade_return', 0):.2%}

## 月度收益
"""

    # 添加月度收益表格
    if 'monthly_returns' in results:
        for month, ret in results['monthly_returns'].items():
            report += f"- {month.strftime('%Y-%m')}: {ret:.2%}\n"

    return report
```

## 最佳实践

### ✅ 推荐做法

1. **样本外验证**
   - Walk-Forward分析
   - 滚动窗口回测
   - 避免过拟合

2. **考虑交易成本**
   - 佣金
   - 滑点
   - 市场冲击

3. **压力测试**
   - 不同市场环境
   - 极端行情
   - 黑天鹅事件

### ❌ 避免错误

1. **过拟合**
   - 参数过多
   - 规则复杂
   - 数据挖掘

2. **前视偏差**
   - 使用未来数据
   - 信号滞后
   - 重新计算

3. **忽略流动性**
   - 大额交易
   - 市场冲击
   - 退出困难

## 相关资源

- [回测框架详解](backtest-framework.md)
- [绩效指标参考](performance-metrics.md)
- [参数优化方法](parameter-optimization.md)

---

**版本**: 1.0.0
**最后更新**: 2026-01-10
