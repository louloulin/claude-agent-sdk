# Contributing to Claude Agent SDK Rust

感谢您对 Claude Agent SDK Rust 项目的关注！我们欢迎任何形式的贡献。

## 🚀 快速开始

### 开发环境设置

1. **Fork 并克隆仓库**
   ```bash
   # Fork 后替换 your-username 为你的 GitHub 用户名
   git clone https://github.com/louloulin/claude-agent-sdk.git
   cd claude-agent-sdk
   ```

2. **安装 Rust**
   ```bash
   # 使用 rustup 安装 Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # 确保使用最新稳定版本
   rustup update
   ```

3. **验证环境**
   ```bash
   # 检查 Rust 版本（需要 1.85+）
   rustc --version

   # 检查 Cargo 版本
   cargo --version
   ```

### 构建项目

```bash
# 构建项目
cargo build

# 运行测试
cargo test

# 构建文档
cargo doc --open
```

## 📋 开发流程

### 1. 分支策略

```bash
# 创建功能分支
git checkout -b feature/your-feature-name

# 或者修复分支
git checkout -b fix/your-bug-fix
```

**分支命名规范**：
- `feature/` - 新功能
- `fix/` - Bug 修复
- `docs/` - 文档更新
- `refactor/` - 代码重构
- `test/` - 测试相关
- `perf/` - 性能优化

### 2. 代码规范

#### Rust 代码风格

```bash
# 格式化代码
cargo fmt

# 检查格式（CI 中使用）
cargo fmt -- --check
```

#### 代码检查

```bash
# 运行 Clippy
cargo clippy --all-targets --all-features

# 修复 Clippy 警告
cargo clippy --all-targets --all-features --fix
```

**代码质量要求**：
- ✅ 零 Clippy 警告
- ✅ 零编译警告
- ✅ 通过所有测试
- ✅ 公共 API 必须有文档注释

### 3. 测试要求

#### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example() {
        // 测试代码
        assert_eq!(2 + 2, 4);
    }
}
```

#### 测试覆盖

```bash
# 运行所有测试
cargo test

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_name

# 运行文档测试
cargo test --doc
```

**测试要求**：
- ✅ 新功能必须有单元测试
- ✅ 测试覆盖率 > 80%
- ✅ 关键路径必须有集成测试
- ✅ 异步代码使用 `#[tokio::test]`

### 4. 文档要求

#### 公共 API 文档

```rust
//! 模块级文档
//!
//! 详细说明...

/// 函数/结构体文档
///
/// # Examples
///
/// ```
/// use claude_agent_sdk::function_name;
///
/// let result = function_name();
/// assert!(result.is_ok());
/// ```
///
/// # Errors
///
/// - `Error::Variant` - 当...
pub fn function_name() -> Result<()> {
    // 实现
}
```

#### 文档生成

```bash
# 生成并打开文档
cargo doc --open

# 检查文档链接
cargo doc --document-private-items
```

**文档要求**：
- ✅ 所有公共 API 必须有文档
- ✅ 包含使用示例
- ✅ 说明错误情况
- ✅ 文档通过 `cargo doc` 测试

## 🐛 Bug 报告

### 报告 Bug 前请确认

1. ✅ 搜索现有 Issues
2. ✅ 确认是 Bug 而非问题
3. ✅ 准备最小复现示例

### Bug 报告模板

```markdown
### Bug 描述
简要描述 Bug...

### 复现步骤
1. 步骤一
2. 步骤二
3. 步骤三

### 期望行为
应该发生什么...

### 实际行为
实际发生了什么...

### 环境信息
- OS:
- Rust 版本:
- SDK 版本:
- Claude Code CLI 版本:

### 额外信息
日志、截图等...
```

## ✨ 功能请求

### 功能请求前请考虑

1. ✅ 这是否符合项目目标？
2. ✅ 是否已有类似功能？
3. ✅ 是否值得添加？

### 功能请求模板

```markdown
### 功能描述
简要描述功能...

### 使用场景
描述具体使用场景...

### 期望的 API
```rust
// 期望的 API 设计
```

### 替代方案
描述其他可能的实现方式...

