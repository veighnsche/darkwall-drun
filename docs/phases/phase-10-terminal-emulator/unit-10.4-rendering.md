# Unit 10.4: Rendering

> **Phase:** 10 - Terminal Emulator Integration  
> **Complexity:** Medium  
> **Estimated Time:** 3-4 hours  
> **Prerequisites:** Unit 10.3

---

## Objective

Render the termwiz `Surface` to ratatui for display. This bridges the terminal emulator with our TUI framework.

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    PTY Output                        │
│                        │                             │
│                        ▼                             │
│              ┌─────────────────┐                     │
│              │ EmbeddedTerminal │                    │
│              │   (termwiz)      │                    │
│              └────────┬────────┘                     │
│                       │                              │
│                       ▼                              │
│              ┌─────────────────┐                     │
│              │  Surface/Cells  │                     │
│              └────────┬────────┘                     │
│                       │                              │
│                       ▼                              │
│              ┌─────────────────┐                     │
│              │TerminalWidget   │ ◄── This unit      │
│              │  (ratatui)      │                     │
│              └────────┬────────┘                     │
│                       │                              │
│                       ▼                              │
│              ┌─────────────────┐                     │
│              │  Frame/Buffer   │                     │
│              └─────────────────┘                     │
└─────────────────────────────────────────────────────┘
```

---

## Tasks

### 1. Color Conversion

Convert termwiz colors to ratatui colors:

```rust
use ratatui::style::Color as RatatuiColor;
use termwiz::color::ColorAttribute;

/// Convert termwiz color to ratatui color
pub fn convert_color(color: &ColorAttribute) -> RatatuiColor {
    match color {
        ColorAttribute::Default => RatatuiColor::Reset,
        ColorAttribute::PaletteIndex(idx) => {
            // Map 0-15 to named colors for better compatibility
            match *idx {
                0 => RatatuiColor::Black,
                1 => RatatuiColor::Red,
                2 => RatatuiColor::Green,
                3 => RatatuiColor::Yellow,
                4 => RatatuiColor::Blue,
                5 => RatatuiColor::Magenta,
                6 => RatatuiColor::Cyan,
                7 => RatatuiColor::White,
                8 => RatatuiColor::DarkGray,
                9 => RatatuiColor::LightRed,
                10 => RatatuiColor::LightGreen,
                11 => RatatuiColor::LightYellow,
                12 => RatatuiColor::LightBlue,
                13 => RatatuiColor::LightMagenta,
                14 => RatatuiColor::LightCyan,
                15 => RatatuiColor::Gray,
                _ => RatatuiColor::Indexed(*idx),
            }
        }
        ColorAttribute::TrueColorWithDefaultFallback(c)
        | ColorAttribute::TrueColorWithPaletteFallback(c, _) => {
            let (r, g, b, _) = c.to_srgb_u8();
            RatatuiColor::Rgb(r, g, b)
        }
    }
}
```

### 2. Attribute Conversion

Convert termwiz cell attributes to ratatui style:

```rust
use ratatui::style::{Style, Modifier};
use termwiz::cell::CellAttributes;
use termwiz::cell::Intensity;

/// Convert termwiz attributes to ratatui style
pub fn convert_attrs(attrs: &CellAttributes) -> Style {
    let mut style = Style::default();
    
    // Colors
    style = style.fg(convert_color(attrs.foreground()));
    style = style.bg(convert_color(attrs.background()));
    
    // Modifiers
    let mut modifiers = Modifier::empty();
    
    match attrs.intensity() {
        Intensity::Bold => modifiers |= Modifier::BOLD,
        Intensity::Half => modifiers |= Modifier::DIM,
        Intensity::Normal => {}
    }
    
    if attrs.italic() {
        modifiers |= Modifier::ITALIC;
    }
    
    if attrs.underline() != termwiz::cell::Underline::None {
        modifiers |= Modifier::UNDERLINED;
    }
    
    if attrs.blink() != termwiz::cell::Blink::None {
        modifiers |= Modifier::SLOW_BLINK;
    }
    
    if attrs.reverse() {
        modifiers |= Modifier::REVERSED;
    }
    
    if attrs.invisible() {
        modifiers |= Modifier::HIDDEN;
    }
    
    if attrs.strikethrough() {
        modifiers |= Modifier::CROSSED_OUT;
    }
    
    style.add_modifier(modifiers)
}
```

### 3. Terminal Widget

Create a ratatui widget that renders the terminal:

```rust
use ratatui::widgets::Widget;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// Widget for rendering an embedded terminal
pub struct TerminalWidget<'a> {
    terminal: &'a EmbeddedTerminal,
    /// Whether to show cursor
    show_cursor: bool,
}

