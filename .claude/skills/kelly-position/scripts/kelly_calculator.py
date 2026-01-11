#!/usr/bin/env python3
"""
Kelly仓位计算器 - 命令行工具

快速计算Kelly最优仓位,支持完整Kelly和分数Kelly。
"""

import sys
import argparse
from typing import Dict, Any, Optional
import math


class KellyCalculator:
    """Kelly仓位计算器"""

    @staticmethod
    def calculate_kelly_full(win_rate: float, avg_win: float, avg_loss: float) -> float:
        """
        计算完整Kelly

        f* = (bp - q) / b

        Args:
            win_rate: 胜率 (0-1之间)
            avg_win: 平均盈利
            avg_loss: 平均亏损

        Returns:
            Kelly仓位比例
        """
        b = avg_win / avg_loss  # 盈亏比
        p = win_rate
        q = 1.0 - p

        kelly = (b * p - q) / b
        return kelly

    @staticmethod
    def calculate_kelly_simplified(returns: list) -> float:
        """
        计算简化Kelly

        f = μ / σ²

        Args:
            returns: 历史收益率列表

        Returns:
            Kelly仓位比例
        """
        n = len(returns)
        if n < 2:
            return 0.0

        # 计算平均收益
        mu = sum(returns) / n

        # 计算方差
        variance = sum((r - mu) ** 2 for r in returns) / (n - 1)

        if variance == 0:
            return 0.0

        kelly = mu / variance
        return kelly

    @staticmethod
    def apply_fractional_kelly(kelly: float, fraction: float = 0.25) -> float:
        """
        应用分数Kelly

        Args:
            kelly: 完整Kelly结果
            fraction: 分数 (默认0.25即1/4 Kelly)

        Returns:
            分数Kelly仓位
        """
        return kelly * fraction

    @staticmethod
    def apply_position_limits(kelly: float) -> tuple[float, bool]:
        """
        应用仓位限制

        Args:
            kelly: Kelly结果

        Returns:
            (最终仓位, 是否被限制)
        """
        MAX_POSITION = 0.25  # 单只股票最大25%
        MIN_POSITION = 0.02  # 最小2%

        if kelly < MIN_POSITION:
            return (0.0, True)  # 信号太弱
        elif kelly > MAX_POSITION:
            return (MAX_POSITION, True)  # 触发上限
        else:
            return (kelly, False)

    @staticmethod
    def assess_risk(kelly: float) -> str:
        """
        评估风险等级

        Args:
            kelly: Kelly结果

        Returns:
            风险等级描述
        """
        if kelly > 0.20:
            return "高"
        elif kelly > 0.10:
            return "中"
        elif kelly > 0.05:
            return "低"
        else:
            return "极低"

    def analyze_from_stats(
        self,
        win_rate: float,
        avg_win: float,
        avg_loss: float,
        fraction: float = 0.25
    ) -> Dict[str, Any]:
        """
        从交易统计数据进行分析

        Args:
            win_rate: 胜率
            avg_win: 平均盈利
            avg_loss: 平均亏损
            fraction: 分数Kelly系数

        Returns:
            分析结果字典
        """
        # 计算完整Kelly
        kelly = self.calculate_kelly_full(win_rate, avg_win, avg_loss)

        # 应用分数Kelly
        fractional_kelly = self.apply_fractional_kelly(kelly, fraction)

        # 应用仓位限制
        final_position, is_limited = self.apply_position_limits(fractional_kelly)

        # 评估风险
        risk_level = self.assess_risk(fractional_kelly)

        return {
            "kelly_full": round(kelly * 100, 2),
            "fractional_kelly": round(fractional_kelly * 100, 2),
            "recommended_position": round(final_position * 100, 2),
            "risk_level": risk_level,
            "is_limited": is_limited,
            "win_rate": round(win_rate * 100, 1),
            "avg_win": avg_win,
            "avg_loss": avg_loss,
        }

    def format_report(self, analysis: Dict[str, Any]) -> str:
        """
        格式化分析报告

        Args:
            analysis: 分析结果字典

        Returns:
            格式化的报告字符串
        """
        kelly_full = analysis["kelly_full"]
        fractional = analysis["fractional_kelly"]
        recommended = analysis["recommended_position"]
        risk = analysis["risk_level"]
        win_rate = analysis["win_rate"]

        report = f"""
📊 Kelly仓位计算结果

━━━━━━━━━━━━━━━━━━━━━━━━━━━━
【统计数据】
胜率: {win_rate}%
平均盈利: ${analysis['avg_win']:.2f}
平均亏损: ${analysis['avg_loss']:.2f}

【Kelly计算】
完整Kelly: {kelly_full}%
1/4分数Kelly: {fractional}%

【推荐仓位】
{recommended}%
风险等级: {risk}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━
"""

        if analysis["is_limited"]:
            if recommended == 0:
                report += "⚠️ Kelly信号不足(<2%),不建议建仓\n"
            elif recommended >= 25:
                report += "⚠️ 触发单只股票上限(25%)\n"

        return report


def main():
    """命令行入口"""
    parser = argparse.ArgumentParser(
        description="Kelly仓位计算器 - 科学计算投资仓位"
    )

    # 输入模式
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--stats", nargs=3, metavar=("WIN_RATE", "AVG_WIN", "AVG_LOSS"),
                       type=float, help="交易统计: 胜率(0-1) 平均盈利 平均亏损")
    group.add_argument("--returns", nargs='+', type=float,
                       help="历史收益率列表(例如: 0.05 -0.03 0.08)")

    # 可选参数
    parser.add_argument("--fraction", type=float, default=0.25,
                       help="分数Kelly系数(默认0.25即1/4 Kelly)")
    parser.add_argument("--json", action="store_true", help="输出JSON格式")

    args = parser.parse_args()

    calculator = KellyCalculator()

    if args.stats:
        # 从交易统计计算
        win_rate, avg_win, avg_loss = args.stats

        if not 0 <= win_rate <= 1:
            print("错误: 胜率必须在0-1之间")
            sys.exit(1)

        analysis = calculator.analyze_from_stats(win_rate, avg_win, avg_loss, args.fraction)

    elif args.returns:
        # 从收益率序列计算
        kelly = calculator.calculate_kelly_simplified(args.returns)
        fractional = calculator.apply_fractional_kelly(kelly, args.fraction)
        final, is_limited = calculator.apply_position_limits(fractional)
        risk = calculator.assess_risk(fractional)

        analysis = {
            "kelly_full": round(kelly * 100, 2),
            "fractional_kelly": round(fractional * 100, 2),
            "recommended_position": round(final * 100, 2),
            "risk_level": risk,
            "is_limited": is_limited,
            "returns_count": len(args.returns),
        }

    # 输出结果
    if args.json:
        import json
        print(json.dumps(analysis, indent=2, ensure_ascii=False))
    else:
        print(calculator.format_report(analysis))


if __name__ == "__main__":
    main()
