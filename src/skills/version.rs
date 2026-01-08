//! Semantic version management for Agent Skills

use semver::{Version, VersionReq};
use std::collections::HashMap;
use std::fmt;

/// Version compatibility check result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatibilityResult {
    /// Version is compatible
    Compatible {
        /// The version that satisfies the requirement
        version: String,
        /// The requirement that was matched
        requirement: String,
    },
    /// Version is not compatible
    Incompatible {
        /// The version that was checked
        version: String,
        /// The requirement that was not met
        requirement: String,
        /// Reason for incompatibility
        reason: String,
    },
    /// Parse error
    ParseError {
        /// The input that failed to parse
        input: String,
        /// Error message
        error: String,
    },
}

impl fmt::Display for CompatibilityResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompatibilityResult::Compatible {
                version,
                requirement,
            } => {
                write!(
                    f,
                    "✅ Version {} satisfies requirement {}",
                    version, requirement
                )
            },
            CompatibilityResult::Incompatible {
                version,
                requirement,
                reason,
            } => {
                write!(
                    f,
                    "❌ Version {} does not satisfy requirement {}: {}",
                    version, requirement, reason
                )
            },
            CompatibilityResult::ParseError { input, error } => {
                write!(f, "❌ Failed to parse '{}': {}", input, error)
            },
        }
    }
}

/// Version manager for handling semantic versioning operations
pub struct VersionManager {
    /// Available versions for each skill
    available: HashMap<String, Version>,
}

impl VersionManager {
    /// Create a new version manager
    pub fn new() -> Self {
        Self {
            available: HashMap::new(),
        }
    }

    /// Add a skill version
    pub fn add_version(
        &mut self,
        skill_id: impl Into<String>,
        version: &str,
    ) -> Result<(), String> {
        let version = Version::parse(version).map_err(|e| e.to_string())?;
        self.available.insert(skill_id.into(), version);
        Ok(())
    }

    /// Check if a version satisfies a requirement
    pub fn check_requirement(&self, version: &str, requirement: &str) -> CompatibilityResult {
        let version = match Version::parse(version) {
            Ok(v) => v,
            Err(e) => {
                return CompatibilityResult::ParseError {
                    input: version.to_string(),
                    error: e.to_string(),
                };
            },
        };

        let req = match VersionReq::parse(requirement) {
            Ok(r) => r,
            Err(e) => {
                return CompatibilityResult::ParseError {
                    input: requirement.to_string(),
                    error: e.to_string(),
                };
            },
        };

        if req.matches(&version) {
            CompatibilityResult::Compatible {
                version: version.to_string(),
                requirement: requirement.to_string(),
            }
        } else {
            CompatibilityResult::Incompatible {
                version: version.to_string(),
                requirement: requirement.to_string(),
                reason: format!(
                    "Version {} does not match requirement {}",
                    version, requirement
                ),
            }
        }
    }

    /// Find the best compatible version for a requirement
    pub fn find_compatible_version(&self, skill_id: &str, requirement: &str) -> Option<String> {
        let available_version = self.available.get(skill_id)?;

        let req = VersionReq::parse(requirement).ok()?;

        if req.matches(available_version) {
            Some(available_version.to_string())
        } else {
            None
        }
    }

    /// Compare two versions
    ///
    /// Returns:
    /// - Ok(Greater) if v1 > v2
    /// - Ok(Equal) if v1 == v2
    /// - Ok(Less) if v1 < v2
    /// - Err if parsing fails
    pub fn compare_versions(&self, v1: &str, v2: &str) -> Result<std::cmp::Ordering, String> {
        let version1 = Version::parse(v1).map_err(|e| e.to_string())?;
        let version2 = Version::parse(v2).map_err(|e| e.to_string())?;
        Ok(version1.cmp(&version2))
    }

    /// Get the latest version from a list of versions
    pub fn latest_version(&self, versions: &[String]) -> Option<String> {
        versions
            .iter()
            .filter_map(|v| Version::parse(v).ok())
            .max()
            .map(|v| v.to_string())
    }

