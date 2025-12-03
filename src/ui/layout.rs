//! Grid layout system for DRUN
//!
//! Unit 5.4.1: Grid Layout
//!
//! Provides:
//! - 2-column (configurable) grid layout
//! - Column-major ordering (like rofi)
//! - Navigation helpers (up/down/left/right)
//! - Pagination

use std::ops::Range;

/// Grid layout configuration
#[derive(Debug, Clone, Copy)]
pub struct GridLayout {
    /// Number of columns (default: 2)
    pub columns: u16,
    /// Number of visible rows (default: 5)
    pub visible_rows: u16,
}

impl Default for GridLayout {
    fn default() -> Self {
        Self {
            columns: 2,
            visible_rows: 5,
        }
    }
}

impl GridLayout {
    /// Create a new grid layout
    pub fn new(columns: u16, visible_rows: u16) -> Self {
        Self {
            columns: columns.max(1).min(10),
            visible_rows: visible_rows.max(1).min(20),
        }
    }

    /// Total number of visible entries (columns × rows)
    pub fn visible_count(&self) -> usize {
        (self.columns as usize) * (self.visible_rows as usize)
    }

    /// Calculate the range of entries visible for a given selection
    /// Returns the start..end indices of entries to display
    pub fn visible_range(&self, selected: usize, total: usize) -> Range<usize> {
        if total == 0 {
            return 0..0;
        }
        
        let page_size = self.visible_count();
        let page = selected / page_size;
        let start = page * page_size;
        let end = (start + page_size).min(total);
        start..end
    }

    /// Convert flat index to (row, col) position
    /// Uses column-major ordering (rofi-style):
    /// ```text
    /// Index:  0 5
    ///         1 6
    ///         2 7
    ///         3 8
    ///         4 9
    /// ```
    pub fn index_to_position(&self, index: usize) -> (u16, u16) {
        let rows = self.visible_rows as usize;
        let col = index / rows;
        let row = index % rows;
        (row as u16, col as u16)
    }

    /// Convert (row, col) position to flat index
    pub fn position_to_index(&self, row: u16, col: u16) -> usize {
        let rows = self.visible_rows as usize;
        (col as usize) * rows + (row as usize)
    }

    /// Calculate new selection after moving up
    pub fn move_up(&self, current: usize) -> usize {
        current.saturating_sub(1)
    }

    /// Calculate new selection after moving down
    pub fn move_down(&self, current: usize, total: usize) -> usize {
        if current + 1 < total {
            current + 1
        } else {
            current
        }
    }

    /// Calculate new selection after moving left (previous column)
    pub fn move_left(&self, current: usize) -> usize {
        let rows = self.visible_rows as usize;
        current.saturating_sub(rows)
    }

    /// Calculate new selection after moving right (next column)
    pub fn move_right(&self, current: usize, total: usize) -> usize {
        let rows = self.visible_rows as usize;
        (current + rows).min(total.saturating_sub(1))
    }

    /// Calculate new selection after page up
    pub fn page_up(&self, current: usize) -> usize {
        let page_size = self.visible_count();
        current.saturating_sub(page_size)
    }

    /// Calculate new selection after page down
    pub fn page_down(&self, current: usize, total: usize) -> usize {
        let page_size = self.visible_count();
        (current + page_size).min(total.saturating_sub(1))
    }

    /// Move to first entry
    pub fn move_home(&self) -> usize {
        0
    }

    /// Move to last entry
    pub fn move_end(&self, total: usize) -> usize {
        total.saturating_sub(1)
    }

    /// Tab navigation (next with wrap)
    pub fn tab_next(&self, current: usize, total: usize) -> usize {
        if total == 0 {
            0
        } else {
            (current + 1) % total
        }
    }

    /// Shift+Tab navigation (previous with wrap)
    pub fn tab_prev(&self, current: usize, total: usize) -> usize {
        if total == 0 {
            0
        } else if current == 0 {
            total - 1
        } else {
            current - 1
        }
    }

    /// Calculate entry height in lines based on display config
    pub fn entry_height(&self, show_generic: bool, show_comment: bool, show_categories: bool) -> u16 {
        let mut height = 1; // Name line always shown
        if show_generic { height += 1; }
        if show_comment { height += 1; }
        if show_categories { height += 1; }
        height
    }
}

/// Navigation direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_count() {
        let layout = GridLayout::new(2, 5);
        assert_eq!(layout.visible_count(), 10);
    }

    #[test]
    fn test_visible_range() {
        let layout = GridLayout::new(2, 5);
        
        // First page
        assert_eq!(layout.visible_range(0, 25), 0..10);
        assert_eq!(layout.visible_range(5, 25), 0..10);
        assert_eq!(layout.visible_range(9, 25), 0..10);
        
        // Second page
        assert_eq!(layout.visible_range(10, 25), 10..20);
        assert_eq!(layout.visible_range(15, 25), 10..20);
        
        // Third page (partial)
        assert_eq!(layout.visible_range(20, 25), 20..25);
    }

    #[test]
    fn test_index_to_position() {
        let layout = GridLayout::new(2, 5);
        
        // Column 0
        assert_eq!(layout.index_to_position(0), (0, 0));
        assert_eq!(layout.index_to_position(1), (1, 0));
        assert_eq!(layout.index_to_position(4), (4, 0));
        
        // Column 1
        assert_eq!(layout.index_to_position(5), (0, 1));
        assert_eq!(layout.index_to_position(6), (1, 1));
        assert_eq!(layout.index_to_position(9), (4, 1));
    }

    #[test]
    fn test_position_to_index() {
        let layout = GridLayout::new(2, 5);
        
        assert_eq!(layout.position_to_index(0, 0), 0);
        assert_eq!(layout.position_to_index(4, 0), 4);
        assert_eq!(layout.position_to_index(0, 1), 5);
        assert_eq!(layout.position_to_index(4, 1), 9);
    }

    #[test]
    fn test_navigation() {
        let layout = GridLayout::new(2, 5);
        let total = 15;
        
        // Up/Down
        assert_eq!(layout.move_up(5), 4);
        assert_eq!(layout.move_up(0), 0); // Can't go below 0
        assert_eq!(layout.move_down(5, total), 6);
        assert_eq!(layout.move_down(14, total), 14); // Can't exceed total
        
        // Left/Right (column jump)
        assert_eq!(layout.move_left(7), 2); // 7 - 5 = 2
        assert_eq!(layout.move_left(2), 0); // saturating_sub
        assert_eq!(layout.move_right(2, total), 7); // 2 + 5 = 7
        assert_eq!(layout.move_right(12, total), 14); // clamped to total-1
    }

    #[test]
    fn test_tab_wrap() {
        let layout = GridLayout::new(2, 5);
        let total = 15;
        
        assert_eq!(layout.tab_next(14, total), 0); // Wrap to start
        assert_eq!(layout.tab_prev(0, total), 14); // Wrap to end
    }
}
