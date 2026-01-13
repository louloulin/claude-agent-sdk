// 真实验证 SKILL.md 功能
//
// 这个程序将：
// 1. 扫描所有 SKILL.md 文件
// 2. 解析每个文件的 YAML frontmatter
// 3. 验证元数据完整性
// 4. 转换为 SkillPackage
// 5. 打印详细统计信息

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
struct SkillMetadata {
    name: String,
    description: String,
    version: String,
    author: Option<String>,
    tags: Vec<String>,
    dependencies: Vec<String>,
}

#[derive(Debug)]
struct SkillFile {
    metadata: SkillMetadata,
    content: String,
    skill_dir: PathBuf,
}

#[derive(Debug)]
struct ValidationError {
    skill_path: String,
    error: String,
}

fn parse_frontmatter(content: &str) -> Result<(HashMap<String, String>, String), String> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() < 2 {
        return Err("文件内容太少".to_string());
    }

    if !lines[0].trim_start_matches('#').contains("---") {
        return Err("缺少 frontmatter 开始标记".to_string());
    }

    let mut frontmatter_end = None;
    let mut yaml_content = Vec::new();

    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim_start_matches('#').contains("---") {
            frontmatter_end = Some(i);
            break;
        }
        yaml_content.push(*line);
    }

    let end = frontmatter_end.ok_or("缺少 frontmatter 结束标记")?;

    // 解析 YAML 字段
    let mut metadata = HashMap::new();
    let mut current_field: Option<String> = None;
    let mut current_value: Vec<String> = Vec::new();

    for line in &yaml_content {
        let trimmed = line.trim();

        // 跳过空行和注释
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // 检查是否是字段定义 (key: value)
        if trimmed.contains(':') && !trimmed.starts_with('-') {
            // 保存前一个字段
            if let Some(field) = current_field.take() {
                let value = current_value.join("\n");
                metadata.insert(field, value);
                current_value.clear();
            }

            // 解析新字段
            let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                if !value.is_empty() {
                    metadata.insert(key, value);
                } else {
                    current_field = Some(key);
                }
            }
        } else if trimmed.starts_with('-') {
            // 列表项
            let item = trimmed[1..].trim().to_string();
            current_value.push(item);
        } else if let Some(ref _field) = current_field {
            // 多行值
            current_value.push(trimmed.to_string());
        }
    }

    // 保存最后一个字段
    if let Some(field) = current_field {
        metadata.insert(field, current_value.join("\n"));
    }

    // 提取 markdown 内容
    let markdown_content = lines[end + 1..].join("\n");

    Ok((metadata, markdown_content))
}

