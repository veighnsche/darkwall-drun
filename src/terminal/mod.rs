//! Terminal emulation using termwiz
//!
//! This module provides a proper terminal emulator that handles:
//! - ANSI escape sequences
//! - Cursor positioning
//! - Colors and attributes
//! - Screen buffer management
//!
//! # Module Structure
//!
//! - `widget` - Ratatui widget for rendering terminal content
//! - `input` - Crossterm key conversion utilities

mod input;
mod widget;

// Re-export public API
pub use input::{convert_keycode, convert_modifiers};
pub use widget::TerminalWidget;

use termwiz::cell::{Cell, CellAttributes};
use termwiz::color::ColorAttribute;
use termwiz::escape::csi::{Cursor, Edit, Sgr, CSI};
use termwiz::escape::parser::Parser;
use termwiz::escape::{Action, ControlCode};
use termwiz::input::{KeyCode, KeyCodeEncodeModes, KeyboardEncoding, Modifiers};
use termwiz::surface::Surface;

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

/// Embedded terminal emulator
pub struct EmbeddedTerminal {
    /// The terminal surface (screen buffer)
    surface: Surface,
    /// Escape sequence parser
    parser: Parser,
    /// Configuration
    config: TerminalConfig,
    /// Current cursor position
    cursor: CursorPosition,
    /// Scrollback buffer (lines that scrolled off top)
    scrollback: Vec<Vec<Cell>>,
    /// Scroll offset for viewing (0 = bottom)
    scroll_offset: usize,
    /// Whether in alternate screen mode
    in_alternate_screen: bool,
    /// Saved primary screen (when in alternate)
    saved_primary: Option<Surface>,
    /// Whether to auto-scroll when new content arrives
    follow_mode: bool,
    /// Current text attributes (colors, bold, etc.)
    current_attrs: CellAttributes,
    /// Saved cursor position (for save/restore)
    saved_cursor: Option<CursorPosition>,
    /// Whether application cursor keys mode is enabled
    application_cursor_keys: bool,
    /// Whether newline mode is enabled
    newline_mode: bool,
    /// Keyboard encoding mode
    keyboard_encoding: KeyboardEncoding,
    /// Mouse reporting mode
    mouse_reporting: bool,
}

impl EmbeddedTerminal {
    /// Create a new embedded terminal
    pub fn new(config: TerminalConfig) -> Self {
        let surface = Surface::new(config.cols, config.rows);

        Self {
            surface,
            parser: Parser::new(),
            config,
            cursor: CursorPosition::default(),
            scrollback: Vec::new(),
            scroll_offset: 0,
            in_alternate_screen: false,
            saved_primary: None,
            follow_mode: true,
            current_attrs: CellAttributes::default(),
            saved_cursor: None,
            application_cursor_keys: false,
            newline_mode: false,
            keyboard_encoding: KeyboardEncoding::Xterm,
            mouse_reporting: false,
        }
    }

    /// Create with default 80x24 size
    #[allow(dead_code)] // Public API for tests and future use
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

    /// Get a reference to the surface
    #[allow(dead_code)] // Public API for future use
    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    /// Get the current cursor position
    pub fn cursor(&self) -> CursorPosition {
        self.cursor
    }

    /// Get the scroll offset
    #[allow(dead_code)] // Public API for future use
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get the scrollback buffer
    pub fn scrollback(&self) -> &[Vec<Cell>] {
        &self.scrollback
    }

    /// Check if in alternate screen mode
    #[allow(dead_code)] // Public API for future use
    pub fn in_alternate_screen(&self) -> bool {
        self.in_alternate_screen
    }

    /// Get mutable reference to surface
    #[allow(dead_code)] // Public API for future use
    pub fn surface_mut(&mut self) -> &mut Surface {
        &mut self.surface
    }

    // ========== Scrollback Management ==========

    /// Add a line to scrollback
    fn push_to_scrollback(&mut self, line: Vec<Cell>) {
        self.scrollback.push(line);

        // Enforce max scrollback
        while self.scrollback.len() > self.config.scrollback {
            self.scrollback.remove(0);
        }
    }

