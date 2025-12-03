# Unit 5.1: Frecency Sorting

> **Phase:** 5 - Polish & Features  
> **Complexity:** Medium  
> **Skills:** Algorithms, persistence  
> **Status:** ⚠️ Implemented but not wired up

---

## Implementation Status

The `history.rs` module is **complete** with:
- `UsageStats` - per-entry usage count and last-used timestamp
- `HistoryFile` - JSON serialization format with version field  
- `History` - main manager with load/save/record/frecency methods

**Remaining work:**
1. Update `default_path()` to use `XDG_STATE_HOME` (currently uses DATA_HOME)
2. Add `History` field to `App` struct
3. Call `history.load()` on startup
4. Call `history.record_usage(&entry.id)` after successful execution
5. Integrate frecency into `update_filtered()` sorting
6. Call `history.save()` on exit

---

## Objective

Implement frecency-based sorting so frequently and recently used entries appear first.

---

## Tasks

### 1. Track Usage Count Per Entry

```rust
pub struct UsageStats {
    pub count: u32,
    pub last_used: DateTime<Utc>,
}

pub struct History {
    entries: HashMap<String, UsageStats>,  // key = desktop entry ID
}
```

### 2. Track Last-Used Timestamp

Update timestamp on each execution.

### 3. Frecency Algorithm

```rust
impl History {
    pub fn frecency_score(&self, entry_id: &str) -> f64 {
        let stats = match self.entries.get(entry_id) {
            Some(s) => s,
            None => return 0.0,
        };
        
        let frequency = stats.count as f64;
        let recency = self.recency_weight(stats.last_used);
        
        frequency * recency
    }
    
    fn recency_weight(&self, last_used: DateTime<Utc>) -> f64 {
        let hours_ago = (Utc::now() - last_used).num_hours() as f64;
        
        match hours_ago {
            h if h < 1.0 => 4.0,      // Last hour
            h if h < 24.0 => 2.0,     // Last day
            h if h < 168.0 => 1.5,    // Last week
            h if h < 720.0 => 1.0,    // Last month
            _ => 0.5,                  // Older
        }
    }
}
```

### 4. Persist to History File (XDG Compliant)

Per the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/latest/):

> `$XDG_STATE_HOME` contains state data that should persist between (application) restarts,
> but that is not important or portable enough to the user that it should be stored in `$XDG_DATA_HOME`.
> It may contain: **actions history** (logs, history, recently used files, …)

**History file location:** `$XDG_STATE_HOME/darkwall-drun/history.json`

Default: `~/.local/state/darkwall-drun/history.json`

```rust
fn default_history_path() -> PathBuf {
    // Use XDG_STATE_HOME per spec (NOT DATA_HOME)
    if let Ok(state_home) = std::env::var("XDG_STATE_HOME") {
        return PathBuf::from(state_home).join("darkwall-drun/history.json");
    }
    
    // Fall back to ~/.local/state
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/state/darkwall-drun/history.json")
}

impl History {
    pub fn load() -> Result<Self>;
    pub fn save(&self) -> Result<()>;
    pub fn record_usage(&mut self, entry_id: &str);
}
```

**Why STATE_HOME, not DATA_HOME?**
- History is machine-specific (different apps on different machines)
- Not important enough to sync across machines
- Losing it is not catastrophic (just resets frecency)

---

## Implementation Notes

### History File Format

```json
{
  "version": 1,
  "entries": {
    "firefox.desktop": {
      "count": 42,
      "last_used": "2024-01-15T10:30:00Z"
    },
    "alacritty.desktop": {
      "count": 128,
      "last_used": "2024-01-15T11:45:00Z"
    }
  }
}
```

### Sorting Integration

```rust
impl App {
    fn sort_entries(&mut self) {
        self.filtered_entries.sort_by(|a, b| {
            let score_a = self.history.frecency_score(&a.id);
            let score_b = self.history.frecency_score(&b.id);
            
            // Higher score first
            score_b.partial_cmp(&score_a).unwrap_or(Ordering::Equal)
        });
    }
}
```

### Combined with Fuzzy Search

```rust
fn rank_entry(&self, entry: &DesktopEntry, query: &str) -> f64 {
    let fuzzy_score = self.fuzzy_match(entry, query);
    let frecency_score = self.history.frecency_score(&entry.id);
    
    // Weighted combination
    fuzzy_score * 0.7 + frecency_score * 0.3
}
```

---

## Configuration

```toml
[history]
# Enable frecency sorting
enabled = true

# Maximum entries to track
max_entries = 1000

# Decay old entries after N days
decay_after_days = 90

# Weight of frecency vs fuzzy match (0.0 - 1.0)
frecency_weight = 0.3
```

---

## Acceptance Criteria

- [ ] Usage count tracked per entry
- [ ] Last-used timestamp updated
- [ ] Frecency score calculated correctly
- [ ] History persists across restarts
- [ ] Recently used entries appear first
- [ ] Frequently used entries ranked higher

---

## Testing

### Unit Tests

```rust
#[test]
fn test_frecency_recent_boost() {
    let mut history = History::new();
    
    // Entry used once, just now
    history.record_usage("recent.desktop");
    
    // Entry used 10 times, a month ago
    history.entries.insert("old.desktop".to_string(), UsageStats {
        count: 10,
        last_used: Utc::now() - Duration::days(30),
    });
    
    // Recent should score higher despite lower count
    assert!(history.frecency_score("recent.desktop") > 
            history.frecency_score("old.desktop"));
}

#[test]
fn test_history_persistence() {
    let mut history = History::new();
    history.record_usage("test.desktop");
    history.save().unwrap();
    
    let loaded = History::load().unwrap();
    assert_eq!(loaded.entries.get("test.desktop").unwrap().count, 1);
}
```

### Manual Tests

1. Launch app A 5 times - should rise to top
2. Launch app B once - should appear above unused apps
3. Restart darkwall-drun - order should persist
4. Wait a day, launch app C - should rank above old app A

---

## Edge Cases

| Case | Handling |
|------|----------|
| New entry (no history) | Score = 0, sort alphabetically |
| Corrupted history file | Log warning, start fresh |
| Very old entries | Apply decay, optionally prune |
| Entry removed from system | Keep history (might return) |

---

## Related Units

- **Depends on:** None
- **Related:** Unit 5.2 (Categories - frecency within categories)
