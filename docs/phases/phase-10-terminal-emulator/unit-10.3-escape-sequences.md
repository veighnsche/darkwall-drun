# Unit 10.3: Escape Sequence Handling

> **Phase:** 10 - Terminal Emulator Integration  
> **Complexity:** Medium  
> **Estimated Time:** 2-3 hours  
> **Prerequisites:** Unit 10.2

---

## Objective

Parse ANSI escape sequences and apply them to the terminal surface. This is the core of terminal emulation.

---

## Background: Escape Sequences

Terminals use escape sequences for:
- **Cursor movement** - `\x1b[H` (home), `\x1b[5;10H` (row 5, col 10)
- **Colors** - `\x1b[31m` (red foreground), `\x1b[48;2;255;0;0m` (RGB background)
- **Attributes** - `\x1b[1m` (bold), `\x1b[4m` (underline)
- **Screen control** - `\x1b[2J` (clear screen), `\x1b[K` (clear to end of line)
- **Scrolling** - `\x1b[S` (scroll up), `\x1b[T` (scroll down)

---

## Tasks

### 1. Parser Integration

termwiz provides a `Parser` that converts bytes into `Action` events:

```rust
use termwiz::escape::parser::Parser;
use termwiz::escape::Action;

impl EmbeddedTerminal {
    /// Process raw bytes from PTY
    pub fn write(&mut self, data: &[u8]) {
        let actions = self.parser.parse_as_vec(data);
        
        for action in actions {
            self.handle_action(action);
        }
        
        // Notify that content was added
        self.on_content_added();
    }
}
```

### 2. Action Handler

Handle each type of action:

```rust
impl EmbeddedTerminal {
    fn handle_action(&mut self, action: Action) {
        use termwiz::escape::Action::*;
        
        match action {
            Print(c) => self.print_char(c),
            PrintString(s) => self.print_string(&s),
            Control(ctrl) => self.handle_control(ctrl),
            Esc(esc) => self.handle_esc(esc),
            CSI(csi) => self.handle_csi(csi),
            OperatingSystemCommand(osc) => self.handle_osc(osc),
            DeviceControl(dcs) => self.handle_dcs(dcs),
            Sixel(sixel) => { /* Future: image support */ }
            XtGetTcap(_) | KittyImage(_) => { /* Future: graphics */ }
        }
    }
}
```

### 3. Character Printing

```rust
impl EmbeddedTerminal {
    /// Print a single character at cursor position
    fn print_char(&mut self, c: char) {
        let (col, row) = (self.cursor.x, self.cursor.y);
        
        // Set cell content with current attributes
        self.surface.set_cell(
            col,
            row,
            &Cell::new(c, self.current_attrs.clone()),
        );
        
        // Advance cursor
        self.cursor.x += 1;
        
        // Handle line wrap
        if self.cursor.x >= self.config.cols {
            self.cursor.x = 0;
            self.newline();
        }
    }
    
    /// Print a string
    fn print_string(&mut self, s: &str) {
        for c in s.chars() {
            self.print_char(c);
        }
    }
}
```

### 4. Control Characters

```rust
use termwiz::escape::ControlCode;

impl EmbeddedTerminal {
    fn handle_control(&mut self, ctrl: ControlCode) {
        use ControlCode::*;
        
        match ctrl {
            Null => {}
            Bell => { /* Could trigger visual bell */ }
            Backspace => {
                self.cursor.x = self.cursor.x.saturating_sub(1);
            }
            HorizontalTab => {
                // Move to next tab stop (every 8 columns)
                self.cursor.x = ((self.cursor.x / 8) + 1) * 8;
                if self.cursor.x >= self.config.cols {
                    self.cursor.x = self.config.cols - 1;
                }
            }
            LineFeed | VerticalTab | FormFeed => {
                self.newline();
            }
            CarriageReturn => {
                self.cursor.x = 0;
            }
            _ => {}
        }
    }
    
    /// Handle newline - move cursor down, scroll if needed
    fn newline(&mut self) {
        self.cursor.y += 1;
        
        if self.cursor.y >= self.config.rows {
            // Scroll screen up
            self.scroll_up(1);
            self.cursor.y = self.config.rows - 1;
        }
    }
}
```

