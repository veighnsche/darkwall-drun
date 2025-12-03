use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use ratatui_image::{StatefulImage, Resize};
use std::sync::Arc;
use parking_lot::Mutex;

use crate::app::{App, AppMode};
use crate::executor::CommandStatus;
use crate::icons::IconManager;
use super::theme::Theme;
use super::entry_card::{EntryCard, EntryDisplayConfig};

/// Main draw function
/// TEAM_000: Phase 2 - Updated for execution modes
/// TEAM_002: Added icon manager parameter
/// TEAM_004: Added theme parameter for theming support
pub fn draw(f: &mut Frame, app: &App, icon_manager: Option<&Arc<Mutex<IconManager>>>) {
    // TEAM_004: Resolve theme from config
    let theme = app.config().resolve_theme();
    match app.mode() {
        AppMode::Launcher => draw_launcher(f, app, icon_manager, &theme),
        AppMode::Executing { command, .. } => draw_executing(f, app, command, &theme),
        AppMode::PostExecution { command, exit_status, preserved_output } => {
            draw_post_execution(f, app, command, exit_status, preserved_output, icon_manager, &theme)
        }
        AppMode::TuiHandover { .. } => {
            // TUI handover - we shouldn't be drawing, but show a message just in case
            let msg = Paragraph::new("Running TUI application...")
                .style(Style::default().fg(theme.accent));
            f.render_widget(msg, f.area());
        }
    }
}