    /// Check if an update is available for a skill
    pub fn check_update_available(&self, skill_id: &str, current: &str) -> Result<bool, String> {
        let current_version = Version::parse(current).map_err(|e| e.to_string())?;

        if let Some(available_version) = self.available.get(skill_id) {
            Ok(available_version > &current_version)
        } else {
            Err(format!(
                "No available version found for skill: {}",
                skill_id
            ))
        }
    }

    /// Validate that all dependencies have compatible versions
    pub fn validate_dependencies(
        &self,
        _skill_id: &str,
        dependencies: &[(String, String)], // (skill_id, version_requirement)
    ) -> Result<(), String> {
        for (dep_id, version_req) in dependencies {
            if let Some(available_version) = self.available.get(dep_id) {
                let req = VersionReq::parse(version_req).map_err(|e| e.to_string())?;

                if !req.matches(available_version) {
                    return Err(format!(
                        "Dependency {} version {} does not satisfy requirement {}",
                        dep_id, available_version, version_req
                    ));
                }
            } else {
                return Err(format!("Dependency {} not found", dep_id));
            }
        }

        Ok(())
    }

    /// Get all available skill versions
    pub fn available_versions(&self) -> &HashMap<String, Version> {
        &self.available
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_manager_creation() {
        let manager = VersionManager::new();
        assert_eq!(manager.available_versions().len(), 0);
    }

    #[test]
    fn test_add_version() {
        let mut manager = VersionManager::new();
        assert!(manager.add_version("skill1", "1.0.0").is_ok());
        assert!(manager.add_version("skill2", "2.1.3").is_ok());
        assert_eq!(manager.available_versions().len(), 2);
    }

    #[test]
    fn test_add_invalid_version() {
        let mut manager = VersionManager::new();
        assert!(manager.add_version("skill1", "invalid").is_err());
    }

    #[test]
    fn test_check_requirement_compatible() {
        let manager = VersionManager::new();
        let result = manager.check_requirement("1.2.3", "^1.0.0");

        match result {
            CompatibilityResult::Compatible {
                version,
                requirement,
            } => {
                assert_eq!(version, "1.2.3");
                assert_eq!(requirement, "^1.0.0");
            },
            _ => panic!("Expected compatible result"),
        }
    }

    #[test]
    fn test_check_requirement_incompatible() {
        let manager = VersionManager::new();
        let result = manager.check_requirement("2.0.0", "^1.0.0");

        match result {
            CompatibilityResult::Incompatible {
                version,
                requirement,
                ..
            } => {
                assert_eq!(version, "2.0.0");
                assert_eq!(requirement, "^1.0.0");
            },
            _ => panic!("Expected incompatible result"),
        }
    }

    #[test]
    fn test_check_requirement_invalid() {
        let manager = VersionManager::new();
        let result = manager.check_requirement("invalid", "^1.0.0");

        match result {
            CompatibilityResult::ParseError { input, .. } => {
                assert_eq!(input, "invalid");
            },
            _ => panic!("Expected parse error"),
        }
    }

    #[test]
    fn test_find_compatible_version() {
        let mut manager = VersionManager::new();
        manager.add_version("skill1", "1.5.0").unwrap();

        assert_eq!(
            manager.find_compatible_version("skill1", "^1.0.0"),
            Some("1.5.0".to_string())
        );
        assert_eq!(manager.find_compatible_version("skill1", "^2.0.0"), None);
        assert_eq!(manager.find_compatible_version("skill2", "^1.0.0"), None);
    }

    #[test]
    fn test_compare_versions() {
        let manager = VersionManager::new();

        assert_eq!(
            manager.compare_versions("2.0.0", "1.0.0").unwrap(),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            manager.compare_versions("1.0.0", "2.0.0").unwrap(),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            manager.compare_versions("1.0.0", "1.0.0").unwrap(),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_latest_version() {
        let manager = VersionManager::new();
        let versions = vec![
            "1.0.0".to_string(),
            "2.0.0".to_string(),
            "1.5.0".to_string(),
        ];

        assert_eq!(manager.latest_version(&versions), Some("2.0.0".to_string()));
    }

    #[test]
    fn test_latest_version_with_invalid() {
        let manager = VersionManager::new();
        let versions = vec![
            "1.0.0".to_string(),
            "invalid".to_string(),
            "2.0.0".to_string(),
        ];

        assert_eq!(manager.latest_version(&versions), Some("2.0.0".to_string()));
    }

    #[test]
    fn test_check_update_available() {
        let mut manager = VersionManager::new();
        manager.add_version("skill1", "2.0.0").unwrap();

        assert!(manager.check_update_available("skill1", "1.0.0").unwrap());
        assert!(!manager.check_update_available("skill1", "2.0.0").unwrap());
        assert!(!manager.check_update_available("skill1", "3.0.0").unwrap());
    }

    #[test]
    fn test_validate_dependencies() {
        let mut manager = VersionManager::new();
        manager.add_version("dep1", "1.5.0").unwrap();
        manager.add_version("dep2", "2.0.0").unwrap();

        let deps = vec![
            ("dep1".to_string(), "^1.0.0".to_string()),
            ("dep2".to_string(), "^2.0.0".to_string()),
        ];

        assert!(manager.validate_dependencies("skill1", &deps).is_ok());

        let incompatible_deps = vec![("dep1".to_string(), "^2.0.0".to_string())];
        assert!(
            manager
                .validate_dependencies("skill1", &incompatible_deps)
                .is_err()
        );
    }

    #[test]
    fn test_compatibility_result_display() {
        let compatible = CompatibilityResult::Compatible {
            version: "1.0.0".to_string(),
            requirement: "^1.0.0".to_string(),
        };
        assert!(compatible.to_string().contains("✅"));

        let incompatible = CompatibilityResult::Incompatible {
            version: "2.0.0".to_string(),
            requirement: "^1.0.0".to_string(),
            reason: "version mismatch".to_string(),
        };
        assert!(incompatible.to_string().contains("❌"));

        let parse_error = CompatibilityResult::ParseError {
            input: "invalid".to_string(),
            error: "parse error".to_string(),
        };
        assert!(parse_error.to_string().contains("❌"));
    }

    #[test]
    fn test_prerelease_versions() {
        let manager = VersionManager::new();

        // Prerelease versions should be less than release versions
        assert_eq!(
            manager.compare_versions("1.0.0-alpha", "1.0.0").unwrap(),
            std::cmp::Ordering::Less
        );

        // Prerelease comparison
        assert_eq!(
            manager
                .compare_versions("1.0.0-alpha", "1.0.0-beta")
                .unwrap(),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn test_complex_version_requirements() {
        let manager = VersionManager::new();

        // Test caret requirement (^1.2.3 = >=1.2.3 <2.0.0)
        assert!(matches!(
            manager.check_requirement("1.5.0", "^1.2.3"),
            CompatibilityResult::Compatible { .. }
        ));
        assert!(matches!(
            manager.check_requirement("2.0.0", "^1.2.3"),
            CompatibilityResult::Incompatible { .. }
        ));

        // Test tilde requirement (~1.2.3 = >=1.2.3 <1.3.0)
        assert!(matches!(
            manager.check_requirement("1.2.5", "~1.2.3"),
            CompatibilityResult::Compatible { .. }
        ));
        assert!(matches!(
            manager.check_requirement("1.3.0", "~1.2.3"),
            CompatibilityResult::Incompatible { .. }
        ));

        // Test wildcard requirement (*)
        assert!(matches!(
            manager.check_requirement("1.0.0", "*"),
            CompatibilityResult::Compatible { .. }
        ));

        // Test multiple requirements (>=1.0.0, <2.0.0)
        assert!(matches!(
            manager.check_requirement("1.5.0", ">=1.0.0, <2.0.0"),
            CompatibilityResult::Compatible { .. }
        ));
    }
}