fn parse_yaml_string(value: &str) -> Vec<String> {
    value.lines()
        .map(|line| line.trim().trim_start_matches('-').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn load_skill(skill_dir: &Path) -> Result<SkillFile, ValidationError> {
    let skill_md = skill_dir.join("SKILL.md");

    if !skill_md.exists() {
        return Err(ValidationError {
            skill_path: skill_dir.display().to_string(),
            error: "SKILL.md 文件不存在".to_string(),
        });
    }

    let content = fs::read_to_string(&skill_md).map_err(|e| ValidationError {
        skill_path: skill_dir.display().to_string(),
        error: format!("读取文件失败: {}", e),
    })?;

    let (metadata_map, markdown_content) = parse_frontmatter(&content).map_err(|e| ValidationError {
        skill_path: skill_dir.display().to_string(),
        error: format!("解析 frontmatter 失败: {}", e),
    })?;

    // 提取必需字段
    let name = metadata_map.get("name")
        .ok_or_else(|| ValidationError {
            skill_path: skill_dir.display().to_string(),
            error: "缺少 name 字段".to_string(),
        })?
        .clone();

    let description = metadata_map.get("description")
        .ok_or_else(|| ValidationError {
            skill_path: skill_dir.display().to_string(),
            error: "缺少 description 字段".to_string(),
        })?
        .clone();

    let version = metadata_map.get("version")
        .ok_or_else(|| ValidationError {
            skill_path: skill_dir.display().to_string(),
            error: "缺少 version 字段".to_string(),
        })?
        .clone();

    let author = metadata_map.get("author").cloned();
    let tags = metadata_map.get("tags")
        .map(|s| parse_yaml_string(s))
        .unwrap_or_default();
    let dependencies = metadata_map.get("dependencies")
        .map(|s| parse_yaml_string(s))
        .unwrap_or_default();

    Ok(SkillFile {
        metadata: SkillMetadata {
            name,
            description,
            version,
            author,
            tags,
            dependencies,
        },
        content: markdown_content,
        skill_dir: skill_dir.to_path_buf(),
    })
}

fn scan_skills_dir(skills_dir: &Path) -> (Vec<SkillFile>, Vec<ValidationError>) {
    let mut skills = Vec::new();
    let mut errors = Vec::new();

    if !skills_dir.exists() {
        errors.push(ValidationError {
            skill_path: skills_dir.display().to_string(),
            error: "Skills 目录不存在".to_string(),
        });
        return (skills, errors);
    }

    let entries = match fs::read_dir(skills_dir) {
        Ok(entries) => entries,
        Err(e) => {
            errors.push(ValidationError {
                skill_path: skills_dir.display().to_string(),
                error: format!("读取目录失败: {}", e),
            });
            return (skills, errors);
        }
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        if path.is_dir() {
            match load_skill(&path) {
                Ok(skill) => skills.push(skill),
                Err(e) => errors.push(e),
            }
        }
    }

    (skills, errors)
}

fn print_statistics(skills: &[SkillFile], errors: &[ValidationError]) {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║           🎯 SKILL.md 功能验证报告                        ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    println!("\n📊 总体统计:");
    println!("   ✅ 成功加载: {} 个 SKILL.md 文件", skills.len());
    println!("   ❌ 加载失败: {} 个文件", errors.len());
    println!("   📁 总计扫描: {} 个技能", skills.len() + errors.len());

    if skills.is_empty() && errors.is_empty() {
        println!("\n⚠️  警告: 未找到任何 SKILL.md 文件");
        return;
    }

    // 成功加载的技能详情
    if !skills.is_empty() {
        println!("\n✅ 成功加载的技能:");

        let mut total_lines = 0;
        let mut tags_count: HashMap<&str, usize> = HashMap::new();
        let mut langs: HashMap<&str, usize> = HashMap::new();

        for (i, skill) in skills.iter().enumerate() {
            let line_count = skill.content.lines().count();
            total_lines += line_count;

            println!("\n   {}. {}", i + 1, skill.metadata.name);
            println!("      📂 路径: {}", skill.skill_dir.display());
            println!("      📝 描述: {}", skill.metadata.description);
            println!("      🏷️  版本: {}", skill.metadata.version);
            if let Some(author) = &skill.metadata.author {
                println!("      👤 作者: {}", author);
            }
            println!("      📄 内容行数: {} 行", line_count);

            if !skill.metadata.tags.is_empty() {
                println!("      🏷️  标签: {}", skill.metadata.tags.join(", "));
                for tag in &skill.metadata.tags {
                    *tags_count.entry(tag.as_str()).or_insert(0) += 1;
                }
            }

            if !skill.metadata.dependencies.is_empty() {
                println!("      🔗 依赖: {}", skill.metadata.dependencies.join(", "));
            }

            // 统计语言（基于内容特征）
            let content_lower = skill.content.to_lowercase();
            if content_lower.contains("rust") || content_lower.contains("fn ") || content_lower.contains("let mut") {
                *langs.entry("Rust").or_insert(0) += 1;
            } else if content_lower.contains("python") || content_lower.contains("def ") || content_lower.contains("import ") {
                *langs.entry("Python").or_insert(0) += 1;
            } else if content_lower.contains("javascript") || content_lower.contains("typescript") || content_lower.contains("const ") {
                *langs.entry("JavaScript/TypeScript").or_insert(0) += 1;
            } else if content_lower.contains("swift") || content_lower.contains("@main") {
                *langs.entry("Swift").or_insert(0) += 1;
            } else if content_lower.contains("kotlin") || content_lower.contains("fun ") {
                *langs.entry("Kotlin").or_insert(0) += 1;
            } else if content_lower.contains("go") || content_lower.contains("func ") {
                *langs.entry("Go").or_insert(0) += 1;
            } else if content_lower.contains("中文") || content_lower.contains("专家") {
                *langs.entry("中文").or_insert(0) += 1;
            } else if content_lower.contains("sql") || content_lower.contains("select ") {
                *langs.entry("SQL").or_insert(0) += 1;
            } else {
                *langs.entry("其他").or_insert(0) += 1;
            }
        }

        println!("\n📈 内容统计:");
        println!("   📝 总内容行数: {} 行", total_lines);
        println!("   📊 平均行数: {} 行/技能", total_lines / skills.len());

        if !langs.is_empty() {
            println!("\n🌐 编程语言分布:");
            let mut lang_vec: Vec<_> = langs.iter().collect();
            lang_vec.sort_by(|a, b| b.1.cmp(a.1));
            for (lang, count) in lang_vec {
                println!("      - {}: {} 个技能", lang, count);
            }
        }

        if !tags_count.is_empty() {
            println!("\n🏷️  热门标签:");
            let mut tag_vec: Vec<_> = tags_count.iter().collect();
            tag_vec.sort_by(|a, b| b.1.cmp(a.1));
            tag_vec.truncate(10);
            for (tag, count) in tag_vec {
                println!("      - {}: {} 个技能", tag, count);
            }
        }
    }

    // 加载失败的文件
    if !errors.is_empty() {
        println!("\n❌ 加载失败的文件:");
        for (i, error) in errors.iter().enumerate() {
            println!("\n   {}. {}", i + 1, error.skill_path);
            println!("      ⚠️  错误: {}", error.error);
        }
    }

    // 版本分布
    println!("\n📊 版本分布:");
    let mut versions: HashMap<&str, usize> = HashMap::new();
    for skill in skills {
        *versions.entry(skill.metadata.version.as_str()).or_insert(0) += 1;
    }

    if !versions.is_empty() {
        let mut version_vec: Vec<_> = versions.iter().collect();
        version_vec.sort_by(|a, b| b.1.cmp(a.1));
        for (version, count) in version_vec {
            println!("   v{}: {} 个技能", version, count);
        }
    }

    println!("\n✅ 验证完成!");
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║              SKILL.md 功能验证完成                         ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
}

fn main() {
    println!("🔍 开始验证 SKILL.md 功能...\n");

    // 获取项目根目录
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let skills_dir = manifest_dir.join("examples/.claude/skills");

    println!("📁 扫描目录: {}", skills_dir.display());

    // 扫描所有技能
    let (skills, errors) = scan_skills_dir(&skills_dir);

    // 打印统计信息
    print_statistics(&skills, &errors);

    // 返回适当的退出码
    if !errors.is_empty() {
        println!("⚠️  发现 {} 个错误", errors.len());
        std::process::exit(1);
    } else if skills.is_empty() {
        println!("⚠️  未找到任何 SKILL.md 文件");
        std::process::exit(1);
    } else {
        println!("✅ 所有 SKILL.md 文件验证成功!");
        std::process::exit(0);
    }
}
