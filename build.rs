//! Claude Agent SDK Rust - Build Script
//!
//! 这个 build.rs 在编译时自动检查 Claude Code CLI 是否安装
//! 如果未安装或版本过低，会显示友好的安装提示

use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Claude Code CLI 的最低版本要求
const MIN_CLAUDE_VERSION: &str = "2.0.0";

fn main() {
    // 在 cargo doc 时跳过检查（避免生成文档时的警告）
    if is_cargo_doc() {
        return;
    }

    // 检查 Claude Code CLI
    check_claude_cli();

    // 重新构建条件：当 Claude Code 状态变化时重新编译
    println!("cargo:rerun-if-changed=build.rs");
}

/// 检查是否是 cargo doc 命令
fn is_cargo_doc() -> bool {
    env::var("CARGO_DOC_RUNNER").is_ok()
        || env::var("RUSTDOCFLAGS").is_ok()
        || std::env::args().any(|arg| arg.contains("doc"))
}

/// 检查 Claude Code CLI 是否安装
fn check_claude_cli() {
    // 检查环境变量，允许跳过检查
    if env::var("SKIP_CLAUDE_CHECK").is_ok() {
        return;
    }

    // 尝试找到 claude 命令
    let claude_path = find_claude_executable();

    match claude_path {
        Some(path) => {
            // 找到了 Claude CLI，检查版本
            let version = get_claude_version(&path);

            match version {
                Some(version_str) => {
                    if version_meets_requirement(&version_str) {
                        print_success(&version_str);
                    } else {
                        print_version_warning(&version_str);
                    }
                }
                None => {
                    // 无法获取版本，但文件存在
                    print_found_but_unknown_version();
                }
            }
        }
        None => {
            // 未找到 Claude CLI
            print_install_guide();
        }
    }
}

/// 查找 Claude 可执行文件
fn find_claude_executable() -> Option<PathBuf> {
    // 方法1: 使用 which/where 命令
    if cfg!(unix) {
        let output = Command::new("which")
            .arg("claude")
            .output()
            .ok()?;

        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Some(PathBuf::from(path));
        }
    } else if cfg!(windows) {
        let output = Command::new("where")
            .arg("claude")
            .output()
            .ok()?;

        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()?
                .to_string();
            return Some(PathBuf::from(path));
        }
    }

    // 方法2: 检查常见路径
    let home = env::var("HOME").ok()
        .or_else(|| env::var("USERPROFILE").ok());

    if let Some(home_dir) = home {
        let common_paths = vec![
            // npm 全局安装路径
            PathBuf::from(home_dir.clone()).join(".npm-global/bin/claude"),
            PathBuf::from(&home_dir).join("AppData/Roaming/npm/claude"),
        ];

        for path in common_paths {
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

/// 获取 Claude CLI 版本
fn get_claude_version(path: &PathBuf) -> Option<String> {
    let output = Command::new(path)
        .arg("--version")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    // 解析版本号，格式: "2.0.76 (Claude Code)"
    version_str
        .split_whitespace()
        .nth(0)
        .map(|s| s.to_string())
}

/// 检查版本是否满足要求
fn version_meets_requirement(version: &str) -> bool {
    // 简单的版本比较
    let parts: Vec<u32> = version
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if parts.len() < 2 {
        return false;
    }

    let min_parts: Vec<u32> = MIN_CLAUDE_VERSION
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    // 比较主版本和次版本
    for i in 0..2 {
        let current = parts.get(i).unwrap_or(&0);
        let minimum = min_parts.get(i).unwrap_or(&0);
        if current < minimum {
            return false;
        }
    }

    true
}

/// 打印成功消息
fn print_success(version: &str) {
    println!("cargo:warning=✅ Claude Code CLI 已安装 (版本: {})", version);
    println!("cargo:warning=   SDK 可以使用完整的 AI 交互功能");
}

/// 打印找到但无法获取版本的消息
fn print_found_but_unknown_version() {
    println!("cargo:warning=⚠️  找到 Claude Code CLI，但无法确定版本");
    println!("cargo:warning=   请确保版本 >= {} (可选)", MIN_CLAUDE_VERSION);
}

/// 打印版本警告
fn print_version_warning(current_version: &str) {
    println!("cargo:warning=⚠️  Claude Code CLI 版本过低");
    println!("cargo:warning=   当前版本: {}", current_version);
    println!("cargo:warning=   推荐版本: >= {}", MIN_CLAUDE_VERSION);
    println!("cargo:warning=   更新命令: npm update -g @anthropic-ai/claude-code");
}

/// 打印安装指南
fn print_install_guide() {
    println!("cargo:warning=╔════════════════════════════════════════════════════════════╗");
    println!("cargo:warning=║  ℹ️  Claude Code CLI 未找到                                      ║");
    println!("cargo:warning=╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("cargo:warning=Claude Code CLI 是使用 SDK 的 AI 交互功能所必需的。");
    println!();
    println!("cargo:warning=📦 安装方法:");
    println!("cargo:warning=   npm install -g @anthropic-ai/claude-code");
    println!();
    println!("cargo:warning=   或者使用自动安装脚本:");
    println!("cargo:warning=   ./scripts/check_and_install_claude.sh");
    println!();
    println!("cargo:warning=📚 更多信息:");
    println!("cargo:warning=   https://docs.claude.com/claude-code/installation");
    println!();
    println!("cargo:warning=⏭️  如果只想编译库而不运行示例，可以设置:");
    println!("cargo:warning=   export SKIP_CLAUDE_CHECK=1");
    println!();
}
