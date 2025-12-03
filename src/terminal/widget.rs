//! Terminal widget for rendering to ratatui
//!
//! Provides:
//! - `TerminalWidget` - ratatui widget for rendering terminal content
//! - Color and attribute conversion from termwiz to ratatui

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::Widget;
use termwiz::cell::CellAttributes;
use termwiz::color::ColorAttribute;

use super::EmbeddedTerminal;

/// Convert termwiz color to ratatui color
pub fn termwiz_to_ratatui_color(color: &ColorAttribute) -> ratatui::style::Color {
    use ratatui::style::Color;

    match color {
        ColorAttribute::Default => Color::Reset,
        ColorAttribute::PaletteIndex(idx) => {
            // Map 0-15 to named colors for better compatibility
            match *idx {
                0 => Color::Black,
                1 => Color::Red,
                2 => Color::Green,
                3 => Color::Yellow,
                4 => Color::Blue,
                5 => Color::Magenta,
                6 => Color::Cyan,
                7 => Color::White,
                8 => Color::DarkGray,
                9 => Color::LightRed,
                10 => Color::LightGreen,
                11 => Color::LightYellow,
                12 => Color::LightBlue,
                13 => Color::LightMagenta,
                14 => Color::LightCyan,
                15 => Color::Gray,
                _ => Color::Indexed(*idx),
            }
        }
        ColorAttribute::TrueColorWithDefaultFallback(c)
        | ColorAttribute::TrueColorWithPaletteFallback(c, _) => {
            let (r, g, b, _) = c.to_srgb_u8();
            Color::Rgb(r, g, b)
        }
    }
}

/// Convert termwiz cell attributes to ratatui style
pub fn convert_attrs(attrs: &CellAttributes) -> Style {
    use termwiz::cell::{Blink, Intensity, Underline};

    let mut style = Style::default();

    // Colors
    style = style.fg(termwiz_to_ratatui_color(&attrs.foreground()));
    style = style.bg(termwiz_to_ratatui_color(&attrs.background()));

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

    if attrs.underline() != Underline::None {
        modifiers |= Modifier::UNDERLINED;
    }

    if attrs.blink() != Blink::None {
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

/// Widget for rendering an embedded terminal to ratatui
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

        // Render cursor if visible and at bottom (following)
        if self.show_cursor && self.terminal.is_at_bottom() {
            let cursor = self.terminal.cursor();
            if cursor.col < area.width as usize && cursor.row < area.height as usize {
                let buf_x = area.x + cursor.col as u16;
                let buf_y = area.y + cursor.row as u16;

                // Invert the cell at cursor position
                if let Some(buf_cell) = buf.cell_mut((buf_x, buf_y)) {
                    buf_cell.set_style(buf_cell.style().add_modifier(Modifier::REVERSED));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::TerminalConfig;

    #[test]
    fn test_widget_rendering() {
        let mut term = EmbeddedTerminal::new(TerminalConfig {
            cols: 10,
            rows: 5,
            scrollback: 100,
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
        assert_eq!(buf.cell((2, 0)).unwrap().symbol(), "l");
        assert_eq!(buf.cell((3, 0)).unwrap().symbol(), "l");
        assert_eq!(buf.cell((4, 0)).unwrap().symbol(), "o");
    }

    #[test]
    fn test_color_conversion() {
        use ratatui::style::Color;

        // Test basic palette colors
        assert_eq!(
            termwiz_to_ratatui_color(&ColorAttribute::PaletteIndex(1)),
            Color::Red
        );
        assert_eq!(
            termwiz_to_ratatui_color(&ColorAttribute::PaletteIndex(2)),
            Color::Green
        );
        assert_eq!(
            termwiz_to_ratatui_color(&ColorAttribute::PaletteIndex(4)),
            Color::Blue
        );

        // Test default
        assert_eq!(
            termwiz_to_ratatui_color(&ColorAttribute::Default),
            Color::Reset
        );

        // Test 256-color palette (above 15)
        assert_eq!(
            termwiz_to_ratatui_color(&ColorAttribute::PaletteIndex(100)),
            Color::Indexed(100)
        );
    }

    #[test]
    fn test_attr_conversion() {
        use termwiz::cell::Intensity;

        // Test bold
        let mut attrs = CellAttributes::default();
        attrs.set_intensity(Intensity::Bold);
        let style = convert_attrs(&attrs);
        assert!(style.add_modifier.contains(Modifier::BOLD));

        // Test italic
        let mut attrs = CellAttributes::default();
        attrs.set_italic(true);
        let style = convert_attrs(&attrs);
        assert!(style.add_modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn test_widget_cursor_rendering() {
        let mut term = EmbeddedTerminal::new(TerminalConfig {
            cols: 10,
            rows: 5,
            scrollback: 100,
            ..Default::default()
        });

        term.write(b"Hi");

        // Cursor should be at position (2, 0)
        assert_eq!(term.cursor().col, 2);
        assert_eq!(term.cursor().row, 0);

        // Create a test buffer
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 5));

        // Render widget with cursor
        let widget = TerminalWidget::new(&term).show_cursor(true);
        widget.render(Rect::new(0, 0, 10, 5), &mut buf);

        // Cursor position should have REVERSED modifier
        let cursor_cell = buf.cell((2, 0)).unwrap();
        assert!(cursor_cell.modifier.contains(Modifier::REVERSED));
    }
}
