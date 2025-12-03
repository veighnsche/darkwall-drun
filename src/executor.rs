//! Command execution with output capture and terminal mode detection.
//!
//! TEAM_000: Phase 2, Units 2.2-2.4

use crate::desktop_entry::Entry;
use crate::pty::ExitStatus;
use std::collections::VecDeque;

/// Terminal mode determines how a command should be executed
/// TEAM_000: Phase 4, Unit 4.1 - Terminal Mode Schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalMode {
    /// Simple command, capture output (ls, echo, etc.)
    Oneshot,
    /// Needs input but not full screen (bash, python REPL)
    Interactive,
    /// Full screen TUI app (btop, htop, vim)
    Tui,
    /// Long-running process (server, watch)
    LongRunning,
}

impl std::str::FromStr for TerminalMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "oneshot" => Ok(TerminalMode::Oneshot),
            "interactive" => Ok(TerminalMode::Interactive),
            "tui" => Ok(TerminalMode::Tui),
            "long-running" | "longrunning" | "long_running" => Ok(TerminalMode::LongRunning),
            _ => Err(()),
        }
    }
}

impl TerminalMode {
    /// Detect terminal mode from command and optional desktop entry
    pub fn detect(cmd: &str, entry: Option<&Entry>) -> Self {
        // 1. Check custom X-DarkwallTerminalMode field first
        if let Some(entry) = entry {
            if let Some(mode) = entry.get_darkwall_field("TerminalMode") {
                if let Ok(m) = mode.parse() {
                    return m;
                }
            }
        }
        
        // 2. Extract the base command (first word)
        let base_cmd = cmd
            .split_whitespace()
            .next()
            .unwrap_or("")
            .rsplit('/')
            .next()
            .unwrap_or("");

        // 3. Known TUI apps (full screen)
        const TUI_APPS: &[&str] = &[
            "htop", "btop", "top", "vim", "nvim", "nano", "less", "man",
            "mc", "ranger", "nnn", "lf", "vifm", "tmux", "screen",
            "mutt", "neomutt", "weechat", "irssi", "cmus", "ncmpcpp",
            "lazygit", "tig", "gitui", "k9s", "dive",
        ];
        if TUI_APPS.iter().any(|&app| base_cmd == app) {
            return TerminalMode::Tui;
        }

        // 4. Known interactive commands (REPL-style)
        const INTERACTIVE: &[&str] = &[
            "bash", "zsh", "fish", "sh", "dash",
            "python", "python3", "ipython", "bpython",
            "node", "deno", "bun",
            "irb", "pry", "ruby",
            "ghci", "stack ghci",
            "lua", "luajit",
            "erl", "iex",
            "psql", "mysql", "sqlite3",
            "redis-cli", "mongosh",
        ];
        if INTERACTIVE.iter().any(|&app| base_cmd == app) {
            return TerminalMode::Interactive;
        }

        // 5. Long-running patterns
        if cmd.contains("watch ") || cmd.contains("tail -f") || cmd.contains("journalctl -f") {
            return TerminalMode::LongRunning;
        }

        // 6. Check if entry has Terminal=true (suggests interactive)
        if let Some(entry) = entry {
            if entry.terminal {
                // Terminal apps that aren't TUI are likely interactive
                return TerminalMode::Interactive;
            }
        }

        // Default to oneshot
        TerminalMode::Oneshot
    }
}

/// Buffer for captured command output
pub struct OutputBuffer {
    lines: VecDeque<OutputLine>,
    max_lines: usize,
    scroll_offset: usize,
    /// Current incomplete line (no newline yet)
    partial_line: String,
}

/// A single line of output with optional ANSI styling info
#[derive(Debug, Clone)]
pub struct OutputLine {
    pub content: String,
    // TODO: Add ANSI style spans for colored output
}

impl OutputBuffer {
    /// Create a new output buffer with the given max line capacity
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_lines),
            max_lines,
            scroll_offset: 0,
            partial_line: String::new(),
        }
    }

    /// Push raw bytes to the buffer, parsing newlines
    pub fn push(&mut self, data: &[u8]) {
        let text = String::from_utf8_lossy(data);
        
        for ch in text.chars() {
            if ch == '\n' {
                // Complete the current line
                let line = std::mem::take(&mut self.partial_line);
                self.push_line(line);
            } else if ch == '\r' {
                // Carriage return - for now just ignore (handle \r\n as \n)
                // TODO: Handle \r properly for progress bars
            } else {
                self.partial_line.push(ch);
            }
        }
    }

    /// Push a complete line
    fn push_line(&mut self, content: String) {
        // Strip ANSI escape codes for now (Phase 2.2 basic implementation)
        // TODO: Parse and preserve ANSI styles
        let stripped = strip_ansi_escapes(&content);
        
        self.lines.push_back(OutputLine { content: stripped });
        
        // Enforce max lines
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
            // Adjust scroll offset if we removed lines above viewport
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
        }
    }

    /// Flush any partial line (call when command exits)
    pub fn flush(&mut self) {
        if !self.partial_line.is_empty() {
            let line = std::mem::take(&mut self.partial_line);
            self.push_line(line);
        }
    }

    /// Get all lines
    /// NOTE: Used in tests; kept for API completeness
    #[allow(dead_code)]
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().map(|l| l.content.as_str())
    }

    /// Get the number of lines
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if buffer is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Get current scroll offset
    #[allow(dead_code)]
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Scroll up by n lines
    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    /// Scroll down by n lines
    pub fn scroll_down(&mut self, n: usize, viewport_height: usize) {
        let max_scroll = self.lines.len().saturating_sub(viewport_height);
        self.scroll_offset = (self.scroll_offset + n).min(max_scroll);
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self, viewport_height: usize) {
        self.scroll_offset = self.lines.len().saturating_sub(viewport_height);
    }

    /// Get visible lines for the given viewport height
    pub fn visible_lines(&self, viewport_height: usize) -> impl Iterator<Item = &str> {
        self.lines
            .iter()
            .skip(self.scroll_offset)
            .take(viewport_height)
            .map(|l| l.content.as_str())
    }

    /// Get the last N lines (for preservation after command exit)
    pub fn last_n_lines(&self, n: usize) -> Vec<String> {
        self.lines
            .iter()
            .rev()
            .take(n)
            .map(|l| l.content.clone())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.lines.clear();
        self.partial_line.clear();
        self.scroll_offset = 0;
    }
}

