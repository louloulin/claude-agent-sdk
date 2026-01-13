//! # Performance Optimization for Agent Skills
//!
//! This module provides performance optimizations for large-scale skill operations,
//! including indexing, caching, and batch processing.

use crate::skills::tags::TagFilter;
use crate::skills::types::SkillPackage;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::time::{Duration, Instant};
use tracing::debug;

/// Performance statistics for operations
#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    /// Number of operations performed
    pub operations: usize,

    /// Total time spent
    pub total_duration: Duration,

    /// Cache hits
    pub cache_hits: usize,

    /// Cache misses
    pub cache_misses: usize,

    /// Items processed
    pub items_processed: usize,
}

impl PerformanceStats {
    /// Create new statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate average time per operation
    pub fn avg_time_per_operation(&self) -> Option<Duration> {
        if self.operations > 0 {
            Some(self.total_duration / self.operations as u32)
        } else {
            None
        }
    }

    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Calculate throughput (items per second)
    pub fn throughput(&self) -> f64 {
        let seconds = self.total_duration.as_secs_f64();
        if seconds > 0.0 {
            self.items_processed as f64 / seconds
        } else {
            0.0
        }
    }
}

/// Simple LRU cache implementation
#[derive(Debug, Clone)]
pub struct LruCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    capacity: usize,
    map: HashMap<K, V>,
    access_order: Vec<K>,
}

impl<K, V> LruCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new LRU cache with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::new(),
            access_order: Vec::with_capacity(capacity),
        }
    }

    /// Get a value from the cache
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(_) = self.map.get(key) {
            // Move to end (most recently used)
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
                self.access_order.push(key.clone());
            }
            self.map.get(key)
        } else {
            None
        }
    }

    /// Insert a value into the cache
    pub fn put(&mut self, key: K, value: V) {
        // Remove existing if present
        if self.map.contains_key(&key) {
            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.remove(pos);
            }
        }

        // Evict oldest if at capacity
        if self.access_order.len() >= self.capacity {
            if let Some(old_key) = self.access_order.first() {
                self.map.remove(old_key);
                self.access_order.remove(0);
            }
        }

        // Insert new
        self.map.insert(key.clone(), value);
        self.access_order.push(key);
    }

    /// Check if cache contains a key
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.map.clear();
        self.access_order.clear();
    }

    /// Get current size
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

/// Multi-indexed skill collection for fast queries
#[derive(Debug, Clone)]
pub struct IndexedSkillCollection {
    /// All skills
    skills: Vec<SkillPackage>,

    /// Index by name
    by_name: HashMap<String, usize>,

    /// Index by tags (tag -> skill indices)
    by_tag: HashMap<String, Vec<usize>>,

    /// Cache for query results
    query_cache: LruCache<String, Vec<usize>>,
}

impl Default for IndexedSkillCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexedSkillCollection {
    /// Create a new indexed collection
    pub fn new() -> Self {
        Self {
            skills: Vec::new(),
            by_name: HashMap::new(),
            by_tag: HashMap::new(),
            query_cache: LruCache::new(100),
        }
    }

    /// Create with capacity hint
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            skills: Vec::with_capacity(capacity),
            by_name: HashMap::with_capacity(capacity),
            by_tag: HashMap::new(),
            query_cache: LruCache::new(100),
        }
    }

    /// Add a skill to the collection
    pub fn add(&mut self, skill: SkillPackage) {
        let index = self.skills.len();

        // Index by name
        self.by_name.insert(skill.metadata.name.clone(), index);

        // Index by tags
        for tag in &skill.metadata.tags {
            self.by_tag
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(index);
        }

        self.skills.push(skill);
    }

    /// Add multiple skills in batch
    pub fn add_batch(&mut self, skills: Vec<SkillPackage>) {
        let start = Instant::now();
        let initial_count = self.skills.len();

        for skill in skills {
            self.add(skill);
        }

        let added = self.skills.len() - initial_count;
        debug!("Added {} skills in {:?}", added, start.elapsed());
    }

    /// Get a skill by name (O(1))
    pub fn get_by_name(&self, name: &str) -> Option<&SkillPackage> {
        self.by_name.get(name).map(|&index| &self.skills[index])
    }

    /// Get all skills with a specific tag (O(1) average)
    pub fn get_by_tag(&self, tag: &str) -> Vec<&SkillPackage> {
        self.by_tag
            .get(tag)
            .map(|indices| indices.iter().map(|&i| &self.skills[i]).collect())
            .unwrap_or_default()
    }

    /// Query skills using a tag filter with caching
    pub fn query(&mut self, filter: &TagFilter) -> Vec<&SkillPackage> {
        let cache_key = format!("{:?}", filter);

        // Check cache
        if let Some(indices) = self.query_cache.get(&cache_key) {
            debug!("Cache hit for query: {}", cache_key);
            return indices.iter().map(|&i| &self.skills[i]).collect();
        }

        // Perform query and collect indices first
        let indices: Vec<usize> = self
            .skills
            .iter()
            .enumerate()
            .filter(|(_, skill)| {
                let tags: HashSet<String> = skill.metadata.tags.iter().cloned().collect();
                filter.matches(&tags)
            })
            .map(|(i, _)| i)
            .collect();

        // Cache the indices
        self.query_cache.put(cache_key, indices.clone());

        // Return references using indices
        indices.iter().map(|&i| &self.skills[i]).collect()
    }

    /// Get all skills
    pub fn all(&self) -> Vec<&SkillPackage> {
        self.skills.iter().collect()
    }

    /// Count of skills
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Clear the collection
    pub fn clear(&mut self) {
        self.skills.clear();
        self.by_name.clear();
        self.by_tag.clear();
        self.query_cache.clear();
    }

    /// Rebuild all indexes
    pub fn rebuild_indexes(&mut self) {
        let skills = std::mem::take(&mut self.skills);
        self.clear();
        self.add_batch(skills);
    }
}

