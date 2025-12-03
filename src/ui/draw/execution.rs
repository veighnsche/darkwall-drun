//! Execution mode drawing functions
//!
//! This module handles rendering the command execution UI:
//! - Executing mode (live output)
//! - Post-execution mode (results display)

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::executor::CommandStatus;
use crate::terminal::TerminalWidget;
use crate::ui::theme::Theme;

/// Draw the executing UI - shows command output using terminal emulator
/// TEAM_000: Phase 2, Unit 2.2 - Output display
/// TEAM_004: Updated to use theme
/// TEAM_010: Updated to use TerminalWidget
pub(crate) fn draw_executing(f: &mut Frame, app: &App, command: &str, theme: &Theme) {
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
        .style(
            Style::default()
                .fg(theme.exit_success)
                .bg(theme.background),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.exit_success))
                .title(" Running ")
                .style(Style::default().bg(theme.background)),
        );
    f.render_widget(header, chunks[0]);

    // Output area - render terminal widget inside a block
    let output_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dimmed_alt))
        .title(" Output ")
        .style(Style::default().bg(theme.background));

    let inner_area = output_block.inner(chunks[1]);
    f.render_widget(output_block, chunks[1]);

    // Render terminal widget
    let terminal = app.terminal();
    let widget = TerminalWidget::new(terminal).show_cursor(true);
    f.render_widget(widget, inner_area);

    // Status bar - show follow mode indicator
    let is_following = terminal.is_following();
    let total_lines = terminal.total_lines();
    let follow_indicator = if is_following {
        "[following]"
    } else {
        "[paused]"
    };
    let status = format!(
        " {} lines {} | Ctrl+C: kill | j/k: scroll | g/G: top/bottom",
        total_lines, follow_indicator
    );
    let status_bar =
        Paragraph::new(status).style(Style::default().fg(theme.accent).bg(theme.background));
    f.render_widget(status_bar, chunks[2]);
}

/// Draw the post-execution UI - reuses TerminalWidget like Executing mode
/// TEAM_000: Phase 2, Unit 2.3 - Return to launcher
/// TEAM_004: Updated to use theme
pub(crate) fn draw_post_execution(
    f: &mut Frame,
    app: &App,
    command: &str,
    exit_status: &CommandStatus,
    copy_feedback: &Option<std::time::Instant>,
    theme: &Theme,
) {
    // Determine colors based on exit status
    let (exit_text, exit_color) = match exit_status {
        CommandStatus::Exited(0) => ("Exit: 0".to_string(), theme.exit_success),
        CommandStatus::Exited(code) => (format!("Exit: {}", code), theme.exit_failure),
        CommandStatus::Signaled(sig) => (format!("Signal: {}", sig), theme.exit_failure),
        CommandStatus::Running => ("Running".to_string(), theme.accent),
        CommandStatus::Unknown => ("Unknown".to_string(), theme.dimmed),
    };

    // Fill background
    let bg_block = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(bg_block, f.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Command header with exit status
            Constraint::Min(1),    // Output (terminal widget)
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    // Command header with exit status
    let header = Paragraph::new(format!("$ {} [{}]", command, exit_text))
        .style(Style::default().fg(exit_color).bg(theme.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(exit_color))
                .title(" Last Command ")
                .style(Style::default().bg(theme.background)),
        );
    f.render_widget(header, chunks[0]);

    // Output area - render terminal widget (same as Executing mode)
    let output_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dimmed_alt))
        .title(" Output ")
        .style(Style::default().bg(theme.background));

    let inner_area = output_block.inner(chunks[1]);
    f.render_widget(output_block, chunks[1]);

    // Render terminal widget (no cursor in post-execution)
    let terminal = app.terminal();
    let widget = TerminalWidget::new(terminal).show_cursor(false);
    f.render_widget(widget, inner_area);

    // Status bar - show scroll info and copy feedback
    let is_at_bottom = terminal.is_at_bottom();
    let total_lines = terminal.total_lines();
    let scroll_indicator = if is_at_bottom {
        "[bottom]"
    } else {
        "[scrolled]"
    };

    // Check if we should show copy feedback
    let copy_feedback = if let Some(instant) = copy_feedback {
        if instant.elapsed().as_secs() < 2 {
            Some("Copied!")
        } else {
            None
        }
    } else {
        None
    };

    let status = if let Some(msg) = copy_feedback {
        format!(
            " {} lines {} | {} | y: copy | Enter: dismiss | q: quit",
            total_lines, scroll_indicator, msg
        )
    } else {
        format!(
            " {} lines {} | y: copy | Enter: dismiss | q: quit",
            total_lines, scroll_indicator
        )
    };

    let status_color = if copy_feedback.is_some() {
        theme.exit_success
    } else {
        theme.dimmed
    };

    let status_bar =
        Paragraph::new(status).style(Style::default().fg(status_color).bg(theme.background));
    f.render_widget(status_bar, chunks[2]);
}