/// Strip ANSI escape sequences from a string
/// Basic implementation - strips CSI sequences
fn strip_ansi_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Start of escape sequence
            if let Some(&'[') = chars.peek() {
                chars.next(); // consume '['
                // Skip until we hit a letter (end of CSI sequence)
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// Status of a command execution
#[derive(Debug, Clone)]
#[allow(dead_code)] // Variants used in pattern matching in ui.rs
pub enum CommandStatus {
    /// Command is still running
    Running,
    /// Command exited with code
    Exited(i32),
    /// Command was killed by signal
    Signaled(i32),
    /// Unknown status
    Unknown,
}

impl CommandStatus {
    /// Create from portable_pty::ExitStatus
    pub fn from_exit_status(status: ExitStatus) -> Self {
        if status.success() {
            CommandStatus::Exited(0)
        } else {
            // portable_pty doesn't expose the exact code easily
            // We can only tell success vs failure
            CommandStatus::Exited(1)
        }
    }
    
    /// Create from std::process::ExitStatus (for TUI handover)
    /// NOTE: Reserved for future TUI exit status reporting
    #[allow(dead_code)]
    pub fn from_std_exit_status(status: std::process::ExitStatus) -> Self {
        if let Some(code) = status.code() {
            CommandStatus::Exited(code)
        } else {
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                if let Some(signal) = status.signal() {
                    return CommandStatus::Signaled(signal);
                }
            }
            CommandStatus::Unknown
        }
    }

    /// Check if this represents a successful exit
    /// NOTE: Reserved for future use in exit status display
    #[allow(dead_code)]
    pub fn is_success(&self) -> bool {
        matches!(self, CommandStatus::Exited(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_mode_detection() {
        assert_eq!(TerminalMode::detect("ls -la", None), TerminalMode::Oneshot);
        assert_eq!(TerminalMode::detect("htop", None), TerminalMode::Tui);
        assert_eq!(TerminalMode::detect("vim test.txt", None), TerminalMode::Tui);
        assert_eq!(TerminalMode::detect("python", None), TerminalMode::Interactive);
        assert_eq!(TerminalMode::detect("bash", None), TerminalMode::Interactive);
        assert_eq!(TerminalMode::detect("watch ls", None), TerminalMode::LongRunning);
    }

    #[test]
    fn test_terminal_mode_from_str() {
        assert_eq!("oneshot".parse::<TerminalMode>().unwrap(), TerminalMode::Oneshot);
        assert_eq!("interactive".parse::<TerminalMode>().unwrap(), TerminalMode::Interactive);
        assert_eq!("tui".parse::<TerminalMode>().unwrap(), TerminalMode::Tui);
        assert_eq!("long-running".parse::<TerminalMode>().unwrap(), TerminalMode::LongRunning);
        assert_eq!("longrunning".parse::<TerminalMode>().unwrap(), TerminalMode::LongRunning);
        assert!("invalid".parse::<TerminalMode>().is_err());
    }

    #[test]
    fn test_output_buffer_basic() {
        let mut buf = OutputBuffer::new(100);
        buf.push(b"line1\nline2\n");
        assert_eq!(buf.len(), 2);
        
        let lines: Vec<_> = buf.lines().collect();
        assert_eq!(lines, vec!["line1", "line2"]);
    }

    #[test]
    fn test_output_buffer_max_lines() {
        let mut buf = OutputBuffer::new(5);
        for i in 0..10 {
            buf.push(format!("line{}\n", i).as_bytes());
        }
        assert_eq!(buf.len(), 5);
        
        let lines: Vec<_> = buf.lines().collect();
        assert_eq!(lines, vec!["line5", "line6", "line7", "line8", "line9"]);
    }

    #[test]
    fn test_output_buffer_partial_line() {
        let mut buf = OutputBuffer::new(100);
        buf.push(b"partial");
        assert_eq!(buf.len(), 0); // Not complete yet
        
        buf.push(b" line\n");
        assert_eq!(buf.len(), 1);
        
        let lines: Vec<_> = buf.lines().collect();
        assert_eq!(lines, vec!["partial line"]);
    }

    #[test]
    fn test_strip_ansi() {
        let input = "\x1b[31mred\x1b[0m normal";
        let stripped = strip_ansi_escapes(input);
        assert_eq!(stripped, "red normal");
    }

    #[test]
    fn test_last_n_lines() {
        let mut buf = OutputBuffer::new(100);
        for i in 0..10 {
            buf.push(format!("line{}\n", i).as_bytes());
        }
        
        let last3 = buf.last_n_lines(3);
        assert_eq!(last3, vec!["line7", "line8", "line9"]);
    }
}