    /// Scroll the screen up by n lines, saving to scrollback
    pub fn scroll_screen_up(&mut self, n: usize) {
        use termwiz::surface::{Change, Position};

        for _ in 0..n {
            // Save top line to scrollback
            let lines = self.surface.screen_lines();
            if let Some(top_line) = lines.first() {
                // Extract cells from the line
                let cells: Vec<Cell> = (0..self.config.cols)
                    .map(|i| {
                        top_line
                            .get_cell(i)
                            .map(|cr| cr.as_cell())
                            .unwrap_or_default()
                    })
                    .collect();
                self.push_to_scrollback(cells);
            }

            // Use termwiz's scroll region to scroll up
            self.surface.add_change(Change::ScrollRegionUp {
                first_row: 0,
                region_size: self.config.rows,
                scroll_count: 1,
            });

            // Clear the bottom line
            self.surface.add_change(Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Absolute(self.config.rows - 1),
            });
            self.surface.add_change(Change::ClearToEndOfLine(
                termwiz::color::ColorAttribute::Default,
            ));
        }
    }

    /// Get total scrollable lines (scrollback + visible)
    pub fn total_lines(&self) -> usize {
        self.scrollback.len() + self.config.rows
    }

    /// Set scroll offset (for user scrolling)
    pub fn set_scroll_offset(&mut self, offset: usize) {
        let max_offset = self.scrollback.len();
        self.scroll_offset = offset.min(max_offset);
        // Disable follow mode when user scrolls
        if self.scroll_offset > 0 {
            self.follow_mode = false;
        }
    }

    /// Scroll viewport up (into scrollback history)
    pub fn scroll_up(&mut self, lines: usize) {
        let max_offset = self.scrollback.len();
        self.scroll_offset = (self.scroll_offset + lines).min(max_offset);
        self.follow_mode = false;
    }

    /// Scroll viewport down (toward current output)
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
        if self.scroll_offset == 0 {
            self.follow_mode = true;
        }
    }

    /// Scroll to bottom (follow mode)
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
        self.follow_mode = true;
    }

    /// Check if at bottom (following)
    pub fn is_at_bottom(&self) -> bool {
        self.scroll_offset == 0
    }

    // ========== Visible Content Retrieval ==========

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

        // Get from surface using screen_lines()
        let lines = self.surface.screen_lines();
        if actual_row < lines.len() {
            (0..self.config.cols)
                .map(|i| {
                    lines[actual_row]
                        .get_cell(i)
                        .map(|cr| cr.as_cell())
                        .unwrap_or_default()
                })
                .collect()
        } else {
            // Empty row
            vec![Cell::default(); self.config.cols]
        }
    }

    /// Get all visible rows
    pub fn get_visible_rows(&self) -> Vec<Vec<Cell>> {
        (0..self.config.rows).map(|row| self.get_row(row)).collect()
    }

    // ========== Follow Mode ==========

    /// Enable/disable follow mode
    #[allow(dead_code)] // Public API for future use
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
    pub fn on_content_added(&mut self) {
        if self.follow_mode {
            self.scroll_to_bottom();
        }
    }

    /// Clear the terminal (screen and scrollback)
    pub fn clear(&mut self) {
        self.scrollback.clear();
        self.scroll_offset = 0;
        self.cursor = CursorPosition::default();
        self.current_attrs = CellAttributes::default();
        // Clear surface by recreating it
        self.surface = Surface::new(self.config.cols, self.config.rows);
    }

    // ========== Input Handling ==========

    /// Encode a key for sending to the PTY
    pub fn encode_key(&self, key: KeyCode, modifiers: Modifiers) -> String {
        let modes = KeyCodeEncodeModes {
            encoding: self.keyboard_encoding,
            application_cursor_keys: self.application_cursor_keys,
            newline_mode: self.newline_mode,
            modify_other_keys: None,
        };

        // Encode the key (is_down = true for key press)
        key.encode(modifiers, modes, true).unwrap_or_default()
    }

    /// Check if mouse reporting is enabled
    #[allow(dead_code)] // Public API for future use
    pub fn mouse_enabled(&self) -> bool {
        self.mouse_reporting
    }

    /// Check if application cursor keys mode is enabled
    #[allow(dead_code)] // Public API for future use
    pub fn application_cursor_keys(&self) -> bool {
        self.application_cursor_keys
    }

    // ========== Escape Sequence Handling ==========

    /// Process raw bytes from PTY
    pub fn write(&mut self, data: &[u8]) {
        let actions = self.parser.parse_as_vec(data);

        for action in actions {
            self.handle_action(action);
        }

        // Notify that content was added
        self.on_content_added();
    }

    /// Handle a parsed action
    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Print(c) => self.print_char(c),
            Action::PrintString(s) => self.print_string(&s),
            Action::Control(ctrl) => self.handle_control(ctrl),
            Action::CSI(csi) => self.handle_csi(csi),
            Action::Esc(esc) => self.handle_esc(esc),
            Action::OperatingSystemCommand(osc) => self.handle_osc(*osc),
            Action::DeviceControl(_) => {
                // Device control - uncommon, ignore for now
            }
            Action::Sixel(_) => {
                // Future: image support
            }
            Action::XtGetTcap(_) | Action::KittyImage(_) => {
                // Future: graphics protocol support
            }
        }
    }

    // ========== Character Printing ==========

    /// Print a single character at cursor position
    fn print_char(&mut self, c: char) {
        use termwiz::surface::{Change, Position};

        // Set cell content with current attributes
        self.surface.add_change(Change::CursorPosition {
            x: Position::Absolute(self.cursor.col),
            y: Position::Absolute(self.cursor.row),
        });
        self.surface.add_change(Change::AllAttributes(
            self.current_attrs.clone(),
        ));
        self.surface.add_change(Change::Text(c.to_string()));

        // Advance cursor
        self.cursor.col += 1;

        // Handle line wrap
        if self.cursor.col >= self.config.cols {
            self.cursor.col = 0;
            self.newline();
        }
    }

    /// Print a string
    fn print_string(&mut self, s: &str) {
        for c in s.chars() {
            self.print_char(c);
        }
    }

    // ========== Control Characters ==========

    fn handle_control(&mut self, ctrl: ControlCode) {
        match ctrl {
            ControlCode::Null => {}
            ControlCode::Bell => {
                // Could trigger visual bell - ignore for now
            }
            ControlCode::Backspace => {
                self.cursor.col = self.cursor.col.saturating_sub(1);
            }
            ControlCode::HorizontalTab => {
                // Move to next tab stop (every 8 columns)
                self.cursor.col = ((self.cursor.col / 8) + 1) * 8;
                if self.cursor.col >= self.config.cols {
                    self.cursor.col = self.config.cols - 1;
                }
            }
            ControlCode::LineFeed | ControlCode::VerticalTab | ControlCode::FormFeed => {
                self.newline();
            }
            ControlCode::CarriageReturn => {
                // Move cursor to beginning of line (standard CR behavior)
                self.cursor.col = 0;
            }
            _ => {}
        }
    }

    /// Handle newline - move cursor down, scroll if needed
    fn newline(&mut self) {
        self.cursor.row += 1;

        if self.cursor.row >= self.config.rows {
            // Scroll screen up
            self.scroll_screen_up(1);
            self.cursor.row = self.config.rows - 1;
        }
    }

    // ========== CSI (Control Sequence Introducer) ==========

    fn handle_csi(&mut self, csi: CSI) {
        match csi {
            CSI::Cursor(cursor_op) => self.handle_cursor(cursor_op),
            CSI::Edit(edit_op) => self.handle_edit(edit_op),
            CSI::Sgr(sgr) => self.handle_sgr(sgr),
            CSI::Mode(mode) => self.handle_mode(mode),
            CSI::Device(_device) => {
                // Device queries - ignore for now
            }
            CSI::Window(_) => {
                // Window manipulation - usually ignore
            }
            _ => {
                tracing::debug!("Unhandled CSI: {:?}", csi);
            }
        }
    }

    // ========== Mode Handling ==========

    fn handle_mode(&mut self, mode: termwiz::escape::csi::Mode) {
        use termwiz::escape::csi::{DecPrivateMode, Mode};

        match mode {
            Mode::SetDecPrivateMode(DecPrivateMode::Code(code)) => {
                self.set_dec_mode(code, true);
            }
            Mode::ResetDecPrivateMode(DecPrivateMode::Code(code)) => {
                self.set_dec_mode(code, false);
            }
            _ => {
                tracing::debug!("Unhandled mode: {:?}", mode);
            }
        }
    }

    fn set_dec_mode(&mut self, code: termwiz::escape::csi::DecPrivateModeCode, enable: bool) {
        use termwiz::escape::csi::DecPrivateModeCode;

        match code {
            DecPrivateModeCode::ApplicationCursorKeys => {
                self.application_cursor_keys = enable;
            }
            DecPrivateModeCode::AutoWrap => {
                // Auto-wrap mode - we always wrap, ignore
            }
            DecPrivateModeCode::ShowCursor => {
                // Cursor visibility - could track for rendering
            }
            DecPrivateModeCode::MouseTracking
            | DecPrivateModeCode::HighlightMouseTracking
            | DecPrivateModeCode::ButtonEventMouse
            | DecPrivateModeCode::AnyEventMouse => {
                self.mouse_reporting = enable;
            }
            DecPrivateModeCode::SGRMouse => {
                // SGR mouse encoding - we use this by default
            }
            DecPrivateModeCode::ClearAndEnableAlternateScreen
            | DecPrivateModeCode::EnableAlternateScreen => {
                if enable {
                    // Save primary screen and switch to alternate
                    if !self.in_alternate_screen {
                        self.saved_primary = Some(std::mem::replace(
                            &mut self.surface,
                            Surface::new(self.config.cols, self.config.rows),
                        ));
                        self.in_alternate_screen = true;
                    }
                } else {
                    // Restore primary screen
                    if self.in_alternate_screen {
                        if let Some(primary) = self.saved_primary.take() {
                            self.surface = primary;
                        }
                        self.in_alternate_screen = false;
                    }
                }
            }
            DecPrivateModeCode::BracketedPaste => {
                // Bracketed paste mode - could track for input handling
            }
            _ => {
                tracing::debug!("Unhandled DEC mode: {:?} = {}", code, enable);
            }
        }
    }

    // ========== Cursor Operations ==========

    fn handle_cursor(&mut self, op: Cursor) {
        match op {
            Cursor::Up(n) => {
                self.cursor.row = self.cursor.row.saturating_sub(n as usize);
            }
            Cursor::Down(n) => {
                self.cursor.row = (self.cursor.row + n as usize).min(self.config.rows - 1);
            }
            Cursor::Left(n) => {
                self.cursor.col = self.cursor.col.saturating_sub(n as usize);
            }
            Cursor::Right(n) => {
                self.cursor.col = (self.cursor.col + n as usize).min(self.config.cols - 1);
            }
            Cursor::Position { line, col } => {
                // CSI row;col H - 1-indexed in escape sequence
                // OneBased::as_one_based() returns u32
                self.cursor.row = (line.as_one_based() as usize).saturating_sub(1).min(self.config.rows - 1);
                self.cursor.col = (col.as_one_based() as usize).saturating_sub(1).min(self.config.cols - 1);
            }
            Cursor::CharacterAndLinePosition { line, col } => {
                // HVP - same as Position
                self.cursor.row = (line.as_one_based() as usize).saturating_sub(1).min(self.config.rows - 1);
                self.cursor.col = (col.as_one_based() as usize).saturating_sub(1).min(self.config.cols - 1);
            }
            Cursor::CharacterPositionAbsolute(col) | Cursor::CharacterAbsolute(col) => {
                // OneBased column position
                self.cursor.col = (col.as_one_based() as usize).saturating_sub(1).min(self.config.cols - 1);
            }
            Cursor::LinePositionAbsolute(row) => {
                // VPA - 1-indexed row as u32
                self.cursor.row = (row as usize).saturating_sub(1).min(self.config.rows - 1);
            }
            Cursor::SaveCursor => {
                self.saved_cursor = Some(self.cursor);
            }
            Cursor::RestoreCursor => {
                if let Some(pos) = self.saved_cursor {
                    self.cursor = pos;
                }
            }
            Cursor::NextLine(n) => {
                self.cursor.col = 0;
                self.cursor.row = (self.cursor.row + n as usize).min(self.config.rows - 1);
            }
            Cursor::PrecedingLine(n) => {
                self.cursor.col = 0;
                self.cursor.row = self.cursor.row.saturating_sub(n as usize);
            }
            Cursor::CharacterPositionForward(n) => {
                self.cursor.col = (self.cursor.col + n as usize).min(self.config.cols - 1);
            }
            Cursor::CharacterPositionBackward(n) => {
                self.cursor.col = self.cursor.col.saturating_sub(n as usize);
            }
            Cursor::LinePositionForward(n) => {
                self.cursor.row = (self.cursor.row + n as usize).min(self.config.rows - 1);
            }
            Cursor::LinePositionBackward(n) => {
                self.cursor.row = self.cursor.row.saturating_sub(n as usize);
            }
            _ => {
                tracing::debug!("Unhandled cursor op: {:?}", op);
            }
        }
    }

    // ========== Edit Operations ==========

    fn handle_edit(&mut self, op: Edit) {
        use termwiz::escape::csi::{EraseInDisplay, EraseInLine};
        use termwiz::surface::{Change, Position};

        match op {
            Edit::EraseInLine(erase) => {
                match erase {
                    EraseInLine::EraseToEndOfLine => {
                        // Clear from cursor to end of line
                        self.surface.add_change(Change::CursorPosition {
                            x: Position::Absolute(self.cursor.col),
                            y: Position::Absolute(self.cursor.row),
                        });
                        self.surface.add_change(Change::ClearToEndOfLine(
                            ColorAttribute::Default,
                        ));
                    }
                    EraseInLine::EraseToStartOfLine => {
                        // Clear from start to cursor
                        self.surface.add_change(Change::CursorPosition {
                            x: Position::Absolute(0),
                            y: Position::Absolute(self.cursor.row),
                        });
                        for _ in 0..=self.cursor.col {
                            self.surface.add_change(Change::Text(" ".to_string()));
                        }
                    }
                    EraseInLine::EraseLine => {
                        // Clear entire line
                        self.surface.add_change(Change::CursorPosition {
                            x: Position::Absolute(0),
                            y: Position::Absolute(self.cursor.row),
                        });
                        self.surface.add_change(Change::ClearToEndOfLine(
                            ColorAttribute::Default,
                        ));
                    }
                }
            }
            Edit::EraseInDisplay(erase) => {
                match erase {
                    EraseInDisplay::EraseToEndOfDisplay => {
                        // Clear from cursor to end of screen
                        self.surface.add_change(Change::CursorPosition {
                            x: Position::Absolute(self.cursor.col),
                            y: Position::Absolute(self.cursor.row),
                        });
                        self.surface.add_change(Change::ClearToEndOfLine(
                            ColorAttribute::Default,
                        ));
                        for y in (self.cursor.row + 1)..self.config.rows {
                            self.surface.add_change(Change::CursorPosition {
                                x: Position::Absolute(0),
                                y: Position::Absolute(y),
                            });
                            self.surface.add_change(Change::ClearToEndOfLine(
                                ColorAttribute::Default,
                            ));
                        }
                    }
                    EraseInDisplay::EraseToStartOfDisplay => {
                        // Clear from start to cursor
                        for y in 0..self.cursor.row {
                            self.surface.add_change(Change::CursorPosition {
                                x: Position::Absolute(0),
                                y: Position::Absolute(y),
                            });
                            self.surface.add_change(Change::ClearToEndOfLine(
                                ColorAttribute::Default,
                            ));
                        }
                        self.surface.add_change(Change::CursorPosition {
                            x: Position::Absolute(0),
                            y: Position::Absolute(self.cursor.row),
                        });
                        for _ in 0..=self.cursor.col {
                            self.surface.add_change(Change::Text(" ".to_string()));
                        }
                    }
                    EraseInDisplay::EraseDisplay => {
                        // Clear entire screen
                        self.surface.add_change(Change::ClearScreen(
                            ColorAttribute::Default,
                        ));
                    }
                    EraseInDisplay::EraseScrollback => {
                        self.scrollback.clear();
                    }
                }
            }
            Edit::DeleteCharacter(n) => {
                // Delete n characters at cursor, shifting rest left
                // For simplicity, just clear them
                self.surface.add_change(Change::CursorPosition {
                    x: Position::Absolute(self.cursor.col),
                    y: Position::Absolute(self.cursor.row),
                });
                for _ in 0..n {
                    self.surface.add_change(Change::Text(" ".to_string()));
                }
            }
            Edit::DeleteLine(n) => {
                // Delete n lines at cursor, scrolling rest up
                for _ in 0..n {
                    self.surface.add_change(Change::ScrollRegionUp {
                        first_row: self.cursor.row,
                        region_size: self.config.rows - self.cursor.row,
                        scroll_count: 1,
                    });
                }
            }
            Edit::InsertLine(n) => {
                // Insert n blank lines at cursor, scrolling rest down
                for _ in 0..n {
                    self.surface.add_change(Change::ScrollRegionDown {
                        first_row: self.cursor.row,
                        region_size: self.config.rows - self.cursor.row,
                        scroll_count: 1,
                    });
                }
            }
            _ => {
                tracing::debug!("Unhandled edit op: {:?}", op);
            }
        }
    }

    // ========== SGR (Select Graphic Rendition) ==========

    fn handle_sgr(&mut self, sgr: Sgr) {
        match sgr {
            Sgr::Reset => {
                self.current_attrs = CellAttributes::default();
            }
            Sgr::Intensity(intensity) => {
                self.current_attrs.set_intensity(intensity);
            }
            Sgr::Underline(underline) => {
                self.current_attrs.set_underline(underline);
            }
            Sgr::Blink(blink) => {
                self.current_attrs.set_blink(blink);
            }
            Sgr::Italic(italic) => {
                self.current_attrs.set_italic(italic);
            }
            Sgr::Inverse(inverse) => {
                self.current_attrs.set_reverse(inverse);
            }
            Sgr::Invisible(invisible) => {
                self.current_attrs.set_invisible(invisible);
            }
            Sgr::StrikeThrough(strike) => {
                self.current_attrs.set_strikethrough(strike);
            }
            Sgr::Foreground(color) => {
                self.current_attrs.set_foreground(color);
            }
            Sgr::Background(color) => {
                self.current_attrs.set_background(color);
            }
            Sgr::UnderlineColor(color) => {
                self.current_attrs.set_underline_color(color);
            }
            Sgr::Overline(overline) => {
                self.current_attrs.set_overline(overline);
            }
            Sgr::Font(_) => {
                // Font selection - ignore
            }
            Sgr::VerticalAlign(_) => {
                // Vertical alignment - ignore
            }
        }
    }

    // ========== ESC Sequences ==========

    fn handle_esc(&mut self, esc: termwiz::escape::Esc) {
        use termwiz::escape::esc::EscCode;
        use termwiz::escape::Esc;

        match esc {
            Esc::Code(EscCode::DecSaveCursorPosition) => {
                self.saved_cursor = Some(self.cursor);
            }
            Esc::Code(EscCode::DecRestoreCursorPosition) => {
                if let Some(pos) = self.saved_cursor {
                    self.cursor = pos;
                }
            }
            Esc::Code(EscCode::ReverseIndex) => {
                // Move cursor up, scroll down if at top
                if self.cursor.row == 0 {
                    self.surface.add_change(termwiz::surface::Change::ScrollRegionDown {
                        first_row: 0,
                        region_size: self.config.rows,
                        scroll_count: 1,
                    });
                } else {
                    self.cursor.row -= 1;
                }
            }
            Esc::Code(EscCode::Index) => {
                // Move cursor down, scroll up if at bottom
                self.newline();
            }
            Esc::Code(EscCode::NextLine) => {
                self.cursor.col = 0;
                self.newline();
            }
            Esc::Code(EscCode::FullReset) => {
                self.clear();
            }
            _ => {
                tracing::debug!("Unhandled ESC: {:?}", esc);
            }
        }
    }

    // ========== OSC (Operating System Command) ==========

    fn handle_osc(&mut self, osc: termwiz::escape::osc::OperatingSystemCommand) {
        use termwiz::escape::osc::OperatingSystemCommand;

        match osc {
            OperatingSystemCommand::SetWindowTitle(title)
            | OperatingSystemCommand::SetWindowTitleSun(title)
            | OperatingSystemCommand::SetIconNameAndWindowTitle(title) => {
                // Could store title for display - ignore for now
                tracing::debug!("Window title: {}", title);
            }
            _ => {
                tracing::debug!("Unhandled OSC: {:?}", osc);
            }
        }
    }
}

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
        term.cursor = CursorPosition::new(10, 5);
        term.scroll_offset = 3;

        // Clear
        term.clear();

        // Verify reset
        assert_eq!(term.cursor.col, 0);
        assert_eq!(term.cursor.row, 0);
        assert_eq!(term.scroll_offset, 0);
        assert!(term.scrollback.is_empty());
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
        assert_eq!(term.cursor.col, 5);
        assert_eq!(term.cursor.row, 0);

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
        assert_eq!(term.cursor.row, 1);

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
        assert_eq!(term.cursor.row, 4);
        assert_eq!(term.cursor.col, 9);
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
        assert_eq!(term.cursor.row, 3);

        // Move down 1
        term.write(b"\x1b[1B");
        assert_eq!(term.cursor.row, 4);

        // Move right 3
        term.write(b"\x1b[3C");
        assert_eq!(term.cursor.col, 8);

        // Move left 2
        term.write(b"\x1b[2D");
        assert_eq!(term.cursor.col, 6);
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
        assert_eq!(
            term.current_attrs.foreground(),
            ColorAttribute::Default
        );
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
        assert_eq!(term.cursor.row, 1);
        assert_eq!(term.cursor.col, 3); // "ABC" = 3 chars

        let row0 = term.get_row(0);
        let text0: String = row0.iter().map(|c| c.str()).collect();
        assert_eq!(text0, "1234567890");

        let row1 = term.get_row(1);
        let text1: String = row1.iter().map(|c| c.str()).collect();
        assert!(text1.starts_with("ABC"));
    }
}
