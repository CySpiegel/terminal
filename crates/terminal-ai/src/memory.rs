//! Persistent memory system for the AI assistant.
//!
//! Stores key-value memory entries in a JSON file so the LLM can
//! remember context across sessions. Memory content is injected into
//! the system prompt at conversation start.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// A single memory entry with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// The key used to look up this entry.
    pub key: String,
    /// The stored value.
    pub value: String,
    /// When this entry was first created.
    pub created_at: DateTime<Utc>,
    /// When this entry was last updated.
    pub updated_at: DateTime<Utc>,
    /// Which conversation or source created this entry.
    pub source: String,
}

/// Persistent key-value memory store backed by a JSON file.
pub struct MemoryStore {
    entries: HashMap<String, MemoryEntry>,
    path: PathBuf,
}

impl MemoryStore {
    /// Load the memory store from the given path.
    ///
    /// If the file does not exist or is invalid, returns an empty store.
    pub fn load(path: &Path) -> Self {
        let entries = if path.exists() {
            match std::fs::read_to_string(path) {
                Ok(contents) => match serde_json::from_str::<HashMap<String, MemoryEntry>>(&contents) {
                    Ok(entries) => {
                        debug!("Loaded {} memory entries from {}", entries.len(), path.display());
                        entries
                    }
                    Err(e) => {
                        warn!("Failed to parse memory file {}: {}", path.display(), e);
                        HashMap::new()
                    }
                },
                Err(e) => {
                    warn!("Failed to read memory file {}: {}", path.display(), e);
                    HashMap::new()
                }
            }
        } else {
            debug!("Memory file does not exist, starting empty: {}", path.display());
            HashMap::new()
        };

        Self {
            entries,
            path: path.to_path_buf(),
        }
    }

    /// Load from the default path (~/.config/terminal/memory.json).
    pub fn load_default() -> Self {
        let path = dirs_path();
        Self::load(&path)
    }

    /// Save the memory store to disk.
    pub fn save(&self) -> Result<(), MemoryError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| MemoryError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let json = serde_json::to_string_pretty(&self.entries)
            .map_err(MemoryError::Serialize)?;

        std::fs::write(&self.path, json).map_err(|e| MemoryError::Io {
            path: self.path.clone(),
            source: e,
        })?;

        debug!("Saved {} memory entries to {}", self.entries.len(), self.path.display());
        Ok(())
    }

    /// Get a memory entry by key.
    pub fn get(&self, key: &str) -> Option<&MemoryEntry> {
        self.entries.get(key)
    }

    /// Set a memory entry. Creates a new entry or updates an existing one.
    pub fn set(&mut self, key: &str, value: &str, source: &str) {
        let now = Utc::now();
        let entry = self.entries.entry(key.to_string()).or_insert_with(|| MemoryEntry {
            key: key.to_string(),
            value: String::new(),
            created_at: now,
            updated_at: now,
            source: source.to_string(),
        });
        entry.value = value.to_string();
        entry.updated_at = now;
        entry.source = source.to_string();
    }

    /// Delete a memory entry by key. Returns true if the entry existed.
    pub fn delete(&mut self, key: &str) -> bool {
        self.entries.remove(key).is_some()
    }

    /// List all memory entries, sorted by key.
    pub fn list(&self) -> Vec<&MemoryEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by_key(|e| &e.key);
        entries
    }

    /// Search for memory entries whose key or value contains the query string.
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<_> = self
            .entries
            .values()
            .filter(|entry| {
                entry.key.to_lowercase().contains(&query_lower)
                    || entry.value.to_lowercase().contains(&query_lower)
            })
            .collect();
        results.sort_by_key(|e| &e.key);
        results
    }

    /// Generate a section to include in the system prompt so the LLM
    /// has access to all remembered context.
    pub fn to_system_prompt_section(&self) -> String {
        if self.entries.is_empty() {
            return String::new();
        }

        let mut section = String::from("\n\n## Persistent Memory\n");
        section.push_str("The following key-value pairs are remembered from previous sessions:\n\n");

        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by_key(|e| &e.key);

        for entry in entries {
            section.push_str(&format!("- **{}**: {}\n", entry.key, entry.value));
        }

        section
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the file path of this store.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Default memory file path.
fn dirs_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("terminal")
        .join("memory.json")
}

/// Errors that can occur in the memory system.
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Serialization error: {0}")]
    Serialize(serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn temp_store() -> (MemoryStore, PathBuf) {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        // Write empty JSON so load works
        std::fs::write(&path, "{}").unwrap();
        let store = MemoryStore::load(&path);
        (store, path)
    }

    #[test]
    fn test_set_and_get() {
        let (mut store, _path) = temp_store();
        store.set("project", "terminal-ai", "test");
        let entry = store.get("project").unwrap();
        assert_eq!(entry.value, "terminal-ai");
        assert_eq!(entry.source, "test");
    }

    #[test]
    fn test_delete() {
        let (mut store, _path) = temp_store();
        store.set("key", "value", "test");
        assert!(store.delete("key"));
        assert!(!store.delete("key"));
        assert!(store.get("key").is_none());
    }

    #[test]
    fn test_search() {
        let (mut store, _path) = temp_store();
        store.set("language", "rust", "test");
        store.set("framework", "tokio", "test");
        store.set("editor", "vim", "test");

        let results = store.search("rust");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "language");
    }

    #[test]
    fn test_list() {
        let (mut store, _path) = temp_store();
        store.set("b_key", "val", "test");
        store.set("a_key", "val", "test");
        let list = store.list();
        assert_eq!(list[0].key, "a_key");
        assert_eq!(list[1].key, "b_key");
    }

    #[test]
    fn test_system_prompt_section() {
        let (mut store, _path) = temp_store();
        assert!(store.to_system_prompt_section().is_empty());

        store.set("lang", "rust", "test");
        let section = store.to_system_prompt_section();
        assert!(section.contains("Persistent Memory"));
        assert!(section.contains("lang"));
        assert!(section.contains("rust"));
    }

    #[test]
    fn test_load_missing_file() {
        let store = MemoryStore::load(Path::new("/tmp/nonexistent_memory_test_file.json"));
        assert!(store.is_empty());
    }
}
