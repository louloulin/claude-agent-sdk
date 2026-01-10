#!/usr/bin/env python3
"""
SKILL.md 深度分析和效果验证脚本

不仅验证文件格式，还分析：
1. 内容质量和深度
2. 代码示例的完整性
3. 技术覆盖的广度
4. 实用性评分
5. 学习路径完整性
"""

import os
import re
from pathlib import Path
from typing import Dict, List, Tuple
from collections import Counter
import json


def analyze_code_examples(content: str) -> Dict:
    """分析代码示例"""
    # 检测代码块
    code_blocks = re.findall(r'```[\w]*\n(.*?)\n```', content, re.DOTALL)

    # 统计编程语言
    languages = re.findall(r'```(\w+)', content)

    # 统计代码行数
    total_code_lines = sum(len(block.split('\n')) for block in code_blocks)

    # 检测代码质量指标
    has_comments = any('//' in block or '#' in block or '/*' in block
                       for block in code_blocks)
    has_error_handling = any('try' in block.lower() or 'catch' in block.lower()
                             or 'error' in block.lower() for block in code_blocks)
    has_best_practices = any(word in content.lower() for word in
                            ['best practice', '最佳实践', 'recommend', '建议'])

    return {
        'code_blocks': len(code_blocks),
        'languages': Counter(languages),
        'total_code_lines': total_code_lines,
        'has_comments': has_comments,
        'has_error_handling': has_error_handling,
        'has_best_practices': has_best_practices,
        'avg_code_lines_per_block': total_code_lines / len(code_blocks) if code_blocks else 0
    }


def analyze_content_quality(content: str) -> Dict:
    """分析内容质量"""
    # 统计章节
    sections = re.findall(r'^#+\s+(.+)$', content, re.MULTILINE)

    # 统计列表项
    list_items = len(re.findall(r'^\s*[-*+]\s+', content, re.MULTILINE))

    # 统计表格
    tables = len(re.findall(r'\|.*\|', content))

    # 检测关键内容
    has_introduction = any(word in content.lower() for word in
                          ['introduction', '介绍', 'overview', '概述'])
    has_examples = any(word in content.lower() for word in
                      ['example', '示例', 'demo', '演示'])
    has_best_practices = any(word in content.lower() for word in
                            ['best practice', '最佳实践', 'recommendation', '建议'])
    has_troubleshooting = any(word in content.lower() for word in
                             ['troubleshooting', '故障排除', 'common issue', '常见问题'])
    has_tools = any(word in content.lower() for word in
                   ['tools', '工具', 'resources', '资源'])

    # 检测中文内容
    has_chinese = bool(re.search(r'[\u4e00-\u9fff]', content))

    # 内容深度指标
    lines = content.split('\n')
    non_empty_lines = [l for l in lines if l.strip()]
    content_depth_score = len(non_empty_lines) / 100  # 每100行1分

    return {
        'sections': len(sections),
        'list_items': list_items,
        'tables': tables,
        'has_introduction': has_introduction,
        'has_examples': has_examples,
        'has_best_practices': has_best_practices,
        'has_troubleshooting': has_troubleshooting,
        'has_tools': has_tools,
        'has_chinese': has_chinese,
        'content_depth_score': min(content_depth_score, 10),  # 最高10分
        'non_empty_lines': len(non_empty_lines)
    }


