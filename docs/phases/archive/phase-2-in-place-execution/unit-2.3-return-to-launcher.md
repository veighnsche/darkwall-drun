# Unit 2.3: Return to Launcher

> **Phase:** 2 - In-Place Execution  
> **Complexity:** Medium  
> **Skills:** State management, TUI layout

---

## Objective

Detect command completion, display exit status, preserve output, and return to the launcher interface.

---

## Tasks

### 1. Detect Command Exit

```rust
pub enum CommandStatus {
    Running,
    Exited(i32),
    Signaled(i32),
    Unknown,
}

impl PtySession {
    pub fn poll_status(&mut self) -> CommandStatus;
}
```

### 2. Show Exit Code

- Display exit code in status bar
- Color code: green (0), red (non-zero)
- Show signal name if killed

### 3. Preserve Last N Lines

```rust
pub struct AppState {
    // ...
    preserved_output: Option<Vec<String>>,
    preserve_lines: usize, // from config
}
```

### 4. Re-render Launcher Below Output

Layout when returning:
```
┌─────────────────────────────┐
│ [Preserved Output - 5 lines]│
│ $ command                   │
│ output line 1               │
│ output line 2               │
│ [Exit: 0]                   │
├─────────────────────────────┤
│ > search query              │
│ ● App 1                     │
│   App 2                     │
│   App 3                     │
└─────────────────────────────┘
```

---

## Implementation Notes

### State Transitions

```rust
pub enum AppMode {
    Launcher,
    Executing { session: PtySession, buffer: OutputBuffer },
    PostExecution { output: Vec<String>, exit_code: i32 },
}
```

### Exit Detection Loop

```rust
async fn execution_loop(session: &mut PtySession, buffer: &mut OutputBuffer) -> i32 {
    loop {
        tokio::select! {
            result = session.read() => {
                match result {
                    Ok(data) => buffer.push(&data),
                    Err(_) => break,
                }
            }
            status = session.wait() => {
                return status.code().unwrap_or(-1);
            }
        }
    }
    -1
}
```

### Layout Calculation

```rust
fn calculate_layout(area: Rect, has_preserved: bool) -> (Option<Rect>, Rect) {
    if has_preserved {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(PRESERVED_LINES + 2), // +2 for borders
                Constraint::Min(10),
            ])
            .split(area);
        (Some(chunks[0]), chunks[1])
    } else {
        (None, area)
    }
}
```

---

## Configuration

```toml
[execution]
preserve_lines = 5  # Number of output lines to keep after return
clear_on_success = false  # Clear output if exit code is 0
```

---

## Acceptance Criteria

- [ ] Command exit is detected promptly
- [ ] Exit code displays correctly
- [ ] Preserved output shows above launcher
- [ ] Launcher is fully functional after return
- [ ] `clear_on_success` option works
- [ ] Can immediately run another command

---

## Testing

### Unit Tests

```rust
#[test]
fn test_exit_code_detection() {
    let session = PtySession::spawn("exit 42").unwrap();
    let status = session.wait().unwrap();
    assert_eq!(status.code(), Some(42));
}

#[test]
fn test_output_preservation() {
    let mut state = AppState::new();
    state.preserve_lines = 3;
    // Run command with 10 lines output
    // Verify only last 3 preserved
}
```

### Manual Tests

1. Run `echo hello && exit 0` - should show green exit status
2. Run `exit 1` - should show red exit status
3. Run `seq 100` - should preserve last N lines
4. After return, search and run another app

---

## UX Considerations

- **Immediate feedback:** Show "Command finished" message
- **Keyboard hint:** Show "Press Enter to dismiss output"
- **Auto-dismiss option:** Clear after N seconds (configurable)

---

## Related Units

- **Depends on:** Unit 2.1 (PTY), Unit 2.2 (Output Capture)
- **Blocks:** None (end of critical path)
