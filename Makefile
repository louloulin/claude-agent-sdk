# Claude Agent SDK Rust - Makefile

.PHONY: all build release test clean docs check lint ci fmt help install

# 默认目标
all: build

# 开发构建
build:
	cargo build

# 发布构建（最大优化）
release:
	cargo build --release

# LTO 构建（链接时优化）
lto:
	cargo build --profile release-lto

# 运行测试
test:
	cargo test

# 运行测试（发布模式）
test-release:
	cargo test --release

# 运行基准测试
bench:
	cargo bench

# 清理构建缓存
clean:
	cargo clean

# 构建文档
docs:
	cargo doc --no-deps

# 打开文档（在浏览器中）
docs-open:
	cargo doc --open

# 代码格式化
fmt:
	cargo fmt

# 检查格式
fmt-check:
	cargo fmt -- --check

# 代码检查
check:
	cargo check --all-targets

# Lint 检查
lint:
	cargo clippy --all-targets -- -D warnings

# 完整 CI 流程
ci: fmt-check lint check test
	@echo "✅ CI 流程完成"

# 安装到本地
install: release
	cargo install --path .

# 开发依赖安装
deps:
	cargo fetch

# 更新依赖
update:
	cargo update

# 发布到 crates.io
publish: check test
	cargo publish --dry-run
	@echo "请运行: cargo publish 进行正式发布"

# 显示帮助
help:
	@echo "Claude Agent SDK Rust - Makefile 命令"
	@echo ""
	@echo "构建命令:"
	@echo "  make build      - 开发构建"
	@echo "  make release    - 发布构建"
	@echo "  make lto        - LTO 构建"
	@echo ""
	@echo "测试命令:"
	@echo "  make test       - 运行测试"
	@echo "  make test-rel   - 测试（发布模式）"
	@echo "  make bench      - 基准测试"
	@echo ""
	@echo "代码质量:"
	@echo "  make fmt        - 格式化代码"
	@echo "  make check      - 代码检查"
	@echo "  make lint       - Lint 检查"
	@echo "  make ci         - 完整 CI 流程"
	@echo ""
	@echo "文档命令:"
	@echo "  make docs       - 构建文档"
	@echo "  make docs-open  - 打开文档"
	@echo ""
	@echo "清理命令:"
	@echo "  make clean      - 清理缓存"
	@echo ""
	@echo "发布命令:"
	@echo "  make publish    - 发布到 crates.io"
