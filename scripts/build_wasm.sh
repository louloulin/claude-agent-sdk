#!/bin/bash
# WebAssembly Build Script
# This script builds the Claude Agent SDK for WebAssembly

set -e

echo "=== Claude Agent SDK - WASM Build Script ==="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found${NC}"
    exit 1
fi

if ! command -v wasm-pack &> /dev/null; then
    echo -e "${YELLOW}Installing wasm-pack...${NC}"
    cargo install wasm-pack
fi

# Add WASM target
echo -e "${YELLOW}Adding WASM target...${NC}"
rustup target add wasm32-unknown-unknown 2>/dev/null || echo "Target already installed"

# Build options
TARGET=${1:-"web"}        # web, bundler, nodejs, mod
RELEASE_FLAG=${2:-"--release"}

echo ""
echo -e "${GREEN}Building for WASM (target: ${TARGET})${NC}"
echo ""

# Clean previous build
echo -e "${YELLOW}Cleaning previous build...${NC}"
rm -rf pkg/

# Build the WASM package
echo -e "${YELLOW}Compiling to WASM...${NC}"
wasm-pack build \
    --target ${TARGET} \
    --out-dir pkg \
    ${RELEASE_FLAG}

echo ""
echo -e "${GREEN}Build complete!${NC}"
echo ""

# Show build results
echo "Build artifacts:"
ls -lh pkg/*.wasm 2>/dev/null || echo "No WASM files found"
ls -lh pkg/*.js 2>/dev/null || echo "No JS files found"

echo ""
echo -e "${GREEN}Next steps:${NC}"
echo ""
echo "1. Test the build:"
echo "   node test_wasm.js"
echo ""
echo "2. Serve locally:"
echo "   python -m http.server 8000"
echo "   # Then open examples/wasm/simple.html"
echo ""
echo "3. Optimize bundle size:"
echo "   wasm-opt pkg/claude_agent_sdk_bg.wasm -O3 -o pkg/claude_agent_sdk_bg_opt.wasm"
echo ""

# Optional: Optimize with wasm-opt if available
if command -v wasm-opt &> /dev/null; then
    echo -e "${YELLOW}Opt WASM optimization available${NC}"
    echo "Run: wasm-opt pkg/claude_agent_sdk_bg.wasm -O3 -o pkg/optimized.wasm"
fi
