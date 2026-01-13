#!/usr/bin/env python3
"""
Skills Optimization Script

Analyzes and optimizes existing SKILL.md files to follow Claude Code best practices.
"""

import os
import re
import yaml
from pathlib import Path
from typing import Dict, List, Tuple

class SkillAnalyzer:
    """Analyze SKILL.md files for optimization opportunities."""

    def __init__(self, skills_dir: str):
        self.skills_dir = Path(skills_dir)
        self.skills = self._discover_skills()

    def _discover_skills(self) -> List[Path]:
        """Find all SKILL.md files."""
        return list(self.skills_dir.glob("*/SKILL.md"))

    def analyze_skill(self, skill_path: Path) -> Dict:
        """Analyze a single skill file."""
        with open(skill_path, 'r', encoding='utf-8') as f:
            content = f.read()

        # Extract frontmatter
        frontmatter_match = re.match(r'^---\n(.*?)\n---', content, re.DOTALL)
        if not frontmatter_match:
            return {'error': 'No frontmatter found'}

        try:
            metadata = yaml.safe_load(frontmatter_match.group(1))
        except yaml.YAMLError as e:
            return {'error': f'YAML parsing error: {e}'}

        # Analyze issues and recommendations
        issues = []
        recommendations = []

        # Check description quality
        description = metadata.get('description', '')

        # 1. Check for trigger words
        if not self._has_trigger_words(description):
            issues.append('Description lacks clear trigger words')
            recommendations.append('Add trigger words like "Use when working with..." or "Use when user mentions..."')

        # 2. Check name format (should be lowercase)
        name = metadata.get('name', '')
        if name != name.lower():
            issues.append('Name should be lowercase')
            recommendations.append(f'Change name from "{name}" to "{name.lower()}"')

        # 3. Check for advanced fields
        if 'allowed_tools' not in metadata:
            recommendations.append('Consider adding allowed-tools field to restrict tool access')

        # 4. Check for progressive disclosure
        skill_dir = skill_path.parent
        has_reference = (skill_dir / 'reference.md').exists()
        has_examples = (skill_dir / 'examples.md').exists()
        has_forms = (skill_dir / 'forms.md').exists()
        has_scripts = (skill_dir / 'scripts').exists() and (skill_dir / 'scripts').is_dir()

        if not has_reference and len(content) > 500:
            recommendations.append('Consider splitting detailed content into reference.md (progressive disclosure)')

        if not has_examples:
            recommendations.append('Add examples.md with practical usage examples')

        # 5. Check for scripts
        if not has_scripts and 'script' in content.lower():
            recommendations.append('Add scripts/ directory with utility scripts')

        # 6. Check description length (max 1024 chars per spec)
        if len(description) > 1024:
            issues.append(f'Description too long ({len(description)} chars, max 1024)')
            recommendations.append('Shorten description to under 1024 characters')

        # 7. Check name length (max 64 chars per spec)
        if len(name) > 64:
            issues.append(f'Name too long ({len(name)} chars, max 64)')
            recommendations.append('Shorten name to under 64 characters')

        return {
            'path': str(skill_path.relative_to(self.skills_dir.parent)),
            'name': name,
            'description': description,
            'metadata': metadata,
            'issues': issues,
            'recommendations': recommendations,
            'has_reference': has_reference,
            'has_examples': has_examples,
            'has_forms': has_forms,
            'has_scripts': has_scripts,
            'content_lines': len(content.split('\n')),
        }

    def _has_trigger_words(self, description: str) -> bool:
        """Check if description has trigger words."""
        trigger_patterns = [
            r'use when',
            r'use for',
            r'helps? (you|to)',
            r'when (the )?user (mentions?|asks?|requests?)',
            r'for working with',
            r'call when',
        ]
        description_lower = description.lower()
        return any(re.search(pattern, description_lower) for pattern in trigger_patterns)

    def analyze_all(self) -> List[Dict]:
        """Analyze all skills."""
        results = []
        for skill_path in self.skills:
            result = self.analyze_skill(skill_path)
            results.append(result)
        return results

    def generate_report(self, results: List[Dict]) -> str:
        """Generate optimization report."""
        report_lines = [
            "# Skills Optimization Report\n",
            f"## Summary",
            f"- Total Skills: {len(results)}",
            f"- Skills with Issues: {sum(1 for r in results if r.get('issues'))}",
            f"- Skills with Recommendations: {sum(1 for r in results if r.get('recommendations'))}",
            "",
            "## Detailed Analysis\n",
        ]

        for i, result in enumerate(results, 1):
            if 'error' in result:
                report_lines.append(f"### {i}. {result.get('path', 'Unknown')}")
                report_lines.append(f"**Error**: {result['error']}\n")
                continue

            report_lines.append(f"### {i}. {result['name']}")
            report_lines.append(f"**Path**: `{result['path']}`")
            report_lines.append(f"**Lines**: {result['content_lines']}")
            report_lines.append(f"**Description**: {result['description'][:100]}...")

            if result['issues']:
                report_lines.append(f"\n**Issues**:")
                for issue in result['issues']:
                    report_lines.append(f"- ❌ {issue}")

            if result['recommendations']:
                report_lines.append(f"\n**Recommendations**:")
                for rec in result['recommendations']:
                    report_lines.append(f"- 💡 {rec}")

            # Show structure
            report_lines.append(f"\n**Structure**:")
            report_lines.append(f"- SKILL.md: ✓")
            report_lines.append(f"- reference.md: {'✓' if result['has_reference'] else '✗'}")
            report_lines.append(f"- examples.md: {'✓' if result['has_examples'] else '✗'}")
            report_lines.append(f"- forms.md: {'✓' if result['has_forms'] else '✗'}")
            report_lines.append(f"- scripts/: {'✓' if result['has_scripts'] else '✗'}")

            report_lines.append("\n" + "-" * 80 + "\n")

        return "\n".join(report_lines)

    def generate_priority_list(self, results: List[Dict]) -> List[Dict]:
        """Generate prioritized list of skills needing optimization."""
        prioritized = []

        for result in results:
            if 'error' in result:
                continue

            score = 0
            # Higher score = more urgent optimization needed

            # Critical issues
            if result['issues']:
                score += len(result['issues']) * 10

            # Large files benefit from progressive disclosure
            if result['content_lines'] > 500:
                if not result['has_reference']:
                    score += 5
                if not result['has_examples']:
                    score += 3

            # Missing best practices
            if not result['has_scripts']:
                score += 2

            if result['recommendations']:
                score += len(result['recommendations'])

            if score > 0:
                result['priority_score'] = score
                prioritized.append(result)

        # Sort by priority score (highest first)
        prioritized.sort(key=lambda x: x['priority_score'], reverse=True)
        return prioritized


