#!/bin/bash
# SKILL.md Validation Script
# Validates SKILL.md files for proper format and completeness

set -e

SKILL_FILE="$1"

if [ -z "$SKILL_FILE" ]; then
    echo "❌ Usage: $0 <path-to-SKILL.md>"
    exit 1
fi

if [ ! -f "$SKILL_FILE" ]; then
    echo "❌ File not found: $SKILL_FILE"
    exit 1
fi

echo "🔍 Validating $SKILL_FILE..."
echo

# Check if file starts with ---
if ! head -n 1 "$SKILL_FILE" | grep -q "^---"; then
    echo "❌ Missing YAML frontmatter start (---)"
    exit 1
fi
echo "✅ YAML frontmatter starts correctly"

# Count YAML delimiters
delimiter_count=$(grep -c "^---" "$SKILL_FILE" || true)
if [ "$delimiter_count" -lt 2 ]; then
    echo "❌ Missing YAML frontmatter end (---)"
    exit 1
fi
echo "✅ YAML frontmatter formatted correctly"

# Check for required fields in YAML frontmatter
required_fields=("name:" "description:" "version:")
for field in "${required_fields[@]}"; do
    if ! sed -n '/^---$/,/^---$/p' "$SKILL_FILE" | grep -q "$field"; then
        echo "❌ Missing required field: $field"
        exit 1
    fi
    echo "✅ Found required field: $field"
done

# Check that content exists after frontmatter
content=$(sed -n '1,/^---$/d; /^---$/{p; :a; n; p; ba; }' "$SKILL_FILE")
if [ -z "$content" ]; then
    echo "⚠️  Warning: No content found after YAML frontmatter"
else
    content_length=$(echo "$content" | wc -c)
    if [ "$content_length" -lt 50 ]; then
        echo "⚠️  Warning: Content seems very short (< 50 chars)"
    else
        echo "✅ Content present (${content_length} characters)"
    fi
fi

# Check for common Markdown issues
if echo "$content" | grep -q "^#"; then
    echo "✅ Contains Markdown headers"
else
    echo "⚠️  Warning: No Markdown headers found"
fi

echo
echo "🎉 Validation complete! SKILL.md looks good."
