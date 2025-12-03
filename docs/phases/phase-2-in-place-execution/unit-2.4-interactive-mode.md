# Unit 2.4: Interactive Mode Detection

> **Phase:** 2 - In-Place Execution  
> **Complexity:** High  
> **Skills:** Terminal emulation, process control

---

## Objective

Detect when a command needs full terminal control (TUI apps like btop, htop) and hand over the terminal appropriately.

---

## Tasks

### 1. Detect Interactive Commands

```rust
pub enum TerminalMode {
    Oneshot,      // Simple command, capture output
    Interactive,  // Needs input (bash, python REPL)
    Tui,          // Full screen TUI (btop, htop, vim)
    LongRunning,  // Server, watch command
}

pub fn detect_terminal_mode(cmd: &str, entry: &DesktopEntry) -> TerminalMode;
```

### 2. Hand Over Terminal Control

For TUI apps:
- Suspend ratatui rendering
- Give child process raw terminal
- Disable our input handling

### 3. Reclaim After Exit

- Detect child exit
- Restore terminal state
- Resume ratatui rendering

---

## Implementation Notes

### Detection Heuristics

```rust
fn detect_terminal_mode(cmd: &str, entry: Option<&DesktopEntry>) -> TerminalMode {
    // 1. Check custom field first
    if let Some(mode) = entry.and_then(|e| e.get("X-DarkwallTerminalMode")) {
        return parse_terminal_mode(mode);
    }
    
    // 2. Known TUI apps
    let tui_apps = ["htop", "btop", "vim", "nvim", "nano", "less", "man"];
    if tui_apps.iter().any(|app| cmd.starts_with(app)) {
        return TerminalMode::Tui;
    }
    
    // 3. Known interactive commands
    let interactive = ["bash", "zsh", "fish", "python", "node"];
    if interactive.iter().any(|app| cmd.starts_with(app)) {
        return TerminalMode::Interactive;
    }
    
    // 4. Long-running patterns
    if cmd.contains("watch ") || cmd.contains("tail -f") {
        return TerminalMode::LongRunning;
    }
    
    TerminalMode::Oneshot
}
```

### Terminal Handover

```rust
impl App {
    fn enter_tui_mode(&mut self) -> Result<()> {
        // 1. Disable ratatui
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;
        
        // 2. Spawn command directly (not in PTY)
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(&self.current_command)
            .status()?;
        
        // 3. Restore ratatui
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
        
        self.last_exit_code = status.code();
        Ok(())
    }
}
```

### Alternative: PTY Passthrough

For more control, use PTY but pass through all I/O:

```rust
async fn tui_passthrough(session: &mut PtySession) -> Result<i32> {
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    
    loop {
        tokio::select! {
            // Forward stdin to PTY
            result = stdin.read(&mut buf) => {
                session.write(&buf[..result?])?;
            }
            // Forward PTY to stdout
            result = session.read(&mut buf) => {
                stdout.write_all(&buf[..result?]).await?;
            }
            // Check for exit
            status = session.poll_exit() => {
                return Ok(status.code().unwrap_or(-1));
            }
        }
    }
}
```

---

## Mode Behaviors

| Mode | PTY | Capture Output | Our UI Visible |
|------|-----|----------------|----------------|
| Oneshot | Yes | Yes | Yes (split) |
| Interactive | Yes | Partial | Yes (split) |
| Tui | No* | No | No |
| LongRunning | Yes | Yes | Yes (split) |

*TUI mode uses direct exec or full passthrough

---

## Acceptance Criteria

- [ ] `htop` runs full-screen, exits cleanly
- [ ] `vim` works with all keybindings
- [ ] `bash` allows interactive input
- [ ] Terminal state restored after TUI exit
- [ ] No visual glitches on mode transitions

---

## Testing

### Unit Tests

```rust
#[test]
fn test_detect_htop() {
    assert_eq!(detect_terminal_mode("htop", None), TerminalMode::Tui);
}

#[test]
fn test_detect_ls() {
    assert_eq!(detect_terminal_mode("ls -la", None), TerminalMode::Oneshot);
}

#[test]
fn test_custom_field_override() {
    let entry = DesktopEntry::with_field("X-DarkwallTerminalMode", "tui");
    assert_eq!(detect_terminal_mode("myapp", Some(&entry)), TerminalMode::Tui);
}
```

### Manual Tests

1. Run `htop` - should take over screen, Ctrl+C exits
2. Run `vim test.txt` - full vim functionality
3. Run `python` - interactive REPL works
4. After each, launcher should restore cleanly

---

## Edge Cases

| Case | Handling |
|------|----------|
| TUI app crashes | Restore terminal anyway |
| SIGWINCH during TUI | Forward to child |
| Ctrl+C in TUI | Forward to child, don't exit launcher |
| TUI spawns subprocess | Should work (PTY inherited) |

---

## Related Units

- **Depends on:** Unit 2.1 (PTY Allocation)
- **Parallel with:** Unit 2.3 (Return to Launcher)
