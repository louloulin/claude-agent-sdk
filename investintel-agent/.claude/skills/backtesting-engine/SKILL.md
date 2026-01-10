---
name: backtesting-engine
description: |
  回测引擎Skill,提供完整的策略回测框架:
  - 历史数据回放
  - 策略模拟执行
  - 性能指标计算
  - 风险调整收益
  - 参数优化
  - 蒙特卡洛模拟

  基于Claude Agent SDK,提供专业级的量化策略回测能力。

allowed-tools:
  - run_backtest
  - calculate_performance_metrics
  - optimize_parameters
  - monte_carlo_simulation
  - scenario_analysis
  - compare_strategies

model: claude-sonnet-4-20250514
tags:
  - backtesting
  - strategy-evaluation
  - performance-analysis
  - optimization

examples:
  - description: "简单策略回测"
    input: |
      回测双均线策略:
      股票: AAPL
      时间: 2020-2024
      策略:
        - 短期均线: 50天
        - 长期均线: 200天
        - 金叉买入,死叉卖出
      初始资金: $100,000

  - description: "多因子策略回测"
    input: |
      回测动量策略:
      股票池: Russell 1000
      因子: 12-1月动量
      持仓: 前50只
      再平衡: 月度
      时间: 2015-2024
      交易成本: 20bp

  - description: "参数优化"
    input: |
      优化均线策略参数:
      短期均线: 测试20/30/40/50天
      长期均线: 测试150/180/200/250天
      优化目标: 夏普比率
      数据: S&P 500 2010-2024

  - description: "蒙特卡洛模拟"
    input: |
      对策略进行蒙特卡洛模拟:
      策略: 60/40股债组合
      模拟次数: 1000次
      时间跨度: 5年
      分析: VaR, CVaR, 概率分布
---
# Backtesting Engine Skill

## 概述

Backtesting Engine Skill提供完整的量化策略回测框架,支持从简单到复杂的各种策略类型。

## 核心功能

### 1. 回测引擎

- **run_backtest**: 策略回测
  - 事件驱动架构
  - 支持多种订单类型
  - 滑点和交易成本模拟
  - 保证金和杠杆
  - 多资产组合

### 2. 性能分析

- **calculate_performance_metrics**: 全面的性能指标
  - 收益指标(年化收益、累计收益)
  - 风险指标(波动率、最大回撤)
  - 风险调整收益(夏普、索提诺、卡尔玛)
  - 统计指标(胜率、盈亏比)
  - 基准对比

### 3. 参数优化

- **optimize_parameters**: 策略参数优化
  - 网格搜索
  - 遗传算法
  - 贝叶斯优化
  - 交叉验证
  - 过拟合检测

### 4. 蒙特卡洛模拟

- **monte_carlo_simulation**: 不确定性分析
  - 收益分布模拟
  - VaR/CVaR计算
  - 压力测试
  - 概率情景分析

### 5. 情景分析

- **scenario_analysis**: 特定情景分析
  - 历史危机重现
  - 黑天鹅事件
  - 宏观情景
  - 行业冲击

### 6. 策略对比

- **compare_strategies**: 多策略对比
  - 并排对比
  - 相关性分析
  - 策略组合
  - 分散化收益

## 回测流程

### 1. 数据准备

```python
# 加载历史数据
data = load_historical_data(
    symbols=['AAPL', 'MSFT', 'GOOGL'],
    start_date='2010-01-01',
    end_date='2024-12-31',
    fields=['open', 'high', 'low', 'close', 'volume', 'returns']
)
```

### 2. 策略定义

```python
def moving_average_crossover_strategy(data, short_window=50, long_window=200):
    """
    双均线策略

    Args:
        data: 价格数据
        short_window: 短期均线周期
        long_window: 长期均线周期

    Returns:
        signals: 交易信号 (1=买入, -1=卖出, 0=持有)
    """
    signals = pd.DataFrame(index=data.index)
    signals['signal'] = 0

    # 计算均线
    signals['short_mavg'] = data['close'].rolling(window=short_window).mean()
    signals['long_mavg'] = data['close'].rolling(window=long_window).mean()

    # 生成信号
    signals['signal'][short_window:] = np.where(
        signals['short_mavg'][short_window:] > signals['long_mavg'][short_window:],
        1,
        0
    )

    # 生成交易订单
    signals['positions'] = signals['signal'].diff()

    return signals
```

