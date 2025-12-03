use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use ratatui_image::{StatefulImage, Resize};
use std::sync::Arc;
use parking_lot::Mutex;

use crate::app::{App, AppMode};
use crate::executor::CommandStatus;
use crate::icons::IconManager;

/// Main draw function
/// TEAM_000: Phase 2 - Updated for execution modes
/// TEAM_002: Added icon manager parameter
pub fn draw(f: &mut Frame, app: &App, icon_manager: Option<&Arc<Mutex<IconManager>>>) {
    match app.mode() {
        AppMode::Launcher => draw_launcher(f, app, icon_manager),
        AppMode::Executing { command, .. } => draw_executing(f, app, command),
        AppMode::PostExecution { command, exit_status, preserved_output } => {
            draw_post_execution(f, app, command, exit_status, preserved_output, icon_manager)
        }
        AppMode::TuiHandover { .. } => {
            // TUI handover - we shouldn't be drawing, but show a message just in case
            let msg = Paragraph::new("Running TUI application...")
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(msg, f.area());
        }
    }
}

/// Draw the launcher UI (original behavior)
fn draw_launcher(f: &mut Frame, app: &App, icon_manager: Option<&Arc<Mutex<IconManager>>>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(1),    // Entry list
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    draw_search_bar(f, app, chunks[0]);
    draw_entry_list(f, app, chunks[1], icon_manager);
    draw_status_bar(f, app, chunks[2]);
}

/// Draw the search/filter bar
fn draw_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let config = app.config();

    let filter_text = if app.is_filtering() || !app.filter_text().is_empty() {
        format!("{}{}", config.appearance.prompt, app.filter_text())
    } else {
        format!("{}Type to filter...", config.appearance.prompt)
    };

    let style = if app.is_filtering() {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let search = Paragraph::new(filter_text)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(" darkwall-drun "),
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
const ICON_COLUMN_WIDTH: usize = 6;

/// Draw the list of entries
/// TEAM_002: Graphics icons only - no fallbacks
fn draw_entry_list(f: &mut Frame, app: &App, area: Rect, icon_manager: Option<&Arc<Mutex<IconManager>>>) {
    let config = app.config();
    let entries = app.visible_entries();
    let selected = app.selected_index();

    // Check if we have graphics support
    let has_graphics = icon_manager
        .as_ref()
        .map(|m| m.lock().supports_graphics())
        .unwrap_or(false);

    // Reserve space for icon column if graphics supported
    let icon_padding = if has_graphics { " ".repeat(ICON_COLUMN_WIDTH) } else { String::new() };
    let line_indent = if has_graphics { " ".repeat(ICON_COLUMN_WIDTH + 2) } else { "  ".to_string() };

    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == selected;

            let prefix = if is_selected {
                &config.appearance.selected_prefix
            } else {
                &config.appearance.unselected_prefix
            };

            // Build the display lines
            let mut lines = vec![];

            // Line 1: Name (with icon padding if graphics supported)
            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::styled(prefix.clone(), name_style),
                Span::raw(icon_padding.clone()),
                Span::styled(&entry.name, name_style),
            ]));

            // Line 2: Generic name (if different from name)
            if config.behavior.show_generic_name {
                if let Some(ref gn) = entry.generic_name {
                    if gn != &entry.name {
                        lines.push(Line::from(vec![
                            Span::raw(line_indent.clone()),
                            Span::styled(gn, Style::default().fg(Color::Gray)),
                        ]));
                    }
                }
            }

            // Line 3: Comment
            if let Some(ref comment) = entry.comment {
                lines.push(Line::from(vec![
                    Span::raw(line_indent.clone()),
                    Span::styled(comment, Style::default().fg(Color::DarkGray)),
                ]));
            }

            // Line 4: Categories
            if config.behavior.show_categories && !entry.categories.is_empty() {
                let cats = entry.categories.join(",");
                lines.push(Line::from(vec![
                    Span::raw(line_indent.clone()),
                    Span::styled(cats, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                ]));
            }

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(list, area);

    // Render graphics icons if available
    if has_graphics {
        if let Some(mgr) = icon_manager {
            render_graphics_icons(f, app, area, mgr);
        }
    }
}

