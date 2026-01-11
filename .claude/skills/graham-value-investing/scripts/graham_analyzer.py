#!/usr/bin/env python3
"""
Graham Value Analyzer - 快速Graham价值分析工具

这个脚本提供命令行方式的Graham价值分析，可以独立运行。
Claude可以直接调用这个脚本而不需要加载完整实现到context。
"""

import sys
import argparse
from typing import Dict, Any, Optional


class GrahamAnalyzer:
    """Graham价值分析器"""

    @staticmethod
    def calculate_intrinsic_value(eps: float, growth_rate: float) -> float:
        """
        计算Graham内在价值

        V = EPS × (8.5 + 2g)

        Args:
            eps: 每股收益
            growth_rate: 预期增长率 (小数形式，例如 0.05 表示 5%)

        Returns:
            内在价值
        """
        return eps * (8.5 + 2.0 * growth_rate)

    @staticmethod
    def calculate_margin_of_safety(intrinsic_value: float, current_price: float) -> float:
        """
        计算安全边际

        Margin = (Intrinsic Value - Current Price) / Intrinsic Value

        Args:
            intrinsic_value: 内在价值
            current_price: 当前价格

        Returns:
            安全边际 (小数形式)
        """
        return (intrinsic_value - current_price) / intrinsic_value

    @staticmethod
    def calculate_valuation_score(margin_of_safety: float) -> int:
        """
        计算估值得分 (0-20分)

        Args:
            margin_of_safety: 安全边际

        Returns:
            得分
        """
        if margin_of_safety >= 0.50:
            return 20
        elif margin_of_safety >= 0.40:
            return 16
        elif margin_of_safety >= 0.30:
            return 12
        elif margin_of_safety >= 0.20:
            return 8
        elif margin_of_safety >= 0.10:
            return 4
        else:
            return 0

    @staticmethod
    def get_recommendation(margin_of_safety: float) -> str:
        """
        根据安全边际给出投资建议

        Args:
            margin_of_safety: 安全边际

        Returns:
            投资建议
        """
        if margin_of_safety >= 0.50:
            return "强烈买入 (5/5)"
        elif margin_of_safety >= 0.30:
            return "买入 (4/5)"
        elif margin_of_safety >= 0.15:
            return "持有 (3/5)"
        elif margin_of_safety >= 0.00:
            return "观望 (2/5)"
        else:
            return "避免 (1/5)"

    def analyze(self, symbol: str, eps: float, current_price: float, growth_rate: float = 0.05) -> Dict[str, Any]:
        """
        执行完整的Graham分析

        Args:
            symbol: 股票代码
            eps: 每股收益
            current_price: 当前价格
            growth_rate: 预期增长率 (默认5%)

        Returns:
            分析结果字典
        """
        # 计算内在价值
        intrinsic_value = self.calculate_intrinsic_value(eps, growth_rate)

        # 计算安全边际
        margin = self.calculate_margin_of_safety(intrinsic_value, current_price)

        # 计算得分
        score = self.calculate_valuation_score(margin)

        # 获取建议
        recommendation = self.get_recommendation(margin)

        return {
            "symbol": symbol.upper(),
            "eps": eps,
            "current_price": current_price,
            "growth_rate": growth_rate,
            "intrinsic_value": round(intrinsic_value, 2),
            "margin_of_safety": round(margin * 100, 2),
            "valuation_score": score,
            "recommendation": recommendation
        }

    def format_report(self, analysis: Dict[str, Any]) -> str:
        """
        格式化分析报告

        Args:
            analysis: 分析结果字典

        Returns:
            格式化的报告字符串
        """
        symbol = analysis["symbol"]
        intrinsic = analysis["intrinsic_value"]
        current = analysis["current_price"]
        margin = analysis["margin_of_safety"]
        score = analysis["valuation_score"]
        rec = analysis["recommendation"]

        report = f"""
📊 Graham快速估值 - {symbol}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━
内在价值: ${intrinsic:.2f}
当前价格: ${current:.2f}
安全边际: {margin:.1f}%

Graham评分: {score}/20
投资建议: {rec}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━
核心指标:
• EPS: ${analysis['eps']:.2f}
• 预期增长率: {analysis['growth_rate']*100:.1f}%
"""
        return report


def main():
    """命令行入口"""
    parser = argparse.ArgumentParser(
        description="Graham Value Analyzer - 快速Graham价值分析"
    )
    parser.add_argument("symbol", help="股票代码")
    parser.add_argument("--eps", type=float, required=True, help="每股收益 (EPS)")
    parser.add_argument("--price", type=float, required=True, help="当前价格")
    parser.add_argument("--growth", type=float, default=0.05,
                       help="预期增长率 (默认5%%，例如0.05表示5%%)")

    args = parser.parse_args()

    # 创建分析器
    analyzer = GrahamAnalyzer()

    # 执行分析
    analysis = analyzer.analyze(
        symbol=args.symbol,
        eps=args.eps,
        current_price=args.price,
        growth_rate=args.growth
    )

    # 打印报告
    print(analyzer.format_report(analysis))


if __name__ == "__main__":
    main()
