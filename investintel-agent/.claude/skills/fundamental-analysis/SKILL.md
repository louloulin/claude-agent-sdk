---
name: fundamental-analysis
description: 基本面分析专家，包括财务报表分析、估值模型、盈利质量评估、竞争优势分析。在进行公司基本面分析、估值计算、投资价值评估时使用。
allowed-tools:
  - Bash(python:*, R:*)
  - Read
  - Write
  - WebFetch
model: claude-opus-4-20250514
tags:
  - fundamental-analysis
  - valuation
  - financial-ratios
  - earnings-quality
dependencies: []
---

# Fundamental Analysis Skill

## 核心能力

你是基本面分析专家，专注于公司内在价值评估。

### 1. 财务报表分析

#### 利润表分析

**收入质量**
```python
def revenue_quality(revenue_growth, accounts_receivable_growth):
    """
    收入增长 > 应收账款增长 = 高质量收入
    收入增长 < 应收账款增长 = 低质量收入(可能通过赊销粉饰)
    """
    if revenue_growth > accounts_receivable_growth:
        return "High quality - Cash collected efficiently"
    else:
        return "Low quality - Receivables growing faster"
```

**毛利率分析**
```python
def gross_margin_analysis(gross_margin, industry_avg, historical_avg):
    """
    毛利率 > 行业平均: 竞争优势
    毛利率上升: 定价权或成本控制
    """
    if gross_margin > industry_avg * 1.1:
        return "Competitive advantage detected"
    elif gross_margin < industry_avg * 0.9:
        return "Competitive disadvantage"
    else:
        return "Industry average"
```

**营业利润率**
```python
def operating_margin_analysis(operating_margin, trend):
    """
    营业利润率 = 核心业务盈利能力
    上升趋势: 运营效率提升
    """
    if operating_margin > 0.15:  # 15%+
        return "Excellent - Strong profitability"
    elif operating_margin > 0.10:
        return "Good - Healthy profitability"
    elif operating_margin > 0.05:
        return "Average - Moderate profitability"
    else:
        return "Poor - Low profitability"
```

#### 资产负债表分析

**流动性比率**
```python
def current_ratio_analysis(current_ratio, quick_ratio):
    """
    流动比率 > 2: 流动性强
    速动比率 > 1: 短期偿债能力好
    """
    if current_ratio > 2 and quick_ratio > 1:
        return "Strong liquidity"
    elif current_ratio > 1.5 and quick_ratio > 0.8:
        return "Adequate liquidity"
    else:
        return "Weak liquidity - potential risk"
```

**资产负债率**
```python
def debt_ratio_analysis(total_debt, total_equity, industry_avg):
    debt_to_equity = total_debt / total_equity

    if debt_to_equity < 0.3:
        return "Conservative - Low financial leverage"
    elif debt_to_equity < 0.6:
        return "Moderate - Reasonable leverage"
    else:
        return "Aggressive - High financial risk"
```

**股东权益回报率 (ROE)**
```python
def roe_analysis(roe, industry_avg):
    """
    ROE = Net Income / Shareholder Equity
    ROE > 15%: 优秀
    ROE > 行业平均: 超额收益
    """
    if roe > 0.20:
        return "Excellent - Very high returns"
    elif roe > 0.15:
        return "Good - Above average"
    elif roe > industry_avg:
        return "Above industry average"
    else:
        return "Below average"
```

#### 现金流量表分析

**经营现金流**
```python
def operating_cash_flow_analysis(ocf, net_income):
    """
    OCF > Net Income: 盈利质量高
    OCF < Net Income: 盈利质量低(可能通过应计项目粉饰)
    """
    if ocf > net_income * 1.1:
        return "High quality - Cash exceeds earnings"
    elif ocf > net_income * 0.9:
        return "Good quality - Cash supports earnings"
    else:
        return "Poor quality - Earnings not backed by cash"
```

**自由现金流**
```python
def free_cash_flow_analysis(fcf, ocf, capex):
    """
    FCF = OCF - Capex
    FCF > 0: 产生现金用于分红/回购
    FCF < 0: 烧钱阶段或高资本支出
    """
    if fcf > 0:
        return "Positive FCF - Value creation"
    else:
        return "Negative FCF - Investment or burning cash"
```

### 2. 估值模型

#### DCF (现金流折现) 模型

```python
def dcf_valuation(fcf, growth_rate, discount_rate, terminal_growth, years=10):
    """
    DCF = Σ(FCF_t / (1+r)^t) + Terminal Value

    Args:
        fcf: 当前自由现金流
        growth_rate: 预测增长率
        discount_rate: WACC或要求回报率
        terminal_growth: 永续增长率
        years: 预测年数
    """
    pv_cash_flows = 0
    for t in range(1, years + 1):
        projected_fcf = fcf * (1 + growth_rate) ** t
        pv = projected_fcf / (1 + discount_rate) ** t
        pv_cash_flows += pv

    # 终值
    terminal_fcf = fcf * (1 + growth_rate) ** years * (1 + terminal_growth)
    terminal_value = terminal_fcf / (discount_rate - terminal_growth)
    pv_terminal = terminal_value / (1 + discount_rate) ** years

    enterprise_value = pv_cash_flows + pv_terminal
    return enterprise_value
```

#### 相对估值模型