### 附加信息
参考资料、设计思路等...
```

## 📝 Pull Request 流程

### 1. 提交 PR 前

- [ ] 代码通过 `cargo fmt` 格式化
- [ ] 代码通过 `cargo clippy` 检查
- [ ] 所有测试通过 `cargo test`
- [ ] 文档完整且通过 `cargo doc`
- [ ] 更新相关文档和 CHANGELOG

### 2. 提交信息规范

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Type 类型**：
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具相关

**示例**：
```
feat(skills): add tag query system

Implement TagFilter and TagQueryBuilder for flexible
tag-based queries on skill packages.

Closes #123
```

### 3. PR 描述模板

```markdown
## 变更描述
简要描述变更内容...

## 变更类型
- [ ] Bug 修复
- [ ] 新功能
- [ ] 破坏性变更
- [ ] 文档更新

## 测试
描述如何测试...

## 截图/日志
如果有...

## 相关 Issue
Closes #123
```

### 4. Code Review 准则

- ✅ 保持友好和建设性
- ✅ 专注于代码质量而非风格
- ✅ 解释问题而不仅指出问题
- ✅ 认可好的代码实践

## 🎯 开发重点

我们特别欢迎以下方面的贡献：

### 高优先级

1. **集成测试**
   - 端到端测试场景
   - 与 Claude Code CLI 集成测试

2. **性能优化**
   - 基准测试
   - 性能分析和优化

3. **文档改进**
   - 教程和最佳实践
   - 多语言翻译

4. **示例程序**
   - 实用场景示例
   - 性能对比示例

### 中优先级

1. **WASM 支持**
   - WASM 编译配置
   - 浏览器环境测试

2. **生态集成**
   - Rig 框架适配
   - 其他 Rust AI 框架

### 低优先级

1. **多 Agent 编排**
2. **社区工具**
3. **UI/CLI 工具**

## 🧪 测试策略

### 测试层级

1. **单元测试** - 测试单个函数/方法
2. **集成测试** - 测试模块间交互
3. **端到端测试** - 测试完整工作流

### 测试最佳实践

```rust
// ✅ 好的测试
#[tokio::test]
async fn test_query_with_valid_input() {
    let result = query("Hello", None).await;
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

// ❌ 不好的测试（过于简单）
#[tokio::test]
async fn test_query() {
    assert!(true);
}
```

## 📚 资源链接

### 官方文档

- [Claude Code CLI Docs](https://docs.claude.com/claude-code)
- [Anthropic API Docs](https://docs.anthropic.com/)
- [MCP Specification](https://modelcontextprotocol.io/)
- [Rust Book](https://doc.rust-lang.org/book/)

### 项目文档

- [README](README.md)
- [中文 README](README_zh-CN.md)
- [Examples](examples/)
- [API Documentation](https://docs.rs/claude-agent-sdk-rs)

### 相关项目

- [Python SDK](https://github.com/anthropics/claude-agent-sdk-python)
- [TypeScript SDK](https://github.com/anthropics/claude-agent-sdk-ts)
- [Agent Skills](https://github.com/anthropics/skills)

## 🤝 社区准则

### 行为准则

1. **尊重他人** - 保持友好和专业
2. **建设性反馈** - 专注于代码而非人
3. **包容性** - 欢迎不同背景的贡献者
4. **协作优先** - 寻求共识而非冲突

### 沟通渠道

- **GitHub Issues** - Bug 报告和功能请求
- **GitHub Discussions** - 一般讨论和问题
- **Pull Requests** - 代码审查和技术讨论

## 📄 许可证

贡献的代码将采用 [MIT License](LICENSE.md) 发布。

## ❓ 获取帮助

### 常见问题

**Q: 我该如何开始？**
A: 从修复简单的 Bug 或添加文档开始！

**Q: 我不熟悉 Rust，可以贡献吗？**
A: 当然！文档、测试、示例等都需要帮助。

**Q: 我的 PR 会被接受吗？**
A: 只要符合项目目标和代码规范，我们都欢迎！

### 联系方式

- 创建 GitHub Issue
- 在 Discussions 中提问
- 查看 [FAQ](FAQ.md)（如果有）

---

**再次感谢您的贡献！** 🎉

让我们一起打造最好的 Rust AI Agent SDK！
