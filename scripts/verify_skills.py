#!/usr/bin/env python3
"""
SKILL.md 功能验证脚本

验证所有 SKILL.md 文件的完整性，包括：
- YAML frontmatter 解析
- 元数据完整性
- 内容统计
- 依赖关系分析
"""

import os
import re
import sys
from pathlib import Path
from typing import Dict, List, Tuple
from collections import Counter

def parse_frontmatter(content: str) -> Tuple[Dict[str, str], str]:
    """解析 YAML frontmatter 和 markdown 内容"""
    lines = content.split('\n')

    if len(lines) < 2:
        raise ValueError("文件内容太少")

    # 查找 frontmatter 开始和结束
    if not lines[0].strip().startswith('---'):
        raise ValueError("缺少 frontmatter 开始标记")

    yaml_lines = []
    frontmatter_end = None

    for i, line in enumerate(lines[1:], 1):
        if line.strip().startswith('---'):
            frontmatter_end = i
            break
        yaml_lines.append(line)

    if frontmatter_end is None:
        raise ValueError("缺少 frontmatter 结束标记")

    # 解析 YAML 字段
    metadata = {}
    current_field = None
    current_value = []

    for line in yaml_lines:
        line = line.strip()

        # 跳过空行和注释
        if not line or line.startswith('#'):
            continue

        # 检查是否是字段定义
        if ':' in line and not line.startswith('-'):
            # 保存前一个字段
            if current_field:
                if isinstance(current_value, list):
                    metadata[current_field] = current_value
                else:
                    metadata[current_field] = '\n'.join(current_value)
                current_value = []

            # 解析新字段
            key, value = line.split(':', 1)
            key = key.strip()
            value = value.strip()

            if value:
                metadata[key] = value
                current_field = None
            else:
                current_field = key
                current_value = []
        elif line.startswith('-'):
            # 列表项
            item = line[1:].strip()
            if current_field:
                if not isinstance(current_value, list):
                    current_value = []
                current_value.append(item)
        elif current_field:
            # 多行值
            current_value.append(line)

    # 保存最后一个字段
    if current_field:
        metadata[current_field] = current_value

    # 提取 markdown 内容
    markdown_content = '\n'.join(lines[frontmatter_end + 1:])

    return metadata, markdown_content


def load_skill(skill_dir: Path) -> Tuple[Dict, str, List[str]]:
    """加载单个 SKILL.md 文件"""
    skill_md = skill_dir / "SKILL.md"

    if not skill_md.exists():
        raise FileNotFoundError(f"SKILL.md 文件不存在: {skill_md}")

    with open(skill_md, 'r', encoding='utf-8') as f:
        content = f.read()

    metadata, markdown_content = parse_frontmatter(content)

    # 提取必需字段
    errors = []

    if 'name' not in metadata:
        errors.append("缺少 name 字段")

    if 'description' not in metadata:
        errors.append("缺少 description 字段")

    if 'version' not in metadata:
        errors.append("缺少 version 字段")

    return metadata, markdown_content, errors


def scan_skills_dir(skills_dir: Path) -> Tuple[List[Dict], List[Dict]]:
    """扫描所有技能目录"""
    skills = []
    errors = []

    if not skills_dir.exists():
        errors.append({
            'path': str(skills_dir),
            'error': 'Skills 目录不存在'
        })
        return skills, errors

    for entry in skills_dir.iterdir():
        if entry.is_dir():
            try:
                metadata, content, parse_errors = load_skill(entry)

                if parse_errors:
                    errors.append({
                        'path': str(entry),
                        'error': f"解析错误: {', '.join(parse_errors)}"
                    })
                else:
                    skills.append({
                        'metadata': metadata,
                        'content': content,
                        'path': entry
                    })
            except Exception as e:
                errors.append({
                    'path': str(entry),
                    'error': str(e)
                })

    return skills, errors


