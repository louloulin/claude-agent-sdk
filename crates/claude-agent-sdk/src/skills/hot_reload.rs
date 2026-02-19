//! Hot reloading support for Agent Skills
//!
//! This module provides file system monitoring capabilities to automatically
//! reload skill configurations when they change on disk.

use crate::skills::{SkillError, SkillPackage};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn};

#[cfg(feature = "hot-reload")]
use tracing::{debug, error};

/// Configuration for hot reload behavior
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// Debounce duration to avoid rapid reloads (default: 100ms)
    pub debounce_duration: Duration,
    /// Whether to recursively watch subdirectories
    pub recursive: bool,
    /// File patterns to watch (e.g., ["*.yaml", "*.json"])
    pub file_patterns: Vec<String>,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            debounce_duration: Duration::from_millis(100),
            recursive: true,
            file_patterns: vec!["*.yaml".to_string(), "*.json".to_string()],
        }
    }
}

/// Event emitted when a skill file changes
#[derive(Debug, Clone)]
pub enum HotReloadEvent {
    /// A skill was created
    SkillCreated { path: PathBuf, skill: SkillPackage },
    /// A skill was modified
    SkillModified { path: PathBuf, skill: SkillPackage },
    /// A skill was deleted
    SkillDeleted { path: PathBuf },
    /// An error occurred
    Error { path: PathBuf, error: String },
}

/// Hot reload watcher for skill files
#[cfg(feature = "hot-reload")]
pub struct HotReloadWatcher {
    #[allow(dead_code)]
    config: HotReloadConfig,
    #[allow(dead_code)]
    event_sender: mpsc::UnboundedSender<HotReloadEvent>,
    _watcher: notify::RecommendedWatcher,
}

#[cfg(feature = "hot-reload")]
impl HotReloadWatcher {
    /// Create a new hot reload watcher
    ///
    /// # Arguments
    /// * `watch_path` - Directory to watch for changes
    /// * `config` - Configuration for the watcher
    /// * `event_sender` - Channel to send events through
    ///
    /// # Returns
    /// A new watcher or an error if setup fails
    pub fn new(
        watch_path: impl AsRef<Path>,
        config: HotReloadConfig,
        event_sender: mpsc::UnboundedSender<HotReloadEvent>,
    ) -> Result<Self, SkillError> {
        use notify::Watcher;

        let watch_path = watch_path.as_ref();

        if !watch_path.exists() {
            return Err(SkillError::Configuration(format!(
                "Watch path does not exist: {:?}",
                watch_path
            )));
        }

        let sender_clone = event_sender.clone();
        let file_patterns = config.file_patterns.clone();

        let mut watcher = notify::recommended_watcher(
            move |result: notify::Result<notify::Event>| match result {
                Ok(event) => {
                    Self::handle_event(event, &sender_clone, &file_patterns);
                },
                Err(e) => {
                    error!("Hot reload error: {:?}", e);
                },
            },
        )
        .map_err(|e| SkillError::Configuration(format!("Failed to create watcher: {}", e)))?;

        watcher
            .watch(watch_path, notify::RecursiveMode::Recursive)
            .map_err(|e| SkillError::Configuration(format!("Failed to watch path: {}", e)))?;

        info!(
            "Hot reload watcher started for: {:?} (patterns: {:?})",
            watch_path, config.file_patterns
        );

        Ok(Self {
            config,
            event_sender,
            _watcher: watcher,
        })
    }

    /// Handle a single file system event
    fn handle_event(
        event: notify::Event,
        sender: &mpsc::UnboundedSender<HotReloadEvent>,
        patterns: &[String],
    ) {
        use notify::EventKind;

        // Get the first path from the event
        let path = match event.paths.first() {
            Some(p) => p,
            None => return,
        };

        // Skip if not a file
        if !path.is_file() {
            return;
        }

        // Check file pattern
        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => return,
        };

        let matches_pattern = patterns
            .iter()
            .any(|pattern| match pattern.strip_prefix('*') {
                Some(ext) => file_name.ends_with(ext),
                None => file_name == *pattern,
            });

        if !matches_pattern {
            debug!("Skipping file (pattern mismatch): {:?}", path);
            return;
        }

        debug!("File event: kind={:?}, path={:?}", event.kind, path);

