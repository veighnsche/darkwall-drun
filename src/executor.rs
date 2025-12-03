//! Command execution with terminal mode detection.
//!
//! TEAM_000: Phase 2, Units 2.2-2.4

use crate::desktop_entry::Entry;
use crate::pty::ExitStatus;

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
            "htop", "btop", "top", "vim", "nvim", "neovim", "nano", "less", "man",
            "mc", "ranger", "nnn", "lf", "vifm", "tmux", "screen",
            "mutt", "neomutt", "weechat", "irssi", "cmus", "ncmpcpp",
            "lazygit", "tig", "gitui", "k9s", "dive", "helix", "hx",
        ];
        // Check exact match OR if base_cmd ends with the app name (for Nix wrapper scripts like "desktop-btop")
        if TUI_APPS.iter().any(|&app| base_cmd == app || base_cmd.ends_with(&format!("-{}", app))) {
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
        
        // Nix store paths with wrapper scripts
        assert_eq!(TerminalMode::detect("/nix/store/abc123-desktop-btop", None), TerminalMode::Tui);
        assert_eq!(TerminalMode::detect("/nix/store/xyz789-desktop-neovim", None), TerminalMode::Tui);
        assert_eq!(TerminalMode::detect("/nix/store/def456-desktop-nvim", None), TerminalMode::Tui);
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
}