/// Draw the launcher UI (original behavior)
/// TEAM_004: Updated to use theme
fn draw_launcher(f: &mut Frame, app: &App, icon_manager: Option<&Arc<Mutex<IconManager>>>, theme: &Theme) {
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
        Style::default().fg(theme.search_highlight).bg(theme.background)
    } else {
        Style::default().fg(theme.dimmed).bg(theme.background)
    };

    let search = Paragraph::new(filter_text)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent))
                .title(" darkwall-drun ")
                .style(Style::default().bg(theme.background)),
        );

    f.render_widget(search, area);

    // Show cursor in filter mode
    if app.is_filtering() {
        let cursor_x = area.x + 1 + app.config().appearance.prompt.len() as u16 + app.filter_text().len() as u16;
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

/// Width of icon column in characters when graphics are supported
const ICON_COLUMN_WIDTH: u16 = 6;
/// Gap between columns
const COLUMN_GAP: u16 = 2;

/// Draw the list of entries using grid layout
/// TEAM_004: Rewritten to use GridLayout and EntryCard
fn draw_entry_list(f: &mut Frame, app: &App, area: Rect, icon_manager: Option<&Arc<Mutex<IconManager>>>, theme: &Theme) {
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
    let card_height = entry_config.card_height();
    let columns = grid.columns as usize;
    let column_width = if columns > 1 {
        (inner.width.saturating_sub(COLUMN_GAP * (columns as u16 - 1))) / columns as u16
    } else {
        inner.width
    };

    // Render each visible entry as a card
    for (local_idx, entry) in visible_entries.iter().enumerate() {
        let global_idx = page_start + local_idx;
        let is_selected = global_idx == selected;

        // Calculate grid position (column-major order)
        let (row, col) = grid.index_to_position(local_idx);

        // Calculate card area
        let card_x = inner.x + col * (column_width + COLUMN_GAP);
        let card_y = inner.y + row * card_height;
        let card_area = Rect {
            x: card_x,
            y: card_y,
            width: column_width,
            height: card_height,
        };

        // Skip if card is outside visible area
        if card_y + card_height > inner.y + inner.height {
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
    page_start: usize,
) {
    let entries = app.visible_entries();
    let grid = app.grid_layout();
    
    let card_height = entry_config.card_height();
    let columns = grid.columns as usize;
    let column_width = if columns > 1 {
        (inner.width.saturating_sub(COLUMN_GAP * (columns as u16 - 1))) / columns as u16
    } else {
        inner.width
    };
    
    // Icon dimensions
    let icon_width = ICON_COLUMN_WIDTH;
    let icon_height = card_height.min(2); // Max 2 rows per icon
    
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
            let card_x = inner.x + col * (column_width + COLUMN_GAP);
            let card_y = inner.y + row * card_height;
            
            // Skip if outside visible area
            if card_y + card_height > inner.y + inner.height {
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
            " {}/{} | Page {}/{} | ↑↓←→/hjkl: nav | Tab: next | Enter: run | ESC: quit",
            app.selected_index() + 1,
            total,
            page,
            total_pages.max(1)
        )
    };

    let status_bar = Paragraph::new(status)
        .style(Style::default().fg(theme.dimmed).bg(theme.background));

    f.render_widget(status_bar, area);
}

/// Draw the executing UI - shows command output
/// TEAM_000: Phase 2, Unit 2.2 - Output display
/// TEAM_004: Updated to use theme
fn draw_executing(f: &mut Frame, app: &App, command: &str, theme: &Theme) {
    // Fill background
    let bg_block = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(bg_block, f.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Command header
            Constraint::Min(1),    // Output
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    // Command header
    let header = Paragraph::new(format!("$ {}", command))
        .style(Style::default().fg(theme.exit_success).bg(theme.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.exit_success))
                .title(" Running ")
                .style(Style::default().bg(theme.background)),
        );
    f.render_widget(header, chunks[0]);

    // Output area
    let output_height = chunks[1].height.saturating_sub(2) as usize; // -2 for borders
    let buffer = app.output_buffer();
    let lines: Vec<Line> = buffer
        .visible_lines(output_height)
        .map(|s| Line::from(s.to_string()))
        .collect();

    let output = Paragraph::new(lines)
        .style(Style::default().fg(theme.foreground).bg(theme.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.dimmed_alt))
                .title(" Output ")
                .style(Style::default().bg(theme.background)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(output, chunks[1]);

    // Status bar
    let status = format!(
        " {} lines | Ctrl+C: kill | j/k: scroll | g/G: top/bottom",
        buffer.len()
    );
    let status_bar = Paragraph::new(status)
        .style(Style::default().fg(theme.accent).bg(theme.background));
    f.render_widget(status_bar, chunks[2]);
}

/// Draw the post-execution UI - shows preserved output above launcher
/// TEAM_000: Phase 2, Unit 2.3 - Return to launcher
/// TEAM_004: Updated to use theme
fn draw_post_execution(
    f: &mut Frame,
    app: &App,
    command: &str,
    exit_status: &CommandStatus,
    preserved_output: &[String],
    icon_manager: Option<&Arc<Mutex<IconManager>>>,
    theme: &Theme,
) {
    // Fill background
    let bg_block = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(bg_block, f.area());

    // Calculate layout based on preserved output
    let output_lines = preserved_output.len() as u16;
    let output_height = output_lines.min(10) + 4; // +4 for borders and header

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(output_height), // Preserved output
            Constraint::Length(3),             // Search bar
            Constraint::Min(1),                // Entry list
            Constraint::Length(1),             // Status bar
        ])
        .split(f.area());

    // Preserved output section
    draw_preserved_output(f, chunks[0], command, exit_status, preserved_output, theme);

    // Regular launcher below
    draw_search_bar(f, app, chunks[1], theme);
    draw_entry_list(f, app, chunks[2], icon_manager, theme);
    draw_post_execution_status_bar(f, chunks[3], theme);
}

/// Draw the preserved output section
/// TEAM_004: Updated to use theme
fn draw_preserved_output(
    f: &mut Frame,
    area: Rect,
    command: &str,
    exit_status: &CommandStatus,
    preserved_output: &[String],
    theme: &Theme,
) {
    let (exit_text, exit_color) = match exit_status {
        CommandStatus::Exited(0) => ("Exit: 0".to_string(), theme.exit_success),
        CommandStatus::Exited(code) => (format!("Exit: {}", code), theme.exit_failure),
        CommandStatus::Signaled(sig) => (format!("Signal: {}", sig), theme.exit_failure),
        CommandStatus::Running => ("Running".to_string(), theme.accent),
        CommandStatus::Unknown => ("Unknown".to_string(), theme.dimmed),
    };

    let mut lines: Vec<Line> = Vec::new();
    
    // Command line
    lines.push(Line::from(vec![
        Span::styled("$ ", Style::default().fg(theme.dimmed)),
        Span::styled(command, Style::default().fg(theme.foreground)),
    ]));

    // Output lines
    for line in preserved_output {
        lines.push(Line::from(Span::styled(line.as_str(), Style::default().fg(theme.foreground))));
    }

    // Exit status
    lines.push(Line::from(vec![
        Span::styled(format!("[{}]", exit_text), Style::default().fg(exit_color)),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(exit_color))
        .title(" Last Command ")
        .style(Style::default().bg(theme.background));

    let paragraph = Paragraph::new(lines)
        .style(Style::default().bg(theme.background))
        .block(block);
    f.render_widget(paragraph, area);
}

/// Status bar for post-execution mode
/// TEAM_004: Updated to use theme
fn draw_post_execution_status_bar(f: &mut Frame, area: Rect, theme: &Theme) {
    let status = " Enter: dismiss | q: quit";
    let status_bar = Paragraph::new(status)
        .style(Style::default().fg(theme.dimmed).bg(theme.background));
    f.render_widget(status_bar, area);
}
