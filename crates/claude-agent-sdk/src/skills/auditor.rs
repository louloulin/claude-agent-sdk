//! # Skills Security Auditor
//!
//! This module provides security auditing capabilities for agent skills,
//! detecting potentially dangerous patterns in skill code and configurations.
//!
//! ## Features
//!
//! - Network access detection
//! - Dangerous command detection (eval, exec, system)
//! - File access pattern analysis
//! - Risk level assessment
//!
//! ## Example
//!
//! ```no_run
//! use claude_agent_sdk_rs::skills::auditor::{SkillAuditor, AuditConfig};
//! use claude_agent_sdk_rs::skills::skill_md::SkillMdFile;
//!
//! let config = AuditConfig {
//!     strict_mode: true,
//!     allow_network: false,
//! };
//!
//! let auditor = SkillAuditor::new(config);
//! let skill = SkillMdFile::parse("path/to/SKILL.md")?;
//! let report = auditor.audit(&skill)?;
//!
//! if report.risk_level == RiskLevel::Safe {
//!     println!("Skill is safe to use");
//! } else {
//!     println!("Skill has risks: {:?}", report.issues);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::skills::skill_md::SkillMdFile;
use std::path::Path;

/// Risk level for a skill
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum RiskLevel {
    /// Safe - only from trusted sources
    #[default]
    Safe,
    /// Low - minor issues
    Low,
    /// Medium - needs review
    Medium,
    /// High - dangerous, should not run
    High,
    /// Critical - malicious, block execution
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Safe => write!(f, "Safe"),
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::Critical => write!(f, "Critical"),
        }
    }
}

/// Individual security issue found during audit
#[derive(Debug, Clone)]
pub struct SkillAuditIssue {
    /// Type of issue
    pub issue_type: IssueType,
    /// Severity level
    pub severity: RiskLevel,
    /// File where issue was found (if applicable)
    pub file: Option<String>,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Description of the issue
    pub message: String,
    /// Code snippet that triggered the issue
    pub snippet: Option<String>,
}

/// Type of security issue
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueType {
    /// Network access detected
    NetworkAccess,
    /// Dangerous command (eval, exec, system)
    DangerousCommand,
    /// File system access
    FileAccess,
    /// Code execution
    CodeExecution,
    /// External command execution
    ExternalCommand,
    /// Sensitive data access
    SensitiveDataAccess,
    /// Other security concern
    Other,
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueType::NetworkAccess => write!(f, "Network Access"),
            IssueType::DangerousCommand => write!(f, "Dangerous Command"),
            IssueType::FileAccess => write!(f, "File Access"),
            IssueType::CodeExecution => write!(f, "Code Execution"),
            IssueType::ExternalCommand => write!(f, "External Command"),
            IssueType::SensitiveDataAccess => write!(f, "Sensitive Data Access"),
            IssueType::Other => write!(f, "Other"),
        }
    }
}

/// Audit report for a skill
#[derive(Debug, Clone, Default)]
pub struct SkillAuditReport {
    /// Overall safety assessment
    pub safe: bool,
    /// All issues found
    pub issues: Vec<SkillAuditIssue>,
    /// Warnings (non-critical issues)
    pub warnings: Vec<String>,
    /// Errors (critical issues)
    pub errors: Vec<String>,
    /// Overall risk level
    pub risk_level: RiskLevel,
    /// Number of files scanned
    pub files_scanned: usize,
}

impl SkillAuditReport {
    /// Check if the report has any issues
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// Get issues by severity
    pub fn issues_by_severity(&self, severity: RiskLevel) -> Vec<&SkillAuditIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == severity)
            .collect()
    }

    /// Get all high and critical issues
    pub fn critical_issues(&self) -> Vec<&SkillAuditIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity >= RiskLevel::High)
            .collect()
    }
}

/// Configuration for skill auditing
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Enable strict mode (more conservative checks)
    pub strict_mode: bool,
    /// Allow network access (otherwise treats as high risk)
    pub allow_network: bool,
    /// Check scripts directory
    pub check_scripts: bool,
    /// Check resource files
    pub check_resources: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            allow_network: false,
            check_scripts: true,
            check_resources: true,
        }
    }
}