/// Batch operations for efficient processing
pub struct BatchOperations;

impl BatchOperations {
    /// Batch filter skills
    pub fn filter_skills(
        skills: &[SkillPackage],
        predicate: impl Fn(&SkillPackage) -> bool,
    ) -> Vec<SkillPackage> {
        skills.iter().filter(|s| predicate(s)).cloned().collect()
    }

    /// Batch map skills
    pub fn map_skills(
        skills: Vec<SkillPackage>,
        mapper: impl Fn(SkillPackage) -> SkillPackage,
    ) -> Vec<SkillPackage> {
        skills.into_iter().map(mapper).collect()
    }

    /// Batch filter and map
    pub fn filter_map_skills(
        skills: Vec<SkillPackage>,
        predicate: impl Fn(&SkillPackage) -> bool,
        mapper: impl Fn(SkillPackage) -> SkillPackage,
    ) -> Vec<SkillPackage> {
        skills
            .into_iter()
            .filter(|s| predicate(s))
            .map(mapper)
            .collect()
    }

    /// Partition skills into two groups
    pub fn partition_skills(
        skills: Vec<SkillPackage>,
        predicate: impl Fn(&SkillPackage) -> bool,
    ) -> (Vec<SkillPackage>, Vec<SkillPackage>) {
        skills.into_iter().partition(predicate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::types::SkillMetadata;

    fn create_test_skill(name: &str, tags: Vec<&str>) -> SkillPackage {
        SkillPackage {
            metadata: SkillMetadata {
                id: uuid::Uuid::new_v4().to_string(),
                name: name.to_string(),
                description: String::new(),
                version: "1.0.0".to_string(),
                author: None,
                dependencies: Vec::new(),
                tags: tags.into_iter().map(String::from).collect(),
            },
            instructions: String::new(),
            scripts: Vec::new(),
            resources: Default::default(),
        }
    }

    #[test]
    fn test_performance_stats_default() {
        let stats = PerformanceStats::new();
        assert_eq!(stats.operations, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
    }

    #[test]
    fn test_performance_stats_avg_time() {
        let mut stats = PerformanceStats::new();
        stats.operations = 10;
        stats.total_duration = Duration::from_secs(1);

        let avg = stats.avg_time_per_operation();
        assert_eq!(avg, Some(Duration::from_millis(100)));
    }

    #[test]
    fn test_performance_stats_cache_hit_rate() {
        let mut stats = PerformanceStats::new();
        stats.cache_hits = 80;
        stats.cache_misses = 20;

        let rate = stats.cache_hit_rate();
        assert!((rate - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_performance_stats_throughput() {
        let mut stats = PerformanceStats::new();
        stats.items_processed = 1000;
        stats.total_duration = Duration::from_secs(2);

        let throughput = stats.throughput();
        assert!((throughput - 500.0).abs() < 1.0);
    }

    #[test]
    fn test_lru_cache_basic() {
        let mut cache = LruCache::new(2);

        cache.put("key1", "value1");
        cache.put("key2", "value2");

        assert_eq!(cache.get(&"key1"), Some(&"value1"));
        assert_eq!(cache.get(&"key2"), Some(&"value2"));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = LruCache::new(2);

        cache.put("key1", "value1");
        cache.put("key2", "value2");
        cache.put("key3", "value3"); // Should evict key1

        assert_eq!(cache.get(&"key1"), None);
        assert_eq!(cache.get(&"key2"), Some(&"value2"));
        assert_eq!(cache.get(&"key3"), Some(&"value3"));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_lru_cache_access_order() {
        let mut cache = LruCache::new(2);

        cache.put("key1", "value1");
        cache.put("key2", "value2");
        cache.get(&"key1"); // Access key1
        cache.put("key3", "value3"); // Should evict key2

        assert_eq!(cache.get(&"key1"), Some(&"value1"));
        assert_eq!(cache.get(&"key2"), None);
        assert_eq!(cache.get(&"key3"), Some(&"value3"));
    }

    #[test]
    fn test_indexed_collection_add() {
        let mut collection = IndexedSkillCollection::new();
        let skill = create_test_skill("skill1", vec!["tag1", "tag2"]);

        collection.add(skill);

        assert_eq!(collection.len(), 1);
        assert!(collection.get_by_name("skill1").is_some());
    }

    #[test]
    fn test_indexed_collection_by_name() {
        let mut collection = IndexedSkillCollection::new();
        collection.add(create_test_skill("skill1", vec!["tag1"]));
        collection.add(create_test_skill("skill2", vec!["tag2"]));

        assert_eq!(
            collection.get_by_name("skill1").unwrap().metadata.name,
            "skill1"
        );
        assert_eq!(
            collection.get_by_name("skill2").unwrap().metadata.name,
            "skill2"
        );
        assert!(collection.get_by_name("skill3").is_none());
    }

    #[test]
    fn test_indexed_collection_by_tag() {
        let mut collection = IndexedSkillCollection::new();
        collection.add(create_test_skill("skill1", vec!["rust", "web"]));
        collection.add(create_test_skill("skill2", vec!["rust", "cli"]));
        collection.add(create_test_skill("skill3", vec!["python", "web"]));

        let rust_skills = collection.get_by_tag("rust");
        assert_eq!(rust_skills.len(), 2);

        let web_skills = collection.get_by_tag("web");
        assert_eq!(web_skills.len(), 2);

        let python_skills = collection.get_by_tag("python");
        assert_eq!(python_skills.len(), 1);
    }

    #[test]
    fn test_indexed_collection_query() {
        let mut collection = IndexedSkillCollection::new();
        collection.add(create_test_skill("skill1", vec!["rust", "web"]));
        collection.add(create_test_skill("skill2", vec!["rust", "cli"]));
        collection.add(create_test_skill("skill3", vec!["python", "web"]));

        let filter = TagFilter::new().has("rust");
        let results = collection.query(&filter);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_indexed_collection_query_cache() {
        let mut collection = IndexedSkillCollection::new();
        collection.add(create_test_skill("skill1", vec!["rust"]));
        collection.add(create_test_skill("skill2", vec!["python"]));

        let filter = TagFilter::new().has("rust");

        // First query - cache miss
        let results1 = collection.query(&filter);
        assert_eq!(results1.len(), 1);

        // Second query - cache hit
        let results2 = collection.query(&filter);
        assert_eq!(results2.len(), 1);
    }

    #[test]
    fn test_indexed_collection_batch_add() {
        let mut collection = IndexedSkillCollection::new();
        let skills = vec![
            create_test_skill("skill1", vec!["tag1"]),
            create_test_skill("skill2", vec!["tag2"]),
            create_test_skill("skill3", vec!["tag3"]),
        ];

        collection.add_batch(skills);

        assert_eq!(collection.len(), 3);
    }

    #[test]
    fn test_batch_operations_filter() {
        let skills = vec![
            create_test_skill("skill1", vec!["rust"]),
            create_test_skill("skill2", vec!["python"]),
            create_test_skill("skill3", vec!["rust"]),
        ];

        let filtered = BatchOperations::filter_skills(&skills, |s| {
            s.metadata.tags.contains(&"rust".to_string())
        });

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_batch_operations_partition() {
        let skills = vec![
            create_test_skill("skill1", vec!["rust"]),
            create_test_skill("skill2", vec!["python"]),
            create_test_skill("skill3", vec!["go"]),
        ];

        let (rust_skills, other_skills) = BatchOperations::partition_skills(skills, |s| {
            s.metadata.tags.contains(&"rust".to_string())
        });

        assert_eq!(rust_skills.len(), 1);
        assert_eq!(other_skills.len(), 2);
    }

    #[test]
    fn test_indexed_collection_rebuild() {
        let mut collection = IndexedSkillCollection::new();
        collection.add(create_test_skill("skill1", vec!["tag1"]));
        collection.add(create_test_skill("skill2", vec!["tag2"]));

        assert_eq!(collection.by_tag.len(), 2);

        collection.rebuild_indexes();

        assert_eq!(collection.len(), 2);
        assert!(collection.get_by_name("skill1").is_some());
        assert!(collection.get_by_name("skill2").is_some());
    }
}