**P/E (市盈率) 估值**
```python
def pe_valuation(current_price, eps, industry_pe, growth_rate):
    """
    PEG = P/E / Growth Rate
    PEG < 1: 低估
    PEG > 2: 高估
    """
    current_pe = current_price / eps
    peg = current_pe / growth_rate

    if peg < 1:
        return "Undervalued - PEG < 1"
    elif peg < 1.5:
        return "Fair value"
    else:
        return "Overvalued - PEG > 1.5"
```

**P/B (市净率) 估值**
```python
def pb_valuation(current_price, book_value_per_share, industry_pb, roe):
    """
    P/B 应与 ROE 相关
    高ROE + 低P/B = 价值机会
    """
    current_pb = current_price / book_value_per_share

    if roe > 0.15 and current_pb < industry_pb * 0.8:
        return "Value opportunity - High ROE, low P/B"
    elif current_pb < industry_pb * 0.7:
        return "Potentially undervalued"
    else:
        return "Fair or overvalued"
```

**EV/EBITDA 估值**
```python
def ev_ebitda_valuation(ev, ebitda, industry_ev_ebitda):
    """
    EV/EBITDA 适合资本结构差异大的公司比较
    """
    current_ev_ebitda = ev / ebitda

    if current_ev_ebitda < industry_ev_ebitda * 0.8:
        return "Undervalued vs industry"
    elif current_ev_ebitda > industry_ev_ebitda * 1.2:
        return "Overvalued vs industry"
    else:
        return "Fair value vs industry"
```

### 3. 盈利质量评估

**盈利可持续性评分**
```python
def earnings_sustainability_score(
    cash_conversion,  # OCF/NI
    earnings_stability,  # EPS波动率
    recurring_ratio,  # 经常性利润占比
    margin_trend  # 利润率趋势
):
    score = 0

    # 现金转换率 (0-30分)
    if cash_conversion > 1.1:
        score += 30
    elif cash_conversion > 0.9:
        score += 20
    else:
        score += 10

    # 盈利稳定性 (0-30分)
    if earnings_stability < 0.1:
        score += 30
    elif earnings_stability < 0.2:
        score += 20
    else:
        score += 10

    # 经常性利润占比 (0-20分)
    score += int(recurring_ratio * 20)

    # 利润率趋势 (0-20分)
    if margin_trend > 0:
        score += 20
    elif margin_trend > -0.05:
        score += 10

    return score  # 0-100分
```

**应计项目分析**
```python
def accruals_analysis(net_income, operating_cash_flow):
    """
    应计项目 = 净利润 - 经营现金流
    高应计项目 = 盈利质量低
    """
    accruals = net_income - operating_cash_flow
    accruals_ratio = accruals / abs(net_income)

    if accruals_ratio > 0.2:
        return "High accruals - Low earnings quality"
    elif accruals_ratio > 0:
        return "Moderate accruals"
    else:
        return "Negative accruals - Conservative accounting"
```

### 4. 竞争优势分析 (护城河)

**巴菲特护城河评估**
```python
def moat_analysis(company):
    """
    评估四大护城河:
    1. 无形资产 (品牌、专利)
    2. 成本优势
    3. 转换成本
    4. 网络效应
    """
    moat_score = 0

    # 无形资产
    if has_strong_brand:
        moat_score += 1
    if has_patents:
        moat_score += 1

    # 成本优势
    if has_scale_economy:
        moat_score += 1
    if has_low_cost_provider:
        moat_score += 1

    # 转换成本
    if has_high_switching_cost:
        moat_score += 1

    # 网络效应
    if has_network_effects:
        moat_score += 2

    if moat_score >= 5:
        return "Wide moat - Strong competitive advantage"
    elif moat_score >= 3:
        return "Narrow moat - Moderate advantage"
    else:
        return "No moat - Limited advantage"
```

## 分析框架

### 完整分析流程

1. **收集财务数据**
   - 获取近5年财务报表
   - 收集行业对比数据

2. **财务健康检查**
   - 流动性风险
   - 偿债能力
   - 盈利能力

3. **盈利质量评估**
   - 现金转换
   - 应计项目
   - 盈利稳定性

4. **估值计算**
   - DCF估值
   - 相对估值
   - 综合估值区间

5. **护城河分析**
   - 竞争优势
   - 行业地位
   - 可持续性

6. **投资建议**
   - 综合评分
   - 风险提示
   - 关键假设

## 最佳实践

### ✅ 推荐做法

1. **多模型验证**
   - DCF + 相对估值
   - 敏感性分析
   - 保守估计

2. **关注现金流**
   - 现金为王
   - OCF > 净利润
   - FCF用于分红/回购

3. **长期视角**
   - 3-5年预测
   - 忽略短期波动
   - 关注内在价值

### ❌ 避免错误

1. **过度依赖单一指标**
   - P/E可能误导
   - 需要多维度分析

2. **忽视行业特性**
   - 不同行业不同估值
   - 周期性行业需谨慎

3. **假设过于乐观**
   - 保守预测
   - 留出安全边际

## 相关资源

- [财务比率详解](financial-ratios.md)
- [估值模型参考](valuation-models.md)
- [盈利质量评估](earnings-quality.md)

---

**版本**: 1.0.0
**最后更新**: 2026-01-10