/// Skills security auditor
pub struct SkillAuditor {
    config: AuditConfig,
}

impl SkillAuditor {
    /// Create a new auditor with the given configuration
    pub fn new(config: AuditConfig) -> Self {
        Self { config }
    }

    /// Create an auditor with default configuration
    pub fn default_auditor() -> Self {
        Self::new(AuditConfig::default())
    }

    /// Audit a skill and return a security report
    pub fn audit(&self, skill: &SkillMdFile) -> Result<SkillAuditReport, AuditError> {
        let mut report = SkillAuditReport::default();

        // Scan main SKILL.md content
        self.check_content(&skill.content, "SKILL.md", &mut report);

        // Scan scripts if enabled
        if self.config.check_scripts {
            for script_path in &skill.scripts {
                if let Some(content) = self.read_file(script_path) {
                    self.check_script(
                        &content,
                        script_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown"),
                        &mut report,
                    );
                    report.files_scanned += 1;
                }
            }
        }

        // Scan resource files if enabled
        if self.config.check_resources {
            for resource_path in &skill.resources {
                if let Some(content) = self.read_file(resource_path) {
                    self.check_content(
                        &content,
                        resource_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown"),
                        &mut report,
                    );
                    report.files_scanned += 1;
                }
            }
        }

        // Calculate overall risk level
        report.risk_level = self.calculate_risk_level(&report);
        report.safe = report.risk_level <= RiskLevel::Low;

        Ok(report)
    }

    /// Read file content safely
    fn read_file(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    /// Calculate overall risk level from issues
    fn calculate_risk_level(&self, report: &SkillAuditReport) -> RiskLevel {
        if report.issues.is_empty() {
            return RiskLevel::Safe;
        }

        // Find the highest severity issue
        report
            .issues
            .iter()
            .map(|issue| issue.severity)
            .max()
            .unwrap_or(RiskLevel::Safe)
    }

    /// Check main markdown content
    fn check_content(&self, content: &str, file: &str, report: &mut SkillAuditReport) {
        // Check for network patterns in code blocks
        self.check_network_access(content, file, report);

        // Check for dangerous commands
        self.check_dangerous_commands(content, file, report);

        // Check for file access patterns
        self.check_file_access_patterns(content, file, report);
    }

    /// Check script file
    fn check_script(&self, content: &str, file: &str, report: &mut SkillAuditReport) {
        // Scripts get more stringent checks
        self.check_network_access(content, file, report);
        self.check_dangerous_commands(content, file, report);
        self.check_file_access_patterns(content, file, report);
        self.check_code_execution(content, file, report);
    }

    /// Check for network access patterns
    fn check_network_access(&self, content: &str, file: &str, report: &mut SkillAuditReport) {
        let patterns = [
            "http://",
            "https://",
            "ftp://",
            "requests.",
            "urllib.",
            "fetch(",
            "axios.",
            "socket.",
            "connect(",
            "wget",
            "curl",
        ];

        let mut found_patterns = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &patterns {
                if line.contains(pattern) {
                    let severity = if self.config.allow_network {
                        RiskLevel::Low
                    } else {
                        RiskLevel::Medium
                    };

                    found_patterns.push((line_num + 1, pattern, severity));
                }
            }
        }

        for (line_num, pattern, severity) in found_patterns {
            report.issues.push(SkillAuditIssue {
                issue_type: IssueType::NetworkAccess,
                severity,
                file: Some(file.to_string()),
                line: Some(line_num),
                message: format!("Network access pattern detected: {}", pattern),
                snippet: self.get_line_snippet(content, line_num),
            });
        }
    }

