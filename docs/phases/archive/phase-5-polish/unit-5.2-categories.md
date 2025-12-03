# Unit 5.2: Categories/Groups

> **Phase:** 5 - Polish & Features  
> **Complexity:** Medium  
> **Skills:** UI layout, state management

---

## Objective

Group entries by XDG category with collapsible sections and filter shortcuts.

---

## Tasks

### 1. Group Entries by Category

```rust
pub struct CategoryGroup {
    pub name: String,
    pub entries: Vec<DesktopEntry>,
    pub collapsed: bool,
}

impl App {
    fn group_by_category(&self) -> Vec<CategoryGroup>;
}
```

### 2. Collapsible Sections in TUI

```
┌─────────────────────────────┐
│ > _                         │
├─────────────────────────────┤
│ ▼ Internet (3)              │
│   ● Firefox                 │
│     Chromium                │
│     Thunderbird             │
│ ▶ Development (5)           │
│ ▼ Utilities (2)             │
│     Terminal                │
│     Files                   │
└─────────────────────────────┘
```

### 3. Category Filter Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Cycle through categories |
| `Shift+Tab` | Cycle backwards |
| `:cat <name>` | Filter to category |

---

## Implementation Notes

### XDG Categories

Standard categories from the XDG spec:

```rust
const MAIN_CATEGORIES: &[&str] = &[
    "AudioVideo",
    "Audio",
    "Video",
    "Development",
    "Education",
    "Game",
    "Graphics",
    "Network",
    "Office",
    "Science",
    "Settings",
    "System",
    "Utility",
];

// Display names
fn category_display_name(cat: &str) -> &str {
    match cat {
        "AudioVideo" => "Media",
        "Network" => "Internet",
        "Utility" => "Utilities",
        _ => cat,
    }
}
```

### Grouping Logic

```rust
fn group_by_category(entries: &[DesktopEntry]) -> Vec<CategoryGroup> {
    let mut groups: HashMap<String, Vec<DesktopEntry>> = HashMap::new();
    
    for entry in entries {
        let category = entry.categories
            .iter()
            .find(|c| MAIN_CATEGORIES.contains(&c.as_str()))
            .cloned()
            .unwrap_or_else(|| "Other".to_string());
        
        groups.entry(category).or_default().push(entry.clone());
    }
    
    // Sort groups by name, entries by frecency within group
    let mut result: Vec<CategoryGroup> = groups
        .into_iter()
        .map(|(name, entries)| CategoryGroup {
            name,
            entries,
            collapsed: false,
        })
        .collect();
    
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}
```

### Rendering Collapsible Groups

```rust
fn render_categories(f: &mut Frame, area: Rect, groups: &[CategoryGroup], selected: usize) {
    let mut items = Vec::new();
    let mut current_idx = 0;
    
    for group in groups {
        // Category header
        let icon = if group.collapsed { "▶" } else { "▼" };
        let header = format!("{} {} ({})", icon, group.name, group.entries.len());
        items.push(ListItem::new(header).style(Style::default().bold()));
        current_idx += 1;
        
        // Entries (if not collapsed)
        if !group.collapsed {
            for entry in &group.entries {
                let prefix = if current_idx == selected { "●" } else { " " };
                items.push(ListItem::new(format!("  {} {}", prefix, entry.name)));
                current_idx += 1;
            }
        }
    }
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(list, area);
}
```

### Navigation State

```rust
pub struct CategoryNavigation {
    pub groups: Vec<CategoryGroup>,
    pub selected_group: usize,
    pub selected_entry: usize,  // within group
}

impl CategoryNavigation {
    pub fn toggle_collapse(&mut self) {
        self.groups[self.selected_group].collapsed = 
            !self.groups[self.selected_group].collapsed;
    }
    
    pub fn next(&mut self) { /* ... */ }
    pub fn prev(&mut self) { /* ... */ }
    pub fn selected_entry(&self) -> Option<&DesktopEntry> { /* ... */ }
}
```

---

## Configuration

```toml
[ui]
# Enable category grouping
group_by_category = true

# Start with categories collapsed
start_collapsed = false

# Categories to always show expanded
always_expanded = ["Internet", "Development"]

# Hide empty categories
hide_empty = true
```

---

## Acceptance Criteria

- [ ] Entries grouped by XDG category
- [ ] Category headers show entry count
- [ ] Collapse/expand with Enter on header
- [ ] Navigation works across groups
- [ ] Tab cycles through categories
- [ ] Search filters within all categories

---

## Testing

### Unit Tests

```rust
#[test]
fn test_group_by_category() {
    let entries = vec![
        DesktopEntry::new("Firefox").with_category("Network"),
        DesktopEntry::new("Terminal").with_category("Utility"),
        DesktopEntry::new("Chromium").with_category("Network"),
    ];
    
    let groups = group_by_category(&entries);
    
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].name, "Network");
    assert_eq!(groups[0].entries.len(), 2);
}

#[test]
fn test_navigation_across_groups() {
    let mut nav = CategoryNavigation::new(/* ... */);
    nav.next(); // Move to first entry
    nav.next(); // Move to second entry
    nav.next(); // Move to next group header
    // Verify correct selection
}
```

### Manual Tests

1. View grouped entries - categories should be visible
2. Collapse a category - entries should hide
3. Press Tab - should jump to next category
4. Search "fire" - should show Firefox regardless of category

---

## UX Considerations

- **Flat mode toggle:** Allow switching between grouped and flat list
- **Remember state:** Persist collapsed state across sessions
- **Smart defaults:** Expand categories with recent entries

---

## Related Units

- **Depends on:** None
- **Related:** Unit 5.1 (Frecency - sort within categories)
