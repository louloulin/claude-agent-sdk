---
name: skill-md-validator
description: "Validates SKILL.md files for format, completeness, and best practices"
version: "1.0.0"
author: "Documentation Team <docs@example.com>"
tags:
  - validation
  - documentation
  - quality
  - tooling
dependencies: []
---

# SKILL.md Validator

A utility for validating SKILL.md files to ensure they follow best practices and are complete.

## What It Checks

### Required Fields
- `name:` - Skill name (required)
- `description:` - Skill description (required)
- `version:` - Semantic version (required)

### Format Validation
- YAML frontmatter starts with `---`
- YAML frontmatter ends with `---`
- Proper YAML syntax
- Valid semantic version

### Content Validation
- Content exists after frontmatter
- Minimum content length (50 characters)
- Contains Markdown headers
- No obvious formatting issues

## Usage

### Basic Validation
```bash
./validate_skill.sh path/to/SKILL.md
```

### Validate All Skills
```bash
for skill in examples/.claude/skills/*/SKILL.md; do
    echo "Validating $skill..."
    ./validate_skill.sh "$skill"
    echo
done
```

### Integration with CI/CD
```yaml
# .github/workflows/validate-skills.yml
name: skill-md-validator

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Validate SKILL.md files
        run: |
          for skill in examples/.claude/skills/*/SKILL.md; do
            ./examples/.claude/skills/skill-validator/validate_skill.sh "$skill"
          done
```

## Validation Output

### Success Example
```
🔍 Validating code-reviewer/SKILL.md...

✅ YAML frontmatter starts correctly
✅ YAML frontmatter formatted correctly
✅ Found required field: name:
✅ Found required field: description:
✅ Found required field: version:
✅ Content present (2540 characters)
✅ Contains Markdown headers

🎉 Validation complete! SKILL.md looks good.
```

### Failure Example
```
🔍 Validating bad-example/SKILL.md...

❌ Missing YAML frontmatter start (---)

Validation failed. Please fix the issues above.
```

## Best Practices

### Writing SKILL.md Files

1. **Start with YAML frontmatter**
   ```markdown
   ---
   name: "My Skill"
   description: "What this skill does"
   version: "1.0.0"
   ---
   ```

2. **Add meaningful content**
   - Include clear instructions
   - Provide examples
   - Use proper Markdown formatting
   - Add sections with headers

3. **Follow naming conventions**
   - Use kebab-case for skill IDs
   - Use semantic versioning
   - Be descriptive but concise

4. **Include metadata**
   - Author information
   - Relevant tags
   - Dependencies on other skills

## Common Issues

### Missing Required Fields
```
❌ Error: Missing required field: name:

Fix: Add the name field to YAML frontmatter
---
name: skill-md-validator
---
```

### Invalid YAML
```
❌ Error: YAML parsing error

Fix: Ensure proper YAML syntax
- Use spaces for indentation (not tabs)
- Quote strings with special characters
- Properly format lists
```

### Insufficient Content
```
⚠️  Warning: Content seems very short (< 50 chars)

Fix: Add more detailed instructions and examples
```

## Integration Examples

### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

for file in $(git diff --cached --name-only | grep SKILL.md); do
    if ! ./examples/.claude/skills/skill-validator/validate_skill.sh "$file"; then
        echo "❌ Validation failed for $file"
        exit 1
    fi
done
```

### Makefile Target
```makefile
# Makefile

.PHONY: validate-skills
validate-skills:
	@echo "Validating all SKILL.md files..."
	@for skill in examples/.claude/skills/*/SKILL.md; do \
		./examples/.claude/skills/skill-validator/validate_skill.sh "$$skill"; \
	done
```

## Extending the Validator

You can extend this validator to check for:

1. **Custom fields**
   ```bash
   if ! grep -q "author:" "$SKILL_FILE"; then
       echo "⚠️  Warning: Missing author field"
   fi
   ```

2. **Code examples**
   ```bash
   if ! grep -q '```' "$SKILL_FILE"; then
       echo "⚠️  Warning: No code examples found"
   fi
   ```

3. **Tag conventions**
   ```bash
   tags=$(sed -n '/^---$/,/^---$/p' "$SKILL_FILE" | grep "^tags:" | tail -n +1)
   if [ -z "$tags" ]; then
       echo "⚠️  Warning: No tags defined"
   fi
   ```

## Related Tools

- [yamllint](https://yamllint.readthedocs.io/) - YAML linter
- [markdownlint](https://github.com/igorshubovych/markdownlint-cli) - Markdown linter
- [vale](https://vale.sh/) - Prose linter for documentation
