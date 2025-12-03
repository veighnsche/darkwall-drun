mod app;
mod config;
mod desktop_entry;
mod executor;
mod history;
mod icons;
mod niri;
mod pty;
mod terminal;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use parking_lot::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use app::App;
use config::Config;
use icons::IconManager;

#[derive(Parser, Debug)]
#[command(name = "drun")]
#[command(about = "TUI application launcher - works locally or via SSH")]
#[command(version)]
struct Cli {
    /// Config file path
    #[arg(long, default_value = "~/.config/darkwall-drun/config.toml")]
    config: String,

    /// Run in daemon mode (stay open after command execution)
    #[arg(long, short)]
    daemon: bool,

    /// Disable niri IPC integration (auto-disabled when NIRI_SOCKET is absent)
    #[arg(long)]
    no_niri: bool,

    /// Enable mouse support (may not work well over SSH)
    #[arg(long)]
    mouse: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "darkwall_drun=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
        .init();

    let cli = Cli::parse();

    // Load config
    let config = Config::load(&cli.config)?;

    // Load desktop entries
    let entries = desktop_entry::load_all(&config.desktop_entry_dirs)?;
    tracing::info!("Loaded {} desktop entries", entries.len());

    // TEAM_002: Initialize icon manager BEFORE entering raw mode
    // This queries the terminal for graphics protocol support
    // Skip over SSH to avoid hanging on terminal queries
    let icon_manager = if config.icons.enabled && std::env::var("SSH_CONNECTION").is_err() {
        // Use a timeout to avoid hanging if terminal doesn't respond
        let mgr = IconManager::new(config.icons.size);
        if mgr.supports_graphics() {
            tracing::info!("Graphics icons enabled");
        } else {
            tracing::info!("No graphics protocol detected, icons disabled");
        }
        Some(Arc::new(Mutex::new(mgr)))
    } else {
        if std::env::var("SSH_CONNECTION").is_ok() {
            tracing::info!("Icons disabled over SSH");
        } else {
            tracing::info!("Icons disabled in config");
        }
        None
    };

    // Setup terminal
    // NOTE: DRUN is terminal-agnostic. It uses stdin/stdout/stderr only.
    // No assumptions about specific terminal emulators (kitty, foot, etc.)
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    
    // Mouse support is off by default for SSH compatibility
    if cli.mouse {
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    } else {
        execute!(stdout, EnterAlternateScreen)?;
    }
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    // Niri IPC is auto-disabled if socket doesn't exist (common over SSH)
    let mut app = App::new(entries, config, !cli.no_niri);

    // Run main loop
    let result = run_app(&mut terminal, &mut app, icon_manager).await;

    // TEAM_001: Save history before exit
    app.save_history();

    // Restore terminal
    disable_raw_mode()?;
    if cli.mouse {
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
    } else {
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    }
    terminal.show_cursor()?;

    result
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    icon_manager: Option<Arc<Mutex<IconManager>>>,
) -> Result<()> {
    loop {
        // Get terminal size for PTY
        let size = terminal.size()?;
        
        // Preload one icon per frame (non-blocking gradual loading)
        if let Some(ref mgr) = icon_manager {
            let entries = app.visible_entries();
            let icon_iter = entries.iter().map(|e| (e.id.as_str(), e.icon.as_deref()));
            mgr.lock().try_load_one(icon_iter);
        }
        
        terminal.draw(|f| ui::draw(f, app, icon_manager.as_ref()))?;

        // Handle TUI handover mode
        if let app::AppMode::TuiHandover { command } = app.mode() {
            let cmd = command.clone();
            app.execute_tui(&cmd)?;
            continue;
        }

        // Handle exit mode (GUI app launched)
        if matches!(app.mode(), app::AppMode::Exit) {
            return Ok(());
        }

        // Poll PTY if executing
        if app.is_executing() {
            app.poll_execution()?;
        }

        // Use shorter poll timeout when executing to be responsive
        let poll_timeout = if app.is_executing() {
            std::time::Duration::from_millis(16) // ~60fps
        } else {
            std::time::Duration::from_millis(100)
        };

        if event::poll(poll_timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if handle_key_event(app, key, size.width, size.height).await? {
                        return Ok(());
                    }
                }
                Event::Resize(cols, rows) => {
                    // Propagate resize to PTY (adjusted for UI chrome)
                    let output_cols = cols.saturating_sub(2);
                    let output_rows = rows.saturating_sub(6);
                    app.resize_pty(output_cols, output_rows).ok();
                }
                _ => {}
            }
        }
    }
}

/// Handle key events based on current app mode
/// Returns true if the app should exit
async fn handle_key_event(
    app: &mut App,
    key: event::KeyEvent,
    cols: u16,
    rows: u16,
) -> Result<bool> {
    use app::AppMode;

    match app.mode() {
        AppMode::Launcher => handle_launcher_keys(app, key, cols, rows).await,
        AppMode::Executing { .. } => handle_executing_keys(app, key),
        AppMode::PostExecution { .. } => handle_post_execution_keys(app, key),
        AppMode::TuiHandover { .. } => Ok(false), // Handled in main loop
        AppMode::Exit => Ok(true), // Exit immediately
    }
}

