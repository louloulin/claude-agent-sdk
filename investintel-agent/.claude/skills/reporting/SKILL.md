---
name: reporting
description: 报告生成专家，包括投资组合报告、研究报告、风险报告、可视化图表生成。在生成各类投资分析报告、创建可视化图表、导出多种格式时使用。
allowed-tools:
  - Bash(python:*, R:*)
  - Read
  - Write
model: claude-sonnet-4-20250514
tags:
  - reporting
  - visualization
  - document-generation
  - export
dependencies: []
---

# Reporting Skill

## 核心能力

你是报告生成专家，专注于创建专业的投资分析报告。

### 1. 投资组合报告

#### 组合概览报告

```python
def generate_portfolio_summary(portfolio):
    """
    生成投资组合概览报告

    包含:
    - 总资产
    - 总收益
    - 收益率
    - 风险指标
    - 资产配置
    """
    report = {
        'summary': {
            'total_assets': portfolio['total_value'],
            'total_cost': portfolio['total_cost'],
            'total_gain_loss': portfolio['total_value'] - portfolio['total_cost'],
            'total_return': (portfolio['total_value'] / portfolio['total_cost']) - 1,
            'daily_change': portfolio['daily_change'],
            'daily_change_pct': portfolio['daily_change_pct']
        },
        'performance': {
            'ytd_return': calculate_ytd_return(portfolio),
            'mtd_return': calculate_mtd_return(portfolio),
            '1y_return': calculate_1y_return(portfolio),
            '3y_return': calculate_3y_return(portfolio),
            'since_inception': calculate_since_inception(portfolio)
        },
        'risk_metrics': {
            'volatility': calculate_volatility(portfolio),
            'var_95': calculate_var_95(portfolio),
            'max_drawdown': calculate_max_drawdown(portfolio),
            'sharpe_ratio': calculate_sharpe(portfolio)
        },
        'holdings': portfolio['holdings']
    }

    return report
```

#### 资产配置报告

```python
def generate_allocation_report(portfolio):
    """
    生成资产配置报告

    包含:
    - 当前配置
    - 目标配置
    - 偏离分析
    - 再平衡建议
    """
    current_alloc = calculate_current_allocation(portfolio)
    target_alloc = get_target_allocation(portfolio)

    allocation_report = {
        'by_asset_class': {
            'equities': current_alloc.get('equities', 0),
            'fixed_income': current_alloc.get('fixed_income', 0),
            'cash': current_alloc.get('cash', 0),
            'alternatives': current_alloc.get('alternatives', 0)
        },
        'by_sector': calculate_sector_allocation(portfolio),
        'by_region': calculate_region_allocation(portfolio),
        'drift_analysis': analyze_drift(current_alloc, target_alloc),
        'rebalance_recommendations': generate_rebalance_recommendations(
            current_alloc, target_alloc, threshold=0.05
        )
    }

    return allocation_report
```

### 2. 研究报告

#### 个股研究报告

```python
def generate_stock_research_report(symbol, analysis_data):
    """
    生成个股研究报告

    结构:
    1. 投资摘要
    2. 公司概况
    3. 财务分析
    4. 估值分析
    5. 技术分析
    6. 风险因素
    7. 投资建议
    """
    report = f"""
# {symbol} 投资研究报告

## 投资摘要
**评级**: {analysis_data['rating']}
**目标价**: ${analysis_data['target_price']:.2f}
**当前价**: ${analysis_data['current_price']:.2f}
**上涨空间**: {analysis_data['upside_potential']:.1f}%
**建议**: {analysis_data['recommendation']}

## 公司概况
- 行业: {analysis_data['industry']}
- 市值: {analysis_data['market_cap']}
- 员工数: {analysis_data['employees']}
- 业务描述: {analysis_data['description']}

## 财务分析
### 盈利能力
- ROE: {analysis_data['roe']:.2%}
- 净利率: {analysis_data['net_margin']:.2%}
- 毛利率: {analysis_data['gross_margin']:.2%}

### 成长性
- 营收增长: {analysis_data['revenue_growth']:.2%}
- 利润增长: {analysis_data['earnings_growth']:.2%}

### 财务健康
- 资产负债率: {analysis_data['debt_to_equity']:.2f}
- 流动比率: {analysis_data['current_ratio']:.2f}
- 利息保障倍数: {analysis_data['interest_coverage']:.2f}

## 估值分析
- P/E (TTM): {analysis_data['pe_ttm']:.2f}
- P/B: {analysis_data['pb']:.2f}
- EV/EBITDA: {analysis_data['ev_ebitda']:.2f}
- PEG: {analysis_data['peg']:.2f}
- 内在价值 (DCF): ${analysis_data['intrinsic_value']:.2f}

## 技术分析
- 趋势: {analysis_data['trend']}
- 支撑位: ${analysis_data['support']:.2f}
- 阻力位: ${analysis_data['resistance']:.2f}
- RSI: {analysis_data['rsi']:.1f}
- MACD: {analysis_data['macd']}

## 风险因素
"""

    for risk in analysis_data['risks']:
        report += f"- {risk}\n"

    report += f"""

## 投资建议
{analysis_data['investment_thesis']}

**关键催化剂**:
"""

    for catalyst in analysis_data['catalysts']:
        report += f"- {catalyst}\n"

    report += f"""

**免责声明**: 本报告仅供参考，不构成投资建议。

**报告日期**: {analysis_data['report_date']}
**分析师**: InvestIntel AI
"""

    return report
```

