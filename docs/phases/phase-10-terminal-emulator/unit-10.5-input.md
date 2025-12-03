# Unit 10.5: Input Handling

> **Phase:** 10 - Terminal Emulator Integration  
> **Complexity:** Low  
> **Estimated Time:** 1-2 hours  
> **Prerequisites:** Unit 10.4

---

## Objective

Properly forward keyboard and mouse input to the PTY, encoding special keys correctly.

---

## Current State

The existing input handling in `src/main.rs`:
- Forwards basic characters
- Handles Enter, Backspace, Tab
- Uses hardcoded escape sequences

This works but is incomplete for full terminal emulation.

---

## Tasks

### 1. Key Encoding with termwiz

termwiz provides proper key encoding:

```rust
use termwiz::input::{KeyCode, KeyEvent, Modifiers};
use termwiz::escape::csi::KittyKeyboardFlags;

impl EmbeddedTerminal {
    /// Encode a key event for the PTY
    pub fn encode_key(&self, key: &KeyEvent) -> Vec<u8> {
        // Use termwiz's key encoding
        key.encode(
            self.keyboard_encoding,
            self.application_cursor_keys,
            self.application_keypad,
        )
    }
}
```

### 2. Convert crossterm Keys to termwiz

```rust
use crossterm::event::{KeyCode as CtKeyCode, KeyEvent as CtKeyEvent, KeyModifiers};
use termwiz::input::{KeyCode as TwKeyCode, KeyEvent as TwKeyEvent, Modifiers as TwModifiers};

/// Convert crossterm key event to termwiz key event
pub fn convert_key_event(ct_event: &CtKeyEvent) -> TwKeyEvent {
    let modifiers = convert_modifiers(ct_event.modifiers);
    let key = convert_keycode(ct_event.code);
    
    TwKeyEvent {
        key,
        modifiers,
    }
}

fn convert_modifiers(ct_mods: KeyModifiers) -> TwModifiers {
    let mut mods = TwModifiers::NONE;
    
    if ct_mods.contains(KeyModifiers::SHIFT) {
        mods |= TwModifiers::SHIFT;
    }
    if ct_mods.contains(KeyModifiers::CONTROL) {
        mods |= TwModifiers::CTRL;
    }
    if ct_mods.contains(KeyModifiers::ALT) {
        mods |= TwModifiers::ALT;
    }
    
    mods
}

fn convert_keycode(ct_code: CtKeyCode) -> TwKeyCode {
    match ct_code {
        CtKeyCode::Char(c) => TwKeyCode::Char(c),
        CtKeyCode::Enter => TwKeyCode::Enter,
        CtKeyCode::Backspace => TwKeyCode::Backspace,
        CtKeyCode::Tab => TwKeyCode::Tab,
        CtKeyCode::Esc => TwKeyCode::Escape,
        CtKeyCode::Up => TwKeyCode::UpArrow,
        CtKeyCode::Down => TwKeyCode::DownArrow,
        CtKeyCode::Left => TwKeyCode::LeftArrow,
        CtKeyCode::Right => TwKeyCode::RightArrow,
        CtKeyCode::Home => TwKeyCode::Home,
        CtKeyCode::End => TwKeyCode::End,
        CtKeyCode::PageUp => TwKeyCode::PageUp,
        CtKeyCode::PageDown => TwKeyCode::PageDown,
        CtKeyCode::Insert => TwKeyCode::Insert,
        CtKeyCode::Delete => TwKeyCode::Delete,
        CtKeyCode::F(n) => TwKeyCode::Function(n),
        _ => TwKeyCode::Char(' '), // Fallback
    }
}
```

### 3. Update Key Handler

