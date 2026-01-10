---
name: risk-analytics
description: |
  风险分析Skill,提供全面的投资组合风险管理:
  - VaR/CVaR计算
  - 波动率分析
  - 相关性分析
  - 风险分解
  - 压力测试
  - 风险归因

  基于Claude Agent SDK,提供专业级的风险管理工具。

allowed-tools:
  - calculate_var
  - calculate_cvar
  - volatility_analysis
  - correlation_analysis
  - risk_decomposition
  - stress_testing
  - risk_attribution
  - beta_calculation

model: claude-sonnet-4-20250514
tags:
  - risk-management
  - var
  - portfolio-risk
  - stress-testing

examples:
  - description: "VaR计算"
    input: |
      计算投资组合的VaR:
      组合: 60%股票,40%债券
      置信水平: 95%
      持有期: 1天
      方法: 历史模拟法

  - description: "CVaR分析"
    input: |
      计算CVaR(预期亏损):
      组合: 科技股组合
      置信水平: 99%
      分析尾部风险特征

  - description: "压力测试"
    input: |
      对组合进行压力测试:
      情景1: 2008金融危机
      情景2: 2020新冠崩盘
      情景3: 科技股泡沫破裂

      分析各情景下的潜在损失

  - description: "风险分解"
    input: |
      分解投资组合的风险:
      组合: 多资产组合
      分解: 边际VaR, 成分VaR
      识别主要风险源
---
# Risk Analytics Skill

## 概述

Risk Analytics Skill提供全面的风险管理工具,从VaR计算到压力测试,覆盖投资组合风险的各个方面。

## 核心功能

### 1. VaR计算

- **calculate_var**: 风险价值计算
  - 历史模拟法
  - 参数法(方差-协方差)
  - 蒙特卡洛模拟
  - 条件VaR
  - 增量VaR

### 2. CVaR计算

- **calculate_cvar**: 条件风险价值
  - 预期亏损
  - 尾部风险分析
  - 超过VaR的平均损失

### 3. 波动率分析

- **volatility_analysis**: 多维度波动率
  - 历史波动率
  - 隐含波动率
  - GARCH模型
  - 已实现波动率
  - 波动率期限结构

### 4. 相关性分析

- **correlation_analysis**: 相关性风险
  - 皮尔逊相关系数
  - 滚动相关系数
  - 相关性稳定性
  - 极端事件相关性

### 5. 风险分解

- **risk_decomposition**: 组合风险分解
  - 边际VaR
  - 成分VaR
  - 风险贡献度
  - 因子分解

### 6. 压力测试

- **stress_testing**: 极端情景分析
  - 历史危机重现
  - 假设情景
  - 反向压力测试
  - 流动性压力

### 7. 风险归因

- **risk_attribution**: 风险来源分析
  - 因子风险
  - 特质风险
  - 国家/行业风险
  - 货币风险

### 8. Beta计算

- **beta_calculation**: 系统性风险
  - CAPM Beta
  - 滚动Beta
  - 下行Beta
  - 行业调整Beta

## VaR计算方法

### 1. 历史模拟法

```python
def historical_var(returns, confidence_level=0.95, time_horizon=1):
    """
    历史模拟法VaR

    Args:
        returns: 历史收益率
        confidence_level: 置信水平
        time_horizon: 持有期(天)

    Returns:
        VaR值
    """
    # 计算持有期收益率
    cumulative_returns = returns.rolling(time_horizon).sum().dropna()

    # 计算分位数
    var = np.percentile(cumulative_returns, (1 - confidence_level) * 100)

    return var
```

### 2. 参数法(方差-协方差)

```python
def parametric_var(returns, confidence_level=0.95, time_horizon=1):
    """
    参数法VaR

    假设收益率服从正态分布
    """
    # 计算均值和标准差
    mu = returns.mean()
    sigma = returns.std()

    # 计算持有期参数
    mu_h = mu * time_horizon
    sigma_h = sigma * np.sqrt(time_horizon)

    # 计算VaR
    from scipy.stats import norm
    z_score = norm.ppf(1 - confidence_level)
    var = mu_h + z_score * sigma_h

    return var
```

### 3. 蒙特卡洛模拟

```python
def monte_carlo_var(returns, confidence_level=0.95,
                   time_horizon=1, n_simulations=10000):
    """
    蒙特卡洛模拟VaR
    """
    mu = returns.mean()
    sigma = returns.std()

    # 模拟收益率路径
    simulated_returns = np.random.normal(
        mu * time_horizon,
        sigma * np.sqrt(time_horizon),
        n_simulations
    )

    # 计算VaR
    var = np.percentile(simulated_returns, (1 - confidence_level) * 100)

    return var
```

## CVaR计算

```python
def calculate_cvar(returns, confidence_level=0.95):
    """
    条件风险价值(CVaR)

    也称为Expected Shortfall (ES)
    """
    # 计算VaR
    var = np.percentile(returns, (1 - confidence_level) * 100)

    # 计算超过VaR的平均损失
    tail_losses = returns[returns <= var]
    cvar = tail_losses.mean()

    return cvar
```

