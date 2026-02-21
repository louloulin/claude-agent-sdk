//! Tag-based filtering and querying for Agent Skills

use std::collections::{HashMap, HashSet};
use std::fmt;

/// Tag query operator
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagOperator {
    /// Tag must be present
    Has(String),
    /// Tag must not be present
    NotHas(String),
    /// Any of the tags must be present (OR)
    AnyOf(Vec<String>),
    /// All of the tags must be present (AND)
    AllOf(Vec<String>),
    /// None of the tags must be present
    NoneOf(Vec<String>),
}

impl fmt::Display for TagOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TagOperator::Has(tag) => write!(f, "has:{}", tag),
            TagOperator::NotHas(tag) => write!(f, "!{}", tag),
            TagOperator::AnyOf(tags) => write!(f, "any:{}", tags.join("|")),
            TagOperator::AllOf(tags) => write!(f, "all:{}", tags.join("&")),
            TagOperator::NoneOf(tags) => write!(f, "none:{}", tags.join("|")),
        }
    }
}

/// Tag filter for querying skills
#[derive(Debug, Clone)]
pub struct TagFilter {
    operators: Vec<TagOperator>,
}

impl TagFilter {
    /// Create a new tag filter
    pub fn new() -> Self {
        Self {
            operators: Vec::new(),
        }
    }

    /// Add a "has tag" condition
    pub fn has(mut self, tag: impl Into<String>) -> Self {
        self.operators.push(TagOperator::Has(tag.into()));
        self
    }

    /// Add a "not has tag" condition
    pub fn not_has(mut self, tag: impl Into<String>) -> Self {
        self.operators.push(TagOperator::NotHas(tag.into()));
        self
    }

    /// Add an "any of tags" condition
    pub fn any_of(mut self, tags: Vec<String>) -> Self {
        self.operators.push(TagOperator::AnyOf(tags));
        self
    }

    /// Add an "all of tags" condition
    pub fn all_of(mut self, tags: Vec<String>) -> Self {
        self.operators.push(TagOperator::AllOf(tags));
        self
    }

    /// Add a "none of tags" condition
    pub fn none_of(mut self, tags: Vec<String>) -> Self {
        self.operators.push(TagOperator::NoneOf(tags));
        self
    }

    /// Check if a set of tags matches all filter conditions
    pub fn matches(&self, tags: &HashSet<String>) -> bool {
        for op in &self.operators {
            if !self.matches_operator(op, tags) {
                return false;
            }
        }
        true
    }

    fn matches_operator(&self, op: &TagOperator, item_tags: &HashSet<String>) -> bool {
        match op {
            TagOperator::Has(tag) => item_tags.contains(tag),
            TagOperator::NotHas(tag) => !item_tags.contains(tag),
            TagOperator::AnyOf(tags) => tags.iter().any(|t| item_tags.contains(t)),
            TagOperator::AllOf(tags) => tags.iter().all(|t| item_tags.contains(t)),
            TagOperator::NoneOf(tags) => tags.iter().all(|t| !item_tags.contains(t)),
        }
    }
}

impl Default for TagFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Tag query builder for constructing complex queries
pub struct TagQueryBuilder {
    filters: Vec<TagFilter>,
}

