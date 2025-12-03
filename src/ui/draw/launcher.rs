//! Launcher mode drawing functions
//!
//! This module handles rendering the main launcher UI:
//! - Search bar
//! - Entry list with grid layout
//! - Status bar
//! - Graphics icons

use parking_lot::Mutex;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui_image::{Resize, StatefulImage};
use std::sync::Arc;
use unicode_width::UnicodeWidthStr;

use crate::app::App;
use crate::icons::IconManager;
use crate::ui::entry_card::{EntryCard, EntryDisplayConfig};
use crate::ui::theme::Theme;

/// Width of icon column in characters when graphics are supported
const ICON_COLUMN_WIDTH: u16 = 6;
/// Gap between columns
const COLUMN_GAP: u16 = 2;

/// Pre-computed grid dimensions for rendering
struct GridDimensions {
    card_height: u16,
    column_width: u16,
}

impl GridDimensions {
    fn compute(
        inner_width: u16,
        grid: &crate::ui::layout::GridLayout,
        entry_config: &EntryDisplayConfig,
    ) -> Self {
        let columns = grid.columns as usize;
        let column_width = if columns > 1 {
            (inner_width.saturating_sub(COLUMN_GAP * (columns as u16 - 1))) / columns as u16
        } else {
            inner_width
        };
        Self {
            card_height: entry_config.card_height(),
            column_width,
        }
    }
}

