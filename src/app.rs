use anyhow::Result;
use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Matcher,
};

use crate::config::Config;
use crate::desktop_entry::Entry;
use crate::executor::{CommandStatus, OutputBuffer, TerminalMode};
use crate::history::History;
use crate::niri::NiriClient;
use crate::pty::PtySession;
use crate::ui::layout::GridLayout;

/// Application mode - determines what UI to show and how to handle input
/// TEAM_000: Phase 2, Unit 2.3 - State transitions
#[derive(Debug, Clone)]
pub enum AppMode {
    /// Normal launcher mode - showing entry list
    Launcher,
    /// Executing a command with PTY - showing output
    Executing {
        command: String,
        /// NOTE: Reserved for mode-specific UI behavior (e.g., different status indicators)
        #[allow(dead_code)]
        mode: TerminalMode,
    },
    /// Command finished, showing preserved output above launcher
    PostExecution {
        command: String,
        exit_status: CommandStatus,
        preserved_output: Vec<String>,
    },
    /// TUI mode - full terminal handover (htop, vim, etc.)
    TuiHandover {
        command: String,
    },
}

/// Application state
pub struct App {
    /// Current application mode
    mode: AppMode,
    /// All loaded desktop entries
    entries: Vec<Entry>,
    /// Filtered entries (indices into `entries`)
    filtered: Vec<usize>,
    /// Currently selected index in filtered list
    selected: usize,
    /// Current filter text
    filter: String,
    /// Whether we're in filter input mode
    filtering: bool,
    /// Configuration
    config: Config,
    /// Niri IPC client
    niri: Option<NiriClient>,
    /// PTY session for current execution (if any)
    pty_session: Option<PtySession>,
    /// Output buffer for current execution
    output_buffer: OutputBuffer,
    /// Fuzzy matcher
    matcher: Matcher,
    /// TEAM_001: Usage history for frecency sorting
    history: History,
    /// TEAM_001: Frecency weight from config
    frecency_weight: f64,
    /// TEAM_004: Grid layout for 2-column display
    grid_layout: GridLayout,
}

impl App {
    pub fn new(entries: Vec<Entry>, config: Config, niri_enabled: bool) -> Self {
        let filtered: Vec<usize> = (0..entries.len()).collect();
        
        // Niri IPC: gracefully disabled if socket not found (e.g., over SSH)
        let niri = if niri_enabled {
            NiriClient::try_new()
        } else {
            None
        };

        let max_output_lines = config.behavior.preserve_output_lines.max(1000);
        
        // TEAM_001: Initialize history
        let mut history = History::new(
            config.history.max_entries,
            config.history.decay_after_days,
        );
        if config.history.enabled {
            if let Err(e) = history.load() {
                tracing::warn!("Failed to load history: {}", e);
            }
        }
        let frecency_weight = config.history.frecency_weight;
        
        // TEAM_004: Initialize grid layout from config
        let grid_layout = config.grid_layout();
        
        Self {
            mode: AppMode::Launcher,
            entries,
            filtered,
            selected: 0,
            filter: String::new(),
            filtering: false,
            config,
            niri,
            pty_session: None,
            output_buffer: OutputBuffer::new(max_output_lines),
            matcher: Matcher::new(nucleo_matcher::Config::DEFAULT),
            history,
            frecency_weight,
            grid_layout,
        }
    }

    /// Get currently visible entries
    pub fn visible_entries(&self) -> Vec<&Entry> {
        self.filtered.iter().map(|&i| &self.entries[i]).collect()
    }

    /// Get the currently selected entry
    pub fn selected_entry(&self) -> Option<&Entry> {
        self.filtered.get(self.selected).map(|&i| &self.entries[i])
    }

    /// Get selected index
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Move selection up
    pub fn previous(&mut self) {
        self.selected = self.grid_layout.move_up(self.selected);
    }

    /// Move selection down
    pub fn next(&mut self) {
        self.selected = self.grid_layout.move_down(self.selected, self.filtered.len());
    }

    /// TEAM_004: Move selection left (previous column)
    pub fn move_left(&mut self) {
        self.selected = self.grid_layout.move_left(self.selected);
    }

    /// TEAM_004: Move selection right (next column)
    pub fn move_right(&mut self) {
        self.selected = self.grid_layout.move_right(self.selected, self.filtered.len());
    }

    /// TEAM_004: Tab navigation (next with wrap)
    pub fn tab_next(&mut self) {
        self.selected = self.grid_layout.tab_next(self.selected, self.filtered.len());
    }

    /// TEAM_004: Shift+Tab navigation (previous with wrap)
    pub fn tab_prev(&mut self) {
        self.selected = self.grid_layout.tab_prev(self.selected, self.filtered.len());
    }

