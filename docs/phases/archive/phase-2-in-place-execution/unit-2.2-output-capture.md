# Unit 2.2: Output Capture

> **Phase:** 2 - In-Place Execution  
> **Complexity:** Medium  
> **Skills:** Async Rust, TUI rendering

---

## Objective

Stream command output to a buffer and display it in the TUI with proper ANSI handling.

---

## Tasks

### 1. Output Buffer Implementation

```rust
pub struct OutputBuffer {
    lines: VecDeque<String>,
    max_lines: usize,
    scroll_offset: usize,
}

impl OutputBuffer {
    pub fn push(&mut self, data: &[u8]);
    pub fn lines(&self) -> impl Iterator<Item = &str>;
    pub fn scroll_up(&mut self, n: usize);
    pub fn scroll_down(&mut self, n: usize);
    pub fn clear(&mut self);
}
```

### 2. Stream stdout/stderr to Buffer

- Async read from PTY master
- Parse line boundaries
- Handle partial lines (no newline yet)

### 3. Display Output in TUI

- Scrollable viewport widget
- Line wrapping for long lines
- Scroll indicators

### 4. Handle ANSI Escape Codes

- Use `vte` crate for parsing
- Convert to ratatui styles
- Strip unsupported sequences

---

## Implementation Notes

### ANSI Parsing with `vte`

```toml
[dependencies]
vte = "0.13"
```

```rust
use vte::{Parser, Perform};

struct AnsiHandler {
    current_style: Style,
    output: Vec<Span>,
}

impl Perform for AnsiHandler {
    fn print(&mut self, c: char) { /* ... */ }
    fn csi_dispatch(&mut self, params: &[i64], /* ... */) { /* ... */ }
}
```

### Async Read Loop

```rust
async fn read_output(pty: &mut PtySession, buffer: &mut OutputBuffer) {
    let mut buf = [0u8; 4096];
    loop {
        match pty.read(&mut buf).await {
            Ok(0) => break, // EOF
            Ok(n) => buffer.push(&buf[..n]),
            Err(e) => { /* handle error */ }
        }
    }
}
```

### TUI Widget

```rust
fn render_output(f: &mut Frame, area: Rect, buffer: &OutputBuffer) {
    let block = Block::default()
        .title("Output")
        .borders(Borders::ALL);
    
    let paragraph = Paragraph::new(buffer.lines().collect::<Vec<_>>())
        .block(block)
        .scroll((buffer.scroll_offset as u16, 0));
    
    f.render_widget(paragraph, area);
}
```

---

## Acceptance Criteria

- [ ] Output streams to buffer in real-time
- [ ] Buffer respects max line limit
- [ ] Scrolling works (up/down/page up/page down)
- [ ] Basic ANSI colors render correctly
- [ ] Long lines wrap properly
- [ ] No memory leaks with long-running commands

---

## Testing

### Unit Tests

```rust
#[test]
fn test_buffer_push() {
    let mut buf = OutputBuffer::new(100);
    buf.push(b"line1\nline2\n");
    assert_eq!(buf.lines().count(), 2);
}

#[test]
fn test_buffer_max_lines() {
    let mut buf = OutputBuffer::new(10);
    for i in 0..20 {
        buf.push(format!("line{}\n", i).as_bytes());
    }
    assert_eq!(buf.lines().count(), 10);
}

#[test]
fn test_ansi_color_parsing() {
    let input = b"\x1b[31mred\x1b[0m";
    // Verify red color is extracted
}
```

### Manual Tests

1. Run `ls --color` - colors should display
2. Run `cat /dev/urandom | head -c 10000` - should not crash
3. Scroll through output with j/k

---

## Keybindings (Execution Mode)

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |
| `G` | Scroll to bottom |
| `g` | Scroll to top |

---

## Related Units

- **Depends on:** Unit 2.1 (PTY Allocation)
- **Blocks:** Unit 2.3 (Return to Launcher)