def main():
    """Main entry point."""
    script_dir = Path(__file__).parent
    skills_dir = script_dir.parent / "examples" / ".claude" / "skills"

    if not skills_dir.exists():
        print(f"Error: Skills directory not found: {skills_dir}")
        return 1

    analyzer = SkillAnalyzer(skills_dir)
    results = analyzer.analyze_all()

    # Generate report
    report = analyzer.generate_report(results)
    report_path = script_dir.parent / "SKILLS_OPTIMIZATION_REPORT.md"
    report_path.write_text(report, encoding='utf-8')
    print(f"✓ Generated optimization report: {report_path}")

    # Generate priority list
    prioritized = analyzer.generate_priority_list(results)

    priority_path = script_dir.parent / "SKILLS_OPTIMIZATION_PRIORITY.md"
    priority_lines = [
        "# Skills Optimization Priority List\n",
        f"## Top {len(prioritized)} Skills Needing Optimization\n",
        "Sorted by priority score (highest = most urgent)\n",
    ]

    for i, skill in enumerate(prioritized, 1):
        priority_lines.extend([
            f"### {i}. {skill['name']} (Priority: {skill['priority_score']})",
            f"**Path**: `{skill['path']}`\n",
        ])

        if skill['issues']:
            priority_lines.append("**Critical Issues**:")
            for issue in skill['issues']:
                priority_lines.append(f"- ❌ {issue}")
            priority_lines.append("")

        if skill['recommendations']:
            priority_lines.append("**Recommendations**:")
            for rec in skill['recommendations'][:5]:  # Top 5
                priority_lines.append(f"- 💡 {rec}")
            priority_lines.append("")

        priority_lines.append("-" * 80 + "\n")

    priority_path.write_text("\n".join(priority_lines), encoding='utf-8')
    print(f"✓ Generated priority list: {priority_path}")

    # Print summary
    print("\n📊 Summary:")
    print(f"  Total skills analyzed: {len(results)}")
    print(f"  Skills needing optimization: {len(prioritized)}")
    print(f"  High priority (score >= 10): {sum(1 for s in prioritized if s['priority_score'] >= 10)}")
    print(f"  Medium priority (score 5-9): {sum(1 for s in prioritized if 5 <= s['priority_score'] < 10)}")
    print(f"  Low priority (score < 5): {sum(1 for s in prioritized if s['priority_score'] < 5)}")

    return 0


if __name__ == '__main__':
    exit(main())
