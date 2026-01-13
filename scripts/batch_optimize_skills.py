#!/usr/bin/env python3
"""
Batch Skills Optimization Tool

Quickly optimize multiple skills with best practices.
"""

import os
import re
from pathlib import Path
from typing import List, Dict

def optimize_skill_metadata(skill_path: Path) -> Dict:
    """Optimize a single skill's metadata."""
    with open(skill_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Extract and optimize frontmatter
    frontmatter_match = re.match(r'^(---\n.*?\n---)', content, re.DOTALL)
    if not frontmatter_match:
        return {'error': 'No frontmatter found'}

    frontmatter = frontmatter_match.group(1)
    body_content = content[frontmatter_match.end():]

    # Extract name
    name_match = re.search(r'name:\s*["\']([^"\']+)["\']', frontmatter)
    if not name_match:
        return {'error': 'No name found'}

    name = name_match.group(1)

    # Optimize name to lowercase
    optimized_name = name.lower().replace(' ', '-')
    frontmatter = re.sub(
        r'name:\s*["\'][^"\']+["\']',
        f'name: "{optimized_name}"',
        frontmatter
    )

    # Extract description
    desc_match = re.search(r'description:\s*["\']([^"\']+)["\']', frontmatter)
    if not desc_match:
        return {'error': 'No description found'}

    description = desc_match.group(1)

    # Check if description needs trigger words
    needs_triggers = not any(
        phrase in description.lower()
        for phrase in ['use when', 'working with', 'user mentions', 'call when']
    )

    if needs_triggers:
        # Add trigger words based on name
        trigger_phrases = {
            'backend': 'Use when working with APIs, databases, backend systems, or when the user mentions server-side development.',
            'frontend': 'Use when working with UI components, web interfaces, or when the user mentions client-side development.',
            'data': 'Use when working with data analysis, statistics, visualization, or when the user mentions data processing.',
            'security': 'Use when working with security audits, vulnerability assessments, or when the user mentions security.',
            'devops': 'Use when working with deployment, CI/CD, infrastructure, or when the user mentions operations.',
        }

        for key, trigger in trigger_phrases.items():
            if key in optimized_name:
                description = f"{description} {trigger}"
                break

    frontmatter = re.sub(
        r'description:\s*["\'][^"\']+["\']',
        f'description: "{description}"',
        frontmatter
    )

    # Add allowed-tools if missing
    if 'allowed_tools:' not in frontmatter:
        # Insert after dependencies or tags
        if 'dependencies:' in frontmatter:
            insert_pos = frontmatter.find('dependencies:')
        elif 'tags:' in frontmatter:
            insert_pos = frontmatter.find('tags:')
        else:
            insert_pos = frontmatter.find('\n', frontmatter.find('version:'))

        if insert_pos != -1:
            # Find the end of the section
            section_end = frontmatter.find('\n\n', insert_pos)
            if section_end == -1:
                section_end = len(frontmatter)

            # Determine appropriate tools based on skill name
            tools = ['Read', 'Write', 'Edit', 'Grep', 'Bash']

            tools_section = f'\nallowed_tools:\n'
            tools_section += '\n'.join(f'  - {tool}' for tool in tools)

            frontmatter = frontmatter[:section_end] + tools_section + frontmatter[section_end:]

    # Update version
    frontmatter = re.sub(
        r'version:\s*["\']?[\d.]+"?\n',
        'version: "2.0.0"\n',
        frontmatter
    )

    return {
        'name': optimized_name,
        'original_name': name,
        'description': description,
        'frontmatter': frontmatter,
        'body': body_content,
        'optimized': True
    }

def create_optimized_skill(skill_path: Path, optimization: Dict) -> bool:
    """Write optimized skill back to file."""
    if 'error' in optimization:
        return False

    new_content = optimization['frontmatter'] + '\n' + optimization['body']

    # Backup original
    backup_path = skill_path.with_suffix('.md.backup')
    skill_path.rename(backup_path)

    # Write optimized
    skill_path.write_text(new_content, encoding='utf-8')

    return True

def optimize_all_skills(skills_dir: Path, dry_run: bool = False) -> List[Dict]:
    """Optimize all skills in directory."""
    results = []

    for skill_file in sorted(skills_dir.glob("*/SKILL.md")):
        print(f"\nProcessing: {skill_file.parent.name}")

        optimization = optimize_skill_metadata(skill_file)

        if 'error' in optimization:
            print(f"  ❌ Error: {optimization['error']}")
            results.append({
                'skill': skill_file.parent.name,
                'status': 'error',
                'error': optimization['error']
            })
            continue

        print(f"  Name: {optimization['original_name']} → {optimization['name']}")
        print(f"  Description: {optimization['description'][:80]}...")

        if not dry_run:
            success = create_optimized_skill(skill_file, optimization)
            if success:
                print(f"  ✅ Optimized (backup created)")
                results.append({
                    'skill': skill_file.parent.name,
                    'status': 'optimized',
                    'original_name': optimization['original_name'],
                    'new_name': optimization['name']
                })
            else:
                print(f"  ❌ Failed to write")
                results.append({
                    'skill': skill_file.parent.name,
                    'status': 'write_failed'
                })
        else:
            print(f"  📝 Would optimize (dry run)")
            results.append({
                'skill': skill_file.parent.name,
                'status': 'dry_run',
                'original_name': optimization['original_name'],
                'new_name': optimization['name']
            })

    return results

def main():
    """Main entry point."""
    import argparse

    parser = argparse.ArgumentParser(description='Optimize SKILL.md files')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be done without making changes')
    parser.add_argument('--skills-dir', type=str, help='Path to skills directory')

    args = parser.parse_args()

    # Determine skills directory
    if args.skills_dir:
        skills_dir = Path(args.skills_dir)
    else:
        script_dir = Path(__file__).parent
        skills_dir = script_dir.parent / "examples" / ".claude" / "skills"

    if not skills_dir.exists():
        print(f"Error: Skills directory not found: {skills_dir}")
        return 1

    print(f"Optimizing skills in: {skills_dir}")
    print(f"Mode: {'DRY RUN' if args.dry_run else 'LIVE'}")

    results = optimize_all_skills(skills_dir, dry_run=args.dry_run)

    # Summary
    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)

    optimized = sum(1 for r in results if r['status'] in ['optimized', 'dry_run'])
    errors = sum(1 for r in results if r['status'] in ['error', 'write_failed'])

    print(f"Total skills: {len(results)}")
    print(f"Would optimize: {optimized}")
    print(f"Errors: {errors}")

    if not args.dry_run and optimized > 0:
        print(f"\n✅ {optimized} skills optimized")
        print(f"📁 Backups created with .md.backup extension")

    return 0

if __name__ == '__main__':
    exit(main())