/// Handle keys in launcher mode
/// TEAM_004: Added grid navigation (left/right/tab/page)
async fn handle_launcher_keys(
    app: &mut App,
    key: event::KeyEvent,
    cols: u16,
    rows: u16,
) -> Result<bool> {
    match key.code {
        // Ctrl+C always exits
        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            return Ok(true);
        }
        // Esc clears filter or exits
        KeyCode::Esc => {
            if app.is_filtering() || !app.filter_text().is_empty() {
                app.clear_filter();
            } else {
                return Ok(true); // Exit
            }
        }
        // Enter executes selected entry
        KeyCode::Enter => {
            if let Some(entry) = app.selected_entry() {
                // Adjust size for UI chrome: header(3) + output borders(2) + status(1) = 6 rows
                // And 2 columns for left/right borders
                let output_cols = cols.saturating_sub(2);
                let output_rows = rows.saturating_sub(6);
                app.execute_entry(entry.clone(), output_cols, output_rows).await?;
            }
        }
        // Navigation - arrows always work
        KeyCode::Up => app.previous(),
        KeyCode::Down => app.next(),
        KeyCode::Left => app.move_left(),
        KeyCode::Right => app.move_right(),
        // TEAM_004: Page navigation
        KeyCode::PageUp => app.page_up(),
        KeyCode::PageDown => app.page_down(),
        KeyCode::Home => app.move_home(),
        KeyCode::End => app.move_end(),
        // TEAM_004: Tab navigation (wraps around)
        KeyCode::Tab => {
            if key.modifiers.contains(event::KeyModifiers::SHIFT) {
                app.tab_prev();
            } else {
                app.tab_next();
            }
        }
        KeyCode::BackTab => app.tab_prev(),
        // Backspace in filter mode
        KeyCode::Backspace => {
            if app.is_filtering() || !app.filter_text().is_empty() {
                app.pop_filter_char();
            }
        }
        // Any printable char starts/continues filtering
        KeyCode::Char(c) => {
            if !app.is_filtering() {
                app.start_filter();
            }
            app.push_filter_char(c);
        }
        _ => {}
    }
    Ok(false)
}

/// Handle keys in executing mode
fn handle_executing_keys(app: &mut App, key: event::KeyEvent) -> Result<bool> {
    use crate::terminal::{convert_keycode, convert_modifiers};

    match key.code {
        // Ctrl+C kills the process
        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            app.kill_execution();
        }
        // Scroll output (only when not following/at bottom)
        KeyCode::Up | KeyCode::Char('k') if !app.terminal().is_at_bottom() => {
            app.terminal_mut().scroll_up(1);
        }
        KeyCode::Down | KeyCode::Char('j') if !app.terminal().is_at_bottom() => {
            app.terminal_mut().scroll_down(1);
        }
        KeyCode::Char('u') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            app.terminal_mut().scroll_up(10);
        }
        KeyCode::Char('d') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            app.terminal_mut().scroll_down(10);
        }
        KeyCode::Char('g') => {
            // Scroll to top of scrollback
            let max_offset = app.terminal().scrollback().len();
            app.terminal_mut().set_scroll_offset(max_offset);
        }
        KeyCode::Char('G') => {
            app.terminal_mut().scroll_to_bottom();
        }
        // Forward other input to the process using proper key encoding
        _ => {
            let tw_key = convert_keycode(key.code);
            let tw_mods = convert_modifiers(key.modifiers);
            let encoded = app.terminal().encode_key(tw_key, tw_mods);
            if !encoded.is_empty() {
                app.send_input(encoded.as_bytes())?;
            }
        }
    }
    Ok(false)
}

/// Handle keys in post-execution mode
/// Uses same scroll handling as Executing mode via terminal
fn handle_post_execution_keys(app: &mut App, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        // Enter or Esc dismisses output and returns to launcher
        KeyCode::Enter | KeyCode::Esc => {
            app.dismiss_output();
        }
        // Ctrl+C or q exits
        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            return Ok(true);
        }
        KeyCode::Char('q') => {
            return Ok(true);
        }
        // Copy output to clipboard
        KeyCode::Char('y') => {
            if let Err(e) = app.copy_output_to_clipboard() {
                tracing::warn!("Failed to copy to clipboard: {}", e);
            }
        }
        // Scroll up (into scrollback history)
        KeyCode::Up | KeyCode::Char('k') => {
            app.terminal_mut().scroll_up(1);
        }
        // Scroll down (toward current output)
        KeyCode::Down | KeyCode::Char('j') => {
            app.terminal_mut().scroll_down(1);
        }
        // Page up/down
        KeyCode::Char('u') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            app.terminal_mut().scroll_up(10);
        }
        KeyCode::Char('d') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            app.terminal_mut().scroll_down(10);
        }
        // Go to top of scrollback
        KeyCode::Char('g') => {
            let max_offset = app.terminal().scrollback().len();
            app.terminal_mut().set_scroll_offset(max_offset);
        }
        // Go to bottom
        KeyCode::Char('G') => {
            app.terminal_mut().scroll_to_bottom();
        }
        _ => {}
    }
    Ok(false)
}