    /// Check for dangerous commands
    fn check_dangerous_commands(&self, content: &str, file: &str, report: &mut SkillAuditReport) {
        let dangerous_patterns = [
            ("eval(", RiskLevel::High),
            ("exec(", RiskLevel::High),
            ("system(", RiskLevel::High),
            ("subprocess.call", RiskLevel::Medium),
            ("subprocess.Popen", RiskLevel::Medium),
            ("os.system", RiskLevel::High),
            ("child_process", RiskLevel::Medium),
            ("spawn(", RiskLevel::High),
            ("Runtime.exec", RiskLevel::High),
            ("ProcessBuilder", RiskLevel::Medium),
        ];

        for (line_num, line) in content.lines().enumerate() {
            for (pattern, severity) in &dangerous_patterns {
                if line.contains(pattern) {
                    report.issues.push(SkillAuditIssue {
                        issue_type: IssueType::DangerousCommand,
                        severity: *severity,
                        file: Some(file.to_string()),
                        line: Some(line_num + 1),
                        message: format!("Dangerous command detected: {}", pattern),
                        snippet: Some(line.to_string()),
                    });
                }
            }
        }
    }

    /// Check for file access patterns
    fn check_file_access_patterns(&self, content: &str, file: &str, report: &mut SkillAuditReport) {
        let file_patterns = [
            ("open(", RiskLevel::Low),
            ("File.open", RiskLevel::Low),
            ("fs.readFile", RiskLevel::Low),
            ("fs.writeFile", RiskLevel::Low),
            ("Path.", RiskLevel::Low),
            ("/etc/", RiskLevel::Medium),
            ("/home/", RiskLevel::Medium),
            ("C:\\\\", RiskLevel::Medium),
        ];

        for (line_num, line) in content.lines().enumerate() {
            for (pattern, severity) in &file_patterns {
                if line.contains(pattern) {
                    report.issues.push(SkillAuditIssue {
                        issue_type: IssueType::FileAccess,
                        severity: *severity,
                        file: Some(file.to_string()),
                        line: Some(line_num + 1),
                        message: format!("File access pattern detected: {}", pattern),
                        snippet: Some(line.to_string()),
                    });
                }
            }
        }
    }

    /// Check for code execution patterns
    fn check_code_execution(&self, content: &str, file: &str, report: &mut SkillAuditReport) {
        let code_exec_patterns = [
            "compile(",
            "execfile(",
            "__import__",
            "importlib.",
            "getattr(__builtins__",
        ];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &code_exec_patterns {
                if line.contains(pattern) {
                    report.issues.push(SkillAuditIssue {
                        issue_type: IssueType::CodeExecution,
                        severity: RiskLevel::High,
                        file: Some(file.to_string()),
                        line: Some(line_num + 1),
                        message: format!("Code execution pattern detected: {}", pattern),
                        snippet: Some(line.to_string()),
                    });
                }
            }
        }
    }

    /// Get a snippet of the line for context
    fn get_line_snippet(&self, content: &str, line_num: usize) -> Option<String> {
        content
            .lines()
            .nth(line_num.saturating_sub(1))
            .map(|s| s.to_string())
    }
}

