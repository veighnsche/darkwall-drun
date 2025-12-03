mod app;
mod config;
mod desktop_entry;
mod niri;
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
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use app::App;
use config::Config;

#[derive(Parser, Debug)]
#[command(name = "darkwall-tui")]
#[command(about = "TUI application launcher with niri integration")]
#[command(version)]
struct Cli {
    /// Config file path
    #[arg(long, default_value = "~/.config/darkwall-tui/config.toml")]
    config: String,

    /// Run in daemon mode (stay open after command execution)
    #[arg(long, short)]
    daemon: bool,

    /// Disable niri IPC integration
    #[arg(long)]
    no_niri: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "darkwall_tui=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
        .init();

    let cli = Cli::parse();

    // Load config
    let config = Config::load(&cli.config)?;

    // Load desktop entries
    let entries = desktop_entry::load_all(&config.desktop_entry_dirs)?;
    tracing::info!("Loaded {} desktop entries", entries.len());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(entries, config, !cli.no_niri);

    // Run main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Esc => {
                        if app.is_filtering() {
                            app.clear_filter();
                        } else {
                            return Ok(());
                        }
                    }
                    KeyCode::Char('q') if !app.is_filtering() => return Ok(()),
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        if let Some(entry) = app.selected_entry() {
                            app.execute_entry(entry.clone()).await?;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') if !app.is_filtering() => {
                        app.previous();
                    }
                    KeyCode::Down | KeyCode::Char('j') if !app.is_filtering() => {
                        app.next();
                    }
                    KeyCode::Char('/') if !app.is_filtering() => {
                        app.start_filter();
                    }
                    KeyCode::Char(c) if app.is_filtering() => {
                        app.push_filter_char(c);
                    }
                    KeyCode::Backspace if app.is_filtering() => {
                        app.pop_filter_char();
                    }
                    KeyCode::Char(c) if !app.is_filtering() => {
                        // Start filtering immediately on any char
                        app.start_filter();
                        app.push_filter_char(c);
                    }
                    _ => {}
                }
            }
        }
    }
}