impl<'a> TerminalWidget<'a> {
    pub fn new(terminal: &'a EmbeddedTerminal) -> Self {
        Self {
            terminal,
            show_cursor: true,
        }
    }
    
    pub fn show_cursor(mut self, show: bool) -> Self {
        self.show_cursor = show;
        self
    }
}

impl<'a> Widget for TerminalWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (term_cols, term_rows) = self.terminal.size();
        
        // Render each cell
        for y in 0..area.height.min(term_rows as u16) {
            let row = self.terminal.get_row(y as usize);
            
            for x in 0..area.width.min(term_cols as u16) {
                if let Some(cell) = row.get(x as usize) {
                    let buf_x = area.x + x;
                    let buf_y = area.y + y;
                    
                    // Get character (handle empty cells)
                    let ch = cell.str();
                    let display_char = if ch.is_empty() { " " } else { ch };
                    
                    // Convert style
                    let style = convert_attrs(cell.attrs());
                    
                    // Set in buffer
                    buf.set_string(buf_x, buf_y, display_char, style);
                }
            }
        }
        
        // Render cursor if visible and in view
        if self.show_cursor && self.terminal.is_at_bottom() {
            let cursor = self.terminal.cursor_position();
            if cursor.x < area.width as usize && cursor.y < area.height as usize {
                let buf_x = area.x + cursor.x as u16;
                let buf_y = area.y + cursor.y as u16;
                
                // Invert the cell at cursor position
                if let Some(cell) = buf.cell_mut((buf_x, buf_y)) {
                    cell.set_style(cell.style().add_modifier(Modifier::REVERSED));
                }
            }
        }
    }
}
```

### 4. Integration with draw_executing

Update the executing mode drawing:

```rust
// In src/ui/draw.rs

fn draw_executing(f: &mut Frame, app: &mut App, command: &str, theme: &Theme) {
    // ... header code ...
    
    // Output area
    let output_area = chunks[1];
    let inner_area = output_area.inner(Margin::new(1, 1)); // Account for borders
    
    if let Some(ref terminal) = app.embedded_terminal() {
        // New: Use terminal widget
        let widget = TerminalWidget::new(terminal)
            .show_cursor(true);
        
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.dimmed_alt))
            .title(" Output ");
        
        f.render_widget(block, output_area);
        f.render_widget(widget, inner_area);
    } else {
        // Fallback: Old OutputBuffer rendering
        let buffer = app.output_buffer_mut();
        let output_height = inner_area.height as usize;
        let lines: Vec<Line> = buffer
            .visible_lines(output_height)
            .into_iter()
            .map(|s| Line::from(s))
            .collect();
        
        let output = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.dimmed_alt))
                .title(" Output "));
        
        f.render_widget(output, output_area);
    }
    
    // ... status bar code ...
}
```

### 5. Efficient Rendering

Only re-render changed regions:

```rust
impl EmbeddedTerminal {
    /// Get dirty regions since last render
    pub fn take_dirty_regions(&mut self) -> Vec<Rect> {
        // termwiz Surface tracks changes
        let changes = self.surface.take_changes();
        
        // Convert to ratatui Rects
        changes.iter().map(|change| {
            // Simplified - in practice, parse change types
            Rect::default() // Full redraw for now
        }).collect()
    }
    