## 风险分解

### 边际VaR (Marginal VaR)

单个资产对组合VaR的边际贡献:

```
MVaR_i = ΔVaR / Δw_i
```

### 成分VaR (Component VaR)

单个资产对组合VaR的绝对贡献:

```
CVaR_i = w_i × MVaR_i
```

### 风险贡献度

```
RC_i = CVaR_i / VaR_portfolio
```

```python
def risk_decomposition(portfolio_returns, weights, confidence_level=0.95):
    """
    投资组合风险分解

    Args:
        portfolio_returns: 各资产收益率矩阵
        weights: 组合权重
        confidence_level: 置信水平

    Returns:
        marginal_var: 边际VaR
        component_var: 成分VaR
        risk_contribution: 风险贡献度
    """
    # 计算协方差矩阵
    cov_matrix = np.cov(portfolio_returns.T)

    # 计算组合收益率和方差
    portfolio_return = np.dot(weights, portfolio_returns.mean())
    portfolio_variance = np.dot(weights.T, np.dot(cov_matrix, weights))
    portfolio_std = np.sqrt(portfolio_variance)

    # 计算组合VaR
    from scipy.stats import norm
    z_score = norm.ppf(1 - confidence_level)
    portfolio_var = portfolio_return + z_score * portfolio_std

    # 计算边际VaR
    marginal_var = z_score * (np.dot(cov_matrix, weights) / portfolio_std)

    # 计算成分VaR
    component_var = weights * marginal_var

    # 计算风险贡献度
    risk_contribution = component_var / portfolio_var

    return {
        'portfolio_var': portfolio_var,
        'marginal_var': marginal_var,
        'component_var': component_var,
        'risk_contribution': risk_contribution
    }
```

## 波动率模型

### GARCH(1,1)模型

```python
from arch import arch_model

def garch_volatility(returns, horizon=1):
    """
    GARCH(1,1)模型估计波动率

    σ²ₜ = ω + α₁ε²ₜ₋₁ + β₁σ²ₜ₋₁
    """
    model = arch_model(returns, vol='Garch', p=1, q=1)
    fitted_model = model.fit(disp='off')

    # 预测未来波动率
    forecast = fitted_model.forecast(horizon=horizon)
    predicted_variance = forecast.variance.values[-1, -1]
    predicted_volatility = np.sqrt(predicted_variance)

    return predicted_volatility
```

### EWMA (指数加权移动平均)

```python
def ewma_volatility(returns, lambda_param=0.94):
    """
    EWMA模型估计波动率

    σ²ₜ = λσ²ₜ₋₁ + (1-λ)r²ₜ₋₁
    """
    ewma_var = 0
    for ret in returns:
        ewma_var = lambda_param * ewma_var + (1 - lambda_param) * ret**2

    ewma_volatility = np.sqrt(ewma_var)
    return ewma_volatility
```

## 压力测试

### 历史危机重现

```python
def historical_crisis_scenarios():
    """
    定义历史危机情景
    """
    scenarios = {
        '1929_Great_Depression': {
            'equity_shock': -0.89,  # -89%
            'duration_months': 34
        },
        '1987_Black_Monday': {
            'equity_shock': -0.22,  # -22%
            'duration_days': 1
        },
        '2008_Financial_Crisis': {
            'equity_shock': -0.57,  # -57%
            'duration_months': 17
        },
        '2020_COVID': {
            'equity_shock': -0.34,  # -34%
            'duration_months': 3
        }
    }
    return scenarios
```

### 自定义压力情景

```python
def custom_stress_test(portfolio_value, scenario):
    """
    自定义情景压力测试

    Args:
        portfolio_value: 当前组合价值
        scenario: 情景定义

    Returns:
        stressed_value: 压力后价值
        loss: 损失金额
        loss_pct: 损失百分比
    """
    stressed_value = portfolio_value * (1 + scenario['shock'])
    loss = portfolio_value - stressed_value
    loss_pct = scenario['shock']

    return {
        'stressed_value': stressed_value,
        'loss': loss,
        'loss_pct': loss_pct
    }
```

### 反向压力测试

```python
def reverse_stress_test(portfolio, ruin_threshold=0.5):
    """
    反向压力测试

    找出导致组合损失超过阈值的情景
    """
    scenarios = []

    # 股票冲击
    for equity_shock in np.arange(-0.1, -0.5, -0.05):
        total_loss = portfolio.equity_weight * equity_shock
        if total_loss <= -ruin_threshold:
            scenarios.append({
                'type': 'equity_crash',
                'shock': equity_shock,
                'total_loss': total_loss
            })

    # 利率冲击
    for rate_shock in np.arange(-0.02, 0.02, 0.005):
        # 久期计算
        duration = portfolio.bond_duration
        total_loss = portfolio.bond_weight * (-duration * rate_shock)
        if total_loss <= -ruin_threshold:
            scenarios.append({
                'type': 'rate_shock',
                'shock': rate_shock,
                'total_loss': total_loss
            })

    return scenarios
```

## Beta计算