/// Draw the launcher UI (original behavior)
/// TEAM_004: Updated to use theme
pub(crate) fn draw_launcher(
    f: &mut Frame,
    app: &App,
    icon_manager: Option<&Arc<Mutex<IconManager>>>,
    theme: &Theme,
) {
    // TEAM_004: Fill background with theme color
    let area = f.area();
    let bg_block = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(bg_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(1),    // Entry list
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    draw_search_bar(f, app, chunks[0], theme);
    draw_entry_list(f, app, chunks[1], icon_manager, theme);
    draw_status_bar(f, app, chunks[2], theme);
}

/// Draw the search/filter bar
/// TEAM_004: Updated to use theme colors
fn draw_search_bar(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let config = app.config();

    let filter_text = if app.is_filtering() || !app.filter_text().is_empty() {
        format!("{}{}", config.appearance.prompt, app.filter_text())
    } else {
        format!("{}Type to filter...", config.appearance.prompt)
    };

    let style = if app.is_filtering() {
        Style::default()
            .fg(theme.search_highlight)
            .bg(theme.background)
    } else {
        Style::default().fg(theme.dimmed).bg(theme.background)
    };

    let search = Paragraph::new(filter_text).style(style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent))
            .title(" darkwall-drun ")
            .style(Style::default().bg(theme.background)),
    );

    f.render_widget(search, area);

    // Show cursor in filter mode
    if app.is_filtering() {
        // Use display width (not byte length) for proper Unicode handling
        // +1 for the border on the left side of the block
        let prompt_width = config.appearance.prompt.width() as u16;
        let filter_width = app.filter_text().width() as u16;
        let cursor_x = area.x + 1 + prompt_width + filter_width;
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

/// Draw the list of entries using grid layout
/// TEAM_004: Rewritten to use GridLayout and EntryCard
fn draw_entry_list(
    f: &mut Frame,
    app: &App,
    area: Rect,
    icon_manager: Option<&Arc<Mutex<IconManager>>>,
    theme: &Theme,
) {
    let config = app.config();
    let entries = app.visible_entries();
    let selected = app.selected_index();
    let grid = app.grid_layout();
    let entry_config = config.entry_display_config();

    // Check if we have graphics support
    let has_graphics = icon_manager
        .as_ref()
        .map(|m| m.lock().supports_graphics())
        .unwrap_or(false);

    // Draw border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dimmed_alt))
        .style(Style::default().bg(theme.background));
    f.render_widget(block, area);

    // Calculate inner area (inside border)
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    // Calculate visible range based on selection
    let visible_range = grid.visible_range(selected, entries.len());
    let visible_entries: Vec<_> = entries[visible_range.clone()].to_vec();
    let page_start = visible_range.start;

    // Calculate card dimensions
    let dims = GridDimensions::compute(inner.width, grid, &entry_config);

    // Render each visible entry as a card
    for (local_idx, entry) in visible_entries.iter().enumerate() {
        let global_idx = page_start + local_idx;
        let is_selected = global_idx == selected;

        // Calculate grid position (column-major order)
        let (row, col) = grid.index_to_position(local_idx);

        // Calculate card area
        let card_x = inner.x + col * (dims.column_width + COLUMN_GAP);
        let card_y = inner.y + row * dims.card_height;
        let card_area = Rect {
            x: card_x,
            y: card_y,
            width: dims.column_width,
            height: dims.card_height,
        };

        // Skip if card is outside visible area
        if card_y + dims.card_height > inner.y + inner.height {
            continue;
        }

        // Render entry card
        let card = EntryCard::new(entry, theme)
            .selected(is_selected)
            .config(entry_config)
            .icon_space(has_graphics);
        f.render_widget(card, card_area);
    }

    // Render graphics icons if available
    if has_graphics {
        if let Some(mgr) = icon_manager {
            render_graphics_icons_grid(f, app, inner, mgr, &entry_config, page_start);
        }
    }
}

/// Render graphics icons for visible entries in grid layout
/// TEAM_004: Updated for grid layout
fn render_graphics_icons_grid(
    f: &mut Frame,
    app: &App,
    inner: Rect,
    icon_manager: &Arc<Mutex<IconManager>>,
    entry_config: &EntryDisplayConfig,
    _page_start: usize,
) {
    let entries = app.visible_entries();
    let grid = app.grid_layout();
    let dims = GridDimensions::compute(inner.width, grid, entry_config);

    // Icon dimensions
    let icon_width = ICON_COLUMN_WIDTH;
    let icon_height = dims.card_height.min(2); // Max 2 rows per icon

    // Get visible range
    let visible_range = grid.visible_range(app.selected_index(), entries.len());
    let visible_entries: Vec<_> = entries[visible_range.clone()].to_vec();

    // Collect icons to render (only from cache, non-blocking)
    let mut icons_to_render = Vec::new();
    {
        let mgr = icon_manager.lock();

        for (local_idx, entry) in visible_entries.iter().enumerate() {
            let (row, col) = grid.index_to_position(local_idx);

            // Calculate position
            let card_x = inner.x + col * (dims.column_width + COLUMN_GAP);
            let card_y = inner.y + row * dims.card_height;

            // Skip if outside visible area
            if card_y + dims.card_height > inner.y + inner.height {
                continue;
            }

            // Only get cached icons - don't block rendering
            if let Some(protocol) = mgr.get_cached(&entry.id) {
                icons_to_render.push((card_x, card_y, protocol));
            }
        }
    } // Release lock before rendering

    // Render collected icons
    for (card_x, card_y, protocol) in icons_to_render {
        let icon_area = Rect {
            x: card_x + 1, // After padding
            y: card_y,
            width: icon_width,
            height: icon_height,
        };

        let image = StatefulImage::new(None).resize(Resize::Fit(None));
        let mut proto = protocol.lock();
        f.render_stateful_widget(image, icon_area, &mut *proto);
    }
}

/// Draw the status bar
/// TEAM_004: Updated to use theme and show grid navigation hints
fn draw_status_bar(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let entries = app.visible_entries();
    let total = entries.len();
    let grid = app.grid_layout();

    let status = if app.is_filtering() || !app.filter_text().is_empty() {
        format!(
            " {} matches | ESC: clear | Enter: run | Ctrl+C: quit",
            total
        )
    } else {
        // Show current position and grid info
        let page = app.selected_index() / grid.visible_count() + 1;
        let total_pages = (total + grid.visible_count() - 1) / grid.visible_count();
        format!(
            " {}/{} | Page {}/{} | ↑↓←→: nav | Tab: next | Enter: run | ESC: quit",
            app.selected_index() + 1,
            total,
            page,
            total_pages.max(1)
        )
    };

    let status_bar =
        Paragraph::new(status).style(Style::default().fg(theme.dimmed).bg(theme.background));

    f.render_widget(status_bar, area);
}