#### 行业研究报告

```python
def generate_industry_report(industry, analysis_data):
    """
    生成行业研究报告

    包含:
    - 行业概览
    - 市场规模和增长
    - 竞争格局
    - 驱动因素
    - 风险因素
    - 投资机会
    """
    report = f"""
# {industry} 行业研究报告

## 行业概览
- 行业规模: {analysis_data['market_size']}
- 增长率: {analysis_data['growth_rate']:.2%}
- 生命周期阶段: {analysis_data['life_cycle_stage']}

## 市场分析
### 市场细分
"""

    for segment, data in analysis_data['segments'].items():
        report += f"- **{segment}**: {data['size']} ({data['share']:.1%})\n"

    report += f"""

### 竞争格局
- 市场集中度: {analysis_data['concentration']}
- Top 5公司: {', '.join(analysis_data['top_players'])}

## 行业驱动因素
"""

    for driver in analysis_data['drivers']:
        report += f"- {driver}\n"

    report += f"""

## 投资机会
"""

    for opportunity in analysis_data['opportunities']:
        report += f"- {opportunity}\n"

    report += f"""

## 风险因素
"""

    for risk in analysis_data['risks']:
        report += f"- {risk}\n"

    return report
```

### 3. 风险报告

#### 组合风险报告

```python
def generate_risk_report(portfolio):
    """
    生成投资组合风险报告

    包含:
    - 风险概览
    - VaR分析
    - 压力测试
    - 风险分解
    - 风险建议
    """
    report = f"""
# 投资组合风险报告

## 风险概览
**组合价值**: ${portfolio['total_value']:,.2f}
**风险等级**: {portfolio['risk_level']}

### 主要风险指标
- 1日95% VaR: -${portfolio['var_1d_95']:,.2f} ({portfolio['var_1d_95_pct']:.2%})
- 5日95% VaR: -${portfolio['var_5d_95']:,.2f} ({portfolio['var_5d_95_pct']:.2%})
- 年化波动率: {portfolio['volatility']:.2%}
- 最大回撤: {portfolio['max_drawdown']:.2%}
- 夏普比率: {portfolio['sharpe_ratio']:.2f}
- 索提诺比率: {portfolio['sortino_ratio']:.2f}

## VaR分析
### VaR历史分布
"""

    for var_item in portfolio['var_history']:
        report += f"- {var_item['date']}: {var_item['var']:.2%}\n"

    report += f"""

## 压力测试结果
"""

    for scenario, result in portfolio['stress_test'].items():
        report += f"""
### {scenario}
- 预期损失: {result['loss']:.2%}
- 损失金额: -${result['loss_amount']:,.2f}
"""

    report += f"""

## 风险分解
### 按资产类别
"""

    for asset_class, contribution in portfolio['risk_by_asset_class'].items():
        report += f"- {asset_class}: {contribution:.2%}\n"

    report += f"""

### 按因子
"""

    for factor, exposure in portfolio['factor_exposure'].items():
        report += f"- {factor}: {exposure:.2f}\n"

    report += f"""

## 风险建议
"""

    for recommendation in portfolio['risk_recommendations']:
        report += f"- {recommendation}\n"

    return report
```

### 4. 可视化图表

#### 权益曲线图

```python
import matplotlib.pyplot as plt
import pandas as pd

def plot_equity_curve(equity_curve, benchmark=None):
    """
    绘制权益曲线图

    Args:
        equity_curve: 权益序列
        benchmark: 基准权益序列 (可选)
    """
    plt.figure(figsize=(12, 6))

    # 绘制策略权益曲线
    plt.plot(equity_curve.index, equity_curve.values,
             label='Strategy', linewidth=2)

    # 绘制基准
    if benchmark is not None:
        plt.plot(benchmark.index, benchmark.values,
                 label='Benchmark', linewidth=2, alpha=0.7)

    plt.title('Equity Curve', fontsize=14, fontweight='bold')
    plt.xlabel('Date', fontsize=12)
    plt.ylabel('Portfolio Value ($)', fontsize=12)
    plt.legend(loc='best')
    plt.grid(True, alpha=0.3)

    # 添加零线
    plt.axhline(y=equity_curve.iloc[0], color='gray',
                linestyle='--', linewidth=1, alpha=0.5)

    plt.tight_layout()
    return plt
```