### 5. CSI (Control Sequence Introducer) Handling

This is the bulk of escape sequence handling:

```rust
use termwiz::escape::csi::{Cursor, Edit, Sgr, CSI};

impl EmbeddedTerminal {
    fn handle_csi(&mut self, csi: CSI) {
        use CSI::*;
        
        match csi {
            Cursor(cursor_op) => self.handle_cursor(cursor_op),
            Edit(edit_op) => self.handle_edit(edit_op),
            Sgr(sgr) => self.handle_sgr(sgr),
            Mode(mode) => self.handle_mode(mode),
            Device(device) => self.handle_device(device),
            Window(window) => { /* Window manipulation - usually ignore */ }
            _ => {
                // Log unhandled for debugging
                tracing::debug!("Unhandled CSI: {:?}", csi);
            }
        }
    }
}
```

### 6. Cursor Operations

```rust
use termwiz::escape::csi::Cursor;

impl EmbeddedTerminal {
    fn handle_cursor(&mut self, op: Cursor) {
        use Cursor::*;
        
        match op {
            Up(n) => {
                self.cursor.y = self.cursor.y.saturating_sub(n as usize);
            }
            Down(n) => {
                self.cursor.y = (self.cursor.y + n as usize).min(self.config.rows - 1);
            }
            Left(n) => {
                self.cursor.x = self.cursor.x.saturating_sub(n as usize);
            }
            Right(n) => {
                self.cursor.x = (self.cursor.x + n as usize).min(self.config.cols - 1);
            }
            Position { line, col } => {
                // CSI row;col H - 1-indexed in escape sequence
                self.cursor.y = (line.as_one_based() as usize - 1).min(self.config.rows - 1);
                self.cursor.x = (col.as_one_based() as usize - 1).min(self.config.cols - 1);
            }
            CursorHome => {
                self.cursor.x = 0;
                self.cursor.y = 0;
            }
            SaveCursor => {
                self.saved_cursor = Some(self.cursor);
            }
            RestoreCursor => {
                if let Some(pos) = self.saved_cursor {
                    self.cursor = pos;
                }
            }
            _ => {}
        }
    }
}
```

### 7. Edit Operations (Clear Screen, etc.)

```rust
use termwiz::escape::csi::Edit;

impl EmbeddedTerminal {
    fn handle_edit(&mut self, op: Edit) {
        use Edit::*;
        
        match op {
            EraseInLine(erase) => {
                use termwiz::escape::csi::EraseInLine::*;
                match erase {
                    EraseToEndOfLine => {
                        // Clear from cursor to end of line
                        for x in self.cursor.x..self.config.cols {
                            self.surface.set_cell(x, self.cursor.y, &Cell::default());
                        }
                    }
                    EraseToStartOfLine => {
                        for x in 0..=self.cursor.x {
                            self.surface.set_cell(x, self.cursor.y, &Cell::default());
                        }
                    }
                    EraseLine => {
                        for x in 0..self.config.cols {
                            self.surface.set_cell(x, self.cursor.y, &Cell::default());
                        }
                    }
                }
            }
            EraseInDisplay(erase) => {
                use termwiz::escape::csi::EraseInDisplay::*;
                match erase {
                    EraseToEndOfDisplay => {
                        // Clear from cursor to end of screen
                        // First, rest of current line
                        for x in self.cursor.x..self.config.cols {
                            self.surface.set_cell(x, self.cursor.y, &Cell::default());
                        }
                        // Then all lines below
                        for y in (self.cursor.y + 1)..self.config.rows {
                            for x in 0..self.config.cols {
                                self.surface.set_cell(x, y, &Cell::default());
                            }
                        }
                    }
                    EraseToStartOfDisplay => {
                        // Clear from start to cursor
                        for y in 0..self.cursor.y {
                            for x in 0..self.config.cols {
                                self.surface.set_cell(x, y, &Cell::default());
                            }
                        }
                        for x in 0..=self.cursor.x {
                            self.surface.set_cell(x, self.cursor.y, &Cell::default());
                        }
                    }
                    EraseDisplay => {
                        // Clear entire screen
                        for y in 0..self.config.rows {
                            for x in 0..self.config.cols {
                                self.surface.set_cell(x, y, &Cell::default());
                            }
                        }
                    }
                    EraseScrollback => {
                        self.scrollback.clear();
                    }
                }
            }
            _ => {}
        }
    }
}
```

