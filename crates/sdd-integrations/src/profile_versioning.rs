use crate::model_profile::ModelProfile;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileVersion {
    pub profile_id: String,
    pub version: String,
    pub profile: ModelProfile,
    pub changelog: Vec<VersionChange>,
    pub released_at: DateTime<Utc>,
    pub deprecated: bool,
    pub deprecated_at: Option<DateTime<Utc>>,
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChange {
    pub change_type: ChangeType,
    pub description: String,
    pub breaking: bool,
    pub migration_guide: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Added,
    Changed,
    Deprecated,
    Removed,
    Fixed,
    Security,
}

#[derive(Debug, Clone)]
pub struct ProfileVersionManager {
    versions: HashMap<String, Vec<ProfileVersion>>,
}

impl ProfileVersionManager {
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }

    pub fn register_version(&mut self, version: ProfileVersion) {
        let profile_id = version.profile_id.clone();
        self.versions.entry(profile_id).or_default().push(version);
    }

    pub fn get_version(&self, profile_id: &str, version: &str) -> Option<&ProfileVersion> {
        self.versions
            .get(profile_id)
            .and_then(|versions| versions.iter().find(|v| v.version == version))
    }

    pub fn get_latest_version(&self, profile_id: &str) -> Option<&ProfileVersion> {
        self.versions
            .get(profile_id)
            .and_then(|versions| versions.last())
    }

    pub fn get_all_versions(&self, profile_id: &str) -> Vec<&ProfileVersion> {
        self.versions
            .get(profile_id)
            .map(|versions| versions.iter().collect())
            .unwrap_or_default()
    }

    pub fn deprecate_version(
        &mut self,
        profile_id: &str,
        version: &str,
        reason: &str,
    ) -> Result<(), VersionError> {
        let versions = self
            .versions
            .get_mut(profile_id)
            .ok_or(VersionError::ProfileNotFound(profile_id.to_string()))?;

        let profile_version = versions
            .iter_mut()
            .find(|v| v.version == version)
            .ok_or(VersionError::VersionNotFound(version.to_string()))?;

        profile_version.deprecated = true;
        profile_version.deprecated_at = Some(Utc::now());
        profile_version.deprecation_reason = Some(reason.to_string());

        Ok(())
    }

    pub fn get_active_versions(&self, profile_id: &str) -> Vec<&ProfileVersion> {
        self.versions
            .get(profile_id)
            .map(|versions| versions.iter().filter(|v| !v.deprecated).collect())
            .unwrap_or_default()
    }

    pub fn has_breaking_changes(
        &self,
        profile_id: &str,
        from_version: &str,
        to_version: &str,
    ) -> bool {
        let _from = match self.get_version(profile_id, from_version) {
            Some(v) => v,
            None => return false,
        };

        let to = match self.get_version(profile_id, to_version) {
            Some(v) => v,
            None => return false,
        };

        to.changelog.iter().any(|change| change.breaking)
    }

    pub fn get_migration_guide(
        &self,
        profile_id: &str,
        _from_version: &str,
        to_version: &str,
    ) -> Vec<String> {
        let to = match self.get_version(profile_id, to_version) {
            Some(v) => v,
            None => return Vec::new(),
        };

        to.changelog
            .iter()
            .filter(|change| change.breaking)
            .filter_map(|change| change.migration_guide.clone())
            .collect()
    }
}

impl Default for ProfileVersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("profile not found: {0}")]
    ProfileNotFound(String),
    #[error("version not found: {0}")]
    VersionNotFound(String),
    #[error("invalid version format: {0}")]
    InvalidVersion(String),
}

impl ProfileVersion {
    pub fn new(
        profile_id: impl Into<String>,
        version: impl Into<String>,
        profile: ModelProfile,
    ) -> Self {
        Self {
            profile_id: profile_id.into(),
            version: version.into(),
            profile,
            changelog: Vec::new(),
            released_at: Utc::now(),
            deprecated: false,
            deprecated_at: None,
            deprecation_reason: None,
        }
    }

    pub fn with_changelog(mut self, changelog: Vec<VersionChange>) -> Self {
        self.changelog = changelog;
        self
    }

    pub fn add_change(mut self, change: VersionChange) -> Self {
        self.changelog.push(change);
        self
    }
}

impl VersionChange {
    pub fn new(change_type: ChangeType, description: impl Into<String>) -> Self {
        Self {
            change_type,
            description: description.into(),
            breaking: false,
            migration_guide: None,
        }
    }

    pub fn breaking(mut self) -> Self {
        self.breaking = true;
        self
    }

    pub fn with_migration_guide(mut self, guide: impl Into<String>) -> Self {
        self.migration_guide = Some(guide.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_profile::{ModelProfile, ModelProvider};

    #[test]
    fn test_profile_version_creation() {
        let profile = ModelProfile::new(
            "test",
            "Test",
            "1.0.0",
            ModelProvider::Custom("test".to_string()),
        );
        let version = ProfileVersion::new("test", "1.0.0", profile);

        assert_eq!(version.profile_id, "test");
        assert_eq!(version.version, "1.0.0");
        assert!(!version.deprecated);
    }

    #[test]
    fn test_version_manager() {
        let mut manager = ProfileVersionManager::new();
        let profile = ModelProfile::new(
            "test",
            "Test",
            "1.0.0",
            ModelProvider::Custom("test".to_string()),
        );
        let version = ProfileVersion::new("test", "1.0.0", profile);

        manager.register_version(version);

        assert!(manager.get_version("test", "1.0.0").is_some());
        assert!(manager.get_latest_version("test").is_some());
    }

    #[test]
    fn test_deprecation() {
        let mut manager = ProfileVersionManager::new();
        let profile = ModelProfile::new(
            "test",
            "Test",
            "1.0.0",
            ModelProvider::Custom("test".to_string()),
        );
        let version = ProfileVersion::new("test", "1.0.0", profile);

        manager.register_version(version);
        manager
            .deprecate_version("test", "1.0.0", "Superseded by 2.0.0")
            .unwrap();

        let deprecated = manager.get_version("test", "1.0.0").unwrap();
        assert!(deprecated.deprecated);
        assert!(deprecated.deprecation_reason.is_some());
    }

    #[test]
    fn test_breaking_changes() {
        let mut manager = ProfileVersionManager::new();

        let profile_v1 = ModelProfile::new(
            "test",
            "Test",
            "1.0.0",
            ModelProvider::Custom("test".to_string()),
        );
        let version_v1 = ProfileVersion::new("test", "1.0.0", profile_v1);

        let profile_v2 = ModelProfile::new(
            "test",
            "Test",
            "2.0.0",
            ModelProvider::Custom("test".to_string()),
        );
        let version_v2 = ProfileVersion::new("test", "2.0.0", profile_v2)
            .add_change(VersionChange::new(ChangeType::Removed, "Removed legacy API").breaking());

        manager.register_version(version_v1);
        manager.register_version(version_v2);

        assert!(manager.has_breaking_changes("test", "1.0.0", "2.0.0"));
    }
}