    /// TEAM_004: Page up
    pub fn page_up(&mut self) {
        self.selected = self.grid_layout.page_up(self.selected);
    }

    /// TEAM_004: Page down
    pub fn page_down(&mut self) {
        self.selected = self.grid_layout.page_down(self.selected, self.filtered.len());
    }

    /// TEAM_004: Move to first entry
    pub fn move_home(&mut self) {
        self.selected = self.grid_layout.move_home();
    }

    /// TEAM_004: Move to last entry
    pub fn move_end(&mut self) {
        self.selected = self.grid_layout.move_end(self.filtered.len());
    }

    /// TEAM_004: Get grid layout reference
    pub fn grid_layout(&self) -> &GridLayout {
        &self.grid_layout
    }

    /// Check if currently filtering
    pub fn is_filtering(&self) -> bool {
        self.filtering
    }

    /// Start filter mode
    pub fn start_filter(&mut self) {
        self.filtering = true;
    }

    /// Clear filter and exit filter mode
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.filtering = false;
        self.update_filtered();
    }

    /// Get current filter text
    pub fn filter_text(&self) -> &str {
        &self.filter
    }

    /// Add character to filter
    pub fn push_filter_char(&mut self, c: char) {
        self.filter.push(c);
        self.update_filtered();
    }

    /// Remove last character from filter
    pub fn pop_filter_char(&mut self) {
        self.filter.pop();
        if self.filter.is_empty() {
            self.filtering = false;
        }
        self.update_filtered();
    }

    /// Update filtered list based on current filter
    /// TEAM_001: Integrated frecency scoring
    fn update_filtered(&mut self) {
        if self.filter.is_empty() {
            // No filter: sort by frecency only
            let mut scored: Vec<(usize, f64)> = self
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let frecency = self.history.frecency_score(&entry.id);
                    (i, frecency)
                })
                .collect();

            // Sort by frecency descending, then alphabetically for ties
            scored.sort_by(|a, b| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| {
                        self.entries[a.0].name.cmp(&self.entries[b.0].name)
                    })
            });
            self.filtered = scored.into_iter().map(|(i, _)| i).collect();
        } else {
            let pattern = Pattern::parse(&self.filter, CaseMatching::Ignore, Normalization::Smart);

            // Combine fuzzy score with frecency
            let mut scored: Vec<(usize, f64)> = self
                .entries
                .iter()
                .enumerate()
                .filter_map(|(i, entry)| {
                    let haystack = entry.search_text();
                    let mut buf = Vec::new();
                    pattern
                        .score(nucleo_matcher::Utf32Str::new(&haystack, &mut buf), &mut self.matcher)
                        .map(|fuzzy_score| {
                            let frecency = self.history.frecency_score(&entry.id);
                            // Weighted combination: fuzzy_score normalized + frecency weight
                            // Fuzzy scores are typically 0-1000+, frecency is 0-~500
                            let fuzzy_norm = fuzzy_score as f64;
                            let combined = fuzzy_norm * (1.0 - self.frecency_weight)
                                + frecency * self.frecency_weight * 10.0; // Scale frecency
                            (i, combined)
                        })
                })
                .collect();

            // Sort by combined score descending
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            self.filtered = scored.into_iter().map(|(i, _)| i).collect();
        }

        // Reset selection if out of bounds
        if self.selected >= self.filtered.len() {
            self.selected = 0;
        }
    }

    /// Get current application mode
    pub fn mode(&self) -> &AppMode {
        &self.mode
    }

    /// Check if we're in launcher mode
    /// NOTE: Currently unused but kept for API completeness - may be useful for plugins/extensions
    #[allow(dead_code)]
    pub fn is_launcher_mode(&self) -> bool {
        matches!(self.mode, AppMode::Launcher)
    }

    /// Check if we're executing a command
    pub fn is_executing(&self) -> bool {
        matches!(self.mode, AppMode::Executing { .. })
    }

    /// Check if we're in post-execution mode
    /// NOTE: Currently unused but kept for API completeness - may be useful for plugins/extensions
    #[allow(dead_code)]
    pub fn is_post_execution(&self) -> bool {
        matches!(self.mode, AppMode::PostExecution { .. })
    }

    /// Get output buffer reference
    pub fn output_buffer(&self) -> &OutputBuffer {
        &self.output_buffer
    }

    /// Get mutable output buffer reference
    pub fn output_buffer_mut(&mut self) -> &mut OutputBuffer {
        &mut self.output_buffer
    }

    /// Start executing the selected entry
    /// TEAM_000: Phase 2 - In-place execution with PTY
    /// TEAM_001: Records usage for frecency
    pub async fn execute_entry(&mut self, entry: Entry, cols: u16, rows: u16) -> Result<()> {
        let Some(cmd) = entry.command() else {
            tracing::warn!("Entry {} has no command", entry.id);
            return Ok(());
        };

        tracing::info!("Executing: {}", cmd);

        // TEAM_001: Record usage for frecency sorting
        if self.config.history.enabled {
            self.history.record_usage(&entry.id);
            // Re-sort entries so next time this entry appears higher
            self.update_filtered();
        }

        // Detect terminal mode
        let terminal_mode = TerminalMode::detect(&cmd, Some(&entry));
        tracing::debug!("Terminal mode: {:?}", terminal_mode);

        // Handle TUI apps specially - they need full terminal control
        if terminal_mode == TerminalMode::Tui {
            self.mode = AppMode::TuiHandover { command: cmd };
            return Ok(());
        }

        // Unfloat window if configured
        if self.config.niri.unfloat_on_execute {
            if let Some(ref niri) = self.niri {
                niri.set_floating(false).await.ok();
            }
        }

        // Clear output buffer and filter for new command
        self.output_buffer.clear();
        self.filter.clear();
        self.update_filtered();

        // Spawn PTY session
        let session = PtySession::spawn(&cmd, cols, rows)?;
        self.pty_session = Some(session);

        // Enter executing mode
        self.mode = AppMode::Executing {
            command: cmd,
            mode: terminal_mode,
        };

        Ok(())
    }

    /// Execute a TUI app with full terminal handover
    /// Returns the exit code when the app exits
    pub fn execute_tui(&mut self, cmd: &str) -> Result<Option<i32>> {
        use crossterm::{
            execute,
            terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        };
        use std::io;

        // 1. Disable our TUI
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;

        // 2. Run the command directly
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()?;

        // 3. Restore our TUI
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;

        // Return to launcher mode
        self.mode = AppMode::Launcher;

        Ok(status.code())
    }

    /// Poll PTY for output and check if command has exited
    /// Returns true if command is still running
    pub fn poll_execution(&mut self) -> Result<bool> {
        let Some(ref mut session) = self.pty_session else {
            return Ok(false);
        };

        // Read available output
        let mut buf = [0u8; 4096];
        loop {
            match session.try_read(&mut buf) {
                Ok(Some(n)) if n > 0 => {
                    self.output_buffer.push(&buf[..n]);
                }
                Ok(_) => break, // No more data or EOF
                Err(e) => {
                    tracing::warn!("PTY read error: {}", e);
                    break;
                }
            }
        }

        // Check if process has exited
        match session.try_wait()? {
            Some(status) => {
                // Process exited
                self.output_buffer.flush();
                
                let exit_status = CommandStatus::from_exit_status(status);
                let preserved = self.output_buffer.last_n_lines(
                    self.config.behavior.preserve_output_lines
                );

                // Extract command from current mode
                let command = match &self.mode {
                    AppMode::Executing { command, .. } => command.clone(),
                    _ => String::new(),
                };

                // Transition to post-execution
                self.mode = AppMode::PostExecution {
                    command,
                    exit_status,
                    preserved_output: preserved,
                };

                // Clean up PTY
                self.pty_session = None;

                // Re-float window if configured
                if self.config.niri.float_on_idle {
                    if let Some(ref niri) = self.niri {
                        // Fire and forget - don't block on this
                        let niri = niri.clone();
                        tokio::spawn(async move {
                            niri.set_floating(true).await.ok();
                        });
                    }
                }

                Ok(false)
            }
            None => Ok(true), // Still running
        }
    }

    /// Send input to the running command
    pub fn send_input(&mut self, data: &[u8]) -> Result<()> {
        if let Some(ref mut session) = self.pty_session {
            session.write(data)?;
        }
        Ok(())
    }

    /// Resize the PTY (call on terminal resize)
    pub fn resize_pty(&mut self, cols: u16, rows: u16) -> Result<()> {
        if let Some(ref session) = self.pty_session {
            session.resize(cols, rows)?;
        }
        Ok(())
    }

    /// Dismiss post-execution output and return to launcher
    pub fn dismiss_output(&mut self) {
        if matches!(self.mode, AppMode::PostExecution { .. }) {
            self.output_buffer.clear();
            self.filter.clear();
            self.update_filtered();
            self.mode = AppMode::Launcher;
        }
    }

    /// Kill the current execution
    pub fn kill_execution(&mut self) {
        self.pty_session = None; // Drop will kill the process
        self.mode = AppMode::Launcher;
    }

    /// Get config reference
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// TEAM_001: Save history to disk
    pub fn save_history(&self) {
        if self.config.history.enabled {
            if let Err(e) = self.history.save() {
                tracing::warn!("Failed to save history: {}", e);
            }
        }
    }
}
