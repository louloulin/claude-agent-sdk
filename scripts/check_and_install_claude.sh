#!/bin/bash
# 自动检查并安装 Claude Code CLI
# 用于构建系统和 CI/CD 流程

set -e

MIN_CLAUDE_VERSION="2.0.0"
CLAUDE_EXECUTABLE="claude"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 检查 Claude Code CLI..."

# 检查是否已安装
if command -v "$CLAUDE_EXECUTABLE" &> /dev/null; then
    CURRENT_VERSION=$(claude --version 2>&1 | grep -oP '\d+\.\d+\.\d+' || echo "0.0.0")

    echo "✅ Claude Code CLI 已安装"
    echo "   当前版本: $CURRENT_VERSION"

    # 检查版本是否满足最低要求
    if [ "$(printf '%s\n' "$MIN_CLAUDE_VERSION" "$CURRENT_VERSION" | sort -V | head -n1)" = "$MIN_CLAUDE_VERSION" ]; then
        echo "✅ 版本满足要求 (>= $MIN_CLAUDE_VERSION)"
        exit 0
    else
        echo -e "${YELLOW}⚠️  版本过低，需要 >= $MIN_CLAUDE_VERSION${NC}"
        echo "   当前: $CURRENT_VERSION"
        read -p "是否要更新 Claude Code CLI? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo "🔄 正在更新..."
            npm update -g @anthropic-ai/claude-code
            exit 0
        else
            echo -e "${RED}❌ 版本不满足要求，无法继续${NC}"
            exit 1
        fi
    fi
else
    echo -e "${YELLOW}⚠️  Claude Code CLI 未安装${NC}"
    echo ""
    echo "📦 Claude Code CLI 是运行此项目所必需的"
    echo ""
    echo "安装选项:"
    echo "  1. 全局安装 (推荐)"
    echo "  2. 跳过安装"
    echo ""
    read -p "选择安装选项 (1/2): " choice

    case $choice in
        1)
            echo ""
            echo "🔄 正在安装 Claude Code CLI..."
            echo ""

            # 检查 npm 是否可用
            if command -v npm &> /dev/null; then
                echo "使用 npm 安装..."
                npm install -g @anthropic-ai/claude-code

                if [ $? -eq 0 ]; then
                    echo ""
                    echo -e "${GREEN}✅ 安装成功！${NC}"
                    echo ""
                    claude --version
                    echo ""
                    echo "🎉 现在可以使用 Claude Code CLI 了！"
                    exit 0
                else
                    echo -e "${RED}❌ 安装失败${NC}"
                    echo "请手动安装: npm install -g @anthropic-ai/claude-code"
                    exit 1
                fi
            else
                echo -e "${RED}❌ npm 未找到${NC}"
                echo "请先安装 Node.js 和 npm: https://nodejs.org/"
                echo ""
                echo "或者使用其他安装方法:"
                echo "  https://docs.claude.com/claude-code/installation"
                exit 1
            fi
            ;;
        2)
            echo -e "${YELLOW}⏭️  跳过安装${NC}"
            echo "注意: 某些示例和测试需要 Claude Code CLI"
            exit 0
            ;;
        *)
            echo -e "${RED}❌ 无效选择${NC}"
            exit 1
            ;;
    esac
fi