        // Handle different event kinds
        match event.kind {
            EventKind::Create(_) => {
                Self::load_and_send_event(path, sender, |path, skill| {
                    HotReloadEvent::SkillCreated { path, skill }
                });
            },
            EventKind::Modify(_) => {
                Self::load_and_send_event(path, sender, |path, skill| {
                    HotReloadEvent::SkillModified { path, skill }
                });
            },
            EventKind::Remove(_) => {
                let _ = sender.send(HotReloadEvent::SkillDeleted { path: path.clone() });
            },
            _ => {},
        }
    }

    /// Load a skill file and send the appropriate event
    fn load_and_send_event(
        path: &Path,
        sender: &mpsc::UnboundedSender<HotReloadEvent>,
        event_builder: impl FnOnce(PathBuf, SkillPackage) -> HotReloadEvent,
    ) {
        let skill = match Self::load_skill(path) {
            Ok(skill) => skill,
            Err(e) => {
                let _ = sender.send(HotReloadEvent::Error {
                    path: path.to_path_buf(),
                    error: e.to_string(),
                });
                return;
            },
        };

        let _ = sender.send(event_builder(path.to_path_buf(), skill));
    }

    /// Load a skill from a file (supports both JSON and YAML)
    fn load_skill(path: &Path) -> Result<SkillPackage, SkillError> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| SkillError::Configuration("No file extension".to_string()))?;

        match extension {
            "json" => SkillPackage::load_from_file(path)
                .map_err(|e| SkillError::Io(format!("Failed to load JSON: {}", e))),
            "yaml" | "yml" => {
                #[cfg(feature = "yaml")]
                {
                    SkillPackage::load_from_yaml(path)
                        .map_err(|e| SkillError::Io(format!("Failed to load YAML: {}", e)))
                }
                #[cfg(not(feature = "yaml"))]
                {
                    Err(SkillError::Configuration(
                        "YAML support not enabled".to_string(),
                    ))
                }
            },
            _ => Err(SkillError::Configuration(format!(
                "Unsupported file type: {}",
                extension
            ))),
        }
    }
}

/// Simple hot reload stub without the watcher
#[cfg(not(feature = "hot-reload"))]
pub struct HotReloadWatcher {
    _config: HotReloadConfig,
    _event_sender: mpsc::UnboundedSender<HotReloadEvent>,
}

#[cfg(not(feature = "hot-reload"))]
impl HotReloadWatcher {
    pub fn new(
        _watch_path: impl AsRef<Path>,
        _config: HotReloadConfig,
        _event_sender: mpsc::UnboundedSender<HotReloadEvent>,
    ) -> Result<Self, SkillError> {
        Err(SkillError::Configuration(
            "Hot reload feature not enabled. Enable with --features hot-reload".to_string(),
        ))
    }
}

/// Manages hot reloading for multiple skill files
pub struct HotReloadManager {
    event_receiver: mpsc::UnboundedReceiver<HotReloadEvent>,
    skills: std::collections::HashMap<PathBuf, SkillPackage>,
}

impl HotReloadManager {
    /// Create a new hot reload manager
    pub fn new(event_receiver: mpsc::UnboundedReceiver<HotReloadEvent>) -> Self {
        Self {
            event_receiver,
            skills: std::collections::HashMap::new(),
        }
    }

    /// Get all currently loaded skills
    pub fn get_skills(&self) -> Vec<&SkillPackage> {
        self.skills.values().collect()
    }

    /// Get a skill by path
    pub fn get_skill(&self, path: &Path) -> Option<&SkillPackage> {
        self.skills.get(path)
    }

    /// Process all pending events
    ///
    /// Returns the number of events processed
    pub fn process_events(&mut self) -> usize {
        let mut count = 0;
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(event);
            count += 1;
        }
        count
    }

    /// Handle a single hot reload event
    fn handle_event(&mut self, event: HotReloadEvent) {
        match event {
            HotReloadEvent::SkillCreated { path, skill } => {
                info!("Skill created: {:?}", path);
                self.skills.insert(path, skill);
            },
            HotReloadEvent::SkillModified { path, skill } => {
                info!("Skill modified: {:?}", path);
                self.skills.insert(path, skill);
            },
            HotReloadEvent::SkillDeleted { path } => {
                info!("Skill deleted: {:?}", path);
                self.skills.remove(&path);
            },
            HotReloadEvent::Error { path, error } => {
                warn!("Skill error at {:?}: {}", path, error);
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reload_config_default() {
        let config = HotReloadConfig::default();
        assert_eq!(config.debounce_duration, Duration::from_millis(100));
        assert!(config.recursive);
        assert_eq!(config.file_patterns.len(), 2);
    }

    #[test]
    fn test_hot_reload_config_custom() {
        let config = HotReloadConfig {
            debounce_duration: Duration::from_millis(200),
            recursive: false,
            file_patterns: vec!["*.json".to_string()],
        };
        assert_eq!(config.debounce_duration, Duration::from_millis(200));
        assert!(!config.recursive);
        assert_eq!(config.file_patterns.len(), 1);
    }

    #[test]
    fn test_hot_reload_manager_creation() {
        let (_sender, receiver) = mpsc::unbounded_channel();
        let manager = HotReloadManager::new(receiver);
        assert_eq!(manager.get_skills().len(), 0);
    }

    #[test]
    fn test_hot_reload_manager_no_events() {
        let (_sender, receiver) = mpsc::unbounded_channel();
        let mut manager = HotReloadManager::new(receiver);
        let count = manager.process_events();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_hot_reload_event_send() {
        let (sender, receiver) = mpsc::unbounded_channel();

        // Send a test event
        let event = HotReloadEvent::SkillDeleted {
            path: PathBuf::from("/test/skill.json"),
        };
        sender.send(event).unwrap();

        let mut manager = HotReloadManager::new(receiver);
        let count = manager.process_events();
        assert_eq!(count, 1);
    }
}
