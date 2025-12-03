//! Tests for the terminal emulator

use super::*;
use termwiz::color::ColorAttribute;

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

#[test]
fn test_follow_mode_default() {
    let term = EmbeddedTerminal::default_size();
    assert!(term.is_following());
    assert!(term.is_at_bottom());
}

#[test]
fn test_scroll_offset() {
    let mut term = EmbeddedTerminal::new(TerminalConfig {
        cols: 80,
        rows: 5,
        scrollback: 100,
        ..Default::default()
    });

    // Initially at bottom
    assert!(term.is_at_bottom());
    assert_eq!(term.scroll_offset(), 0);

    // Can't scroll up with no scrollback
    term.set_scroll_offset(10);
    assert_eq!(term.scroll_offset(), 0); // Clamped to max

    // Scroll to bottom
    term.scroll_to_bottom();
    assert!(term.is_at_bottom());
}

#[test]
fn test_follow_mode_toggle() {
    let mut term = EmbeddedTerminal::default_size();

    // Disable follow mode
    term.set_follow_mode(false);
    assert!(!term.is_following());

    // Enable follow mode (should also scroll to bottom)
    term.set_scroll_offset(5); // Pretend we scrolled up
    term.set_follow_mode(true);
    assert!(term.is_following());
    assert!(term.is_at_bottom());
}

#[test]
fn test_total_lines() {
    let term = EmbeddedTerminal::new(TerminalConfig {
        cols: 80,
        rows: 24,
        scrollback: 100,
        ..Default::default()
    });

    // Initially just visible rows
    assert_eq!(term.total_lines(), 24);
}

#[test]
fn test_clear() {
    let mut term = EmbeddedTerminal::new(TerminalConfig {
        cols: 80,
        rows: 5,
        scrollback: 100,
        ..Default::default()
    });

    // Modify state
    term.write(b"Hello");
    term.set_scroll_offset(3);

    // Clear
    term.clear();

    // Verify reset
    assert_eq!(term.cursor().col, 0);
    assert_eq!(term.cursor().row, 0);
    assert_eq!(term.scroll_offset(), 0);
    assert!(term.scrollback().is_empty());
}

#[test]
fn test_get_row_empty() {
    let term = EmbeddedTerminal::new(TerminalConfig {
        cols: 10,
        rows: 5,
        scrollback: 100,
        ..Default::default()
    });

    // Get a row from empty terminal
    let row = term.get_row(0);
    assert_eq!(row.len(), 10);
    // All cells should be default (space)
    for cell in &row {
        assert_eq!(cell.str(), " ");
    }
}

#[test]
fn test_get_visible_rows() {
    let term = EmbeddedTerminal::new(TerminalConfig {
        cols: 10,
        rows: 5,
        scrollback: 100,
        ..Default::default()
    });

    let rows = term.get_visible_rows();
    assert_eq!(rows.len(), 5);
    for row in &rows {
        assert_eq!(row.len(), 10);
    }
}

// ========== Escape Sequence Tests ==========

#[test]
fn test_write_plain_text() {
    let mut term = EmbeddedTerminal::new(TerminalConfig {
        cols: 20,
        rows: 5,
        scrollback: 100,
        ..Default::default()
    });

    term.write(b"Hello");

    // Cursor should have advanced
    assert_eq!(term.cursor().col, 5);
    assert_eq!(term.cursor().row, 0);

    // Check content
    let row = term.get_row(0);
    let text: String = row.iter().map(|c| c.str()).collect();
    assert!(text.starts_with("Hello"));
}

#[test]
fn test_carriage_return() {
    let mut term = EmbeddedTerminal::new(TerminalConfig {
        cols: 20,
        rows: 5,
        scrollback: 100,
        ..Default::default()
    });

    // Simulate progress bar: "Progress: 50%\rProgress: 100%"
    term.write(b"Progress: 50%");
    term.write(b"\r");
    term.write(b"Progress: 100%");

    // Should show "Progress: 100%" overwriting the previous text
    let row = term.get_row(0);
    let text: String = row.iter().map(|c| c.str()).collect();
    assert!(text.starts_with("Progress: 100%"));
}

