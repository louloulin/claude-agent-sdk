#!/bin/bash
# Claude Agent SDK Rust - 优化构建脚本

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

echo_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查系统资源
check_resources() {
    echo_info "检查系统资源..."
    CORES=$(sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo "4")
    echo_info "CPU 核心数: $CORES"
}

# 清理构建缓存
clean_build() {
    echo_info "清理构建缓存..."
    cargo clean
    echo_info "清理完成"
}

# 开发构建
dev_build() {
    echo_info "开始开发构建..."
    cargo build
    echo_info "开发构建完成"
}

# 发布构建
release_build() {
    echo_info "开始发布构建..."
    cargo build --release
    echo_info "发布构建完成"
}

# LTO 构建
lto_build() {
    echo_info "开始 LTO 构建..."
    cargo build --profile release-lto
    echo_info "LTO 构建完成"
}

# 文档构建
build_docs() {
    echo_info "构建文档..."
    cargo doc --no-deps
    echo_info "文档构建完成"
}

# 运行测试
run_tests() {
    echo_info "运行测试..."
    cargo test
    echo_info "测试完成"
}

# 代码检查
run_checks() {
    echo_info "运行代码检查..."
    cargo fmt -- --check
    cargo clippy --all-targets
    cargo test --no-run
    echo_info "代码检查完成"
}

# CI 流程
ci_build() {
    echo_info "开始完整 CI 构建..."
    cargo fmt -- --check
    cargo clippy --all-targets -- -D warnings
    cargo check --all-targets
    cargo test
    echo_info "CI 构建完成"
}

# 显示帮助
show_help() {
    cat << EOF
用法: ./scripts/build.sh [命令]

命令:
  clean      清理构建缓存
  dev        开发构建
  release    发布构建
  lto        LTO 构建
  docs       构建文档
  test       运行测试
  check      代码检查
  ci         完整 CI 流程
  help       显示帮助

示例:
  ./scripts/build.sh release
  ./scripts/build.sh ci
EOF
}

# 主函数
main() {
    check_resources
    
    case "${1:-help}" in
        clean)    clean_build ;;
        dev)      dev_build ;;
        release)  release_build ;;
        lto)      lto_build ;;
        docs)     build_docs ;;
        test)     run_tests ;;
        check)    run_checks ;;
        ci)       ci_build ;;
        help|--help|-h)  show_help ;;
        *)
            echo_error "未知命令: $1"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
