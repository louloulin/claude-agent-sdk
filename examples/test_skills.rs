use std::path::PathBuf;
use claude_agent_sdk_rs::skills::skill_md::SkillsDirScanner;

fn main() -> anyhow::Result<()> {
    let skills_dir = PathBuf::from(".claude/skills");

    println!("正在扫描Skills目录: {:?}\n", skills_dir);

    let scanner = SkillsDirScanner::new(&skills_dir);
    let skills = scanner.scan()?;

    println!("发现 {} 个Skills:\n", skills.len());

    for skill_md in skills {
        let metadata = &skill_md.metadata;
        println!("📦 {}", metadata.name);
        println!("   版本: {}", metadata.version);
        println!("   作者: {:?}", metadata.author);
        println!("   描述: {}", metadata.description);
        println!("   标签: {:?}", metadata.tags);
        println!("   依赖: {:?}", metadata.dependencies);
        println!();
    }

    Ok(())
}
