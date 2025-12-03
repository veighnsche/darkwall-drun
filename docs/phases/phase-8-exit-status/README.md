# Phase 8: TUI Exit Status Reporting

## Overview

Properly capture and display exit status from executed commands, including TUI apps that use terminal handover.

## Current State

- `CommandStatus::from_exit_status()` handles PTY exits (portable_pty)
- `CommandStatus::from_std_exit_status()` exists but is unused (for TUI handover)
- `CommandStatus::is_success()` exists but is unused

## Problem

When a TUI app (vim, htop) exits via `execute_tui()`, we get `std::process::ExitStatus` but don't convert it to `CommandStatus` or display it.

## Implementation

### 1. Capture TUI Exit Status

```rust
// In app.rs
pub fn execute_tui(&mut self, cmd: &str) -> Result<Option<CommandStatus>> {
    // ... existing terminal handover code ...
    
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()?;
    
    // ... restore terminal ...
    
    // Convert to CommandStatus
    let cmd_status = CommandStatus::from_std_exit_status(status);
    
    // Optionally transition to PostExecution to show status
    if !cmd_status.is_success() {
        self.mode = AppMode::PostExecution {
            command: cmd.to_string(),
            exit_status: cmd_status.clone(),
            preserved_output: vec!["(TUI app output not captured)".to_string()],
        };
    }
    
    Ok(Some(cmd_status))
}
```

### 2. Display Exit Status

The UI already handles `CommandStatus` in `draw_post_execution()`:

```rust
let (exit_text, exit_color) = match exit_status {
    CommandStatus::Exited(0) => ("Exit: 0".to_string(), Color::Green),
    CommandStatus::Exited(code) => (format!("Exit: {}", code), Color::Red),
    CommandStatus::Signaled(sig) => (format!("Signal: {}", sig), Color::Red),
    CommandStatus::Running => ("Running".to_string(), Color::Yellow),
    CommandStatus::Unknown => ("Unknown".to_string(), Color::Gray),
};
```

### 3. Signal Names (Enhancement)

Convert signal numbers to names for better UX:

```rust
fn signal_name(sig: i32) -> &'static str {
    match sig {
        1 => "SIGHUP",
        2 => "SIGINT",
        3 => "SIGQUIT",
        6 => "SIGABRT",
        9 => "SIGKILL",
        11 => "SIGSEGV",
        13 => "SIGPIPE",
        14 => "SIGALRM",
        15 => "SIGTERM",
        _ => "Unknown signal",
    }
}

// Usage in UI
CommandStatus::Signaled(sig) => {
    (format!("{} ({})", signal_name(sig), sig), Color::Red)
}
```

### 4. Notification Integration (Future)

Send desktop notification on command completion:

```rust
fn notify_completion(cmd: &str, status: &CommandStatus) {
    let (summary, body) = match status {
        CommandStatus::Exited(0) => ("Command completed", cmd.to_string()),
        CommandStatus::Exited(code) => ("Command failed", format!("{} (exit {})", cmd, code)),
        CommandStatus::Signaled(sig) => ("Command killed", format!("{} ({})", cmd, signal_name(sig))),
        _ => return,
    };
    
    // Use notify-rust or dbus directly
    notify_rust::Notification::new()
        .summary(summary)
        .body(&body)
        .show()
        .ok();
}
```

## Configuration

```toml
[behavior]
# Show exit status for TUI apps
show_tui_exit_status = true

# Send notification on command completion
notify_on_completion = false
notify_on_failure_only = true
```

## Implementation Checklist

- [ ] Update `execute_tui()` to return `CommandStatus`
- [ ] Show exit status after TUI app exits (if non-zero)
- [ ] Add signal name lookup
- [ ] Add notification support (optional)
- [ ] Add configuration options
- [ ] Test with various exit scenarios (Ctrl+C, error, success)

## Testing

```bash
# Test successful exit
# Run vim, :q

# Test error exit
# Create .desktop with: Exec=sh -c "exit 42"

# Test signal
# Run sleep 100, then Ctrl+C from another terminal
```
