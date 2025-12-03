//! Core terminal emulator implementation
//!
//! This module contains the main `EmbeddedTerminal` struct and its implementation,
//! delegating escape sequence handling to the `escape_handlers` module.

use termwiz::cell::{Cell, CellAttributes};
use termwiz::color::ColorAttribute;
use termwiz::input::{KeyCode, KeyCodeEncodeModes, KeyboardEncoding, Modifiers};
use termwiz::escape::parser::Parser;
use termwiz::surface::Surface;

use super::config::{CursorPosition, TerminalConfig};

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

    // ========== Basic Accessors ==========

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

    /// Get current attributes (for tests)
    #[cfg(test)]
    pub(crate) fn current_attrs(&self) -> &CellAttributes {
        &self.current_attrs
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
            self.surface
                .add_change(Change::ClearToEndOfLine(ColorAttribute::Default));
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
    #[allow(dead_code)] // Used in tests
    pub fn get_visible_rows(&self) -> Vec<Vec<Cell>> {
        (0..self.config.rows).map(|row| self.get_row(row)).collect()
    }

    /// Get all terminal content as text (scrollback + visible)
    pub fn content_as_text(&self) -> String {
        let mut lines = Vec::new();

        // Scrollback lines
        for row in &self.scrollback {
            let line: String = row
                .iter()
                .map(|cell| cell.str())
                .collect::<String>()
                .trim_end()
                .to_string();
            lines.push(line);
        }

        // Visible rows
        let screen_lines = self.surface.screen_lines();
        for line in screen_lines {
            let text: String = (0..self.config.cols)
                .map(|i| {
                    line.get_cell(i)
                        .map(|cr| cr.as_cell().str().to_string())
                        .unwrap_or_default()
                })
                .collect::<String>()
                .trim_end()
                .to_string();
            lines.push(text);
        }

        // Trim trailing empty lines
        while lines.last().map(|s| s.is_empty()).unwrap_or(false) {
            lines.pop();
        }

        lines.join("\n")
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

    // ========== Escape Sequence Processing ==========

    /// Process raw bytes from PTY
    pub fn write(&mut self, data: &[u8]) {
        let actions = self.parser.parse_as_vec(data);

        for action in actions {
            self.handle_action(action);
        }

        // Notify that content was added
        self.on_content_added();
    }

    /// Handle a parsed action by delegating to EscapeHandler
    fn handle_action(&mut self, action: termwiz::escape::Action) {
        // Check for full reset first (needs special handling)
        if matches!(
            &action,
            termwiz::escape::Action::Esc(termwiz::escape::Esc::Code(
                termwiz::escape::esc::EscCode::FullReset
            ))
        ) {
            self.clear();
            return;
        }

        match action {
            termwiz::escape::Action::Print(c) => {
                self.print_char(c);
            }
            termwiz::escape::Action::PrintString(s) => {
                for c in s.chars() {
                    self.print_char(c);
                }
            }
            termwiz::escape::Action::Control(ctrl) => {
                self.handle_control(ctrl);
            }
            termwiz::escape::Action::CSI(csi) => {
                self.handle_csi(csi);
            }
            termwiz::escape::Action::Esc(esc) => {
                self.handle_esc(esc);
            }
            termwiz::escape::Action::OperatingSystemCommand(osc) => {
                self.handle_osc(*osc);
            }
            termwiz::escape::Action::DeviceControl(_)
            | termwiz::escape::Action::Sixel(_)
            | termwiz::escape::Action::XtGetTcap(_)
            | termwiz::escape::Action::KittyImage(_) => {
                // Ignored for now
            }
        }
    }

    // ========== Inline Action Handlers ==========
    // These are kept here to avoid complex borrowing issues with the EscapeHandler

    fn print_char(&mut self, c: char) {
        use termwiz::surface::{Change, Position};

        self.surface.add_change(Change::CursorPosition {
            x: Position::Absolute(self.cursor.col),
            y: Position::Absolute(self.cursor.row),
        });
        self.surface
            .add_change(Change::AllAttributes(self.current_attrs.clone()));
        self.surface.add_change(Change::Text(c.to_string()));

        self.cursor.col += 1;

        if self.cursor.col >= self.config.cols {
            self.cursor.col = 0;
            self.newline();
        }
    }

    fn newline(&mut self) {
        self.cursor.row += 1;

        if self.cursor.row >= self.config.rows {
            self.scroll_screen_up(1);
            self.cursor.row = self.config.rows - 1;
        }
    }

    fn handle_control(&mut self, ctrl: termwiz::escape::ControlCode) {
        use termwiz::escape::ControlCode;

        match ctrl {
            ControlCode::Null => {}
            ControlCode::Bell => {}
            ControlCode::Backspace => {
                self.cursor.col = self.cursor.col.saturating_sub(1);
            }
            ControlCode::HorizontalTab => {
                self.cursor.col = ((self.cursor.col / 8) + 1) * 8;
                if self.cursor.col >= self.config.cols {
                    self.cursor.col = self.config.cols - 1;
                }
            }
            ControlCode::LineFeed | ControlCode::VerticalTab | ControlCode::FormFeed => {
                self.newline();
            }
            ControlCode::CarriageReturn => {
                self.cursor.col = 0;
            }
            _ => {}
        }
    }

    fn handle_csi(&mut self, csi: termwiz::escape::csi::CSI) {
        use termwiz::escape::csi::CSI;

        match csi {
            CSI::Cursor(op) => self.handle_cursor(op),
            CSI::Edit(op) => self.handle_edit(op),
            CSI::Sgr(sgr) => self.handle_sgr(sgr),
            CSI::Mode(mode) => self.handle_mode(mode),
            CSI::Device(_) | CSI::Window(_) => {}
            _ => {
                tracing::debug!("Unhandled CSI: {:?}", csi);
            }
        }
    }

    fn handle_cursor(&mut self, op: termwiz::escape::csi::Cursor) {
        use termwiz::escape::csi::Cursor;

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
                self.cursor.row = (line.as_one_based() as usize)
                    .saturating_sub(1)
                    .min(self.config.rows - 1);
                self.cursor.col = (col.as_one_based() as usize)
                    .saturating_sub(1)
                    .min(self.config.cols - 1);
            }
            Cursor::CharacterAndLinePosition { line, col } => {
                self.cursor.row = (line.as_one_based() as usize)
                    .saturating_sub(1)
                    .min(self.config.rows - 1);
                self.cursor.col = (col.as_one_based() as usize)
                    .saturating_sub(1)
                    .min(self.config.cols - 1);
            }
            Cursor::CharacterPositionAbsolute(col) | Cursor::CharacterAbsolute(col) => {
                self.cursor.col = (col.as_one_based() as usize)
                    .saturating_sub(1)
                    .min(self.config.cols - 1);
            }
            Cursor::LinePositionAbsolute(row) => {
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

    fn handle_edit(&mut self, op: termwiz::escape::csi::Edit) {
        use termwiz::escape::csi::{Edit, EraseInDisplay, EraseInLine};
        use termwiz::surface::{Change, Position};

        match op {
            Edit::EraseInLine(erase) => match erase {
                EraseInLine::EraseToEndOfLine => {
                    self.surface.add_change(Change::CursorPosition {
                        x: Position::Absolute(self.cursor.col),
                        y: Position::Absolute(self.cursor.row),
                    });
                    self.surface
                        .add_change(Change::ClearToEndOfLine(ColorAttribute::Default));
                }
                EraseInLine::EraseToStartOfLine => {
                    self.surface.add_change(Change::CursorPosition {
                        x: Position::Absolute(0),
                        y: Position::Absolute(self.cursor.row),
                    });
                    for _ in 0..=self.cursor.col {
                        self.surface.add_change(Change::Text(" ".to_string()));
                    }
                }
                EraseInLine::EraseLine => {
                    self.surface.add_change(Change::CursorPosition {
                        x: Position::Absolute(0),
                        y: Position::Absolute(self.cursor.row),
                    });
                    self.surface
                        .add_change(Change::ClearToEndOfLine(ColorAttribute::Default));
                }
            },
            Edit::EraseInDisplay(erase) => match erase {
                EraseInDisplay::EraseToEndOfDisplay => {
                    self.surface.add_change(Change::CursorPosition {
                        x: Position::Absolute(self.cursor.col),
                        y: Position::Absolute(self.cursor.row),
                    });
                    self.surface
                        .add_change(Change::ClearToEndOfLine(ColorAttribute::Default));
                    for y in (self.cursor.row + 1)..self.config.rows {
                        self.surface.add_change(Change::CursorPosition {
                            x: Position::Absolute(0),
                            y: Position::Absolute(y),
                        });
                        self.surface
                            .add_change(Change::ClearToEndOfLine(ColorAttribute::Default));
                    }
                }
                EraseInDisplay::EraseToStartOfDisplay => {
                    for y in 0..self.cursor.row {
                        self.surface.add_change(Change::CursorPosition {
                            x: Position::Absolute(0),
                            y: Position::Absolute(y),
                        });
                        self.surface
                            .add_change(Change::ClearToEndOfLine(ColorAttribute::Default));
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
                    self.surface
                        .add_change(Change::ClearScreen(ColorAttribute::Default));
                }
                EraseInDisplay::EraseScrollback => {
                    self.scrollback.clear();
                }
            },
            Edit::DeleteCharacter(n) => {
                self.surface.add_change(Change::CursorPosition {
                    x: Position::Absolute(self.cursor.col),
                    y: Position::Absolute(self.cursor.row),
                });
                for _ in 0..n {
                    self.surface.add_change(Change::Text(" ".to_string()));
                }
            }
            Edit::DeleteLine(n) => {
                for _ in 0..n {
                    self.surface.add_change(Change::ScrollRegionUp {
                        first_row: self.cursor.row,
                        region_size: self.config.rows - self.cursor.row,
                        scroll_count: 1,
                    });
                }
            }
            Edit::InsertLine(n) => {
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

    fn handle_sgr(&mut self, sgr: termwiz::escape::csi::Sgr) {
        use termwiz::escape::csi::Sgr;

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
            Sgr::Font(_) | Sgr::VerticalAlign(_) => {}
        }
    }

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
            DecPrivateModeCode::AutoWrap | DecPrivateModeCode::ShowCursor => {}
            DecPrivateModeCode::MouseTracking
            | DecPrivateModeCode::HighlightMouseTracking
            | DecPrivateModeCode::ButtonEventMouse
            | DecPrivateModeCode::AnyEventMouse => {
                self.mouse_reporting = enable;
            }
            DecPrivateModeCode::SGRMouse => {}
            DecPrivateModeCode::ClearAndEnableAlternateScreen
            | DecPrivateModeCode::EnableAlternateScreen => {
                if enable {
                    if !self.in_alternate_screen {
                        self.saved_primary = Some(std::mem::replace(
                            &mut self.surface,
                            Surface::new(self.config.cols, self.config.rows),
                        ));
                        self.in_alternate_screen = true;
                    }
                } else if self.in_alternate_screen {
                    if let Some(primary) = self.saved_primary.take() {
                        self.surface = primary;
                    }
                    self.in_alternate_screen = false;
                }
            }
            DecPrivateModeCode::BracketedPaste => {}
            _ => {
                tracing::debug!("Unhandled DEC mode: {:?} = {}", code, enable);
            }
        }
    }

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
                if self.cursor.row == 0 {
                    self.surface
                        .add_change(termwiz::surface::Change::ScrollRegionDown {
                            first_row: 0,
                            region_size: self.config.rows,
                            scroll_count: 1,
                        });
                } else {
                    self.cursor.row -= 1;
                }
            }
            Esc::Code(EscCode::Index) => {
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

    fn handle_osc(&mut self, osc: termwiz::escape::osc::OperatingSystemCommand) {
        use termwiz::escape::osc::OperatingSystemCommand;

        match osc {
            OperatingSystemCommand::SetWindowTitle(title)
            | OperatingSystemCommand::SetWindowTitleSun(title)
            | OperatingSystemCommand::SetIconNameAndWindowTitle(title) => {
                tracing::debug!("Window title: {}", title);
            }
            _ => {
                tracing::debug!("Unhandled OSC: {:?}", osc);
            }
        }
    }
}
