//! Dependency resolution and management for Agent Skills

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Dependency requirement for a skill
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dependency {
    /// Skill ID that this skill depends on
    pub skill_id: String,

    /// Version requirement (e.g., "^1.0.0", ">=2.0.0")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_requirement: Option<String>,
}

impl Dependency {
    /// Create a new dependency
    pub fn new(skill_id: impl Into<String>) -> Self {
        Self {
            skill_id: skill_id.into(),
            version_requirement: None,
        }
    }

    /// Create a new dependency with version requirement
    pub fn with_version(skill_id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            skill_id: skill_id.into(),
            version_requirement: Some(version.into()),
        }
    }
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref version) = self.version_requirement {
            write!(f, "{}@{}", self.skill_id, version)
        } else {
            write!(f, "{}", self.skill_id)
        }
    }
}

/// Result of dependency resolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionResult {
    /// All dependencies resolved successfully
    Resolved {
        /// Load order (topological sort)
        load_order: Vec<String>,
    },

    /// Circular dependency detected
    CircularDependency {
        /// Cycle of skill IDs
        cycle: Vec<String>,
    },

    /// Missing dependencies
    MissingDependencies {
        /// Missing skill IDs
        missing: Vec<String>,
    },
}

/// Dependency resolver for Agent Skills
pub struct DependencyResolver {
    /// Available skills (skill_id -> version)
    available: HashMap<String, String>,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new() -> Self {
        Self {
            available: HashMap::new(),
        }
    }

    /// Add an available skill
    pub fn add_skill(&mut self, skill_id: impl Into<String>, version: impl Into<String>) {
        self.available.insert(skill_id.into(), version.into());
    }

    /// Add multiple skills from skill packages
    pub fn add_skills<'a, I>(&mut self, packages: I)
    where
        I: IntoIterator<Item = &'a crate::skills::SkillPackage>,
    {
        for package in packages {
            self.add_skill(
                package.metadata.id.clone(),
                package.metadata.version.clone(),
            );
        }
    }

    /// Resolve dependencies for a set of skills
    ///
    /// # Arguments
    /// * `skills` - Map of skill_id to their dependencies
    ///
    /// # Returns
    /// Resolution result with load order or error
    pub fn resolve(&self, skills: &HashMap<String, Vec<Dependency>>) -> ResolutionResult {
        // Check for missing dependencies
        let missing = self.find_missing_dependencies(skills);
        if !missing.is_empty() {
            return ResolutionResult::MissingDependencies { missing };
        }

        // Check for circular dependencies
        if let Some(cycle) = self.detect_cycles(skills) {
            return ResolutionResult::CircularDependency { cycle };
        }

        // Topological sort for load order
        let load_order = self.topological_sort(skills);

        ResolutionResult::Resolved { load_order }
    }

    /// Find all missing dependencies
    fn find_missing_dependencies(&self, skills: &HashMap<String, Vec<Dependency>>) -> Vec<String> {
        let mut missing = HashSet::new();

        for deps in skills.values() {
            for dep in deps {
                if !self.available.contains_key(&dep.skill_id) {
                    missing.insert(dep.skill_id.clone());
                }
            }
        }

        missing.into_iter().collect()
    }

    /// Detect circular dependencies using DFS
    fn detect_cycles(&self, skills: &HashMap<String, Vec<Dependency>>) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for skill_id in skills.keys() {
            if self.dfs_cycle_detect(skill_id, skills, &mut visited, &mut rec_stack, &mut path) {
                return Some(path);
            }
        }