/// Render graphics icons for visible entries
/// TEAM_002: Kitty/Sixel/iTerm2 graphics protocol support
fn render_graphics_icons(
    f: &mut Frame,
    app: &App,
    area: Rect,
    icon_manager: &Arc<Mutex<IconManager>>,
) {
    let entries = app.visible_entries();
    let config = app.config();
    
    // Calculate entry height (depends on config)
    let lines_per_entry = calculate_lines_per_entry(config);
    
    // Icon area dimensions
    let icon_width = ICON_COLUMN_WIDTH as u16;
    let icon_height = lines_per_entry.min(2) as u16; // Max 2 rows per icon
    
    // Start after border
    let content_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
    
    // Collect icons to render (only from cache, non-blocking)
    let mut icons_to_render = Vec::new();
    {
        let mgr = icon_manager.lock();
        let mut y_offset = 0u16;
        
        for entry in entries.iter() {
            if y_offset + icon_height > content_area.height {
                break; // No more room
            }
            
            // Only get cached icons - don't block rendering
            if let Some(protocol) = mgr.get_cached(&entry.id) {
                icons_to_render.push((y_offset, protocol));
            }
            
            y_offset += lines_per_entry as u16;
        }
    } // Release lock before rendering
    
    // Render collected icons
    for (y_offset, protocol) in icons_to_render {
        let icon_area = Rect {
            x: content_area.x + 2, // After prefix "● "
            y: content_area.y + y_offset,
            width: icon_width,
            height: icon_height,
        };
        
        let image = StatefulImage::new(None).resize(Resize::Fit(None));
        let mut proto = protocol.lock();
        f.render_stateful_widget(image, icon_area, &mut *proto);
    }
}

/// Calculate lines per entry based on config
fn calculate_lines_per_entry(config: &crate::config::Config) -> usize {
    1 + if config.behavior.show_generic_name { 1 } else { 0 }
      + 1  // comment
      + if config.behavior.show_categories { 1 } else { 0 }
}

/// Draw the status bar
fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let entries = app.visible_entries();
    let total = entries.len();

    let status = if app.is_filtering() || !app.filter_text().is_empty() {
        format!(
            " {} matches | ESC: clear | Enter: run | Ctrl+C: quit",
            total
        )
    } else {
        format!(
            " {}/{} | Type to filter | ↑↓: navigate | Enter: run | ESC: quit",
            app.selected_index() + 1,
            total
        )
    };

    let status_bar = Paragraph::new(status).style(Style::default().fg(Color::DarkGray));

    f.render_widget(status_bar, area);
}

/// Draw the executing UI - shows command output
/// TEAM_000: Phase 2, Unit 2.2 - Output display
fn draw_executing(f: &mut Frame, app: &App, command: &str) {
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
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(" Running "),
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Output "),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(output, chunks[1]);

    // Status bar
    let status = format!(
        " {} lines | Ctrl+C: kill | j/k: scroll | g/G: top/bottom",
        buffer.len()
    );
    let status_bar = Paragraph::new(status).style(Style::default().fg(Color::Yellow));
    f.render_widget(status_bar, chunks[2]);
}

/// Draw the post-execution UI - shows preserved output above launcher
/// TEAM_000: Phase 2, Unit 2.3 - Return to launcher
fn draw_post_execution(
    f: &mut Frame,
    app: &App,
    command: &str,
    exit_status: &CommandStatus,
    preserved_output: &[String],
    icon_manager: Option<&Arc<Mutex<IconManager>>>,
) {
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
    draw_preserved_output(f, chunks[0], command, exit_status, preserved_output);

    // Regular launcher below
    draw_search_bar(f, app, chunks[1]);
    draw_entry_list(f, app, chunks[2], icon_manager);
    draw_post_execution_status_bar(f, chunks[3]);
}

/// Draw the preserved output section
fn draw_preserved_output(
    f: &mut Frame,
    area: Rect,
    command: &str,
    exit_status: &CommandStatus,
    preserved_output: &[String],
) {
    let (exit_text, exit_color) = match exit_status {
        CommandStatus::Exited(0) => ("Exit: 0".to_string(), Color::Green),
        CommandStatus::Exited(code) => (format!("Exit: {}", code), Color::Red),
        CommandStatus::Signaled(sig) => (format!("Signal: {}", sig), Color::Red),
        CommandStatus::Running => ("Running".to_string(), Color::Yellow),
        CommandStatus::Unknown => ("Unknown".to_string(), Color::Gray),
    };

    let mut lines: Vec<Line> = Vec::new();
    
    // Command line
    lines.push(Line::from(vec![
        Span::styled("$ ", Style::default().fg(Color::DarkGray)),
        Span::styled(command, Style::default().fg(Color::White)),
    ]));

    // Output lines
    for line in preserved_output {
        lines.push(Line::from(line.as_str()));
    }

    // Exit status
    lines.push(Line::from(vec![
        Span::styled(format!("[{}]", exit_text), Style::default().fg(exit_color)),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(exit_color))
        .title(" Last Command ");

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

/// Status bar for post-execution mode
fn draw_post_execution_status_bar(f: &mut Frame, area: Rect) {
    let status = " Enter: dismiss | q: quit";
    let status_bar = Paragraph::new(status).style(Style::default().fg(Color::DarkGray));
    f.render_widget(status_bar, area);
}
