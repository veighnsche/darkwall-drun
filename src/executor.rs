//! Command execution with terminal mode detection.
//!
//! TEAM_000: Phase 2, Units 2.2-2.4

use crate::desktop_entry::Entry;
use crate::pty::ExitStatus;

/// Terminal mode determines how a command should be executed
/// TEAM_000: Phase 4, Unit 4.1 - Terminal Mode Schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalMode {
    /// GUI application - launch detached, no terminal needed
    Gui,
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
            "gui" => Ok(TerminalMode::Gui),
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
    /// 
    /// Detection priority:
    /// 1. X-DarkwallTerminalMode custom field (explicit override)
    /// 2. Terminal=false in desktop entry → Gui (launch detached)
    /// 3. Known TUI apps → Tui
    /// 4. Known interactive apps → Interactive
    /// 5. Long-running patterns → LongRunning
    /// 6. Terminal=true fallback → Interactive
    /// 7. Default (raw command, no entry) → Oneshot
    pub fn detect(cmd: &str, entry: Option<&Entry>) -> Self {
        // 1. Check custom X-DarkwallTerminalMode field first (explicit override)
        if let Some(entry) = entry {
            if let Some(mode) = entry.get_darkwall_field("TerminalMode") {
                if let Ok(m) = mode.parse() {
                    return m;
                }
            }
        }
        
        // 2. GUI detection: Terminal=false means it's a GUI app
        // This is the DETERMINISTIC way to distinguish CLI from GUI
        if let Some(entry) = entry {
            if !entry.terminal {
                return TerminalMode::Gui;
            }
        }
        
        // From here on, we know it's a terminal app (Terminal=true or raw command)
        
        // 3. Extract the base command (first word)
        let base_cmd = cmd
            .split_whitespace()
            .next()
            .unwrap_or("")
            .rsplit('/')
            .next()
            .unwrap_or("");

        // 4. Known TUI apps (full screen)
        const TUI_APPS: &[&str] = &[
            "htop", "btop", "top", "vim", "nvim", "neovim", "nano", "less", "man",
            "mc", "ranger", "nnn", "lf", "vifm", "tmux", "screen",
            "mutt", "neomutt", "weechat", "irssi", "cmus", "ncmpcpp",
            "lazygit", "tig", "gitui", "k9s", "dive", "helix", "hx",
            // Nested TUI launchers - need full terminal handover
            "drun", "rofi", "dmenu", "wofi", "fuzzel", "tofi",
        ];
        // Check exact match OR if base_cmd ends with the app name (for Nix wrapper scripts like "desktop-btop")
        if TUI_APPS.iter().any(|&app| base_cmd == app || base_cmd.ends_with(&format!("-{}", app))) {
            return TerminalMode::Tui;
        }

        // 5. Known interactive commands (REPL-style)
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

        // 6. Long-running patterns
        if cmd.contains("watch ") || cmd.contains("tail -f") || cmd.contains("journalctl -f") {
            return TerminalMode::LongRunning;
        }

        // 7. If we have an entry with Terminal=true but didn't match above, default to Interactive
        if let Some(entry) = entry {
            if entry.terminal {
                return TerminalMode::Interactive;
            }
        }

        // 8. Default for raw commands (no entry) → Oneshot
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
    use std::collections::HashMap;
    use std::path::PathBuf;

    /// Create a mock Entry for testing
    fn mock_entry(terminal: bool) -> Entry {
        Entry {
            id: "test".to_string(),
            name: "Test".to_string(),
            generic_name: None,
            comment: None,
            exec: Some("test-cmd".to_string()),
            icon: None,
            categories: vec![],
            keywords: vec![],
            terminal,
            no_display: false,
            path: PathBuf::from("/test.desktop"),
            custom_fields: HashMap::new(),
        }
    }

    fn mock_entry_with_mode(terminal: bool, mode: &str) -> Entry {
        let mut entry = mock_entry(terminal);
        entry.custom_fields.insert("TerminalMode".to_string(), mode.to_string());
        entry
    }

    #[test]
    fn test_terminal_mode_detection_raw_commands() {
        // Raw commands (no entry) default to Oneshot
        assert_eq!(TerminalMode::detect("ls -la", None), TerminalMode::Oneshot);
        assert_eq!(TerminalMode::detect("echo hello", None), TerminalMode::Oneshot);
        
        // Known TUI apps
        assert_eq!(TerminalMode::detect("htop", None), TerminalMode::Tui);
        assert_eq!(TerminalMode::detect("vim test.txt", None), TerminalMode::Tui);
        
        // Known interactive apps
        assert_eq!(TerminalMode::detect("python", None), TerminalMode::Interactive);
        assert_eq!(TerminalMode::detect("bash", None), TerminalMode::Interactive);
        
        // Long-running patterns
        assert_eq!(TerminalMode::detect("watch ls", None), TerminalMode::LongRunning);
        
        // Nix store paths with wrapper scripts
        assert_eq!(TerminalMode::detect("/nix/store/abc123-desktop-btop", None), TerminalMode::Tui);
        assert_eq!(TerminalMode::detect("/nix/store/xyz789-desktop-neovim", None), TerminalMode::Tui);
        assert_eq!(TerminalMode::detect("/nix/store/def456-desktop-nvim", None), TerminalMode::Tui);
    }

    #[test]
    fn test_gui_detection_from_desktop_entry() {
        // Terminal=false means GUI app
        let gui_entry = mock_entry(false);
        assert_eq!(TerminalMode::detect("firefox", Some(&gui_entry)), TerminalMode::Gui);
        assert_eq!(TerminalMode::detect("code", Some(&gui_entry)), TerminalMode::Gui);
        assert_eq!(TerminalMode::detect("gimp", Some(&gui_entry)), TerminalMode::Gui);
        
        // Terminal=true means CLI app (falls through to other detection)
        let cli_entry = mock_entry(true);
        assert_eq!(TerminalMode::detect("htop", Some(&cli_entry)), TerminalMode::Tui);
        assert_eq!(TerminalMode::detect("python", Some(&cli_entry)), TerminalMode::Interactive);
        // Unknown CLI command with Terminal=true defaults to Interactive
        assert_eq!(TerminalMode::detect("my-custom-cli", Some(&cli_entry)), TerminalMode::Interactive);
    }

    #[test]
    fn test_explicit_mode_override() {
        // X-DarkwallTerminalMode overrides everything
        let gui_entry_forced_tui = mock_entry_with_mode(false, "tui");
        assert_eq!(TerminalMode::detect("firefox", Some(&gui_entry_forced_tui)), TerminalMode::Tui);
        
        let cli_entry_forced_gui = mock_entry_with_mode(true, "gui");
        assert_eq!(TerminalMode::detect("htop", Some(&cli_entry_forced_gui)), TerminalMode::Gui);
        
        let entry_forced_oneshot = mock_entry_with_mode(true, "oneshot");
        assert_eq!(TerminalMode::detect("python", Some(&entry_forced_oneshot)), TerminalMode::Oneshot);
    }

    #[test]
    fn test_terminal_mode_from_str() {
        assert_eq!("gui".parse::<TerminalMode>().unwrap(), TerminalMode::Gui);
        assert_eq!("oneshot".parse::<TerminalMode>().unwrap(), TerminalMode::Oneshot);
        assert_eq!("interactive".parse::<TerminalMode>().unwrap(), TerminalMode::Interactive);
        assert_eq!("tui".parse::<TerminalMode>().unwrap(), TerminalMode::Tui);
        assert_eq!("long-running".parse::<TerminalMode>().unwrap(), TerminalMode::LongRunning);
        assert_eq!("longrunning".parse::<TerminalMode>().unwrap(), TerminalMode::LongRunning);
        assert!("invalid".parse::<TerminalMode>().is_err());
    }
}