    /// Check if any content changed
    pub fn is_dirty(&self) -> bool {
        self.surface.has_changes()
    }
}
```

### 6. Scrollbar Integration

Show scroll position when viewing history:

```rust
fn draw_terminal_with_scrollbar(
    f: &mut Frame,
    terminal: &EmbeddedTerminal,
    area: Rect,
    theme: &Theme,
) {
    use ratatui::widgets::Scrollbar;
    
    let total_lines = terminal.total_lines();
    let visible_lines = area.height as usize;
    let scroll_offset = terminal.scroll_offset();
    
    // Only show scrollbar if there's scrollback
    if total_lines > visible_lines {
        let scrollbar_area = Rect {
            x: area.x + area.width - 1,
            y: area.y,
            width: 1,
            height: area.height,
        };
        
        let terminal_area = Rect {
            width: area.width - 1,
            ..area
        };
        
        // Render terminal
        f.render_widget(TerminalWidget::new(terminal), terminal_area);
        
        // Render scrollbar
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        
        let mut scrollbar_state = ScrollbarState::new(total_lines)
            .position(total_lines - scroll_offset - visible_lines);
        
        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    } else {
        f.render_widget(TerminalWidget::new(terminal), area);
    }
}
```

---

## Performance Considerations

### 1. Avoid Full Redraws

```rust
// Bad: Redraw everything every frame
terminal.draw(|f| {
    let widget = TerminalWidget::new(&term);
    f.render_widget(widget, area);
});

// Good: Only redraw if changed
if term.is_dirty() || needs_redraw {
    terminal.draw(|f| {
        let widget = TerminalWidget::new(&term);
        f.render_widget(widget, area);
    })?;
}
```

### 2. Batch Cell Updates

```rust
// The Widget implementation already batches by iterating once
// But ensure we're not doing extra work per-cell
```

### 3. String Allocation

```rust
// Avoid allocating strings for each cell
// termwiz Cell::str() returns &str, use directly
let ch = cell.str(); // No allocation
```

---

## Testing

### Visual Test

```rust
#[test]
fn test_color_rendering() {
    let mut term = EmbeddedTerminal::default_size();
    
    // Rainbow text
    term.write(b"\x1b[31mR\x1b[32mG\x1b[34mB\x1b[m");
    
    // Verify cells have correct colors
    let r_cell = term.surface.get_cell(0, 0).unwrap();
    let g_cell = term.surface.get_cell(1, 0).unwrap();
    let b_cell = term.surface.get_cell(2, 0).unwrap();
    
    assert_eq!(r_cell.attrs().foreground(), ColorAttribute::PaletteIndex(1));
    assert_eq!(g_cell.attrs().foreground(), ColorAttribute::PaletteIndex(2));
    assert_eq!(b_cell.attrs().foreground(), ColorAttribute::PaletteIndex(4));
}
```

### Integration Test

```rust
#[test]
fn test_widget_rendering() {
    let mut term = EmbeddedTerminal::new(TerminalConfig {
        cols: 10,
        rows: 5,
        ..Default::default()
    });
    
    term.write(b"Hello");
    
    // Create a test buffer
    let mut buf = Buffer::empty(Rect::new(0, 0, 10, 5));
    
    // Render widget
    let widget = TerminalWidget::new(&term);
    widget.render(Rect::new(0, 0, 10, 5), &mut buf);
    
    // Verify content
    assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "H");
    assert_eq!(buf.cell((1, 0)).unwrap().symbol(), "e");
}
```

---

## Files Changed

| File | Change |
|------|--------|
| `src/terminal.rs` | Add color/attr conversion, TerminalWidget |
| `src/ui/draw.rs` | Update draw_executing to use TerminalWidget |

---

## Acceptance Criteria

- [ ] Colors render correctly (basic, 256, true color)
- [ ] Text attributes render (bold, italic, underline)
- [ ] Cursor position visible
- [ ] Scrollbar shows when scrollback exists
- [ ] Performance acceptable (no visible lag)

---

## Next Unit

[Unit 10.5: Input Handling](./unit-10.5-input.md) - Forward keyboard and mouse input correctly
