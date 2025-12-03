//! Drawing functions for the TUI
//!
//! This module contains all rendering logic, split by mode:
//! - `launcher` - Main launcher UI (search, entry list, status)
//! - `execution` - Command execution and post-execution views

mod execution;
mod launcher;

use parking_lot::Mutex;
use ratatui::{style::Style, widgets::Paragraph, Frame};
use std::sync::Arc;

use crate::app::{App, AppMode};
use crate::icons::IconManager;

use execution::{draw_executing, draw_post_execution};
use launcher::draw_launcher;

/// Main draw function
/// TEAM_000: Phase 2 - Updated for execution modes
/// TEAM_002: Added icon manager parameter
/// TEAM_004: Added theme parameter for theming support
pub fn draw(f: &mut Frame, app: &mut App, icon_manager: Option<&Arc<Mutex<IconManager>>>) {
    // TEAM_004: Resolve theme from config
    let theme = app.config().resolve_theme();
    // Clone mode to avoid borrow conflict with &mut app
    let mode = app.mode().clone();
    match mode {
        AppMode::Launcher => draw_launcher(f, app, icon_manager, &theme),
        AppMode::Executing { ref command, .. } => draw_executing(f, app, command, &theme),
        AppMode::PostExecution {
            ref command,
            ref exit_status,
            ref copy_feedback,
        } => draw_post_execution(f, app, command, exit_status, copy_feedback, &theme),
        AppMode::TuiHandover { .. } => {
            // TUI handover - we shouldn't be drawing, but show a message just in case
            let msg = Paragraph::new("Running TUI application...")
                .style(Style::default().fg(theme.accent));
            f.render_widget(msg, f.area());
        }
        AppMode::Exit => {
            // Exit mode - shouldn't be drawing, but handle gracefully
        }
    }
}
