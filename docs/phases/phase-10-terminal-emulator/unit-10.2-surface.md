# Unit 10.2: Terminal Surface

> **Phase:** 10 - Terminal Emulator Integration  
> **Complexity:** Medium  
> **Estimated Time:** 3-4 hours  
> **Prerequisites:** Unit 10.1

---

## Objective

Replace the current `OutputBuffer` with termwiz's `Surface` for proper screen buffer management.

---

## Current State

The existing `OutputBuffer` in `src/executor.rs`:
- Stores lines as `VecDeque<OutputLine>`
- Handles `\n` and `\r` manually
- Strips ANSI escape codes
- No cursor positioning
- No color preservation

---

## Tasks

### 1. Surface Integration

The termwiz `Surface` provides:
- 2D grid of `Cell` objects
- Cursor position tracking
- Dirty region tracking for efficient updates
- Scrollback support

```rust
impl EmbeddedTerminal {
    /// Get a reference to the surface
    pub fn surface(&self) -> &Surface {
        &self.surface
    }
    
    /// Get mutable reference to surface
    pub fn surface_mut(&mut self) -> &mut Surface {
        &mut self.surface
    }
    
    /// Get the current cursor position
    pub fn cursor_position(&self) -> Position {
        self.cursor
    }
}
```

### 2. Scrollback Buffer Management

When lines scroll off the top, save them:

```rust
impl EmbeddedTerminal {
    /// Add a line to scrollback
    fn push_to_scrollback(&mut self, line: Vec<Cell>) {
        self.scrollback.push(line);
        
        // Enforce max scrollback
        while self.scrollback.len() > self.config.scrollback {
            self.scrollback.remove(0);
        }
    }
    
    /// Scroll the screen up by n lines
    fn scroll_up(&mut self, n: usize) {
        for _ in 0..n {
            // Save top line to scrollback
            let top_line: Vec<Cell> = (0..self.config.cols)
                .map(|x| self.surface.get_cell(x, 0).cloned().unwrap_or_default())
                .collect();
            self.push_to_scrollback(top_line);
            
            // Scroll surface content up
            self.surface.scroll_up(1);
        }
    }
    
    /// Get total scrollable lines (scrollback + visible)
    pub fn total_lines(&self) -> usize {
        self.scrollback.len() + self.config.rows
    }
    
    /// Get current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }
    
    /// Set scroll offset (for user scrolling)
    pub fn set_scroll_offset(&mut self, offset: usize) {
        let max_offset = self.scrollback.len();
        self.scroll_offset = offset.min(max_offset);
    }
    
    /// Scroll to bottom (follow mode)
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }
    
    /// Check if at bottom (following)
    pub fn is_at_bottom(&self) -> bool {
        self.scroll_offset == 0
    }
}
```

### 3. Visible Content Retrieval

Get content for rendering, accounting for scroll position:

```rust
impl EmbeddedTerminal {
    /// Get a row of cells for rendering
    /// row 0 is the top of the viewport
    pub fn get_row(&self, viewport_row: usize) -> Vec<Cell> {
        let total_scrollback = self.scrollback.len();
        
        // Calculate which actual row this maps to
        let actual_row = if self.scroll_offset > 0 {
            // Scrolled up - may be in scrollback
            let scrollback_row = total_scrollback.saturating_sub(self.scroll_offset) + viewport_row;
            
            if scrollback_row < total_scrollback {
                // In scrollback buffer
                return self.scrollback[scrollback_row].clone();
            } else {
                // In visible surface
                scrollback_row - total_scrollback
            }
        } else {
            // At bottom - showing current surface
            viewport_row
        };
        
        // Get from surface
        if actual_row < self.config.rows {
            (0..self.config.cols)
                .map(|x| self.surface.get_cell(x, actual_row).cloned().unwrap_or_default())
                .collect()
        } else {
            // Empty row
            vec![Cell::default(); self.config.cols]
        }
    }
    
    /// Get all visible rows
    pub fn get_visible_rows(&self) -> Vec<Vec<Cell>> {
        (0..self.config.rows)
            .map(|row| self.get_row(row))
            .collect()
    }
}
```

### 4. Follow Mode Integration

```rust
impl EmbeddedTerminal {
    /// Whether to auto-scroll when new content arrives
    follow_mode: bool,
}

impl EmbeddedTerminal {
    /// Enable/disable follow mode
    pub fn set_follow_mode(&mut self, follow: bool) {
        self.follow_mode = follow;
        if follow {
            self.scroll_to_bottom();
        }
    }
    
    /// Check if in follow mode
    pub fn is_following(&self) -> bool {
        self.follow_mode
    }
    
    /// Called when new content is written
    fn on_content_added(&mut self) {
        if self.follow_mode {
            self.scroll_to_bottom();
        }
    }
}
```

---

## Migration

**Direct replacement** - no feature flag, no parallel implementation.

The old `OutputBuffer` is fundamentally broken (no escape sequence handling). Replace it entirely:

```rust
// In App - REPLACE, don't add alongside
pub struct App {
    // DELETE: output_buffer: OutputBuffer,
    terminal: EmbeddedTerminal,  // New implementation
}
```

---

## Testing

### Unit Tests

```rust
#[test]
fn test_scrollback() {
    let mut term = EmbeddedTerminal::new(TerminalConfig {
        cols: 80,
        rows: 5,
        scrollback: 100,
        ..Default::default()
    });
    
    // Write more lines than visible
    for i in 0..10 {
        term.write_line(&format!("Line {}", i));
    }
    
    // Should have 5 lines in scrollback
    assert_eq!(term.scrollback.len(), 5);
    assert_eq!(term.total_lines(), 10);
}

#[test]
fn test_scroll_offset() {
    let mut term = EmbeddedTerminal::new(TerminalConfig {
        cols: 80,
        rows: 5,
        scrollback: 100,
        ..Default::default()
    });
    
    // Add content
    for i in 0..20 {
        term.write_line(&format!("Line {}", i));
    }
    
    // Scroll up
    term.set_scroll_offset(10);
    assert!(!term.is_at_bottom());
    
    // Scroll to bottom
    term.scroll_to_bottom();
    assert!(term.is_at_bottom());
}
```

### Integration Test

```rust
#[test]
fn test_carriage_return() {
    let mut term = EmbeddedTerminal::default_size();
    
    // Simulate progress bar: "Progress: 50%\rProgress: 100%"
    term.write(b"Progress: 50%");
    term.write(b"\r");
    term.write(b"Progress: 100%");
    
    // Should show "Progress: 100%" not both lines
    let row = term.get_row(0);
    let text: String = row.iter().map(|c| c.str()).collect();
    assert!(text.starts_with("Progress: 100%"));
}
```

---

## Files Changed

| File | Change |
|------|--------|
| `src/terminal.rs` | Add Surface management, scrollback |
| `src/app.rs` | Add optional EmbeddedTerminal field |
| `src/config.rs` | Add use_terminal_emulator flag |

---

## Acceptance Criteria

- [ ] Surface correctly stores screen content
- [ ] Scrollback buffer accumulates old lines
- [ ] Scroll offset allows viewing history
- [ ] Follow mode auto-scrolls on new content
- [ ] Can retrieve visible rows for rendering

---

## Next Unit

[Unit 10.3: Escape Sequence Handling](./unit-10.3-escape-sequences.md) - Parse and apply ANSI escape sequences