#### 回撤图

```python
def plot_drawdown(drawdown_series):
    """
    绘制回撤图
    """
    plt.figure(figsize=(12, 4))

    # 填充回撤区域
    plt.fill_between(drawdown_series.index, drawdown_series.values, 0,
                     color='red', alpha=0.3)

    plt.plot(drawdown_series.index, drawdown_series.values,
             color='red', linewidth=1.5)

    plt.title('Drawdown', fontsize=14, fontweight='bold')
    plt.xlabel('Date', fontsize=12)
    plt.ylabel('Drawdown (%)', fontsize=12)
    plt.gca().yaxis.set_major_formatter(plt.FuncFormatter(lambda y, _: f'{y:.0%}'))
    plt.grid(True, alpha=0.3)

    plt.tight_layout()
    return plt
```

#### 资产配置饼图

```python
def plot_allocation_pie_chart(allocation):
    """
    绘制资产配置饼图

    Args:
        allocation: {资产类别: 权重}
    """
    fig, ax = plt.subplots(figsize=(10, 8))

    colors = plt.cm.Set3(range(len(allocation)))

    wedges, texts, autotexts = ax.pie(
        allocation.values(),
        labels=allocation.keys(),
        autopct='%1.1f%%',
        colors=colors,
        startangle=90
    )

    # 设置文本样式
    for text in texts:
        text.set_fontsize(12)
        text.set_fontweight('bold')

    for autotext in autotexts:
        autotext.set_fontsize(10)
        autotext.set_color('white')
        autotext.set_fontweight('bold')

    ax.set_title('Asset Allocation', fontsize=14, fontweight='bold', pad=20)

    plt.tight_layout()
    return fig
```

#### 月度收益热力图

```python
import seaborn as sns

def plot_monthly_returns_heatmap(returns):
    """
    绘制月度收益热力图

    Args:
        returns: 日收益率序列
    """
    # 计算月度收益
    monthly_returns = returns.resample('M').apply(lambda x: (1 + x).prod() - 1)

    # 创建年-月矩阵
    monthly_returns_table = monthly_returns.groupby([
        monthly_returns.index.year,
        monthly_returns.index.month
    ]).first().unstack()

    # 绘制热力图
    plt.figure(figsize=(12, 8))
    sns.heatmap(
        monthly_returns_table,
        annot=True,
        fmt='.2%',
        cmap='RdYlGn',
        center=0,
        cbar_kws={'label': 'Monthly Return'}
    )

    plt.title('Monthly Returns Heatmap', fontsize=14, fontweight='bold')
    plt.xlabel('Month', fontsize=12)
    plt.ylabel('Year', fontsize=12)

    # 设置月份标签
    month_labels = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
                   'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
    plt.xticks(np.arange(12) + 0.5, month_labels)

    plt.tight_layout()
    return plt
```

### 5. 报告导出

#### PDF导出

```python
from reportlab.lib.pagesizes import letter, A4
from reportlab.lib import colors
from reportlab.lib.units import inch
from reportlab.platypus import SimpleDocTemplate, Table, TableStyle, Paragraph, Spacer, Image
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle

def export_to_pdf(report_data, filename):
    """
    导出报告为PDF

    Args:
        report_data: 报告数据字典
        filename: 输出文件名
    """
    doc = SimpleDocTemplate(filename, pagesize=A4)
    styles = getSampleStyleSheet()
    story = []

    # 标题
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=styles['Heading1'],
        fontSize=24,
        textColor=colors.HexColor('#2C3E50'),
        spaceAfter=30
    )
    story.append(Paragraph(report_data['title'], title_style))
    story.append(Spacer(1, 12))

    # 添加内容
    for section in report_data['sections']:
        # 章节标题
        story.append(Paragraph(section['title'], styles['Heading2']))
        story.append(Spacer(1, 12))

        # 章节内容
        if 'table' in section:
            # 创建表格
            table_data = section['table']
            table = Table(table_data, hAlign='LEFT')
            table.setStyle(TableStyle([
                ('BACKGROUND', (0, 0), (-1, 0), colors.grey),
                ('TEXTCOLOR', (0, 0), (-1, 0), colors.whitesmoke),
                ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
                ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
                ('FONTSIZE', (0, 0), (-1, 0), 14),
                ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
                ('BACKGROUND', (0, 1), (-1, -1), colors.beige),
                ('GRID', (0, 0), (-1, -1), 1, colors.black)
            ]))
            story.append(table)
        else:
            # 段落内容
            for content in section['content']:
                story.append(Paragraph(content, styles['Normal']))

        story.append(Spacer(1, 12))

    # 构建PDF
    doc.build(story)

    return filename
```