#[test]
fn test_newline() {
    let mut term = EmbeddedTerminal::new(TerminalConfig {
        cols: 20,
        rows: 5,
        scrollback: 100,
        ..Default::default()
    });

    // Use \r\n for proper line break (CR+LF)
    // \n alone only moves down, doesn't reset column
    term.write(b"Line 1\r\nLine 2");

    // Cursor should be on second line
    assert_eq!(term.cursor().row, 1);

    // Check both lines
    let row0 = term.get_row(0);
    let text0: String = row0.iter().map(|c| c.str()).collect();
    // Line 1 should be on first row
    assert!(text0.starts_with("Line 1"), "row0: '{}'", text0);

    let row1 = term.get_row(1);
    let text1: String = row1.iter().map(|c| c.str()).collect();
    // Line 2 should be on second row
    assert!(text1.starts_with("Line 2"), "row1: '{}'", text1);
}

#[test]
fn test_cursor_position() {
    let mut term = EmbeddedTerminal::new(TerminalConfig {
        cols: 20,
        rows: 10,
        scrollback: 100,
        ..Default::default()
    });

    // Move to row 5, col 10 (1-indexed in escape sequence)
    term.write(b"\x1b[5;10H");

    // Should be at row 4, col 9 (0-indexed)
    assert_eq!(term.cursor().row, 4);
    assert_eq!(term.cursor().col, 9);
}

#[test]
fn test_cursor_movement() {
    let mut term = EmbeddedTerminal::new(TerminalConfig {
        cols: 20,
        rows: 10,
        scrollback: 100,
        ..Default::default()
    });

    // Start at 5,5
    term.write(b"\x1b[6;6H"); // 1-indexed

    // Move up 2
    term.write(b"\x1b[2A");
    assert_eq!(term.cursor().row, 3);

    // Move down 1
    term.write(b"\x1b[1B");
    assert_eq!(term.cursor().row, 4);

    // Move right 3
    term.write(b"\x1b[3C");
    assert_eq!(term.cursor().col, 8);

    // Move left 2
    term.write(b"\x1b[2D");
    assert_eq!(term.cursor().col, 6);
}

#[test]
fn test_clear_to_end_of_line() {
    let mut term = EmbeddedTerminal::new(TerminalConfig {
        cols: 20,
        rows: 5,
        scrollback: 100,
        ..Default::default()
    });

    term.write(b"Hello World");
    term.write(b"\x1b[6G"); // Move to column 6 (1-indexed, so col 5)
    term.write(b"\x1b[K"); // Clear to end of line

    let row = term.get_row(0);
    let text: String = row.iter().map(|c| c.str()).collect();
    // "Hello" should remain, " World" should be cleared
    assert!(text.starts_with("Hello"));
    assert!(!text.contains("World"));
}

#[test]
fn test_sgr_reset() {
    let mut term = EmbeddedTerminal::default_size();

    // Set some attributes then reset
    term.write(b"\x1b[1;31m"); // Bold red
    term.write(b"\x1b[m"); // Reset

    // Attributes should be default
    assert_eq!(term.current_attrs().foreground(), ColorAttribute::Default);
}

#[test]
fn test_line_wrap() {
    let mut term = EmbeddedTerminal::new(TerminalConfig {
        cols: 10,
        rows: 5,
        scrollback: 100,
        ..Default::default()
    });

    // Write more than one line's worth
    term.write(b"1234567890ABC");

    // Should have wrapped to second line
    assert_eq!(term.cursor().row, 1);
    assert_eq!(term.cursor().col, 3); // "ABC" = 3 chars

    let row0 = term.get_row(0);
    let text0: String = row0.iter().map(|c| c.str()).collect();
    assert_eq!(text0, "1234567890");

    let row1 = term.get_row(1);
    let text1: String = row1.iter().map(|c| c.str()).collect();
    assert!(text1.starts_with("ABC"));
}
