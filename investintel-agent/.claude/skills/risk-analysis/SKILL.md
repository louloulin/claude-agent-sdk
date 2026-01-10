---
name: risk-analysis
description: 全面风险分析，包括VaR计算、压力测试、波动率分析、相关性矩阵、流动性风险评估。在评估投资风险、计算风险指标、进行压力测试时使用。
model: claude-opus-4-20250514  # 使用更强大的模型
allowed-tools:
  - Bash(python:*, R:*)
  - Read
  - Write
tags:
  - risk
  - var
  - stress-testing
  - volatility
---

# Risk Analysis Skill

## 核心能力

### 1. VaR (Value at Risk) 计算

#### 历史模拟法

```python
def historical_var(returns, confidence_level=0.95):
    """
    历史模拟法计算VaR

    Args:
        returns: 收益率序列
        confidence_level: 置信水平 (默认95%)

    Returns:
        VaR值 (负数表示损失)
    """
    return np.percentile(returns, (1 - confidence_level) * 100)
```

#### 方差-协方差法

```python
def parametric_var(returns, confidence_level=0.95):
    """
    参数法计算VaR (假设正态分布)
    """
    mean = returns.mean()
    std = returns.std()
    from scipy import stats
    var = mean + stats.norm.ppf(1 - confidence_level) * std
    return var
```

#### 蒙特卡洛模拟

```python
def monte_carlo_var(returns, confidence_level=0.95, n_simulations=10000):
    """
    蒙特卡洛模拟计算VaR
    """
    mean = returns.mean()
    std = returns.std()

    # 生成随机收益率
    simulated_returns = np.random.normal(mean, std, n_simulations)

    # 计算VaR
    var = np.percentile(simulated_returns, (1 - confidence_level) * 100)
    return var
```

### 2. 压力测试

#### 历史场景重现

```python
# 2008金融危机场景
scenario_2008 = {
    'equity': -0.40,      # 股票下跌40%
    'bonds': 0.05,        # 债券上涨5%
    'realestate': -0.30,  # 房地产下跌30%
    'cash': 0.00          # 现金不变
}

# COVID-19场景 (2020年3月)
scenario_covid = {
    'equity': -0.35,
    'bonds': 0.10,
    'realestate': -0.20,
    'cash': 0.00
}

def stress_test_portfolio(portfolio, scenario):
    """
    对投资组合进行压力测试

    Args:
        portfolio: 投资组合 {资产: 权重}
        scenario: 压力测试场景 {资产: 收益率}

    Returns:
        组合在压力情景下的损失
    """
    portfolio_loss = 0
    for asset, weight in portfolio.items():
        portfolio_loss += weight * scenario.get(asset, 0)

    return portfolio_loss
```

#### 假设场景

```python
# 定义自定义压力场景
def custom_stress_scenarios():
    scenarios = {
        '利率急升': {'equity': -0.15, 'bonds': -0.10, 'realestate': -0.20},
        '通胀飙升': {'equity': -0.10, 'bonds': -0.20, 'commodities': 0.30},
        '科技泡沫': {'technology': -0.40, 'other_sectors': -0.10},
        '流动性危机': {'equity': -0.25, 'bonds': 0.05, 'cash': 0.00}
    }
    return scenarios
```

### 3. 波动率分析

#### 历史波动率

```python
def historical_volatility(returns, annualize=True):
    """
    计算历史波动率
    """
    vol = returns.std()
    if annualize:
        vol = vol * np.sqrt(252)  # 年化 (假设252个交易日)
    return vol
```

#### GARCH模型

```python
from arch import arch_model

def garch_volatility(returns, p=1, q=1):
    """
    使用GARCH模型预测波动率

    Args:
        returns: 收益率序列
        p: GARCH阶数
        q: ARCH阶数

    Returns:
        条件波动率预测
    """
    model = arch_model(returns, vol='Garch', p=p, q=q)
    fitted_model = model.fit(disp='off')
    forecast = fitted_model.forecast(horizon=1)
    return forecast.variance.values[-1, -1]
```

#### 隐含波动率

```python
# 从期权价格反推隐含波动率 (需要期权数据)
def implied_volatility(option_price, S, K, T, r, option_type='call'):
    """
    使用Black-Scholes模型反推隐含波动率
    """
    from scipy.optimize import fsolve
    from black_scholes import black_scholes

    def objective(vol):
        return black_scholes(S, K, T, r, vol, option_type) - option_price

    implied_vol = fsolve(objective, 0.2)[0]
    return implied_vol
```

### 4. 相关性分析

#### 相关系数矩阵

```python
def correlation_matrix(returns):
    """
    计算资产收益率相关系数矩阵
    """
    return returns.corr()
```

#### 滚动相关性

```python
def rolling_correlation(returns1, returns2, window=60):
    """
    计算滚动相关性 (默认60天)
    """
    return returns1.rolling(window).corr(returns2)
```

### 5. 风险预算

#### 风险贡献

```python
def marginal_var_contribution(weights, cov_matrix):
    """
    计算每个资产的边际VaR贡献
    """
    portfolio_var = np.sqrt(weights.T @ cov_matrix @ weights)
    marginal_var = (cov_matrix @ weights) / portfolio_var
    return marginal_var

def component_var_contribution(weights, cov_matrix):
    """
    计算每个资产的成分VaR贡献
    """
    marginal_var = marginal_var_contribution(weights, cov_matrix)
    component_var = weights * marginal_var
    return component_var
```

## 工作流程

### 风险分析流程

1. **数据准备**
```bash
# 获取历史价格数据
python scripts/get_prices.py --tickers AAPL MSFT GOOGL --period 5y

# 计算收益率
python scripts/calculate_returns.py --prices data/prices.csv
```

2. **VaR计算**
```python
# 计算多种VaR
python scripts/calculate_var.py --portfolio my_portfolio --methods historical parametric monte_carlo
```

3. **压力测试**
```python
# 运行压力测试
python scripts/stress_test.py --portfolio my_portfolio --scenarios 2008 covid custom
```

4. **风险报告**
```python
# 生成风险报告
python scripts/risk_report.py --portfolio my_portfolio --output report.html
```

## 最佳实践

### ✅ 推荐做法

1. **多种方法结合**
   - 不要只依赖一种VaR方法
   - 比较历史法、参数法、蒙特卡洛的结果

2. **定期压力测试**
   - 至少每季度进行一次
   - 更新压力场景

3. **监控风险集中度**
   - 行业集中度
   - 地域集中度
   - 因子暴露

### ❌ 避免错误

1. **过度依赖历史数据**
   - 历史不一定重复
   - 考虑尾部风险

2. **忽视相关性变化**
   - 危机时相关性趋同
   - 定期更新相关系数

3. **忘记流动性风险**
   - 有些资产难以快速变现
   - 考虑市场冲击成本

## 相关资源

- [VaR计算详解](var-calculations.md)
- [压力测试场景](stress-testing.md)
- [风险指标参考](risk-metrics.md)

---

**版本**: 1.0.0
**最后更新**: 2026-01-10
