# build.rs - 自动检查 Claude Code CLI

## 概述

`build.rs` 是一个 Cargo 构建脚本，在编译项目时自动检查 Claude Code CLI 是否已安装。如果未安装或版本过低，会显示友好的安装提示。

## 功能特性

### ✅ 自动检查
- 在每次 `cargo build` 时自动运行
- 检测 Claude Code CLI 是否安装
- 验证版本是否满足要求（>= 2.0.0）

### 🎯 友好提示
- ✅ 已安装：显示版本信息
- ⚠️ 版本过低：显示更新命令
- ❌ 未安装：显示详细安装指南

### 🔄 智能跳过
- `cargo doc` 时跳过检查
- 设置 `SKIP_CLAUDE_CHECK=1` 环境变量可跳过

## 使用示例

### 场景1: Claude Code CLI 已安装

```bash
$ cargo build
warning: claude-agent-sdk-rs@0.6.0: ✅ Claude Code CLI 已安装 (版本: 2.0.76)
warning: claude-agent-sdk-rs@0.6.0:    SDK 可以使用完整的 AI 交互功能
   Compiling claude-agent-sdk-rs v0.6.0
    Finished dev profile [unoptimized + debuginfo] target(s)
```

### 场景2: Claude Code CLI 未安装

```bash
$ cargo build
warning: claude-agent-sdk-rs@0.6.0: ╔════════════════════════════════════════════════════════════╗
warning: claude-agent-sdk-rs@0.6.0: ║  ℹ️  Claude Code CLI 未找到                                      ║
warning: claude-agent-sdk-rs@0.6.0: ╚════════════════════════════════════════════════════════════╝
warning: claude-agent-sdk-rs@0.6.0: 
warning: claude-agent-sdk-rs@0.6.0: Claude Code CLI 是使用 SDK 的 AI 交互功能所必需的。
warning: claude-agent-sdk-rs@0.6.0: 
warning: claude-agent-sdk-rs@0.6.0: 📦 安装方法:
warning: claude-agent-sdk-rs@0.6.0:    npm install -g @anthropic-ai/claude-code
warning: claude-agent-sdk-rs@0.6.0: 
warning: claude-agent-sdk-rs@0.6.0:    或者使用自动安装脚本:
warning: claude-agent-sdk-rs@0.6.0:    ./scripts/check_and_install_claude.sh
```

### 场景3: 跳过检查

```bash
# 方法1: 设置环境变量
$ SKIP_CLAUDE_CHECK=1 cargo build

# 方法2: 生成文档（自动跳过）
$ cargo doc
```

## 配置选项

### 环境变量

| 变量 | 说明 | 示例 |
|------|------|------|
| `SKIP_CLAUDE_CHECK` | 跳过 Claude Code CLI 检查 | `SKIP_CLAUDE_CHECK=1 cargo build` |

### 版本要求

```rust
const MIN_CLAUDE_VERSION: &str = "2.0.0";
```

可以在 `build.rs` 中修改此常量来调整最低版本要求。

## 工作原理

### 检查流程

```
1. 检查是否是 cargo doc → 是则跳过
2. 检查 SKIP_CLAUDE_CHECK 环境变量 → 设置则跳过
3. 查找 claude 可执行文件
   ├── Unix: which claude
   ├── Windows: where claude
   └── 检查常见安装路径
4. 如果找到:
   ├── 获取版本号
   ├── 比较版本
   └── 显示相应消息
5. 如果未找到:
   └── 显示安装指南
```

### 版本比较

使用简单的语义化版本比较：
- 比较主版本号（major）
- 比较次版本号（minor）
- 忽略补丁版本号（patch）

示例：
```
2.0.76 >= 2.0.0  ✅
1.9.0 >= 2.0.0   ❌
2.1.0 >= 2.0.0   ✅
```

## 集成到 CI/CD

### GitHub Actions

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Install Claude Code CLI
        run: npm install -g @anthropic-ai/claude-code
      
      - name: Build
        run: cargo build --verbose
```

### GitLab CI

```yaml
build:
  script:
    - npm install -g @anthropic-ai/claude-code
    - cargo build --verbose
```

### 跳过检查的 CI

```yaml
# 在 CI 环境中跳过检查（如果不需要实际 AI 交互）
env:
  SKIP_CLAUDE_CHECK: 1

steps:
  - cargo build
```

## 故障排除

### 问题1: 警告信息未显示

**可能原因**: cargo 的 warning 级别被过滤

**解决方案**:
```bash
# 显示所有输出
cargo build --message-format=human

# 或使用 verbose 模式
cargo build -vv
```

### 问题2: 找不到 claude 但已安装

**可能原因**: claude 安装在非标准路径

**解决方案**:
```bash
# 确保 claude 在 PATH 中
export PATH="$PATH:$HOME/.npm-global/bin"

# 或者创建符号链接
ln -s $(which claude) /usr/local/bin/claude
```

### 问题3: 版本检测失败

**可能原因**: `claude --version` 输出格式异常

**解决方案**: 手动检查版本
```bash
claude --version
```

## 最佳实践

### 开发环境
```bash
# 1. 克隆项目
git clone <repo-url>
cd claude-agent-sdk-rs

# 2. 构建项目（会自动检查）
cargo build

# 3. 如果提示未安装，运行自动安装脚本
./scripts/check_and_install_claude.sh

# 4. 重新构建
cargo build
```

### CI/CD 环境
```bash
# 选项1: 安装 Claude Code CLI
npm install -g @anthropic-ai/claude-code
cargo build

# 选项2: 跳过检查（仅编译库）
SKIP_CLAUDE_CHECK=1 cargo build
```

### 仅文档生成
```bash
# 自动跳过检查
cargo doc
```

## 相关文件

- `build.rs` - 主构建脚本
- `scripts/check_and_install_claude.sh` - 自动安装脚本
- `scripts/build_with_check.sh` - 集成检查的构建脚本

## 贡献

如果你想改进 `build.rs`，请注意：

1. 保持输出信息简洁友好
2. 遵循 Cargo 的构建脚本最佳实践
3. 确保跨平台兼容性（Linux, macOS, Windows）
4. 添加适当的测试和文档

## 许可证

MIT License - 与项目主许可证相同