### 8. SGR (Select Graphic Rendition) - Colors & Attributes

```rust
use termwiz::escape::csi::Sgr;
use termwiz::cell::CellAttributes;

impl EmbeddedTerminal {
    /// Current text attributes
    current_attrs: CellAttributes,
    
    fn handle_sgr(&mut self, sgr: Sgr) {
        use Sgr::*;
        
        match sgr {
            Reset => {
                self.current_attrs = CellAttributes::default();
            }
            Intensity(intensity) => {
                self.current_attrs.set_intensity(intensity);
            }
            Underline(underline) => {
                self.current_attrs.set_underline(underline);
            }
            Blink(blink) => {
                self.current_attrs.set_blink(blink);
            }
            Italic(italic) => {
                self.current_attrs.set_italic(italic);
            }
            Inverse(inverse) => {
                self.current_attrs.set_reverse(inverse);
            }
            Invisible(invisible) => {
                self.current_attrs.set_invisible(invisible);
            }
            StrikeThrough(strike) => {
                self.current_attrs.set_strikethrough(strike);
            }
            Foreground(color) => {
                self.current_attrs.set_foreground(color);
            }
            Background(color) => {
                self.current_attrs.set_background(color);
            }
            _ => {}
        }
    }
}
```

---

## Priority Escape Sequences

Focus on these first (most commonly used):

| Priority | Sequence | Description |
|----------|----------|-------------|
| P0 | `\r`, `\n` | Carriage return, newline |
| P0 | `\x1b[m` | Reset attributes |
| P0 | `\x1b[30-37m` | Basic foreground colors |
| P0 | `\x1b[40-47m` | Basic background colors |
| P1 | `\x1b[H` | Cursor home |
| P1 | `\x1b[row;colH` | Cursor position |
| P1 | `\x1b[K` | Clear to end of line |
| P1 | `\x1b[2J` | Clear screen |
| P2 | `\x1b[1m` | Bold |
| P2 | `\x1b[38;5;Nm` | 256-color foreground |
| P2 | `\x1b[38;2;R;G;Bm` | True color foreground |
| P3 | `\x1b[?1049h/l` | Alternate screen |
| P3 | `\x1b[?25h/l` | Show/hide cursor |

---

## Testing

### Test Basic Escape Sequences

```rust
#[test]
fn test_color_escape() {
    let mut term = EmbeddedTerminal::default_size();
    
    // Red text
    term.write(b"\x1b[31mHello\x1b[m");
    
    let cell = term.surface.get_cell(0, 0).unwrap();
    assert_eq!(cell.attrs().foreground(), ColorAttribute::PaletteIndex(1)); // Red
}

#[test]
fn test_cursor_position() {
    let mut term = EmbeddedTerminal::default_size();
    
    // Move to row 5, col 10 (1-indexed in escape)
    term.write(b"\x1b[5;10HX");
    
    // Should be at row 4, col 9 (0-indexed)
    let cell = term.surface.get_cell(9, 4).unwrap();
    assert_eq!(cell.str(), "X");
}

#[test]
fn test_clear_line() {
    let mut term = EmbeddedTerminal::default_size();
    
    term.write(b"Hello World");
    term.write(b"\x1b[5G"); // Move to column 5
    term.write(b"\x1b[K");  // Clear to end of line
    
    let row = term.get_row(0);
    let text: String = row.iter().map(|c| c.str()).collect();
    assert!(text.starts_with("Hell ")); // "o World" cleared
}
```

---

## Files Changed

| File | Change |
|------|--------|
| `src/terminal.rs` | Add escape sequence handling |

---

## Acceptance Criteria

- [ ] Basic colors work (8 colors + bright variants)
- [ ] 256-color palette works
- [ ] True color (RGB) works
- [ ] Cursor positioning works
- [ ] Clear screen/line works
- [ ] Bold/italic/underline work
- [ ] Carriage return overwrites line (rsync fix)

---

## Next Unit

[Unit 10.4: Rendering](./unit-10.4-rendering.md) - Render the terminal surface to ratatui
