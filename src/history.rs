//! Usage history and frecency-based sorting.
//!
//! TEAM_000: Phase 5, Unit 5.1 - Frecency Sorting

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Usage statistics for a single entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// Number of times this entry has been launched
    pub count: u32,
    /// Unix timestamp of last use
    pub last_used: u64,
}

impl UsageStats {
    fn new() -> Self {
        Self {
            count: 1,
            last_used: current_timestamp(),
        }
    }
}

/// History file format
#[derive(Debug, Serialize, Deserialize)]
struct HistoryFile {
    version: u32,
    entries: HashMap<String, UsageStats>,
}

impl Default for HistoryFile {
    fn default() -> Self {
        Self {
            version: 1,
            entries: HashMap::new(),
        }
    }
}

/// Usage history manager
pub struct History {
    entries: HashMap<String, UsageStats>,
    path: PathBuf,
    max_entries: usize,
    decay_after_days: u64,
}

impl History {
    /// Create a new history manager
    pub fn new(max_entries: usize, decay_after_days: u64) -> Self {
        let path = Self::default_path();
        Self {
            entries: HashMap::new(),
            path,
            max_entries,
            decay_after_days,
        }
    }

    /// Get the default history file path
    fn default_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("darkwall-drun")
            .join("history.json")
    }

    /// Load history from disk
    pub fn load(&mut self) -> Result<()> {
        if !self.path.exists() {
            tracing::debug!("No history file found, starting fresh");
            return Ok(());
        }

        let content = fs::read_to_string(&self.path)
            .context("Failed to read history file")?;

        let file: HistoryFile = serde_json::from_str(&content)
            .context("Failed to parse history file")?;

        self.entries = file.entries;
        tracing::info!("Loaded {} history entries", self.entries.len());

        // Prune old entries
        self.prune_old_entries();

        Ok(())
    }

    /// Save history to disk
    pub fn save(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create history directory")?;
        }

        let file = HistoryFile {
            version: 1,
            entries: self.entries.clone(),
        };

        let content = serde_json::to_string_pretty(&file)
            .context("Failed to serialize history")?;

        fs::write(&self.path, content)
            .context("Failed to write history file")?;

        tracing::debug!("Saved {} history entries", self.entries.len());
        Ok(())
    }

    /// Record usage of an entry
    pub fn record_usage(&mut self, entry_id: &str) {
        let now = current_timestamp();

        if let Some(stats) = self.entries.get_mut(entry_id) {
            stats.count = stats.count.saturating_add(1);
            stats.last_used = now;
        } else {
            self.entries.insert(entry_id.to_string(), UsageStats::new());
        }

        // Prune if we have too many entries
        if self.entries.len() > self.max_entries {
            self.prune_least_used();
        }
    }

    /// Calculate frecency score for an entry
    /// Higher score = should appear higher in list
    pub fn frecency_score(&self, entry_id: &str) -> f64 {
        let stats = match self.entries.get(entry_id) {
            Some(s) => s,
            None => return 0.0,
        };

        let frequency = stats.count as f64;
        let recency = self.recency_weight(stats.last_used);

        frequency * recency
    }

    /// Calculate recency weight based on last use time
    fn recency_weight(&self, last_used: u64) -> f64 {
        let now = current_timestamp();
        let hours_ago = (now.saturating_sub(last_used)) as f64 / 3600.0;

        match hours_ago {
            h if h < 1.0 => 4.0,    // Last hour
            h if h < 24.0 => 2.0,   // Last day
            h if h < 168.0 => 1.5,  // Last week
            h if h < 720.0 => 1.0,  // Last month
            _ => 0.5,               // Older
        }
    }

    /// Remove entries older than decay_after_days
    fn prune_old_entries(&mut self) {
        let now = current_timestamp();
        let cutoff = now.saturating_sub(self.decay_after_days * 24 * 3600);

        let before = self.entries.len();
        self.entries.retain(|_, stats| stats.last_used >= cutoff);
        let removed = before - self.entries.len();

        if removed > 0 {
            tracing::info!("Pruned {} old history entries", removed);
        }
    }

    /// Remove least-used entries to stay under max_entries
    fn prune_least_used(&mut self) {
        if self.entries.len() <= self.max_entries {
            return;
        }

        // Sort by frecency score and keep top entries
        let mut entries: Vec<_> = self.entries.iter()
            .map(|(id, stats)| (id.clone(), self.frecency_score(id), stats.clone()))
            .collect();

        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        self.entries.clear();
        for (id, _, stats) in entries.into_iter().take(self.max_entries) {
            self.entries.insert(id, stats);
        }
    }

    /// Get the number of tracked entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_usage() {
        let mut history = History::new(100, 90);
        history.record_usage("test.desktop");
        
        assert_eq!(history.len(), 1);
        assert!(history.frecency_score("test.desktop") > 0.0);
    }

    #[test]
    fn test_frecency_recent_boost() {
        let mut history = History::new(100, 90);
        
        // Entry used once, just now
        history.record_usage("recent.desktop");
        
        // Entry used 10 times, a month ago
        let old_timestamp = current_timestamp() - (30 * 24 * 3600);
        history.entries.insert("old.desktop".to_string(), UsageStats {
            count: 10,
            last_used: old_timestamp,
        });
        
        // Recent should score higher despite lower count
        // recent: 1 * 4.0 = 4.0
        // old: 10 * 0.5 = 5.0 (actually old wins here due to high count)
        // Let's use a more extreme example
        history.entries.insert("very_old.desktop".to_string(), UsageStats {
            count: 2,
            last_used: old_timestamp,
        });
        
        // recent: 1 * 4.0 = 4.0
        // very_old: 2 * 0.5 = 1.0
        assert!(history.frecency_score("recent.desktop") > 
                history.frecency_score("very_old.desktop"));
    }

    #[test]
    fn test_unknown_entry_score() {
        let history = History::new(100, 90);
        assert_eq!(history.frecency_score("unknown.desktop"), 0.0);
    }

    #[test]
    fn test_increment_count() {
        let mut history = History::new(100, 90);
        history.record_usage("test.desktop");
        history.record_usage("test.desktop");
        history.record_usage("test.desktop");
        
        assert_eq!(history.entries.get("test.desktop").unwrap().count, 3);
    }
}