def analyze_technical_coverage(content: str) -> Dict:
    """分析技术覆盖"""
    # 技术关键词
    tech_keywords = {
        'languages': ['python', 'javascript', 'typescript', 'rust', 'go', 'java',
                     'swift', 'kotlin', 'ruby', 'php', 'c\+\+', 'c#'],
        'frameworks': ['react', 'vue', 'angular', 'django', 'flask', 'fastapi',
                      'spring', 'express', 'gin', 'echo', 'tensorflow', 'pytorch'],
        'databases': ['postgresql', 'mysql', 'mongodb', 'redis', 'elasticsearch',
                     'dynamodb', 'cassandra', 'neo4j'],
        'cloud': ['aws', 'azure', 'gcp', 'alibaba', 'terraform', 'kubernetes',
                 'docker', 'ansible', 'chef', 'puppet'],
        'tools': ['git', 'jenkins', 'github actions', 'gitlab ci', 'travis ci',
                 'prometheus', 'grafana', 'elk', 'jenkins'],
        'concepts': ['microservices', 'serverless', 'devops', 'cicd', 'tdd',
                    'bdd', 'agile', 'scrum', 'kubernetes', 'docker']
    }

    found_techs = {}
    content_lower = content.lower()

    for category, keywords in tech_keywords.items():
        found = []
        for keyword in keywords:
            if keyword.lower() in content_lower:
                found.append(keyword)
        if found:
            found_techs[category] = found

    return found_techs


def calculate_utility_score(quality: Dict, code: Dict, tech: Dict) -> float:
    """计算实用性评分 (0-100)"""
    score = 0

    # 内容质量 (40分)
    if quality['has_introduction']:
        score += 5
    if quality['has_examples']:
        score += 10
    if quality['has_best_practices']:
        score += 10
    if quality['has_troubleshooting']:
        score += 5
    if quality['has_tools']:
        score += 5
    score += min(quality['content_depth_score'] / 2, 5)  # 最多5分

    # 代码质量 (40分)
    if code['code_blocks'] > 0:
        score += 10
        score += min(code['code_blocks'] * 2, 10)  # 最多10分
    if code['has_comments']:
        score += 10
    if code['has_error_handling']:
        score += 5
    if code['total_code_lines'] > 100:
        score += 5

    # 技术覆盖 (20分)
    tech_categories = len(tech)
    score += min(tech_categories * 4, 20)  # 最多20分

    return min(score, 100)


def analyze_learning_path(skills: List[Dict]) -> Dict:
    """分析学习路径完整性"""
    # 定义技能层级
    skill_levels = {
        'beginner': ['example-calculator', 'api-tester', 'code-reviewer'],
        'intermediate': ['database-migrator', 'git-workflow', 'docker-helper',
                        'deployment-automation'],
        'advanced': ['frontend-developer', 'backend-developer', 'mobile-developer',
                    'performance-optimizer', 'logging-monitoring'],
        'expert': ['cloud-infrastructure', 'data-engineering',
                  'machine-learning-engineer', 'devops-engineer',
                  'security-auditor']
    }

    # 定义领域路径
    domain_paths = {
        'fullstack': ['frontend-developer', 'backend-developer', 'database-migrator'],
        'devops': ['docker-helper', 'git-workflow', 'deployment-automation',
                  'devops-engineer'],
        'data': ['data-engineering', 'machine-learning-engineer', 'performance-optimizer'],
        'mobile': ['mobile-developer', 'api-tester'],
        'cloud': ['cloud-infrastructure', 'devops-engineer', 'security-auditor']
    }

    skill_names = {s['metadata']['name'].lower().replace(' ', '-'): s
                   for s in skills}

    # 检查每条路径的完整性
    path_completion = {}
    for path_name, required_skills in domain_paths.items():
        completed = sum(1 for skill in required_skills if skill in skill_names)
        path_completion[path_name] = {
            'required': len(required_skills),
            'completed': completed,
            'percentage': (completed / len(required_skills)) * 100
        }

    return path_completion


