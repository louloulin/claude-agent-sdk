---
name: portfolio-optimization
description: |
  投资组合优化Skill,提供现代投资组合理论(MPT)实现:
  - 均值-方差模型(Markowitz)
  - Black-Litterman模型
  - 有效前沿计算
  - 风险平价组合
  - 因子模型优化

  基于Claude Agent SDK,提供科学的投资组合构建和优化能力。

allowed-tools:
  - optimize_portfolio
  - calculate_efficient_frontier
  - black_litterman_model
  - risk_parity_portfolio
  - factor_portfolio_optimization

model: claude-sonnet-4-20250514
tags:
  - portfolio
  - optimization
  - mpt
  - risk-management

examples:
  - description: "优化投资组合权重"
    input: |
      使用以下资产优化投资组合:
      - AAPL: 预期收益12%, 波动率25%
      - MSFT: 预期收益10%, 波动率20%
      - GOOGL: 预期收益11%, 波动率22%

      相关系数矩阵:
      AAPL-MSFT: 0.6
      AAPL-GOOGL: 0.7
      MSFT-GOOGL: 0.65

      使用均值-方差模型,目标风险15%。

  - description: "计算有效前沿"
    input: |
      计算以下投资组合的有效前沿:
      资产: BTC, ETH, SOL
      历史数据: 过去1年

      生成20个有效前沿点。

  - description: "Black-Litterman模型"
    input: |
      使用Black-Litterman模型优化组合:
      - 市场基准: S&P 500
      - 观点1: 科技股未来1年超额收益5%
      - 观点2: 能源股未来1年落后市场3%
      - 置信度: 高

  - description: "风险平价组合"
    input: |
      构建风险平价投资组合:
      资产: 股票, 债券, 商品, REITs
      目标: 各资产贡献相同风险
---

# Portfolio Optimization Skill

## 概述

Portfolio Optimization Skill提供现代投资组合理论(MPT)的完整实现,帮助用户构建科学优化的投资组合。

## 核心功能

### 1. 均值-方差优化 (Markowitz)

- **optimize_portfolio**: 基于均值-方差模型优化
  - 最小化风险(给定收益)
  - 最大化收益(给定风险)
  - 最大夏普比率

### 2. 有效前沿

- **calculate_efficient_frontier**: 计算有效前沿曲线
  - 多个最优风险-收益点
  - 可视化支持
  - 最优组合识别

### 3. Black-Litterman模型

- **black_litterman_model**: 结合市场观点
  - 均衡收益(市场隐含)
  - 投资者观点
  - 置信度调整

### 4. 风险平价

- **risk_parity_portfolio**: 等风险贡献
  - 各资产风险贡献相同
  - 杠杆调整
  - 分层风险平价

### 5. 因子模型

- **factor_portfolio_optimization**: 基于因子的优化
  - 风险因子暴露
  - 因子中性组合
  - Smart Beta策略

## 使用示例

### 示例1: 均值-方差优化

```
使用以下资产优化投资组合:
- AAPL: 预期收益12%, 波动率25%
- MSFT: 预期收益10%, 波动率20%
- GOOGL: 预期收益11%, 波动率22%

相关系数:
AAPL-MSFT: 0.6
AAPL-GOOGL: 0.7
MSFT-GOOGL: 0.65

目标: 最大化夏普比率,无风险利率3%
```

### 示例2: 有效前沿

```
计算科技股组合的有效前沿:
资产: AAPL, MSFT, GOOGL, AMZN, META
历史数据: 2020-2024
生成30个前沿点,可视化展示
```

### 示例3: Black-Litterman

```
使用Black-Litterman模型:
市场基准: S&P 500
风险厌恶系数: 3

观点:
1. 科技股超额收益 +5% (置信度75%)
2. 能源股落后 -3% (置信度60%)
3. 医疗股超额收益 +2% (置信度50%)

优化组合权重
```

## MCP工具说明

### optimize_portfolio

投资组合优化

**参数:**
- assets: 资产列表
- expected_returns: 预期收益率向量
- covariance_matrix: 协方差矩阵
- objective: 优化目标 (min_risk/max_return/max_sharpe)
- constraints: 约束条件 (权重上下限、行业中性等)
- risk_free_rate: 无风险利率

**返回:**
- 最优权重
- 预期收益
- 预期风险(标准差)
- 夏普比率

### calculate_efficient_frontier

计算有效前沿

**参数:**
- assets: 资产列表
- expected_returns: 预期收益率
- covariance_matrix: 协方差矩阵
- num_points: 前沿点数
- min_return: 最小收益
- max_return: 最大收益

**返回:**
- 有效前沿点集 (收益, 风险, 权重)
- 最优夏普比率点
- 可视化数据

### black_litterman_model

Black-Litterman组合优化

**参数:**
- market_caps: 市值权重
- covariance_matrix: 协方差矩阵
- views: 投资者观点矩阵
- view_confidences: 置信度向量
- risk_aversion: 风险厌恶系数
- tau: 标量参数

**返回:**
- Black-Litterman预期收益
- 最优权重
- 与市场权重对比

### risk_parity_portfolio

风险平价组合

**参数:**
- assets: 资产列表
- covariance_matrix: 协方差矩阵
- target_risk: 目标组合风险
- leverage_constraint: 杠杆约束

**返回:**
- 风险平价权重
- 各资产风险贡献
- 组合总风险

### factor_portfolio_optimization

因子模型优化

**参数:**
- assets: 资产列表
- factor_exposures: 因子暴露矩阵
- factor_returns: 因子收益
- specific_risks: 特质风险
- target_factor_exposures: 目标因子暴露
- constraints: 约束条件

**返回:**
- 最优权重
- 因子暴露分析
- 风险分解

## 数学模型

### Markowitz均值-方差模型

**目标函数:**
```
最小化: (1/2) * w' * Σ * w
约束条件:
  - w' * μ = target_return
  - Σw = 1
  - w ≥ 0 (允许做空则为无约束)
```

其中:
- w: 权重向量
- Σ: 协方差矩阵
- μ: 预期收益向量

### Black-Litterman模型

**预期收益调整:**
```
E[R] = τ * Σ * w_mkt + P' * G * (V - P * τ * Σ * w_mkt)
```

其中:
- τ: 标量参数
- Σ: 协方差矩阵
- w_mkt: 市场权重
- P: 观点矩阵
- V: 观点收益向量
- G: 置信度矩阵

### 风险平价

**风险贡献:**
```
RC_i = w_i * (Σw)_i / √(w'Σw)
目标: RC_i = RC_j, ∀i,j
```

## 最佳实践

1. **数据质量**: 使用足够长的历史数据估计协方差矩阵
2. **正则化**: 对协方差矩阵进行收缩估计
3. **再平衡**: 定期再平衡(月度/季度)
4. **交易成本**: 考虑交易成本和换手率
5. **约束**: 设置合理的权重上下限
6. **压力测试**: 测试组合在不同市场环境下的表现

## 技术实现

- 优化算法: 二次规划(QP)
- 数值计算: ndarray/ nalgebra
- 求解器: 内点法/序列二次规划
- 可视化: Plotly数据

## 相关文件

- 相关Skill: risk-analytics, technical-indicators
- 理论基础: Modern Portfolio Theory (Markowitz, 1952)
- Black-Litterman: Black & Litterman (1992)

## 参考资料

- Markowitz, H. (1952). "Portfolio Selection"
- Black, F., & Litterman, R. (1992). "Global Portfolio Optimization"
- Qian, E., & Rasamoelina, A. (2018). "Risk Parity Fundamentals"
