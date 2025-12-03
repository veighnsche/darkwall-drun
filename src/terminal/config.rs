//! Terminal configuration and basic types

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
    #[allow(dead_code)] // Config option for future use
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

/// Cursor position (column, row)
#[derive(Debug, Clone, Copy, Default)]
pub struct CursorPosition {
    pub col: usize,
    pub row: usize,
}

impl CursorPosition {
    #[allow(dead_code)] // Used in tests
    pub fn new(col: usize, row: usize) -> Self {
        Self { col, row }
    }
}
