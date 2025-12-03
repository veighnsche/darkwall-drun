use anyhow::Result;
use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Matcher,
};

use crate::config::Config;
use crate::desktop_entry::Entry;
use crate::niri::NiriClient;

/// Application state
pub struct App {
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
    /// Last command output (for display after return)
    last_output: Option<CommandOutput>,
    /// Fuzzy matcher
    matcher: Matcher,
}

/// Output from an executed command
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl App {
    pub fn new(entries: Vec<Entry>, config: Config, niri_enabled: bool) -> Self {
        let filtered: Vec<usize> = (0..entries.len()).collect();
        let niri = if niri_enabled {
            NiriClient::new().ok()
        } else {
            None
        };

        Self {
            entries,
            filtered,
            selected: 0,
            filter: String::new(),
            filtering: false,
            config,
            niri,
            last_output: None,
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

    /// Execute the selected entry
    pub async fn execute_entry(&mut self, entry: Entry) -> Result<()> {
        let Some(cmd) = entry.command() else {
            tracing::warn!("Entry {} has no command", entry.id);
            return Ok(());
        };

        tracing::info!("Executing: {}", cmd);

        // Unfloat window if configured
        if self.config.niri.unfloat_on_execute {
            if let Some(ref niri) = self.niri {
                niri.set_floating(false).await.ok();
            }
        }

        // TODO: Phase 2 - Execute in-place with PTY
        // For now, spawn in new terminal (Phase 1 behavior)
        let status = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .status()
            .await?;

        self.last_output = Some(CommandOutput {
            command: cmd,
            exit_code: status.code(),
            stdout: String::new(),
            stderr: String::new(),
        });

        // Re-float window if configured
        if self.config.niri.float_on_idle {
            if let Some(ref niri) = self.niri {
                niri.set_floating(true).await.ok();
            }
        }

        Ok(())
    }

    /// Get last command output
    pub fn last_output(&self) -> Option<&CommandOutput> {
        self.last_output.as_ref()
    }

    /// Get config reference
    pub fn config(&self) -> &Config {
        &self.config
    }
}
