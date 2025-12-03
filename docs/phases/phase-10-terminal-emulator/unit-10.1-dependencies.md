# Unit 10.1: Dependencies & Types

> **Phase:** 10 - Terminal Emulator Integration  
> **Complexity:** Low  
> **Estimated Time:** 1-2 hours  
> **Prerequisites:** None

---

## Objective

Add the termwiz crate and define the types needed for terminal emulation.

---

## Tasks

### 1. Add termwiz Dependency

```toml
# Cargo.toml
[dependencies]
termwiz = "0.23"  # Check for latest version
```

**Features to enable:**
- Default features are sufficient for our use case
- We don't need `widgets` feature (we use ratatui)

### 2. Create Terminal Module

Create `src/terminal.rs`:

```rust
//! Terminal emulation using termwiz
//! 
//! This module provides a proper terminal emulator that handles:
//! - ANSI escape sequences
//! - Cursor positioning  
//! - Colors and attributes
//! - Screen buffer management

use termwiz::surface::{Surface, Position, CursorShape};
use termwiz::cell::{Cell, CellAttributes};
use termwiz::color::ColorAttribute;
use termwiz::escape::parser::Parser;
use termwiz::escape::Action;

/// Configuration for the embedded terminal
#[derive(Debug, Clone)]
pub struct TerminalConfig {
    /// Number of columns
    pub cols: usize,
    /// Number of rows  
    pub rows: usize,
    /// Maximum scrollback lines
    pub scrollback: usize,
    /// Whether to enable alternate screen buffer
    pub alternate_screen: bool,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            scrollback: 10000,
            alternate_screen: true,
        }
    }
}

/// Embedded terminal emulator
pub struct EmbeddedTerminal {
    /// The terminal surface (screen buffer)
    surface: Surface,
    /// Escape sequence parser
    parser: Parser,
    /// Configuration
    config: TerminalConfig,
    /// Current cursor position
    cursor: Position,
    /// Scrollback buffer (lines that scrolled off top)
    scrollback: Vec<Vec<Cell>>,
    /// Scroll offset for viewing (0 = bottom)
    scroll_offset: usize,
    /// Whether in alternate screen mode
    in_alternate_screen: bool,
    /// Saved primary screen (when in alternate)
    saved_primary: Option<Surface>,
}
```

### 3. Register Module

In `src/main.rs` or `src/lib.rs`:

```rust
mod terminal;
pub use terminal::{EmbeddedTerminal, TerminalConfig};
```

### 4. Basic Constructor

```rust
impl EmbeddedTerminal {
    /// Create a new embedded terminal
    pub fn new(config: TerminalConfig) -> Self {
        let surface = Surface::new(config.cols, config.rows);
        
        Self {
            surface,
            parser: Parser::new(),
            config,
            cursor: Position::new(0, 0),
            scrollback: Vec::new(),
            scroll_offset: 0,
            in_alternate_screen: false,
            saved_primary: None,
        }
    }
    
    /// Create with default 80x24 size
    pub fn default_size() -> Self {
        Self::new(TerminalConfig::default())
    }
    
    /// Resize the terminal
    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.config.cols = cols;
        self.config.rows = rows;
        self.surface.resize(cols, rows);
    }
    
    /// Get terminal dimensions
    pub fn size(&self) -> (usize, usize) {
        (self.config.cols, self.config.rows)
    }
}
```

---

## Type Definitions

### Cell Representation

termwiz's `Cell` contains:
- Character (grapheme cluster)
- Foreground color
- Background color
- Attributes (bold, italic, underline, etc.)

We'll use these directly rather than wrapping.

### Color Mapping

```rust
/// Convert termwiz color to ratatui color
pub fn termwiz_to_ratatui_color(color: &ColorAttribute) -> ratatui::style::Color {
    use ratatui::style::Color;
    use termwiz::color::ColorAttribute;
    
    match color {
        ColorAttribute::Default => Color::Reset,
        ColorAttribute::PaletteIndex(idx) => Color::Indexed(*idx),
        ColorAttribute::TrueColorWithDefaultFallback(c) 
        | ColorAttribute::TrueColorWithPaletteFallback(c, _) => {
            let (r, g, b, _) = c.to_srgb_u8();
            Color::Rgb(r, g, b)
        }
    }
}
```

---

## Verification

### Compile Check

```bash
cargo check
```

### Unit Test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_terminal_creation() {
        let term = EmbeddedTerminal::default_size();
        assert_eq!(term.size(), (80, 24));
    }
    
    #[test]
    fn test_terminal_resize() {
        let mut term = EmbeddedTerminal::default_size();
        term.resize(120, 40);
        assert_eq!(term.size(), (120, 40));
    }
}
```

---

## Files Changed

| File | Change |
|------|--------|
| `Cargo.toml` | Add termwiz dependency |
| `src/terminal.rs` | New file - terminal emulator module |
| `src/main.rs` | Register terminal module |

---

## Next Unit

[Unit 10.2: Terminal Surface](./unit-10.2-surface.md) - Replace OutputBuffer with termwiz Surface