### 3. 回测执行

```python
backtest = Backtest(
    initial_capital=100000,
    start_date='2010-01-01',
    end_date='2024-12-31',
    commission=0.001,  # 0.1% 交易成本
    slippage=0.0005,   # 0.05% 滑点
)

results = backtest.run(
    strategy=moving_average_crossover_strategy,
    data=data,
    strategy_params={'short_window': 50, 'long_window': 200}
)
```

### 4. 性能分析

```python
metrics = calculate_performance_metrics(
    returns=results['returns'],
    benchmark_returns=benchmark_returns,
    risk_free_rate=0.03
)

print(f"年化收益: {metrics['annual_return']:.2%}")
print(f"夏普比率: {metrics['sharpe_ratio']:.2f}")
print(f"最大回撤: {metrics['max_drawdown']:.2%}")
print(f"胜率: {metrics['win_rate']:.2%}")
```

## 性能指标

### 收益指标

1. **累计收益**
```
Cumulative Return = (Final Value - Initial Value) / Initial Value
```

2. **年化收益**
```
Annualized Return = (1 + Total Return)^(365/Days) - 1
```

3. **月度/季度收益**
```
Monthly Return = (1 + Daily Return)^(Days_in_Month) - 1
```

### 风险指标

1. **波动率**
```
Volatility = Std(Returns) * √252
```

2. **最大回撤**
```
Max Drawdown = Max(Peak - Trough) / Peak
```

3. **下行偏差**
```
Downside Deviation = Std(Negative Returns) * √252
```

### 风险调整收益

1. **夏普比率**
```
Sharpe Ratio = (Rp - Rf) / σp
```

2. **索提诺比率**
```
Sortino Ratio = (Rp - Rf) / σd
```

3. **卡尔玛比率**
```
Calmar Ratio = Annualized Return / Max Drawdown
```

4. **信息比率**
```
Information Ratio = (Rp - Rb) / Tracking Error
```

### 统计指标

1. **胜率**
```
Win Rate = Winning Trades / Total Trades
```

2. **盈亏比**
```
Profit/Loss Ratio = Average Win / Average Loss
```

3. **期望收益**
```
Expected Value = (Win Rate × Average Win) - (Loss Rate × Average Loss)
```

## 参数优化方法

### 1. 网格搜索

```python
def grid_search(strategy, param_grid, data):
    """
    网格搜索参数优化
    """
    best_score = -np.inf
    best_params = None

    for params in itertools.product(*param_grid.values()):
        param_dict = dict(zip(param_grid.keys(), params))
        results = backtest.run(strategy, data, param_dict)
        score = results['sharpe_ratio']

        if score > best_score:
            best_score = score
            best_params = param_dict

    return best_params, best_score
```

### 2. 遗传算法

```python
from deap import base, creator, tools

def genetic_optimization(strategy, param_ranges, data, generations=50):
    """
    遗传算法参数优化
    """
    creator.create("FitnessMax", base.Fitness, weights=(1.0,))
    creator.create("Individual", list, fitness=creator.FitnessMax)

    # 初始化
    toolbox = base.Toolbox()
    toolbox.register("attr_float", random.uniform, *param_ranges[0])
    toolbox.register("individual", tools.initCycle, creator.Individual,
                   (toolbox.attr_float,), n=len(param_ranges))
    toolbox.register("population", tools.initRepeat, list, toolbox.individual)

    # 遗传操作
    toolbox.register("evaluate", evaluate_strategy, strategy=strategy, data=data)
    toolbox.register("mate", tools.cxTwoPoint)
    toolbox.register("mutate", tools.mutGaussian, mu=0, sigma=1, indpb=0.2)
    toolbox.register("select", tools.selTournament, tournsize=3)

    # 运行算法
    population = toolbox.population(n=100)
    algorithms.eaSimple(population, toolbox, cxpb=0.5, mutpb=0.2,
                       ngen=generations, stats=stats, halloffame=hall_of_fame)

    return hall_of_fame[0], hall_of_fame[0].fitness.values[0]
```