/// Errors that can occur during auditing
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to read skill content: {0}")]
    ReadError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_skill(content: &str) -> (TempDir, SkillMdFile) {
        let temp_dir = TempDir::new().unwrap();
        let skill_md = temp_dir.path().join("SKILL.md");

        let skill_content = format!(
            r#"---
name: test-skill
description: Test skill for auditing
---

# Test Skill

{}
"#,
            content
        );

        fs::write(&skill_md, skill_content).unwrap();

        let skill = SkillMdFile::parse(&skill_md).unwrap();
        (temp_dir, skill)
    }

    #[test]
    fn test_audit_safe_skill() {
        let content = r#"
This is a safe skill with no dangerous patterns.

## Features

- Simple text processing
- Data validation
"#;

        let (_temp, skill) = create_test_skill(content);
        let auditor = SkillAuditor::default_auditor();
        let report = auditor.audit(&skill).unwrap();

        assert!(report.safe);
        assert_eq!(report.risk_level, RiskLevel::Safe);
        assert!(!report.has_issues());
    }

    #[test]
    fn test_audit_network_access() {
        let content = r#"
## Fetch Data

```python
import requests
response = requests.get("https://api.example.com/data")
```
"#;

        let (_temp, skill) = create_test_skill(content);
        let config = AuditConfig {
            allow_network: false,
            ..Default::default()
        };
        let auditor = SkillAuditor::new(config);
        let report = auditor.audit(&skill).unwrap();

        assert!(!report.safe);
        assert_eq!(report.risk_level, RiskLevel::Medium);
        assert!(report.has_issues());

        // Check that network access was detected
        let network_issues: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.issue_type == IssueType::NetworkAccess)
            .collect();

        assert!(!network_issues.is_empty());
    }

    #[test]
    fn test_audit_dangerous_commands() {
        let content = r#"
## Execute Command

```python
import os
os.system("rm -rf /")
```
"#;

        let (_temp, skill) = create_test_skill(content);
        let auditor = SkillAuditor::default_auditor();
        let report = auditor.audit(&skill).unwrap();

        assert!(!report.safe);
        assert!(report.has_issues());

        // Check that dangerous command was detected
        let dangerous_issues: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.issue_type == IssueType::DangerousCommand)
            .collect();

        assert!(!dangerous_issues.is_empty());
        assert_eq!(dangerous_issues[0].severity, RiskLevel::High);
    }

    #[test]
    fn test_audit_file_access() {
        let content = r#"
## Read File

```python
with open("/etc/passwd", "r") as f:
    content = f.read()
```
"#;

        let (_temp, skill) = create_test_skill(content);
        let auditor = SkillAuditor::default_auditor();
        let report = auditor.audit(&skill).unwrap();

        assert!(report.has_issues());

        // Check that file access was detected
        let file_issues: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.issue_type == IssueType::FileAccess)
            .collect();

        assert!(!file_issues.is_empty());
    }

    #[test]
    fn test_audit_multiple_issues() {
        let content = r#"
## Multi-issue Skill

```python
import os
import requests

os.system("ls")
requests.get("https://example.com")

with open("/etc/passwd", "r") as f:
    pass
```
"#;

        let (_temp, skill) = create_test_skill(content);
        let auditor = SkillAuditor::default_auditor();
        let report = auditor.audit(&skill).unwrap();

        assert!(!report.safe);
        assert!(report.has_issues());

        // Should have multiple issues
        assert!(report.issues.len() >= 3);
    }

    #[test]
    fn test_audit_strict_mode() {
        let content = r#"
## Read File

```python
with open("data.txt", "r") as f:
    content = f.read()
```
"#;

        let (_temp, skill) = create_test_skill(content);

        // Non-strict mode should allow this (low risk)
        let config = AuditConfig {
            strict_mode: false,
            ..Default::default()
        };
        let auditor = SkillAuditor::new(config);
        let report = auditor.audit(&skill).unwrap();

        assert!(report.has_issues());
        assert_eq!(report.risk_level, RiskLevel::Low);

        // Strict mode might have different behavior
        // For now, strict mode mainly affects future extensibility
    }

    #[test]
    fn test_audit_critical_issues() {
        let content = r#"
## Critical Issues

```python
eval(__import__('os').system('rm -rf /'))
```
"#;

        let (_temp, skill) = create_test_skill(content);
        let auditor = SkillAuditor::default_auditor();
        let report = auditor.audit(&skill).unwrap();

        assert!(!report.safe);
        assert!(report.has_issues());

        // Check that we have critical issues
        let critical = report.critical_issues();
        assert!(!critical.is_empty());
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Safe < RiskLevel::Low);
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn test_issue_type_display() {
        assert_eq!(IssueType::NetworkAccess.to_string(), "Network Access");
        assert_eq!(IssueType::DangerousCommand.to_string(), "Dangerous Command");
        assert_eq!(IssueType::FileAccess.to_string(), "File Access");
    }

    #[test]
    fn test_risk_level_display() {
        assert_eq!(RiskLevel::Safe.to_string(), "Safe");
        assert_eq!(RiskLevel::Critical.to_string(), "Critical");
    }
}