impl TagQueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    /// Add a filter (AND logic with previous filters)
    pub fn and(mut self, filter: TagFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Execute the query on a collection of tagged items
    pub fn query<'a, T>(&self, items: &'a [T], tags_getter: impl Fn(&T) -> &[String]) -> Vec<&'a T>
    where
        T: Sized,
    {
        items
            .iter()
            .filter(|item| {
                let tags: HashSet<String> = tags_getter(item).iter().cloned().collect();
                self.filters.iter().all(|filter| filter.matches(&tags))
            })
            .collect()
    }

    /// Count items matching the query
    pub fn count<T>(&self, items: &[T], tags_getter: impl Fn(&T) -> &[String]) -> usize
    where
        T: Sized,
    {
        self.query(items, tags_getter).len()
    }

    /// Get all unique tags from a collection
    pub fn collect_tags<T>(
        &self,
        items: &[T],
        tags_getter: impl Fn(&T) -> &[String],
    ) -> HashSet<String>
    where
        T: Sized,
    {
        items
            .iter()
            .flat_map(|item| tags_getter(item).iter().cloned())
            .collect()
    }

    /// Group items by a specific tag
    pub fn group_by_tag<'a, T>(
        &self,
        items: &'a [T],
        tag: &str,
        tags_getter: impl Fn(&T) -> &[String],
    ) -> HashMap<String, Vec<&'a T>>
    where
        T: Sized,
    {
        let mut groups: HashMap<String, Vec<&T>> = HashMap::new();
        let no_tag = "__no_tag__".to_string();

        for item in items {
            let tags = tags_getter(item);
            if tags.contains(&tag.to_string()) {
                groups
                    .entry(tag.to_string())
                    .or_default()
                    .push(item);
            } else {
                groups
                    .entry(no_tag.clone())
                    .or_default()
                    .push(item);
            }
        }

        groups
    }

    /// Find items with any of the specified tags
    pub fn with_any_tag<'a, T>(
        &self,
        items: &'a [T],
        tags: &[String],
        tags_getter: impl Fn(&T) -> &[String],
    ) -> Vec<&'a T>
    where
        T: Sized,
    {
        items
            .iter()
            .filter(|item| {
                let item_tags = tags_getter(item);
                tags.iter().any(|tag| item_tags.contains(tag))
            })
            .collect()
    }

    /// Find items with all of the specified tags
    pub fn with_all_tags<'a, T>(
        &self,
        items: &'a [T],
        tags: &[String],
        tags_getter: impl Fn(&T) -> &[String],
    ) -> Vec<&'a T>
    where
        T: Sized,
    {
        items
            .iter()
            .filter(|item| {
                let item_tags = tags_getter(item);
                tags.iter().all(|tag| item_tags.contains(tag))
            })
            .collect()
    }

    /// Get tag statistics
    pub fn tag_statistics<T>(
        &self,
        items: &[T],
        tags_getter: impl Fn(&T) -> &[String],
    ) -> HashMap<String, usize>
    where
        T: Sized,
    {
        let mut stats = HashMap::new();
        for item in items {
            for tag in tags_getter(item) {
                *stats.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        stats
    }

    /// Find most popular tags
    pub fn popular_tags<T>(
        &self,
        items: &[T],
        tags_getter: impl Fn(&T) -> &[String],
        limit: usize,
    ) -> Vec<(String, usize)>
    where
        T: Sized,
    {
        let mut stats: Vec<_> = self
            .tag_statistics(items, tags_getter)
            .into_iter()
            .collect();

        stats.sort_by_key(|b| std::cmp::Reverse(b.1));
        stats.into_iter().take(limit).collect()
    }
}

impl Default for TagQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for tag operations
pub struct TagUtils;

impl TagUtils {
    /// Normalize tag names (lowercase, trim, replace spaces with hyphens, remove special chars)
    pub fn normalize_tag(tag: &str) -> String {
        tag.split_whitespace()
            .collect::<Vec<&str>>()
            .join("-")
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect()
    }

    /// Validate tag name
    pub fn is_valid_tag(tag: &str) -> bool {
        if tag.is_empty() || tag.len() > 50 {
            return false;
        }

        tag.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Parse tag string (e.g., "tag1,tag2,tag3")
    pub fn parse_tags(tags_str: &str) -> Vec<String> {
        tags_str
            .split(',')
            .map(Self::normalize_tag)
            .filter(|s| !s.is_empty() && Self::is_valid_tag(s))
            .collect()
    }

    /// Merge two tag lists, removing duplicates
    pub fn merge_tags(tags1: &[String], tags2: &[String]) -> Vec<String> {
        let mut merged: HashSet<String> = HashSet::new();

        for tag in tags1.iter().chain(tags2.iter()) {
            merged.insert(Self::normalize_tag(tag));
        }

        merged.into_iter().collect()
    }

    /// Find common tags between two lists
    pub fn common_tags(tags1: &[String], tags2: &[String]) -> Vec<String> {
        let set1: HashSet<_> = tags1.iter().map(|t| Self::normalize_tag(t)).collect();
        let set2: HashSet<_> = tags2.iter().map(|t| Self::normalize_tag(t)).collect();

        set1.intersection(&set2).cloned().collect()
    }

    /// Calculate tag similarity (Jaccard index)
    pub fn tag_similarity(tags1: &[String], tags2: &[String]) -> f64 {
        if tags1.is_empty() && tags2.is_empty() {
            return 1.0;
        }

        let set1: HashSet<_> = tags1.iter().map(|t| Self::normalize_tag(t)).collect();
        let set2: HashSet<_> = tags2.iter().map(|t| Self::normalize_tag(t)).collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_filter_has() {
        let filter = TagFilter::new().has("rust");
        let tags: HashSet<String> = vec!["rust".to_string(), "sdk".to_string()]
            .into_iter()
            .collect();

        assert!(filter.matches(&tags));
    }

    #[test]
    fn test_tag_filter_not_has() {
        let filter = TagFilter::new().not_has("python");
        let tags: HashSet<String> = vec!["rust".to_string(), "sdk".to_string()]
            .into_iter()
            .collect();

        assert!(filter.matches(&tags));
    }

    #[test]
    fn test_tag_filter_any_of() {
        let filter = TagFilter::new().any_of(vec!["rust".to_string(), "python".to_string()]);
        let tags1: HashSet<String> = vec!["rust".to_string()].into_iter().collect();
        let tags2: HashSet<String> = vec!["java".to_string()].into_iter().collect();

        assert!(filter.matches(&tags1));
        assert!(!filter.matches(&tags2));
    }

    #[test]
    fn test_tag_filter_all_of() {
        let filter = TagFilter::new().all_of(vec![
            "rust".to_string(),
            "sdk".to_string(),
            "agent".to_string(),
        ]);

        let tags1: HashSet<String> =
            vec!["rust".to_string(), "sdk".to_string(), "agent".to_string()]
                .into_iter()
                .collect();
        let tags2: HashSet<String> = vec!["rust".to_string(), "sdk".to_string()]
            .into_iter()
            .collect();

        assert!(filter.matches(&tags1));
        assert!(!filter.matches(&tags2));
    }

    #[test]
    fn test_tag_filter_none_of() {
        let filter = TagFilter::new().none_of(vec!["python".to_string(), "java".to_string()]);
        let tags1: HashSet<String> = vec!["rust".to_string()].into_iter().collect();
        let tags2: HashSet<String> = vec!["rust".to_string(), "python".to_string()]
            .into_iter()
            .collect();

        assert!(filter.matches(&tags1));
        assert!(!filter.matches(&tags2));
    }

    #[test]
    fn test_tag_query_builder_query() {
        struct Item {
            name: String,
            tags: Vec<String>,
        }

        let items = vec![
            Item {
                name: "item1".to_string(),
                tags: vec!["rust".to_string(), "sdk".to_string()],
            },
            Item {
                name: "item2".to_string(),
                tags: vec!["python".to_string()],
            },
            Item {
                name: "item3".to_string(),
                tags: vec!["rust".to_string()],
            },
        ];

        let builder = TagQueryBuilder::new();
        let result = builder.with_any_tag(&items, &["rust".to_string()], |item| &item.tags);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "item1");
        assert_eq!(result[1].name, "item3");
    }

    #[test]
    fn test_tag_query_builder_statistics() {
        struct Item {
            tags: Vec<String>,
        }

        let items = vec![
            Item {
                tags: vec!["rust".to_string(), "sdk".to_string()],
            },
            Item {
                tags: vec!["rust".to_string(), "agent".to_string()],
            },
            Item {
                tags: vec!["python".to_string()],
            },
        ];

        let builder = TagQueryBuilder::new();
        let stats = builder.tag_statistics(&items, |item| &item.tags);

        assert_eq!(stats.get("rust"), Some(&2));
        assert_eq!(stats.get("sdk"), Some(&1));
        assert_eq!(stats.get("agent"), Some(&1));
        assert_eq!(stats.get("python"), Some(&1));
    }

    #[test]
    fn test_tag_query_builder_popular_tags() {
        struct Item {
            tags: Vec<String>,
        }

        let items = vec![
            Item {
                tags: vec!["rust".to_string(), "sdk".to_string()],
            },
            Item {
                tags: vec!["rust".to_string(), "agent".to_string()],
            },
            Item {
                tags: vec!["rust".to_string()],
            },
        ];

        let builder = TagQueryBuilder::new();
        let popular = builder.popular_tags(&items, |item| &item.tags, 2);

        assert_eq!(popular.len(), 2);
        assert_eq!(popular[0].0, "rust");
        assert_eq!(popular[0].1, 3);
    }

    #[test]
    fn test_tag_utils_normalize() {
        assert_eq!(TagUtils::normalize_tag("  Rust SDK  "), "rust-sdk");
        assert_eq!(TagUtils::normalize_tag("Hello_World"), "hello_world");
        assert_eq!(TagUtils::normalize_tag("test@123"), "test123");
    }

    #[test]
    fn test_tag_utils_is_valid() {
        assert!(TagUtils::is_valid_tag("rust"));
        assert!(TagUtils::is_valid_tag("rust-sdk"));
        assert!(TagUtils::is_valid_tag("rust_sdk"));
        assert!(!TagUtils::is_valid_tag(""));
        assert!(!TagUtils::is_valid_tag("rust@sdk"));
        assert!(!TagUtils::is_valid_tag(&"a".repeat(51)));
    }

    #[test]
    fn test_tag_utils_parse() {
        let tags = TagUtils::parse_tags("rust, python, sdk");
        assert_eq!(tags, vec!["rust", "python", "sdk"]);
    }

    #[test]
    fn test_tag_utils_merge() {
        let tags1 = vec!["rust".to_string(), "sdk".to_string()];
        let tags2 = vec!["rust".to_string(), "agent".to_string()];
        let merged = TagUtils::merge_tags(&tags1, &tags2);

        assert_eq!(merged.len(), 3);
        assert!(merged.contains(&"rust".to_string()));
        assert!(merged.contains(&"sdk".to_string()));
        assert!(merged.contains(&"agent".to_string()));
    }

    #[test]
    fn test_tag_utils_common() {
        let tags1 = vec!["rust".to_string(), "sdk".to_string(), "agent".to_string()];
        let tags2 = vec!["rust".to_string(), "sdk".to_string(), "python".to_string()];
        let common = TagUtils::common_tags(&tags1, &tags2);

        assert_eq!(common.len(), 2);
        assert!(common.contains(&"rust".to_string()));
        assert!(common.contains(&"sdk".to_string()));
    }

    #[test]
    fn test_tag_utils_similarity() {
        let tags1 = vec!["rust".to_string(), "sdk".to_string()];
        let tags2 = vec!["rust".to_string(), "sdk".to_string()];
        let tags3 = vec!["rust".to_string(), "agent".to_string()];
        let tags4: Vec<String> = vec![];

        assert_eq!(TagUtils::tag_similarity(&tags1, &tags2), 1.0);
        assert_eq!(TagUtils::tag_similarity(&tags1, &tags3), 1.0 / 3.0);
        assert_eq!(TagUtils::tag_similarity(&tags1, &tags4), 0.0);
        assert_eq!(TagUtils::tag_similarity(&tags4, &tags4), 1.0);
    }
}
