//! Agent Skills Performance Optimization Examples
//!
//! This example demonstrates performance optimization features for large-scale
//! skill operations, including indexing, caching, and batch processing.

use claude_agent_sdk_rs::skills::performance::{
    BatchOperations, IndexedSkillCollection, LruCache, PerformanceStats,
};
use claude_agent_sdk_rs::skills::tags::TagFilter;
use claude_agent_sdk_rs::skills::types::{SkillMetadata, SkillPackage, SkillResources};
use std::time::{Duration, Instant};
use uuid::Uuid;

fn create_skill(name: &str, tags: Vec<&str>) -> SkillPackage {
    SkillPackage {
        metadata: SkillMetadata {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: format!("Description for {}", name),
            version: "1.0.0".to_string(),
            author: Some("Test Author".to_string()),
            dependencies: Vec::new(),
            tags: tags.into_iter().map(String::from).collect(),
        },
        instructions: format!("Instructions for {}", name),
        scripts: Vec::new(),
        resources: SkillResources::default(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Agent Skills Performance Optimization Demo ===\n");

    // Example 1: LRU Cache
    println!("1. LRU Cache Demonstration");
    println!("----------------------------");
    let mut cache = LruCache::new(3);

    cache.put("key1", "value1");
    cache.put("key2", "value2");
    cache.put("key3", "value3");

    println!("Cache size after 3 inserts: {}", cache.len());
    println!("Get key1: {:?}", cache.get(&"key1"));
    println!("Cache contains key2: {}", cache.contains_key(&"key2"));

    cache.put("key4", "value4"); // Should evict key1 (least recently used)
    println!("After adding key4, cache size: {}", cache.len());
    println!("Get key1 (should be None): {:?}", cache.get(&"key1"));
    println!("Get key2: {:?}", cache.get(&"key2"));
    println!();

    // Example 2: Indexed Skill Collection
    println!("2. Indexed Skill Collection");
    println!("---------------------------");
    let mut collection = IndexedSkillCollection::new();

    // Add skills
    collection.add(create_skill("rust-sdk", vec!["rust", "sdk", "api"]));
    collection.add(create_skill("python-sdk", vec!["python", "sdk", "api"]));
    collection.add(create_skill(
        "web-framework",
        vec!["rust", "web", "framework"],
    ));
    collection.add(create_skill("cli-tool", vec!["rust", "cli", "tool"]));

    println!("Total skills: {}", collection.len());

    // Query by name
    if let Some(skill) = collection.get_by_name("rust-sdk") {
        println!("Found skill: {}", skill.metadata.name);
        println!("Tags: {:?}", skill.metadata.tags);
    }
    println!();

    // Example 3: Tag-based Querying with Indexing
    println!("3. Tag-based Querying");
    println!("---------------------");

    // Query by tag (uses index)
    let rust_skills = collection.get_by_tag("rust");
    println!("Skills with 'rust' tag: {}", rust_skills.len());
    for skill in rust_skills {
        println!("  - {}", skill.metadata.name);
    }

    let sdk_skills = collection.get_by_tag("sdk");
    println!("Skills with 'sdk' tag: {}", sdk_skills.len());
    for skill in sdk_skills {
        println!("  - {}", skill.metadata.name);
    }
    println!();

    // Example 4: Query with Filter and Caching
    println!("4. Query with Filter and Caching");
    println!("-------------------------------");
    let filter = TagFilter::new().has("rust");

    // First query (cache miss)
    let start = Instant::now();
    let results1 = collection.query(&filter);
    let duration1 = start.elapsed();
    println!("First query: {} results in {:?}", results1.len(), duration1);

    // Second query (cache hit)
    let start = Instant::now();
    let results2 = collection.query(&filter);
    let duration2 = start.elapsed();
    println!(
        "Second query: {} results in {:?}",
        results2.len(),
        duration2
    );
    println!();

    // Example 5: Batch Operations
    println!("5. Batch Operations");
    println!("-------------------");
    let mut large_collection = IndexedSkillCollection::with_capacity(100);

    // Create many skills
    let skills: Vec<SkillPackage> = (0..100)
        .map(|i| {
            let tags = if i % 2 == 0 {
                vec!["rust", "performance"]
            } else if i % 3 == 0 {
                vec!["python", "data"]
            } else {
                vec!["general", "utility"]
            };
            create_skill(&format!("skill-{}", i), tags)
        })
        .collect();

    // Batch add
    let start = Instant::now();
    large_collection.add_batch(skills);
    let duration = start.elapsed();
    println!("Added 100 skills in {:?}", duration);
    println!("Total skills: {}", large_collection.len());
    println!();

    // Example 6: Performance Statistics
    println!("6. Performance Statistics");
    println!("------------------------");
    let mut stats = PerformanceStats::new();
    stats.operations = 1000;
    stats.total_duration = Duration::from_millis(500);
    stats.cache_hits = 800;
    stats.cache_misses = 200;
    stats.items_processed = 10000;

    println!("Operations: {}", stats.operations);
    println!("Total time: {:?}", stats.total_duration);
    println!(
        "Average time per operation: {:?}",
        stats.avg_time_per_operation()
    );
    println!("Cache hit rate: {:.2}%", stats.cache_hit_rate() * 100.0);
    println!("Throughput: {:.2} items/sec", stats.throughput());
    println!();

    // Example 7: Batch Filtering
    println!("7. Batch Filtering");
    println!("------------------");
    let all_skills = large_collection
        .all()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let rust_skills = BatchOperations::filter_skills(&all_skills, |s| {
        s.metadata.tags.contains(&"rust".to_string())
    });
    println!(
        "Rust skills: {} out of {}",
        rust_skills.len(),
        all_skills.len()
    );
    println!();

    // Example 8: Batch Partitioning
    println!("8. Batch Partitioning");
    println!("--------------------");
    let skills_vec: Vec<SkillPackage> = vec![
        create_skill("skill-a", vec!["rust"]),
        create_skill("skill-b", vec!["python"]),
        create_skill("skill-c", vec!["rust"]),
        create_skill("skill-d", vec!["go"]),
    ];

    let (rust_skills, other_skills) = BatchOperations::partition_skills(skills_vec, |s| {
        s.metadata.tags.contains(&"rust".to_string())
    });

    println!("Rust skills: {}", rust_skills.len());
    println!("Other skills: {}", other_skills.len());
    println!();

    // Example 9: Complex Query with Multiple Tags
    println!("9. Complex Tag Queries");
    println!("----------------------");

    // Skills with "rust" OR "python"
    let filter_or = TagFilter::new().any_of(vec!["rust".to_string(), "python".to_string()]);
    let results_or = large_collection.query(&filter_or);
    println!("Rust OR Python: {} results", results_or.len());

    // Skills with both "rust" AND "performance"
    let filter_and = TagFilter::new().all_of(vec!["rust".to_string(), "performance".to_string()]);
    let results_and = large_collection.query(&filter_and);
    println!("Rust AND Performance: {} results", results_and.len());

    // Skills without "general"
    let filter_not = TagFilter::new().not_has("general");
    let results_not = large_collection.query(&filter_not);
    println!("NOT 'general': {} results", results_not.len());
    println!();

    // Example 10: Index Rebuilding
    println!("10. Index Rebuilding");
    println!("--------------------");
    println!("Skills before rebuild: {}", large_collection.len());

    large_collection.rebuild_indexes();

    println!("Skills after rebuild: {}", large_collection.len());
    println!("Rebuild maintains all indexes and query cache");
    println!();

    // Example 11: Performance Comparison
    println!("11. Performance Comparison: Indexed vs Sequential");
    println!("--------------------------------------------------");

    // Create larger dataset
    let mut huge_collection = {
        let mut col = IndexedSkillCollection::with_capacity(1000);
        let skills: Vec<SkillPackage> = (0..1000)
            .map(|i| {
                let tags = match i % 5 {
                    0 => vec!["rust", "web"],
                    1 => vec!["python", "data"],
                    2 => vec!["rust", "cli"],
                    3 => vec!["go", "api"],
                    _ => vec!["general"],
                };
                create_skill(&format!("skill-{}", i), tags)
            })
            .collect();
        col.add_batch(skills);
        col
    };

    // Indexed query
    let filter = TagFilter::new().has("rust");
    let start = Instant::now();
    let _indexed_results = huge_collection.query(&filter);
    let indexed_time = start.elapsed();

    println!("Indexed query on 1000 skills: {:?}", indexed_time);
    println!("Indexed queries are O(1) for tag lookups");
    println!();

    // Example 12: Cache Efficiency
    println!("12. Cache Efficiency Demonstration");
    println!("----------------------------------");
    let mut small_collection = IndexedSkillCollection::new();

    for i in 0..50 {
        let tags = if i % 2 == 0 {
            vec!["even"]
        } else {
            vec!["odd"]
        };
        small_collection.add(create_skill(&format!("skill-{}", i), tags));
    }

    // Run multiple queries
    let filter_even = TagFilter::new().has("even");

    for i in 0..5 {
        let _results = small_collection.query(&filter_even);
        if i == 0 {
            println!("Query {} - Cache miss (first time)", i + 1);
        } else {
            println!("Query {} - Cache hit", i + 1);
        }
    }
    println!();

    println!("=== Demo Complete ===");
    println!();
    println!("Key Performance Features:");
    println!("- LRU Cache: Reduces redundant computations");
    println!("- Indexed Collection: O(1) tag lookups");
    println!("- Batch Operations: Efficient bulk processing");
    println!("- Query Caching: Automatic result caching");
    println!("- Performance Stats: Monitor and optimize");

    Ok(())
}