```rust
// In src/main.rs

fn handle_executing_keys(app: &mut App, key: event::KeyEvent) -> Result<bool> {
    // Check for our control keys first
    match key.code {
        // Ctrl+C kills the process
        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            app.kill_execution();
            return Ok(false);
        }
        // Scroll controls (only when paused/scrolled up)
        KeyCode::Up | KeyCode::Char('k') if !app.is_terminal_at_bottom() => {
            app.terminal_scroll_up(1);
            return Ok(false);
        }
        KeyCode::Down | KeyCode::Char('j') if !app.is_terminal_at_bottom() => {
            app.terminal_scroll_down(1);
            return Ok(false);
        }
        KeyCode::Char('g') if !app.is_terminal_at_bottom() => {
            app.terminal_scroll_to_top();
            return Ok(false);
        }
        KeyCode::Char('G') => {
            app.terminal_scroll_to_bottom();
            return Ok(false);
        }
        _ => {}
    }
    
    // Forward all other keys to PTY
    if let Some(ref terminal) = app.embedded_terminal() {
        let tw_event = convert_key_event(&key);
        let encoded = terminal.encode_key(&tw_event);
        app.send_input(&encoded)?;
    } else {
        // Fallback for old implementation
        forward_key_legacy(app, key)?;
    }
    
    Ok(false)
}

/// Legacy key forwarding (for OutputBuffer mode)
fn forward_key_legacy(app: &mut App, key: event::KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            app.send_input(s.as_bytes())?;
        }
        KeyCode::Enter => app.send_input(b"\r")?,
        KeyCode::Backspace => app.send_input(&[0x7f])?,
        KeyCode::Tab => app.send_input(b"\t")?,
        KeyCode::Esc => app.send_input(&[0x1b])?,
        KeyCode::Up => app.send_input(b"\x1b[A")?,
        KeyCode::Down => app.send_input(b"\x1b[B")?,
        KeyCode::Right => app.send_input(b"\x1b[C")?,
        KeyCode::Left => app.send_input(b"\x1b[D")?,
        KeyCode::Home => app.send_input(b"\x1b[H")?,
        KeyCode::End => app.send_input(b"\x1b[F")?,
        KeyCode::PageUp => app.send_input(b"\x1b[5~")?,
        KeyCode::PageDown => app.send_input(b"\x1b[6~")?,
        KeyCode::Delete => app.send_input(b"\x1b[3~")?,
        KeyCode::Insert => app.send_input(b"\x1b[2~")?,
        KeyCode::F(n) => {
            let seq = match n {
                1 => b"\x1bOP".to_vec(),
                2 => b"\x1bOQ".to_vec(),
                3 => b"\x1bOR".to_vec(),
                4 => b"\x1bOS".to_vec(),
                5 => b"\x1b[15~".to_vec(),
                6 => b"\x1b[17~".to_vec(),
                7 => b"\x1b[18~".to_vec(),
                8 => b"\x1b[19~".to_vec(),
                9 => b"\x1b[20~".to_vec(),
                10 => b"\x1b[21~".to_vec(),
                11 => b"\x1b[23~".to_vec(),
                12 => b"\x1b[24~".to_vec(),
                _ => return Ok(()),
            };
            app.send_input(&seq)?;
        }
        _ => {}
    }
    Ok(())
}
```

### 4. Mouse Input (Optional)

For applications that support mouse:

```rust
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};

fn handle_mouse_event(app: &mut App, mouse: MouseEvent) -> Result<()> {
    if !app.is_executing() {
        return Ok(());
    }
    
    // Only forward if terminal supports mouse
    if !app.terminal_mouse_enabled() {
        return Ok(());
    }
    
    let encoded = match mouse.kind {
        MouseEventKind::Down(button) => {
            let btn = match button {
                MouseButton::Left => 0,
                MouseButton::Middle => 1,
                MouseButton::Right => 2,
            };
            // SGR mouse encoding: \x1b[<btn;x;yM
            format!("\x1b[<{};{};{}M", btn, mouse.column + 1, mouse.row + 1)
        }
        MouseEventKind::Up(button) => {
            let btn = match button {
                MouseButton::Left => 0,
                MouseButton::Middle => 1,
                MouseButton::Right => 2,
            };
            format!("\x1b[<{};{};{}m", btn, mouse.column + 1, mouse.row + 1)
        }
        MouseEventKind::ScrollUp => {
            format!("\x1b[<64;{};{}M", mouse.column + 1, mouse.row + 1)
        }
        MouseEventKind::ScrollDown => {
            format!("\x1b[<65;{};{}M", mouse.column + 1, mouse.row + 1)
        }
        _ => return Ok(()),
    };
    
    app.send_input(encoded.as_bytes())?;
    Ok(())
}
```

### 5. Application Mode Tracking

Track when the application requests special modes:

```rust
impl EmbeddedTerminal {
    /// Whether application cursor keys mode is enabled
    application_cursor_keys: bool,
    /// Whether application keypad mode is enabled  
    application_keypad: bool,
    /// Whether mouse reporting is enabled
    mouse_reporting: bool,
    /// Keyboard encoding mode
    keyboard_encoding: KeyboardEncoding,
}

#[derive(Default, Clone, Copy)]
pub enum KeyboardEncoding {
    #[default]
    Legacy,
    /// Kitty keyboard protocol
    Kitty(KittyKeyboardFlags),
}

impl EmbeddedTerminal {
    fn handle_mode(&mut self, mode: Mode) {
        use termwiz::escape::csi::Mode::*;
        
        match mode {
            SetDecPrivateMode(DecPrivateMode::Code(code)) => {
                match code {
                    DecPrivateModeCode::ApplicationCursorKeys => {
                        self.application_cursor_keys = true;
                    }
                    DecPrivateModeCode::MouseTracking => {
                        self.mouse_reporting = true;
                    }
                    // ... other modes
                    _ => {}
                }
            }
            ResetDecPrivateMode(DecPrivateMode::Code(code)) => {
                match code {
                    DecPrivateModeCode::ApplicationCursorKeys => {
                        self.application_cursor_keys = false;
                    }
                    DecPrivateModeCode::MouseTracking => {
                        self.mouse_reporting = false;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    
    /// Check if mouse reporting is enabled
    pub fn mouse_enabled(&self) -> bool {
        self.mouse_reporting
    }
}
```

---

## Special Key Handling

### Ctrl+Key Combinations

```rust
// Ctrl+letter produces ASCII control codes
fn ctrl_key(c: char) -> u8 {
    // Ctrl+A = 0x01, Ctrl+B = 0x02, etc.
    (c.to_ascii_uppercase() as u8) - b'A' + 1
}

// Example: Ctrl+C = 0x03 (but we intercept this for kill)
// Ctrl+D = 0x04 (EOF)
// Ctrl+Z = 0x1A (suspend)
```

### Alt+Key Combinations

```rust
// Alt typically sends ESC prefix
fn alt_key(c: char) -> Vec<u8> {
    vec![0x1b, c as u8]
}
```

---

## Testing

### Key Encoding Test

```rust
#[test]
fn test_arrow_key_encoding() {
    let term = EmbeddedTerminal::default_size();
    
    let up = TwKeyEvent {
        key: TwKeyCode::UpArrow,
        modifiers: TwModifiers::NONE,
    };
    
    let encoded = term.encode_key(&up);
    assert_eq!(encoded, b"\x1b[A");
}

#[test]
fn test_application_cursor_mode() {
    let mut term = EmbeddedTerminal::default_size();
    
    // Enable application cursor keys
    term.write(b"\x1b[?1h");
    
    let up = TwKeyEvent {
        key: TwKeyCode::UpArrow,
        modifiers: TwModifiers::NONE,
    };
    
    let encoded = term.encode_key(&up);
    assert_eq!(encoded, b"\x1bOA"); // Different in app mode
}
```

### Integration Test

```rust
#[test]
fn test_vim_navigation() {
    // Simulate vim-like navigation
    let mut term = EmbeddedTerminal::default_size();
    
    // hjkl should pass through as characters
    let h = convert_key_event(&CtKeyEvent::new(CtKeyCode::Char('h'), KeyModifiers::NONE));
    let encoded = term.encode_key(&h);
    assert_eq!(encoded, b"h");
}
```

---

## Files Changed

| File | Change |
|------|--------|
| `src/terminal.rs` | Add key encoding, mode tracking |
| `src/main.rs` | Update handle_executing_keys |

---

## Acceptance Criteria

- [ ] Arrow keys work in vim/nvim
- [ ] Function keys work
- [ ] Ctrl+key combinations work (except our intercepted ones)
- [ ] Alt+key combinations work
- [ ] Application cursor mode switches correctly
- [ ] Mouse events forwarded when enabled

---

## Phase Complete Checklist

After completing all units:

- [ ] rsync progress bars work correctly
- [ ] Colors preserved in output
- [ ] vim/nvim usable without TUI handover
- [ ] htop renders correctly
- [ ] Performance acceptable
- [ ] All existing tests pass
- [ ] No regressions in basic functionality

---

## Future Enhancements (Not This Phase)

- Sixel image support
- Kitty graphics protocol
- OSC 52 clipboard integration
- Bracketed paste mode
- Focus reporting