def analyze_skill(skill_dir: Path) -> Dict:
    """深度分析单个技能"""
    skill_md = skill_dir / "SKILL.md"

    if not skill_md.exists():
        return None

    with open(skill_md, 'r', encoding='utf-8') as f:
        content = f.read()

    # 解析 frontmatter
    metadata = {}
    in_frontmatter = False
    yaml_lines = []

    for line in content.split('\n'):
        if line.strip() == '---':
            if not in_frontmatter:
                in_frontmatter = True
            else:
                break
        elif in_frontmatter:
            yaml_lines.append(line)

    # 简单解析 YAML
    for line in yaml_lines:
        if ':' in line:
            key, value = line.split(':', 1)
            metadata[key.strip()] = value.strip()

    # 提取 markdown 内容
    markdown_start = content.find('---', 1)
    if markdown_start != -1:
        markdown_start = content.find('---', markdown_start + 3) + 3
        markdown_content = content[markdown_start:]
    else:
        markdown_content = content

    # 深度分析
    code_analysis = analyze_code_examples(markdown_content)
    quality_analysis = analyze_content_quality(markdown_content)
    tech_analysis = analyze_technical_coverage(markdown_content)
    utility_score = calculate_utility_score(quality_analysis, code_analysis, tech_analysis)

    return {
        'path': skill_dir.name,
        'metadata': metadata,
        'code_analysis': code_analysis,
        'quality_analysis': quality_analysis,
        'tech_analysis': tech_analysis,
        'utility_score': utility_score
    }


