use crate::context_bundle::{ContextBundle, ContextError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedBundle {
    pub bundle: ContextBundle,
    pub cache_key: String,
    pub cached_at: DateTime<Utc>,
    pub access_count: usize,
    pub last_accessed: DateTime<Utc>,
    pub dependencies: Vec<String>,
}

impl CachedBundle {
    pub fn new(bundle: ContextBundle, dependencies: Vec<String>) -> Self {
        let cache_key = Self::compute_cache_key(&bundle);
        let now = Utc::now();

        Self {
            bundle,
            cache_key,
            cached_at: now,
            access_count: 0,
            last_accessed: now,
            dependencies,
        }
    }

    pub fn compute_cache_key(bundle: &ContextBundle) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bundle.task_id.as_bytes());
        hasher.update(bundle.hash.as_bytes());
        for fragment in &bundle.fragments {
            hasher.update(fragment.hash.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now();
    }

    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        let age = Utc::now().signed_duration_since(self.cached_at);
        age.num_seconds() > max_age_seconds as i64
    }
}

pub struct ContextCache {
    cache_dir: PathBuf,
    entries: HashMap<String, CachedBundle>,
    max_entries: usize,
    max_age_seconds: u64,
}

impl ContextCache {
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        Self {
            cache_dir: cache_dir.as_ref().to_path_buf(),
            entries: HashMap::new(),
            max_entries: 1000,
            max_age_seconds: 3600,
        }
    }

    pub fn with_limits(mut self, max_entries: usize, max_age_seconds: u64) -> Self {
        self.max_entries = max_entries;
        self.max_age_seconds = max_age_seconds;
        self
    }

    pub fn get(&mut self, task_id: &Uuid) -> Option<ContextBundle> {
        let key = task_id.to_string();
        if let Some(entry) = self.entries.get_mut(&key) {
            if !entry.is_stale(self.max_age_seconds) {
                entry.touch();
                return Some(entry.bundle.clone());
            }
        }
        self.entries.remove(&key);
        None
    }

    pub fn get_by_key(&mut self, cache_key: &str) -> Option<ContextBundle> {
        let task_key = self
            .entries
            .iter()
            .find(|(_, entry)| entry.cache_key == cache_key)
            .map(|(key, _)| key.clone());

        if let Some(key) = task_key {
            if let Some(entry) = self.entries.get_mut(&key) {
                if !entry.is_stale(self.max_age_seconds) {
                    entry.touch();
                    return Some(entry.bundle.clone());
                }
            }
        }
        None
    }

    pub fn put(&mut self, bundle: ContextBundle, dependencies: Vec<String>) -> String {
        let cached = CachedBundle::new(bundle, dependencies);
        let cache_key = cached.cache_key.clone();
        let task_key = cached.bundle.task_id.to_string();

        if self.entries.len() >= self.max_entries {
            self.evict_lru();
        }

        self.entries.insert(task_key, cached);
        cache_key
    }

    pub fn invalidate(&mut self, task_id: &Uuid) -> bool {
        self.entries.remove(&task_id.to_string()).is_some()
    }

    pub fn invalidate_by_dependency(&mut self, dependency_id: &str) -> usize {
        let keys_to_remove: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.dependencies.contains(&dependency_id.to_string()))
            .map(|(key, _)| key.clone())
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.entries.remove(&key);
        }
        count
    }

    pub fn invalidate_all(&mut self) {
        self.entries.clear();
    }

    pub fn cleanup_stale(&mut self) -> usize {
        let stale_keys: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.is_stale(self.max_age_seconds))
            .map(|(key, _)| key.clone())
            .collect();

        let count = stale_keys.len();
        for key in stale_keys {
            self.entries.remove(&key);
        }
        count
    }

    fn evict_lru(&mut self) {
        if let Some(oldest_key) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(key, _)| key.clone())
        {
            self.entries.remove(&oldest_key);
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn total_cached_tokens(&self) -> usize {
        self.entries.values().map(|e| e.bundle.total_tokens()).sum()
    }

    pub fn persist_to_disk(&self) -> Result<(), ContextError> {
        std::fs::create_dir_all(&self.cache_dir)
            .map_err(|e| ContextError::CacheError(e.to_string()))?;

        for (key, entry) in &self.entries {
            let path = self.cache_dir.join(format!("{}.json", key));
            let json = serde_json::to_string(entry)
                .map_err(|e| ContextError::CacheError(e.to_string()))?;
            std::fs::write(path, json).map_err(|e| ContextError::CacheError(e.to_string()))?;
        }

        Ok(())
    }

    pub fn load_from_disk(&mut self) -> Result<usize, ContextError> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut loaded = 0;
        for entry in std::fs::read_dir(&self.cache_dir)
            .map_err(|e| ContextError::CacheError(e.to_string()))?
        {
            let entry = entry.map_err(|e| ContextError::CacheError(e.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| ContextError::CacheError(e.to_string()))?;
                let cached: CachedBundle = serde_json::from_str(&content)
                    .map_err(|e| ContextError::CacheError(e.to_string()))?;

                let key = cached.bundle.task_id.to_string();
                self.entries.insert(key, cached);
                loaded += 1;
            }
        }

        Ok(loaded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_bundle::{ContextFragment, ContextPriority, ContextSource, TokenBudget};
    use tempfile::TempDir;

    fn create_test_bundle() -> ContextBundle {
        let task_id = Uuid::new_v4();
        let mut bundle = ContextBundle::new(task_id, TokenBudget::new(4000));

        let fragment = ContextFragment::new(
            "Test content".to_string(),
            ContextSource::Task,
            "task-001".to_string(),
            ContextPriority::P0,
            "Test".to_string(),
        );
        bundle.add_fragment(fragment).unwrap();
        bundle
    }

    #[test]
    fn test_cache_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ContextCache::new(temp_dir.path());
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn test_cache_put_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = ContextCache::new(temp_dir.path());

        let bundle = create_test_bundle();
        let task_id = bundle.task_id;

        cache.put(bundle, vec![]);
        assert_eq!(cache.entry_count(), 1);

        let retrieved = cache.get(&task_id);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_cache_invalidate() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = ContextCache::new(temp_dir.path());

        let bundle = create_test_bundle();
        let task_id = bundle.task_id;

        cache.put(bundle, vec![]);
        assert!(cache.invalidate(&task_id));
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn test_cache_invalidate_by_dependency() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = ContextCache::new(temp_dir.path());

        let bundle = create_test_bundle();
        cache.put(bundle, vec!["dep-001".to_string()]);

        let count = cache.invalidate_by_dependency("dep-001");
        assert_eq!(count, 1);
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn test_cache_max_entries() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = ContextCache::new(temp_dir.path()).with_limits(2, 3600);

        for _ in 0..5 {
            cache.put(create_test_bundle(), vec![]);
        }

        assert!(cache.entry_count() <= 2);
    }

    #[test]
    fn test_cached_bundle_staleness() {
        let bundle = create_test_bundle();
        let cached = CachedBundle::new(bundle, vec![]);

        assert!(!cached.is_stale(3600));
    }

    #[test]
    fn test_cache_persist_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = ContextCache::new(temp_dir.path());

        let bundle = create_test_bundle();
        cache.put(bundle, vec![]);

        cache.persist_to_disk().unwrap();

        let mut new_cache = ContextCache::new(temp_dir.path());
        let loaded = new_cache.load_from_disk().unwrap();
        assert_eq!(loaded, 1);
    }
}