def print_statistics(skills: List[Dict], errors: List[Dict]):
    """打印详细统计信息"""
    print("\n╔════════════════════════════════════════════════════════════╗")
    print("║           🎯 SKILL.md 功能验证报告                        ║")
    print("╚════════════════════════════════════════════════════════════╝")

    print(f"\n📊 总体统计:")
    print(f"   ✅ 成功加载: {len(skills)} 个 SKILL.md 文件")
    print(f"   ❌ 加载失败: {len(errors)} 个文件")
    print(f"   📁 总计扫描: {len(skills) + len(errors)} 个技能")

    if not skills and not errors:
        print("\n⚠️  警告: 未找到任何 SKILL.md 文件")
        return

    # 成功加载的技能详情
    if skills:
        print("\n✅ 成功加载的技能:")

        total_lines = 0
        tags_counter = Counter()
        langs_counter = Counter()
        versions_counter = Counter()

        for i, skill in enumerate(skills, 1):
            metadata = skill['metadata']
            content = skill['content']
            path = skill['path']

            line_count = len(content.split('\n'))
            total_lines += line_count

            print(f"\n   {i}. {metadata.get('name', 'Unknown')}")
            print(f"      📂 路径: {path.name}")
            print(f"      📝 描述: {metadata.get('description', 'N/A')[:80]}...")
            print(f"      🏷️  版本: {metadata.get('version', 'N/A')}")

            if 'author' in metadata:
                print(f"      👤 作者: {metadata['author']}")

            print(f"      📄 内容行数: {line_count} 行")

            # 标签
            tags = metadata.get('tags', [])
            if tags:
                if isinstance(tags, list):
                    tags_str = ', '.join(tags)
                    for tag in tags:
                        tags_counter[tag] += 1
                else:
                    tags_str = tags
                print(f"      🏷️  标签: {tags_str}")

            # 依赖
            deps = metadata.get('dependencies', [])
            if deps:
                if isinstance(deps, list):
                    deps_str = ', '.join(deps)
                else:
                    deps_str = deps
                print(f"      🔗 依赖: {deps_str}")

            # 统计语言
            content_lower = content.lower()

            # 检测中文
            if '中文' in content_lower or '专家' in content_lower:
                langs_counter['中文'] += 1
            # 检测 Rust
            elif 'rust' in content_lower or 'fn ' in content_lower or 'let mut' in content_lower:
                langs_counter['Rust'] += 1
            # 检测 Python
            elif 'python' in content_lower or 'def ' in content_lower or 'import ' in content_lower:
                langs_counter['Python'] += 1
            # 检测 JavaScript/TypeScript
            elif 'javascript' in content_lower or 'typescript' in content_lower or 'const ' in content_lower:
                langs_counter['JavaScript/TypeScript'] += 1
            # 检测 Swift
            elif 'swift' in content_lower or '@main' in content_lower:
                langs_counter['Swift'] += 1
            # 检测 Kotlin
            elif 'kotlin' in content_lower or 'fun ' in content_lower:
                langs_counter['Kotlin'] += 1
            # 检测 Go
            elif ' go ' in content_lower or 'func ' in content_lower:
                langs_counter['Go'] += 1
            # 检测 SQL
            elif 'sql' in content_lower or 'select ' in content_lower:
                langs_counter['SQL'] += 1
            else:
                langs_counter['其他'] += 1

            # 版本统计
            version = metadata.get('version', 'unknown')
            versions_counter[version] += 1

        # 总体统计
        print(f"\n📈 内容统计:")
        print(f"   📝 总内容行数: {total_lines:,} 行")
        print(f"   📊 平均行数: {total_lines // len(skills)} 行/技能")

        # 语言分布
        if langs_counter:
            print(f"\n🌐 编程语言分布:")
            for lang, count in langs_counter.most_common():
                print(f"      - {lang}: {count} 个技能")

        # 热门标签
        if tags_counter:
            print(f"\n🏷️  热门标签:")
            for tag, count in tags_counter.most_common(10):
                print(f"      - {tag}: {count} 个技能")

        # 版本分布
        if versions_counter:
            print(f"\n📊 版本分布:")
            for version, count in versions_counter.most_common():
                print(f"   v{version}: {count} 个技能")

    # 加载失败的文件
    if errors:
        print(f"\n❌ 加载失败的文件:")
        for i, error in enumerate(errors, 1):
            print(f"\n   {i}. {error['path']}")
            print(f"      ⚠️  错误: {error['error']}")

    print(f"\n✅ 验证完成!")
    print("╔════════════════════════════════════════════════════════════╗")
    print("║              SKILL.md 功能验证完成                         ║")
    print("╚════════════════════════════════════════════════════════════╝\n")


def main():
    print("🔍 开始验证 SKILL.md 功能...\n")

    # 获取项目根目录
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    skills_dir = project_root / "examples" / ".claude" / "skills"

    print(f"📁 扫描目录: {skills_dir}")

    # 扫描所有技能
    skills, errors = scan_skills_dir(skills_dir)

    # 打印统计信息
    print_statistics(skills, errors)

    # 返回适当的退出码
    if errors:
        print(f"⚠️  发现 {len(errors)} 个错误")
        sys.exit(1)
    elif not skills:
        print("⚠️  未找到任何 SKILL.md 文件")
        sys.exit(1)
    else:
        print("✅ 所有 SKILL.md 文件验证成功!")
        sys.exit(0)


if __name__ == "__main__":
    main()
