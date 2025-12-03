//! Crossterm key conversion utilities
//!
//! Provides conversion from crossterm key events to termwiz key codes
//! for forwarding input to the embedded terminal.

use termwiz::input::{KeyCode, Modifiers};

use super::EmbeddedTerminal;

/// Convert crossterm key modifiers to termwiz modifiers
pub fn convert_modifiers(ct_mods: crossterm::event::KeyModifiers) -> Modifiers {
    use crossterm::event::KeyModifiers;

    let mut mods = Modifiers::NONE;

    if ct_mods.contains(KeyModifiers::SHIFT) {
        mods |= Modifiers::SHIFT;
    }
    if ct_mods.contains(KeyModifiers::CONTROL) {
        mods |= Modifiers::CTRL;
    }
    if ct_mods.contains(KeyModifiers::ALT) {
        mods |= Modifiers::ALT;
    }

    mods
}

/// Convert crossterm key code to termwiz key code
pub fn convert_keycode(ct_code: crossterm::event::KeyCode) -> KeyCode {
    use crossterm::event::KeyCode as CtKeyCode;

    match ct_code {
        CtKeyCode::Char(c) => KeyCode::Char(c),
        CtKeyCode::Enter => KeyCode::Enter,
        CtKeyCode::Backspace => KeyCode::Backspace,
        CtKeyCode::Tab => KeyCode::Tab,
        CtKeyCode::BackTab => KeyCode::Tab, // Shift+Tab handled via modifiers
        CtKeyCode::Esc => KeyCode::Escape,
        CtKeyCode::Up => KeyCode::UpArrow,
        CtKeyCode::Down => KeyCode::DownArrow,
        CtKeyCode::Left => KeyCode::LeftArrow,
        CtKeyCode::Right => KeyCode::RightArrow,
        CtKeyCode::Home => KeyCode::Home,
        CtKeyCode::End => KeyCode::End,
        CtKeyCode::PageUp => KeyCode::PageUp,
        CtKeyCode::PageDown => KeyCode::PageDown,
        CtKeyCode::Insert => KeyCode::Insert,
        CtKeyCode::Delete => KeyCode::Delete,
        CtKeyCode::F(n) => KeyCode::Function(n),
        CtKeyCode::Null => KeyCode::Char('\0'),
        CtKeyCode::CapsLock => KeyCode::CapsLock,
        CtKeyCode::ScrollLock => KeyCode::ScrollLock,
        CtKeyCode::NumLock => KeyCode::NumLock,
        CtKeyCode::PrintScreen => KeyCode::PrintScreen,
        CtKeyCode::Pause => KeyCode::Pause,
        CtKeyCode::Menu => KeyCode::Menu,
        _ => KeyCode::Char(' '), // Fallback for unmapped keys
    }
}

/// Convert a crossterm key event to encoded bytes for the PTY
#[allow(dead_code)] // Public API for future use
pub fn encode_crossterm_key(
    terminal: &EmbeddedTerminal,
    key: &crossterm::event::KeyEvent,
) -> String {
    let modifiers = convert_modifiers(key.modifiers);
    let keycode = convert_keycode(key.code);
    terminal.encode_key(keycode, modifiers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::{EmbeddedTerminal, TerminalConfig};

    #[test]
    fn test_arrow_key_encoding() {
        let term = EmbeddedTerminal::new(TerminalConfig::default());

        // Normal mode: arrow keys use CSI sequences
        let encoded = term.encode_key(KeyCode::UpArrow, Modifiers::NONE);
        assert_eq!(encoded, "\x1b[A");

        let encoded = term.encode_key(KeyCode::DownArrow, Modifiers::NONE);
        assert_eq!(encoded, "\x1b[B");

        let encoded = term.encode_key(KeyCode::RightArrow, Modifiers::NONE);
        assert_eq!(encoded, "\x1b[C");

        let encoded = term.encode_key(KeyCode::LeftArrow, Modifiers::NONE);
        assert_eq!(encoded, "\x1b[D");
    }

    #[test]
    fn test_char_encoding() {
        let term = EmbeddedTerminal::new(TerminalConfig::default());

        // Regular characters
        let encoded = term.encode_key(KeyCode::Char('a'), Modifiers::NONE);
        assert_eq!(encoded, "a");

        let encoded = term.encode_key(KeyCode::Char('Z'), Modifiers::NONE);
        assert_eq!(encoded, "Z");
    }

    #[test]
    fn test_enter_encoding() {
        let term = EmbeddedTerminal::new(TerminalConfig::default());

        let encoded = term.encode_key(KeyCode::Enter, Modifiers::NONE);
        assert_eq!(encoded, "\r");
    }

    #[test]
    fn test_function_key_encoding() {
        let term = EmbeddedTerminal::new(TerminalConfig::default());

        // F1-F4 use SS3 sequences
        let encoded = term.encode_key(KeyCode::Function(1), Modifiers::NONE);
        assert_eq!(encoded, "\x1bOP");

        let encoded = term.encode_key(KeyCode::Function(2), Modifiers::NONE);
        assert_eq!(encoded, "\x1bOQ");
    }

    #[test]
    fn test_crossterm_key_conversion() {
        use crossterm::event::{KeyCode as CtKeyCode, KeyModifiers};

        // Test modifier conversion
        let mods = convert_modifiers(KeyModifiers::CONTROL | KeyModifiers::SHIFT);
        assert!(mods.contains(Modifiers::CTRL));
        assert!(mods.contains(Modifiers::SHIFT));

        // Test keycode conversion
        assert!(matches!(convert_keycode(CtKeyCode::Enter), KeyCode::Enter));
        assert!(matches!(convert_keycode(CtKeyCode::Up), KeyCode::UpArrow));
        assert!(matches!(convert_keycode(CtKeyCode::Char('x')), KeyCode::Char('x')));
    }

    #[test]
    fn test_application_cursor_mode() {
        let mut term = EmbeddedTerminal::new(TerminalConfig::default());

        // Initially in normal mode
        assert!(!term.application_cursor_keys());
        let encoded = term.encode_key(KeyCode::UpArrow, Modifiers::NONE);
        assert_eq!(encoded, "\x1b[A"); // CSI A

        // Enable application cursor keys mode: \x1b[?1h
        term.write(b"\x1b[?1h");
        assert!(term.application_cursor_keys());

        // Now arrow keys should use SS3 sequences
        let encoded = term.encode_key(KeyCode::UpArrow, Modifiers::NONE);
        assert_eq!(encoded, "\x1bOA"); // SS3 A

        // Disable application cursor keys mode: \x1b[?1l
        term.write(b"\x1b[?1l");
        assert!(!term.application_cursor_keys());

        // Back to CSI sequences
        let encoded = term.encode_key(KeyCode::UpArrow, Modifiers::NONE);
        assert_eq!(encoded, "\x1b[A");
    }

    #[test]
    fn test_mouse_mode() {
        let mut term = EmbeddedTerminal::new(TerminalConfig::default());

        // Initially mouse reporting is off
        assert!(!term.mouse_enabled());

        // Enable mouse tracking: \x1b[?1000h
        term.write(b"\x1b[?1000h");
        assert!(term.mouse_enabled());

        // Disable mouse tracking: \x1b[?1000l
        term.write(b"\x1b[?1000l");
        assert!(!term.mouse_enabled());
    }
}