#### Excel导出

```python
import pandas as pd
from openpyxl import Workbook
from openpyxl.styles import Font, PatternFill, Alignment
from openpyxl.utils.dataframe import dataframe_to_rows

def export_to_excel(data_dict, filename):
    """
    导出数据到Excel

    Args:
        data_dict: {sheet_name: dataframe}
        filename: 输出文件名
    """
    with pd.ExcelWriter(filename, engine='openpyxl') as writer:
        for sheet_name, df in data_dict.items():
            df.to_excel(writer, sheet_name=sheet_name, index=True)

    # 格式化Excel
    wb = load_workbook(filename)
    for sheet in wb.worksheets:
        # 格式化标题行
        for cell in sheet[1]:
            cell.font = Font(bold=True, color='FFFFFF')
            cell.fill = PatternFill(start_color='366092',
                                   end_color='366092',
                                   fill_type='solid')
            cell.alignment = Alignment(horizontal='center')

        # 自动调整列宽
        for column in sheet.columns:
            max_length = 0
            column_letter = column[0].column_letter
            for cell in column:
                try:
                    if len(str(cell.value)) > max_length:
                        max_length = len(str(cell.value))
                except:
                    pass
            adjusted_width = (max_length + 2) * 1.2
            sheet.column_dimensions[column_letter].width = adjusted_width

    wb.save(filename)
    return filename
```

### 6. 报告模板

#### HTML模板

```python
def generate_html_report(report_data, template_path):
    """
    使用Jinja2模板生成HTML报告

    Args:
        report_data: 报告数据
        template_path: 模板文件路径
    """
    from jinja2 import Environment, FileSystemLoader

    env = Environment(loader=FileSystemLoader('.'))
    template = env.get_template(template_path)

    html_output = template.render(**report_data)

    return html_output
```

### 7. 自动化报告生成

```python
def generate_all_reports(portfolio, output_dir='./reports'):
    """
    自动生成所有报告

    包括:
    - PDF组合报告
    - Excel数据文件
    - HTML在线报告
    - 图表文件
    """
    import os
    from datetime import datetime

    # 创建输出目录
    os.makedirs(output_dir, exist_ok=True)

    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    base_filename = f"portfolio_report_{timestamp}"

    # 生成数据
    summary = generate_portfolio_summary(portfolio)
    allocation = generate_allocation_report(portfolio)
    risk = generate_risk_report(portfolio)

    # 合并数据
    all_data = {
        'title': 'InvestIntel AI - Portfolio Report',
        'sections': [
            {'title': 'Portfolio Summary', 'data': summary},
            {'title': 'Asset Allocation', 'data': allocation},
            {'title': 'Risk Analysis', 'data': risk}
        ]
    }

    # 导出PDF
    pdf_file = os.path.join(output_dir, f'{base_filename}.pdf')
    export_to_pdf(all_data, pdf_file)

    # 导出Excel
    excel_file = os.path.join(output_dir, f'{base_filename}.xlsx')
    export_to_excel({
        'Summary': pd.DataFrame(summary),
        'Allocation': pd.DataFrame(allocation),
        'Risk': pd.DataFrame(risk)
    }, excel_file)

    # 生成图表
    equity_chart = plot_equity_curve(portfolio['equity_curve'])
    equity_chart_file = os.path.join(output_dir, f'{base_filename}_equity.png')
    equity_chart.savefig(equity_chart_file)

    return {
        'pdf': pdf_file,
        'excel': excel_file,
        'charts': equity_chart_file
    }
```

## 最佳实践

### ✅ 推荐做法

1. **清晰简洁**
   - 关键信息突出
   - 避免信息过载
   - 使用可视化

2. **准确性**
   - 数据验证
   - 计算复核
   - 来源标注

3. **可定制**
   - 模板化
   - 参数化
   - 灵活配置

### ❌ 避免错误

1. **信息过载**
   - 突出重点
   - 分层展示
   - 提供摘要

2. **误导性图表**
   - 正确比例
   - 清楚标签
   - 完整信息

3. **格式混乱**
   - 统一风格
   - 专业排版
   - 一致格式

## 相关资源

- [报告模板](report-templates.md)
- [可视化参考](visualization.md)
- [导出格式](export-formats.md)

---

**版本**: 1.0.0
**最后更新**: 2026-01-10