### CAPM Beta

```python
def calculate_beta(asset_returns, benchmark_returns):
    """
    计算CAPM Beta

    Beta = Cov(Asset, Benchmark) / Var(Benchmark)
    """
    # 计算协方差和方差
    covariance = np.cov(asset_returns, benchmark_returns)[0, 1]
    benchmark_variance = np.var(benchmark_returns)

    # 计算Beta
    beta = covariance / benchmark_variance

    return beta
```

### 滚动Beta

```python
def rolling_beta(asset_returns, benchmark_returns, window=60):
    """
    滚动窗口Beta
    """
    rolling_betas = []

    for i in range(window, len(asset_returns)):
        asset_window = asset_returns[i-window:i]
        benchmark_window = benchmark_returns[i-window:i]
        beta = calculate_beta(asset_window, benchmark_window)
        rolling_betas.append(beta)

    return pd.Series(rolling_betas,
                     index=asset_returns.index[window:])
```

### 下行Beta

```python
def downside_beta(asset_returns, benchmark_returns):
    """
    下行Beta

    只在市场下跌时计算的Beta
    """
    # 筛选市场下跌的时期
    down_periods = benchmark_returns < 0

    asset_down = asset_returns[down_periods]
    benchmark_down = benchmark_returns[down_periods]

    # 计算下行Beta
    covariance = np.cov(asset_down, benchmark_down)[0, 1]
    benchmark_variance = np.var(benchmark_down)

    downside_beta = covariance / benchmark_variance

    return downside_beta
```

## 相关性分析

### 相关性矩阵

```python
def correlation_matrix(returns):
    """
    计算相关性矩阵
    """
    corr_matrix = returns.corr()

    # 可视化
    import seaborn as sns
    import matplotlib.pyplot as plt

    plt.figure(figsize=(10, 8))
    sns.heatmap(corr_matrix, annot=True, cmap='coolwarm',
                center=0, vmin=-1, vmax=1)
    plt.title('Asset Correlation Matrix')

    return corr_matrix
```

### 滚动相关系数

```python
def rolling_correlation(asset1_returns, asset2_returns, window=60):
    """
    滚动相关系数
    """
    rolling_corr = asset1_returns.rolling(window).corr(asset2_returns)

    return rolling_corr
```

### 极端事件相关性

```python
def extreme_correlation(returns1, returns2, threshold=0.05):
    """
    极端事件相关性

    在市场极端情况下,相关性通常会上升
    """
    # 定义极端事件(最差的5%日子)
    extreme_days = (returns1 < np.percentile(returns1, threshold * 100)) & \
                   (returns2 < np.percentile(returns2, threshold * 100))

    # 计算极端事件相关性
    extreme_corr = np.corrcoef(
        returns1[extreme_days],
        returns2[extreme_days]
    )[0, 1]

    # 正常时期相关性
    normal_corr = np.corrcoef(returns1, returns2)[0, 1]

    return {
        'normal_correlation': normal_corr,
        'extreme_correlation': extreme_corr,
        'correlation_breakdown': extreme_corr - normal_corr
    }
```

## 风险限制

### 常用风险限制

1. **VaR限制**: 日VaR不超过组合价值的X%
2. **集中度限制**: 单个资产不超过Y%
3. **杠杆限制**: 总杠杆不超过Z倍
4. **止损限制**: 损失超过W%强制平仓
5. **流动性限制**: 持仓不超过日均成交量的V%

### 风险预算

```python
def risk_budget(risk_budget_params):
    """
    风险预算分配

    根据风险预算分配权重
    """
    # 计算各资产的风险贡献目标
    marginal_var = risk_budget_params['marginal_var']
    risk_targets = risk_budget_params['risk_targets']

    # 求解权重使得风险贡献与目标一致
    from scipy.optimize import minimize

    def objective(weights):
        current_contributions = weights * marginal_var
        return np.sum((current_contributions - risk_targets)**2)

    # 约束: 权重和为1
    constraints = ({'type': 'eq', 'fun': lambda w: np.sum(w) - 1})

    result = minimize(objective,
                     x0=np.ones(len(marginal_var)) / len(marginal_var),
                     constraints=constraints)

    return result.x
```

## 最佳实践

1. **多方法验证**: 不要依赖单一VaR方法
2. **回测**: 定期回测VaR预测准确性
3. **压力测试**: VaR不是最坏情况
4. **相关性**: 考虑相关性突变
5. **流动性风险**: 包括流动性调整
6. **模型风险**: 模型假设可能不成立

## 技术实现

- 统计: scipy.stats, statsmodels
- VaR: pyfolio, quantlib
- GARCH: arch库
- 优化: scipy.optimize, cvxpy
- 可视化: matplotlib, seaborn

## 相关Skill

- backtesting-engine: 风险调整收益
- portfolio-optimization: 风险预算
- options-trading: 期权风险

## 参考资料

- Jorion, P. "Value at Risk: The New Benchmark for Managing Financial Risk"
- Hull, J. "Risk Management and Financial Institutions"
- Alexander, C. "Market Risk Analysis"
