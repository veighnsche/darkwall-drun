# Unit 5.4.1: Grid Layout

> **Parent:** Unit 5.4 (Theming)  
> **Complexity:** Low  
> **Skills:** Ratatui layout, constraint calculation

---

## Objective

Implement a 2-column grid layout for displaying entries, with configurable column count and visible rows.

---

## Requirements

| Property | Default | Configurable |
|----------|---------|--------------|
| Columns | 2 | Yes |
| Visible Rows | 5 | Yes |
| Total Visible | 10 (2×5) | Derived |

---

## Implementation

### GridLayout Struct

```rust
pub struct GridLayout {
    pub columns: u16,
    pub visible_rows: u16,
}

impl GridLayout {
    pub fn visible_count(&self) -> usize {
        (self.columns * self.visible_rows) as usize
    }

    pub fn visible_range(&self, selected: usize, total: usize) -> Range<usize> {
        let page_size = self.visible_count();
        let page = selected / page_size;
        let start = page * page_size;
        let end = (start + page_size).min(total);
        start..end
    }

    pub fn index_to_grid(&self, index: usize) -> (u16, u16) {
        let row = (index / self.columns as usize) as u16;
        let col = (index % self.columns as usize) as u16;
        (row, col)
    }
}
```

### Visual Layout vs Index Order

Rofi fills columns top-to-bottom, then left-to-right (column-major):

```
Index order in 2×5 grid:
┌─────┬─────┐
│  0  │  5  │
│  1  │  6  │
│  2  │  7  │
│  3  │  8  │
│  4  │  9  │
└─────┴─────┘
```

This means entry indices are **not** row-major. The grid position calculation:

```rust
/// Convert flat index to (row, col) for column-major layout
pub fn index_to_position(&self, index: usize) -> (u16, u16) {
    let rows = self.visible_rows as usize;
    let col = index / rows;
    let row = index % rows;
    (row as u16, col as u16)
}
```

---

## Navigation (Rofi-Style)

Rofi uses **column-major** navigation in multi-column layouts:

```
Layout (2 columns × 5 rows):
┌─────────┬─────────┐
│ 0       │ 5       │  ← Row 0
│ 1       │ 6       │  ← Row 1
│ 2       │ 7       │  ← Row 2
│ 3       │ 8       │  ← Row 3
│ 4       │ 9       │  ← Row 4
└─────────┴─────────┘
  Col 0     Col 1
```

### Key Bindings

| Key | Rofi Action | Movement |
|-----|-------------|----------|
| `↑`/`k` | `kb-row-up` | Previous entry (-1) |
| `↓`/`j` | `kb-row-down` | Next entry (+1) |
| `←`/`h` | `kb-row-left` | Previous column (-rows) |
| `→`/`l` | `kb-row-right` | Next column (+rows) |
| `Tab` | `kb-element-next` | Next entry (+1, wraps) |
| `Shift+Tab` | `kb-element-prev` | Previous entry (-1, wraps) |
| `Page Up` | `kb-page-prev` | Previous page |
| `Page Down` | `kb-page-next` | Next page |
| `Home` | `kb-row-first` | First entry |
| `End` | `kb-row-last` | Last entry |

### Navigation Logic

```rust
fn move_selection(&mut self, direction: Direction) {
    let rows = self.config.layout.visible_rows as usize;
    let total = self.entries.len();
    
    match direction {
        // Up/Down: linear movement through list
        Direction::Up => {
            self.selected = self.selected.saturating_sub(1);
        }
        Direction::Down => {
            if self.selected + 1 < total {
                self.selected += 1;
            }
        }
        // Left/Right: jump by number of rows (column movement)
        Direction::Left => {
            self.selected = self.selected.saturating_sub(rows);
        }
        Direction::Right => {
            self.selected = (self.selected + rows).min(total.saturating_sub(1));
        }
    }
}
```

### Example Navigation (2×5 grid, 10 entries)

Starting at entry 2:
- **Up**: 2 → 1
- **Down**: 2 → 3
- **Left**: 2 → 0 (saturating_sub, can't go to -3)
- **Right**: 2 → 7 (2 + 5 = 7)

Starting at entry 7:
- **Left**: 7 → 2 (7 - 5 = 2)
- **Right**: 7 → 9 (7 + 5 = 12, clamped to 9)

---

## Acceptance Criteria

- [ ] 2 columns display side-by-side
- [ ] 5 rows visible (10 entries total)
- [ ] Up/Down moves by 1 entry
- [ ] Left/Right moves by `visible_rows` (column jump)
- [ ] Tab/Shift+Tab wraps around
- [ ] Page Up/Down moves by full page
- [ ] Home/End jumps to first/last
- [ ] Partial last row handled gracefully