### 3. 贝叶斯优化

```python
from skopt import gp_minimize

def bayesian_optimization(strategy, param_bounds, data, n_calls=50):
    """
    贝叶斯优化
    """
    def objective(params):
        param_dict = dict(zip(param_bounds.keys(), params))
        results = backtest.run(strategy, data, param_dict)
        return -results['sharpe_ratio']  # 最小化负夏普

    result = gp_minimize(
        objective,
        dimensions=list(param_bounds.values()),
        n_calls=n_calls,
        random_state=42
    )

    return dict(zip(param_bounds.keys(), result.x)), -result.fun
```

## 蒙特卡洛模拟

```python
def monte_carlo_simulation(returns, n_simulations=1000, n_days=252):
    """
    蒙特卡洛模拟

    Args:
        returns: 历史收益率
        n_simulations: 模拟次数
        n_days: 模拟天数

    Returns:
        simulated_paths: 模拟路径
        var_95: 95% VaR
        cvar_95: 95% CVaR
    """
    mu = returns.mean()
    sigma = returns.std()

    simulated_paths = []
    for _ in range(n_simulations):
        simulated_returns = np.random.normal(mu, sigma, n_days)
        simulated_path = (1 + simulated_returns).cumprod()
        simulated_paths.append(simulated_path)

    # 计算VaR和CVaR
    final_values = [path[-1] for path in simulated_paths]
    var_95 = np.percentile(final_values, 5)
    cvar_95 = np.mean([v for v in final_values if v <= var_95])

    return simulated_paths, var_95, cvar_95
```

## 前瞻偏差防范

### 常见前瞻偏差

1. **未来函数**: 使用未来数据
2. **生存偏差**: 只分析幸存者
3. **重唱偏差**: 指数成分股变化
4. **数据偏差**: 使用不存在的数据

### 防范措施

1. **时间戳对齐**: 确保使用正确时间点的数据
2. **滞后处理**: 信号生成后T+1执行
3. **成分股历史**: 使用历史成分股数据
4. **交叉验证**: Out-of-sample测试

## 过拟合检测

### 方法

1. **样本外测试**: 保留部分数据不参与优化
2. **参数敏感性**: 小幅参数变化不应大幅影响结果
3. **交易次数**: 过多交易可能过拟合
4. **复杂度惩罚**: 奥卡姆剃刀原则

### Walk-Forward分析

```python
def walk_forward_analysis(strategy, data, train_size=252*2, test_size=252):
    """
    Walk-Forward分析

    滚动窗口训练和测试
    """
    results = []
    for i in range(train_size, len(data) - test_size, test_size):
        train_data = data[:i]
        test_data = data[i:i+test_size]

        # 训练阶段优化参数
        best_params = optimize_parameters(strategy, train_data)

        # 测试阶段评估
        test_result = backtest.run(strategy, test_data, best_params)
        results.append(test_result)

    return aggregate_results(results)
```

## 最佳实践

1. **数据质量**: 使用高质量历史数据
2. **交易成本**: 务必包含 realistic 交易成本
3. **流动性**: 考虑流动性约束
4. **市场冲击**: 大额订单的市场冲击
5. **样本外测试**: 必须进行样本外验证
6. **鲁棒性**: 测试策略在不同市场环境下的表现
7. **简单性**: 简单策略往往更robust

## 技术实现

- 回测框架: Zipline, Backtrader, VectorBT
- 优化: Optuna, Hyperopt, DEAP
- 数值计算: NumPy, Pandas, SciPy
- 可视化: Matplotlib, Plotly

## 相关Skill

- portfolio-optimization: 组合优化回测
- momentum-trading: 动量策略回测
- risk-analytics: 风险分析

## 参考资料

- Chan, E. (2009). "Quantitative Trading: How to Build Your Own Algorithmic Trading Business"
- Lopez de Prado, M. (2018). "Advances in Financial Machine Learning"
- Aronson, J. (2006). "Evidence-Based Technical Analysis"
