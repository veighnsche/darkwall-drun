use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

/// Main draw function
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(1),    // Entry list
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    draw_search_bar(f, app, chunks[0]);
    draw_entry_list(f, app, chunks[1]);
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
                .title(" darkwall-tui "),
        );

    f.render_widget(search, area);

    // Show cursor in filter mode
    if app.is_filtering() {
        let cursor_x = area.x + 1 + app.config().appearance.prompt.len() as u16 + app.filter_text().len() as u16;
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

/// Draw the list of entries
fn draw_entry_list(f: &mut Frame, app: &App, area: Rect) {
    let config = app.config();
    let entries = app.visible_entries();
    let selected = app.selected_index();

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

            // Line 1: Name
            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(vec![
                Span::styled(prefix.clone(), name_style),
                Span::styled(&entry.name, name_style),
            ]));

            // Line 2: Generic name (if different from name)
            if config.behavior.show_generic_name {
                if let Some(ref gn) = entry.generic_name {
                    if gn != &entry.name {
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(gn, Style::default().fg(Color::Gray)),
                        ]));
                    }
                }
            }

            // Line 3: Comment (topology info)
            if let Some(ref comment) = entry.comment {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(comment, Style::default().fg(Color::DarkGray)),
                ]));
            }

            // Line 4: Categories
            if config.behavior.show_categories && !entry.categories.is_empty() {
                let cats = entry.categories.join(",");
                lines.push(Line::from(vec![
                    Span::raw("  "),
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
}

/// Draw the status bar
fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let entries = app.visible_entries();
    let total = entries.len();

    let status = if app.is_filtering() {
        format!(
            " {} matches | ESC: clear filter | Enter: run | q: quit",
            total
        )
    } else {
        format!(
            " {}/{} | /: filter | j/k: navigate | Enter: run | q: quit",
            app.selected_index() + 1,
            total
        )
    };

    let status_bar = Paragraph::new(status).style(Style::default().fg(Color::DarkGray));

    f.render_widget(status_bar, area);
}
