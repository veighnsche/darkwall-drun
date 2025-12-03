use anyhow::Result;
use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Matcher,
};

use crate::config::Config;
use crate::desktop_entry::Entry;
use crate::executor::{CommandStatus, OutputBuffer, TerminalMode};
use crate::niri::NiriClient;
use crate::pty::PtySession;

/// Application mode - determines what UI to show and how to handle input
/// TEAM_000: Phase 2, Unit 2.3 - State transitions
#[derive(Debug)]
pub enum AppMode {
    /// Normal launcher mode - showing entry list
    Launcher,
    /// Executing a command with PTY - showing output
    Executing {
        command: String,
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
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn next(&mut self) {
        if self.selected < self.filtered.len().saturating_sub(1) {
            self.selected += 1;
        }
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
    fn update_filtered(&mut self) {
        if self.filter.is_empty() {
            self.filtered = (0..self.entries.len()).collect();
        } else {
            let pattern = Pattern::parse(&self.filter, CaseMatching::Ignore, Normalization::Smart);

            let mut scored: Vec<(usize, u32)> = self
                .entries
                .iter()
                .enumerate()
                .filter_map(|(i, entry)| {
                    let haystack = entry.search_text();
                    let mut buf = Vec::new();
                    pattern
                        .score(nucleo_matcher::Utf32Str::new(&haystack, &mut buf), &mut self.matcher)
                        .map(|score| (i, score))
                })
                .collect();

            // Sort by score descending
            scored.sort_by(|a, b| b.1.cmp(&a.1));
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
    pub fn is_launcher_mode(&self) -> bool {
        matches!(self.mode, AppMode::Launcher)
    }

    /// Check if we're executing a command
    pub fn is_executing(&self) -> bool {
        matches!(self.mode, AppMode::Executing { .. })
    }

    /// Check if we're in post-execution mode
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
    pub async fn execute_entry(&mut self, entry: Entry, cols: u16, rows: u16) -> Result<()> {
        let Some(cmd) = entry.command() else {
            tracing::warn!("Entry {} has no command", entry.id);
            return Ok(());
        };

        tracing::info!("Executing: {}", cmd);

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

        // Clear output buffer for new command
        self.output_buffer.clear();

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
}
