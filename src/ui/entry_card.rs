//! Entry card widget for DRUN
//!
//! Unit 5.4.2: Entry Cards
//!
//! Renders each entry as a multi-line card with:
//! - Icon + Name (bold)
//! - GenericName
//! - Comment (dimmed)
//! - Categories (more dimmed)

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthStr;

use crate::desktop_entry::Entry;
use super::theme::Theme;

/// Configuration for entry display
#[derive(Debug, Clone, Copy)]
pub struct EntryDisplayConfig {
    /// Show GenericName line
    pub show_generic: bool,
    /// Show Comment line
    pub show_comment: bool,
    /// Show Categories line
    pub show_categories: bool,
}

impl Default for EntryDisplayConfig {
    fn default() -> Self {
        Self {
            show_generic: true,
            show_comment: true,
            show_categories: true,
        }
    }
}

impl EntryDisplayConfig {
    /// Calculate the height of an entry card in lines
    pub fn card_height(&self) -> u16 {
        let mut height = 1; // Name line always shown
        if self.show_generic { height += 1; }
        if self.show_comment { height += 1; }
        if self.show_categories { height += 1; }
        height
    }
}

/// Entry card widget
pub struct EntryCard<'a> {
    entry: &'a Entry,
    selected: bool,
    theme: &'a Theme,
    config: EntryDisplayConfig,
    /// Whether to show icon space (for alignment when graphics are supported)
    icon_space: bool,
}

impl<'a> EntryCard<'a> {
    pub fn new(entry: &'a Entry, theme: &'a Theme) -> Self {
        Self {
            entry,
            selected: false,
            theme,
            config: EntryDisplayConfig::default(),
            icon_space: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn config(mut self, config: EntryDisplayConfig) -> Self {
        self.config = config;
        self
    }

    pub fn icon_space(mut self, icon_space: bool) -> Self {
        self.icon_space = icon_space;
        self
    }
}

impl<'a> Widget for EntryCard<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let bg = if self.selected { self.theme.selection_bg } else { self.theme.background };
        let fg = if self.selected { self.theme.selection_fg } else { self.theme.foreground };

        // Fill background
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_bg(bg);
                }
            }
        }

        // Calculate inner area with padding
        let padding_x = 1u16;
        let inner_width = area.width.saturating_sub(padding_x * 2);
        if inner_width == 0 {
            return;
        }

        let inner_x = area.x + padding_x;
        let max_y = area.y + area.height;
        let mut y = area.y;

        // Icon space offset (for alignment when graphics icons are shown elsewhere)
        let icon_offset = if self.icon_space { 6 } else { 0 };
        let text_x = inner_x + icon_offset;
        let text_width = inner_width.saturating_sub(icon_offset) as usize;

        // Line 1: Name (bold) - always rendered
        let name_style = Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD);
        buf.set_string(text_x, y, truncate(&self.entry.name, text_width), name_style);
        y += 1;

        // Indent for subsequent lines
        let indent = 3u16;
        let sub_x = text_x + indent;
        let sub_width = text_width.saturating_sub(indent as usize);

        // Line 2: GenericName (if enabled and room available)
        if self.config.show_generic && y < max_y {
            if let Some(ref generic) = self.entry.generic_name {
                if generic != &self.entry.name {
                    let style = Style::default().fg(fg).bg(bg);
                    buf.set_string(sub_x, y, truncate(generic, sub_width), style);
                }
            }
            y += 1;
        }

        // Line 3: Comment (if enabled and room available)
        if self.config.show_comment && y < max_y {
            if let Some(ref comment) = self.entry.comment {
                let style = Style::default().fg(self.theme.dimmed).bg(bg);
                buf.set_string(sub_x, y, truncate(comment, sub_width), style);
            }
            y += 1;
        }

        // Line 4: Categories (if enabled, non-empty, and room available)
        if self.config.show_categories && y < max_y && !self.entry.categories.is_empty() {
            let cats = self.entry.categories.join(",");
            let style = Style::default().fg(self.theme.dimmed_alt).bg(bg);
            buf.set_string(sub_x, y, truncate(&cats, sub_width), style);
        }
    }
}

/// Truncate string to fit within max_width, adding ellipsis if needed
fn truncate(s: &str, max_width: usize) -> String {
    let width = s.width();
    if width <= max_width {
        s.to_string()
    } else if max_width <= 1 {
        "…".to_string()
    } else {
        let mut result = String::new();
        let mut current_width = 0;
        
        for c in s.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
            if current_width + char_width + 1 > max_width {
                result.push('…');
                break;
            }
            result.push(c);
            current_width += char_width;
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello w…");
        assert_eq!(truncate("hi", 2), "hi");
        assert_eq!(truncate("hello", 1), "…");
    }

    #[test]
    fn test_card_height() {
        let config = EntryDisplayConfig::default();
        assert_eq!(config.card_height(), 4);
        
        let config = EntryDisplayConfig {
            show_generic: false,
            show_comment: true,
            show_categories: false,
        };
        assert_eq!(config.card_height(), 2);
    }
}
