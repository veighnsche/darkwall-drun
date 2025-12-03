# Phase 5: History & Frecency Sorting

## Overview

Implement usage history tracking with frecency-based sorting to surface frequently and recently used entries.

## XDG Compliance

Per the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/latest/):

> `$XDG_STATE_HOME` defines the base directory relative to which user-specific state files should be stored.
> If `$XDG_STATE_HOME` is either not set or empty, a default equal to `$HOME/.local/state` should be used.
>
> The `$XDG_STATE_HOME` contains state data that should persist between (application) restarts, but that is not important or portable enough to the user that it should be stored in `$XDG_DATA_HOME`. It may contain:
> - actions history (logs, history, recently used files, …)
> - current state of the application that can be reused on a restart

**History file location:** `$XDG_STATE_HOME/darkwall-drun/history.json`

Default: `~/.local/state/darkwall-drun/history.json`

### Why STATE_HOME, not DATA_HOME?

- History is machine-specific (different apps installed on different machines)
- Not important enough to sync across machines
- Losing it is not catastrophic (just resets frecency)

## Current Implementation Status

The `history.rs` module is **complete but not wired up**. It provides:

- `UsageStats` - per-entry usage count and last-used timestamp
- `HistoryFile` - JSON serialization format with version field
- `History` - main manager with load/save/record/frecency methods

## Integration Steps

### 1. Update history.rs to use XDG_STATE_HOME

```rust
fn default_path() -> PathBuf {
    // Use XDG_STATE_HOME per spec
    if let Ok(state_home) = std::env::var("XDG_STATE_HOME") {
        return PathBuf::from(state_home).join("darkwall-drun/history.json");
    }
    
    // Fall back to ~/.local/state
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/state/darkwall-drun/history.json")
}
```

### 2. Add History to App struct

```rust
// In app.rs
pub struct App {
    // ... existing fields ...
    history: History,
}

impl App {
    pub fn new(...) -> Self {
        let mut history = History::new(1000, 90); // max 1000 entries, decay after 90 days
        if let Err(e) = history.load() {
            tracing::warn!("Failed to load history: {}", e);
        }
        // ...
    }
}
```

### 3. Record usage after execution

```rust
// In execute_entry()
pub async fn execute_entry(&mut self, entry: Entry, ...) -> Result<()> {
    self.history.record_usage(&entry.id);
    // ... existing code ...
}
```

### 4. Use frecency in sorting

```rust
// In update_filtered()
fn update_filtered(&mut self) {
    // ... existing filter logic ...
    
    // Sort by frecency, then by fuzzy score
    scored.sort_by(|a, b| {
        let freq_a = self.history.frecency_score(&self.entries[a.0].id);
        let freq_b = self.history.frecency_score(&self.entries[b.0].id);
        
        // Primary: fuzzy score, Secondary: frecency
        match b.1.cmp(&a.1) {
            std::cmp::Ordering::Equal => freq_b.partial_cmp(&freq_a).unwrap_or(std::cmp::Ordering::Equal),
            other => other,
        }
    });
}
```

### 5. Save on exit

```rust
// In main.rs, after run_app()
if let Err(e) = app.save_history() {
    tracing::warn!("Failed to save history: {}", e);
}
```

## File Format

```json
{
  "version": 1,
  "entries": {
    "firefox": {
      "count": 42,
      "last_used": 1701600000
    },
    "kitty": {
      "count": 156,
      "last_used": 1701599000
    }
  }
}
```

## Frecency Algorithm

```
score = count × recency_weight

recency_weight:
  < 1 hour ago:  4.0
  < 1 day ago:   2.0
  < 1 week ago:  1.5
  < 1 month ago: 1.0
  older:         0.5
```

## Configuration Options (Future)

```toml
[history]
enabled = true
max_entries = 1000
decay_after_days = 90
frecency_weight = 0.5  # 0.0 = pure fuzzy, 1.0 = pure frecency
```

## Testing

```bash
# Verify XDG compliance
XDG_STATE_HOME=/tmp/test-state cargo run
ls /tmp/test-state/darkwall-drun/history.json
```

## Checklist

- [ ] Update `default_path()` to use `XDG_STATE_HOME`
- [ ] Add `History` field to `App`
- [ ] Load history on startup
- [ ] Record usage after successful execution
- [ ] Integrate frecency into sorting
- [ ] Save history on exit
- [ ] Add config options for history behavior
- [ ] Add tests for XDG path resolution