        None
    }

    /// DFS helper for cycle detection
    fn dfs_cycle_detect(
        &self,
        skill_id: &str,
        skills: &HashMap<String, Vec<Dependency>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        visited.insert(skill_id.to_string());
        rec_stack.insert(skill_id.to_string());
        path.push(skill_id.to_string());

        if let Some(deps) = skills.get(skill_id) {
            for dep in deps {
                if !visited.contains(&dep.skill_id) {
                    if self.dfs_cycle_detect(&dep.skill_id, skills, visited, rec_stack, path) {
                        return true;
                    }
                } else if rec_stack.contains(&dep.skill_id) {
                    path.push(dep.skill_id.clone());
                    return true;
                }
            }
        }

        rec_stack.remove(skill_id);
        path.pop();
        false
    }

    /// Topological sort to determine load order
    fn topological_sort(&self, skills: &HashMap<String, Vec<Dependency>>) -> Vec<String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut all_skills: HashSet<String> = HashSet::new();

        // Collect all skills and initialize in-degrees
        for (skill_id, deps) in skills {
            all_skills.insert(skill_id.clone());
            in_degree.insert(skill_id.clone(), deps.len());

            for dep in deps {
                all_skills.insert(dep.skill_id.clone());
                if !in_degree.contains_key(&dep.skill_id) {
                    in_degree.insert(dep.skill_id.clone(), 0);
                }
            }
        }

        // Build adjacency list
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for (skill_id, deps) in skills {
            for dep in deps {
                adj.entry(dep.skill_id.clone())
                    .or_default()
                    .push(skill_id.clone());
            }
        }

        // Kahn's algorithm for topological sort
        let mut queue: Vec<String> = all_skills
            .iter()
            .filter(|id| *in_degree.get(*id).unwrap_or(&0) == 0)
            .cloned()
            .collect();

        let mut result = Vec::new();

        while let Some(skill_id) = queue.pop() {
            result.push(skill_id.clone());

            if let Some(neighbors) = adj.get(&skill_id) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        result
    }

    /// Validate version requirements using semantic versioning
    pub fn validate_versions(&self, skills: &HashMap<String, Vec<Dependency>>) -> bool {
        for deps in skills.values() {
            for dep in deps {
                // Check if dependency exists
                let available_version = match self.available.get(&dep.skill_id) {
                    Some(v) => v,
                    None => return false,
                };

                // Validate version requirement if specified
                if let Some(ref req_str) = dep.version_requirement {
                    if !Self::check_version_requirement(available_version, req_str) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Check if a version satisfies a requirement using semantic versioning
    fn check_version_requirement(version: &str, requirement: &str) -> bool {
        use semver::{Version, VersionReq};

        // Parse the available version
        let version = match Version::parse(version) {
            Ok(v) => v,
            Err(_) => return false, // Invalid version format
        };

        // Parse the requirement (handle common prefixes)
        let req_str = requirement.trim();
        let req_str = match req_str.chars().next() {
            Some('^') | Some('~') | Some('=') | Some('>') | Some('<') => req_str,
            _ => &format!("^{}", req_str), // Default to caret requirement
        };

        let req = match VersionReq::parse(req_str) {
            Ok(r) => r,
            Err(_) => return false, // Invalid requirement format
        };

        req.matches(&version)
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_creation() {
        let dep = Dependency::new("test-skill");
        assert_eq!(dep.skill_id, "test-skill");
        assert!(dep.version_requirement.is_none());

        let dep_with_version = Dependency::with_version("test-skill", "^1.0.0");
        assert_eq!(dep_with_version.skill_id, "test-skill");
        assert_eq!(
            dep_with_version.version_requirement,
            Some("^1.0.0".to_string())
        );
    }

    #[test]
    fn test_dependency_display() {
        let dep = Dependency::new("test-skill");
        assert_eq!(dep.to_string(), "test-skill");

        let dep_with_version = Dependency::with_version("test-skill", "^1.0.0");
        assert_eq!(dep_with_version.to_string(), "test-skill@^1.0.0");
    }

    #[test]
    fn test_simple_resolution() {
        let mut resolver = DependencyResolver::new();
        resolver.add_skill("dep1", "1.0.0");
        resolver.add_skill("main", "1.0.0");

        let mut skills = HashMap::new();
        skills.insert("main".to_string(), vec![Dependency::new("dep1")]);
        skills.insert("dep1".to_string(), vec![]);

        let result = resolver.resolve(&skills);
        match result {
            ResolutionResult::Resolved { load_order } => {
                // dep1 should come before main
                let dep1_idx = load_order.iter().position(|id| id == "dep1").unwrap();
                let main_idx = load_order.iter().position(|id| id == "main").unwrap();
                assert!(dep1_idx < main_idx);
            },
            _ => panic!("Expected resolved result"),
        }
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut resolver = DependencyResolver::new();
        resolver.add_skill("skill1", "1.0.0");
        resolver.add_skill("skill2", "1.0.0");

        let mut skills = HashMap::new();
        skills.insert("skill1".to_string(), vec![Dependency::new("skill2")]);
        skills.insert("skill2".to_string(), vec![Dependency::new("skill1")]);

        let result = resolver.resolve(&skills);
        match result {
            ResolutionResult::CircularDependency { cycle } => {
                assert!(cycle.contains(&"skill1".to_string()));
                assert!(cycle.contains(&"skill2".to_string()));
            },
            _ => panic!("Expected circular dependency error"),
        }
    }

    #[test]
    fn test_missing_dependencies() {
        let mut resolver = DependencyResolver::new();
        resolver.add_skill("main", "1.0.0");
        // Note: not adding "missing-dep"

        let mut skills = HashMap::new();
        skills.insert("main".to_string(), vec![Dependency::new("missing-dep")]);

        let result = resolver.resolve(&skills);
        match result {
            ResolutionResult::MissingDependencies { missing } => {
                assert_eq!(missing, vec!["missing-dep".to_string()]);
            },
            _ => panic!("Expected missing dependencies error"),
        }
    }

    #[test]
    fn test_complex_dependency_graph() {
        let mut resolver = DependencyResolver::new();
        resolver.add_skill("a", "1.0.0");
        resolver.add_skill("b", "1.0.0");
        resolver.add_skill("c", "1.0.0");
        resolver.add_skill("d", "1.0.0");

        let mut skills = HashMap::new();
        skills.insert(
            "a".to_string(),
            vec![Dependency::new("b"), Dependency::new("c")],
        );
        skills.insert("b".to_string(), vec![Dependency::new("d")]);
        skills.insert("c".to_string(), vec![Dependency::new("d")]);
        skills.insert("d".to_string(), vec![]);

        let result = resolver.resolve(&skills);
        match result {
            ResolutionResult::Resolved { load_order } => {
                // d should be first (no dependencies)
                assert_eq!(load_order[0], "d");
                // b and c should come before a
                let a_idx = load_order.iter().position(|id| id == "a").unwrap();
                let b_idx = load_order.iter().position(|id| id == "b").unwrap();
                let c_idx = load_order.iter().position(|id| id == "c").unwrap();
                assert!(b_idx < a_idx);
                assert!(c_idx < a_idx);
            },
            _ => panic!("Expected resolved result"),
        }
    }

    #[test]
    fn test_version_requirement_caret() {
        // Test ^1.0.0 matches various versions
        assert!(DependencyResolver::check_version_requirement("1.0.0", "^1.0.0"));
        assert!(DependencyResolver::check_version_requirement("1.2.3", "^1.0.0"));
        assert!(!DependencyResolver::check_version_requirement("2.0.0", "^1.0.0"));
    }

    #[test]
    fn test_version_requirement_tilde() {
        // Test ~1.2.0 allows patch changes but not minor
        assert!(DependencyResolver::check_version_requirement("1.2.0", "~1.2.0"));
        assert!(DependencyResolver::check_version_requirement("1.2.5", "~1.2.0"));
        assert!(!DependencyResolver::check_version_requirement("1.3.0", "~1.2.0"));
    }

    #[test]
    fn test_version_requirement_exact() {
        // Test exact version matching
        assert!(DependencyResolver::check_version_requirement("2.0.0", "=2.0.0"));
        assert!(!DependencyResolver::check_version_requirement("2.0.1", "=2.0.0"));
    }

    #[test]
    fn test_version_requirement_greater_than() {
        // Test >= constraints
        assert!(DependencyResolver::check_version_requirement("2.0.0", ">=1.0.0"));
        assert!(DependencyResolver::check_version_requirement("1.5.0", ">=1.0.0"));
        assert!(!DependencyResolver::check_version_requirement("0.9.0", ">=1.0.0"));
    }

    #[test]
    fn test_version_validation_integration() {
        let mut resolver = DependencyResolver::new();
        resolver.add_skill("dep1", "1.5.0");
        resolver.add_skill("main", "1.0.0");

        let mut skills = HashMap::new();
        skills.insert(
            "main".to_string(),
            vec![Dependency::with_version("dep1", "^1.0.0")],
        );
        skills.insert("dep1".to_string(), vec![]);

        // Should pass - dep1@1.5.0 satisfies ^1.0.0
        assert!(resolver.validate_versions(&skills));

        // Now test with incompatible version
        let mut resolver2 = DependencyResolver::new();
        resolver2.add_skill("dep1", "2.0.0"); // Major version bump

        // Should fail - dep1@2.0.0 does not satisfy ^1.0.0
        assert!(!resolver2.validate_versions(&skills));
    }

    #[test]
    fn test_invalid_version_formats() {
        // Test invalid version strings are handled gracefully
        assert!(!DependencyResolver::check_version_requirement("not-a-version", "^1.0.0"));
        assert!(!DependencyResolver::check_version_requirement("1.0.0", "not-a-req"));
    }
}