def print_analysis_report(skills: List[Dict]):
    """打印深度分析报告"""

    print("\n╔══════════════════════════════════════════════════════════════════╗")
    print("║           🎯 SKILL.md 深度效果分析报告                          ║")
    print("╚══════════════════════════════════════════════════════════════════╝")

    # 总体统计
    print("\n📊 总体统计:")
    print(f"   分析技能数: {len(skills)} 个")

    avg_utility = sum(s['utility_score'] for s in skills) / len(skills)
    print(f"   平均实用性评分: {avg_utility:.1f}/100")

    total_code_blocks = sum(s['code_analysis']['code_blocks'] for s in skills)
    print(f"   总代码块数: {total_code_blocks} 个")

    total_code_lines = sum(s['code_analysis']['total_code_lines'] for s in skills)
    print(f"   总代码行数: {total_code_lines} 行")

    # 实用性排名
    print("\n🏆 实用性排名 (Top 10):")
    sorted_skills = sorted(skills, key=lambda x: x['utility_score'], reverse=True)[:10]

    for i, skill in enumerate(sorted_skills, 1):
        name = skill['metadata'].get('name', 'Unknown')
        score = skill['utility_score']
        emoji = "🥇" if i == 1 else "🥈" if i == 2 else "🥉" if i == 3 else "  "
        print(f"   {emoji} {i}. {name}")
        print(f"      评分: {score:.1f}/100")

        # 显示关键指标
        code_blocks = skill['code_analysis']['code_blocks']
        code_lines = skill['code_analysis']['total_code_lines']
        has_chinese = skill['quality_analysis']['has_chinese']

        print(f"      代码: {code_blocks} 个代码块, {code_lines} 行")
        print(f"      语言: {'中文' if has_chinese else '英文'}")
        print()

    # 内容质量分析
    print("📈 内容质量分析:")

    has_intro = sum(1 for s in skills if s['quality_analysis']['has_introduction'])
    has_examples = sum(1 for s in skills if s['quality_analysis']['has_examples'])
    has_best_practices = sum(1 for s in skills if s['quality_analysis']['has_best_practices'])
    has_troubleshooting = sum(1 for s in skills if s['quality_analysis']['has_troubleshooting'])
    has_tools = sum(1 for s in skills if s['quality_analysis']['has_tools'])

    print(f"   ✅ 有介绍: {has_intro}/{len(skills)} ({has_intro*100//len(skills)}%)")
    print(f"   ✅ 有示例: {has_examples}/{len(skills)} ({has_examples*100//len(skills)}%)")
    print(f"   ✅ 有最佳实践: {has_best_practices}/{len(skills)} ({has_best_practices*100//len(skills)}%)")
    print(f"   ✅ 有故障排除: {has_troubleshooting}/{len(skills)} ({has_troubleshooting*100//len(skills)}%)")
    print(f"   ✅ 有工具资源: {has_tools}/{len(skills)} ({has_tools*100//len(skills)}%)")

    # 代码质量分析
    print("\n💻 代码质量分析:")

    has_comments = sum(1 for s in skills if s['code_analysis']['has_comments'])
    has_error_handling = sum(1 for s in skills if s['code_analysis']['has_error_handling'])

    print(f"   ✅ 有注释: {has_comments}/{len(skills)} ({has_comments*100//len(skills)}%)")
    print(f"   ✅ 有错误处理: {has_error_handling}/{len(skills)} ({has_error_handling*100//len(skills)}%)")

    avg_code_per_block = sum(s['code_analysis']['avg_code_lines_per_block']
                            for s in skills) / len(skills)
    print(f"   📊 平均代码块大小: {avg_code_per_block:.1f} 行")

    # 技术覆盖分析
    print("\n🔧 技术覆盖分析:")

    all_techs = {}
    for skill in skills:
        for category, techs in skill['tech_analysis'].items():
            if category not in all_techs:
                all_techs[category] = Counter()
            for tech in techs:
                all_techs[category][tech] += 1

    for category, techs in sorted(all_techs.items()):
        print(f"   {category}:")
        for tech, count in techs.most_common(5):
            print(f"      - {tech}: {count} 个技能")

    # 学习路径分析
    print("\n🎓 学习路径完整性:")
    path_completion = analyze_learning_path(skills)

    for path_name, stats in sorted(path_completion.items(),
                                   key=lambda x: x[1]['percentage'],
                                   reverse=True):
        percentage = stats['percentage']
        completed = stats['completed']
        required = stats['required']

        status = "✅" if percentage == 100 else "⚠️" if percentage >= 60 else "❌"
        print(f"   {status} {path_name}: {completed}/{required} ({percentage:.0f}%)")

    # 语言支持分析
    print("\n🌐 语言支持分析:")

    chinese_skills = sum(1 for s in skills if s['quality_analysis']['has_chinese'])
    english_skills = len(skills) - chinese_skills

    print(f"   中文技能: {chinese_skills} 个 ({chinese_skills*100//len(skills)}%)")
    print(f"   英文技能: {english_skills} 个 ({english_skills*100//len(skills)}%)")

    # 推荐改进
    print("\n💡 改进建议:")

    if has_troubleshooting < len(skills):
        missing = len(skills) - has_troubleshooting
        print(f"   • 建议为 {missing} 个技能添加故障排除章节")

    if has_tools < len(skills):
        missing = len(skills) - has_tools
        print(f"   • 建议为 {missing} 个技能添加工具和资源章节")

    low_utility = [s for s in skills if s['utility_score'] < 70]
    if low_utility:
        print(f"   • 建议提升以下技能的实用性:")
        for skill in low_utility:
            name = skill['metadata'].get('name', 'Unknown')
            score = skill['utility_score']
            print(f"     - {name} ({score:.1f}/100)")

    # 总结
    print("\n╔══════════════════════════════════════════════════════════════════╗")
    print("║                     分析完成                                    ║")
    print("╚══════════════════════════════════════════════════════════════════╝\n")


def main():
    print("🔍 开始深度分析 SKILL.md 效果...\n")

    # 获取项目根目录
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    skills_dir = project_root / "examples" / ".claude" / "skills"

    print(f"📁 分析目录: {skills_dir}\n")

    # 分析所有技能
    skills = []
    for entry in skills_dir.iterdir():
        if entry.is_dir():
            skill = analyze_skill(entry)
            if skill:
                skills.append(skill)

    if not skills:
        print("❌ 未找到任何 SKILL.md 文件")
        return 1

    # 打印分析报告
    print_analysis_report(skills)

    # 保存详细分析结果
    report_path = Path("skill_analysis_report.json")
    with open(report_path, 'w', encoding='utf-8') as f:
        json.dump(skills, f, ensure_ascii=False, indent=2)

    print(f"📄 详细分析结果已保存到: {report_path}\n")

    return 0


if __name__ == "__main__":
    exit(main())
