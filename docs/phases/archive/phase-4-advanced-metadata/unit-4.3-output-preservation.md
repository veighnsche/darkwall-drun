# Unit 4.3: Output Preservation Logic

> **Phase:** 4 - Advanced Metadata  
> **Complexity:** Medium  
> **Skills:** State management, configuration

---

## Objective

Implement intelligent output preservation based on terminal mode and per-entry settings.

---

## Tasks

### 1. `keepOutput` Per-Entry Setting

```rust
impl DesktopEntry {
    pub fn keep_output(&self) -> bool {
        // 1. Check explicit field
        if let Some(keep) = self.get("X-DarkwallKeepOutput") {
            return keep == "true";
        }
        
        // 2. Default based on terminal mode
        match self.terminal_mode() {
            TerminalMode::Oneshot => true,
            TerminalMode::Interactive => true,
            TerminalMode::Tui => false,
            TerminalMode::LongRunning => true,
        }
    }
}
```

### 2. Clear Screen for TUI Apps

```rust
impl App {
    fn post_execution(&mut self, entry: &DesktopEntry, exit_code: i32) {
        if entry.keep_output() {
            self.preserved_output = Some(self.output_buffer.last_n_lines(
                self.config.preserve_lines
            ));
        } else {
            self.preserved_output = None;
            // Clear any residual TUI artifacts
            self.clear_screen();
        }
    }
}
```

### 3. Preserve for Oneshot Commands

- Keep last N lines (configurable)
- Show exit code
- Allow scrolling through preserved output

---

## Implementation Notes

### Configuration

```toml
[execution]
# Default number of lines to preserve
preserve_lines = 10

# Clear output if exit code is 0
clear_on_success = false

# Always clear for these terminal modes
always_clear_modes = ["tui"]
```

### Preservation Logic

```rust
fn should_preserve_output(
    entry: &DesktopEntry,
    exit_code: i32,
    config: &Config,
) -> bool {
    // 1. Check explicit setting
    if let Some(keep) = entry.get_bool("X-DarkwallKeepOutput") {
        return keep;
    }
    
    // 2. Check clear_on_success
    if config.execution.clear_on_success && exit_code == 0 {
        return false;
    }
    
    // 3. Check always_clear_modes
    let mode = entry.terminal_mode();
    if config.execution.always_clear_modes.contains(&mode) {
        return false;
    }
    
    // 4. Default: preserve
    true
}
```

### Output Buffer Management

```rust
impl OutputBuffer {
    pub fn last_n_lines(&self, n: usize) -> Vec<String> {
        self.lines
            .iter()
            .rev()
            .take(n)
            .rev()
            .cloned()
            .collect()
    }
    
    pub fn with_exit_status(&self, n: usize, exit_code: i32) -> Vec<String> {
        let mut lines = self.last_n_lines(n);
        lines.push(format!("[Exit: {}]", exit_code));
        lines
    }
}
```

### UI Layout with Preserved Output

```
┌─────────────────────────────────┐
│ Previous: ls -la                │
│ drwxr-xr-x  5 user user 4096 ...│
│ -rw-r--r--  1 user user  123 ...│
│ [Exit: 0]                       │
├─────────────────────────────────┤
│ > _                             │
│ ● Firefox                       │
│   Terminal                      │
│   Files                         │
└─────────────────────────────────┘
```

---

## Acceptance Criteria

- [ ] `X-DarkwallKeepOutput` field respected
- [ ] TUI apps clear screen on exit
- [ ] Oneshot commands preserve output
- [ ] `preserve_lines` config works
- [ ] `clear_on_success` option works
- [ ] Exit code shown in preserved output

---

## Testing

### Unit Tests

```rust
#[test]
fn test_keep_output_default_oneshot() {
    let entry = DesktopEntry::new("ls");
    assert!(entry.keep_output());
}

#[test]
fn test_keep_output_default_tui() {
    let entry = DesktopEntry::new("htop");
    assert!(!entry.keep_output());
}

#[test]
fn test_keep_output_explicit_true() {
    let mut entry = DesktopEntry::new("htop");
    entry.set("X-DarkwallKeepOutput", "true");
    assert!(entry.keep_output());
}

#[test]
fn test_preserve_lines() {
    let mut buffer = OutputBuffer::new(100);
    for i in 0..20 {
        buffer.push(format!("line {}\n", i).as_bytes());
    }
    let preserved = buffer.last_n_lines(5);
    assert_eq!(preserved.len(), 5);
    assert!(preserved[0].contains("line 15"));
}
```

### Manual Tests

1. Run `ls` - output should be preserved
2. Run `htop`, exit - screen should be clean
3. Run `exit 1` - should show red exit code
4. Configure `preserve_lines = 3`, run `seq 10` - only 3 lines kept

---

## Edge Cases

| Case | Handling |
|------|----------|
| Empty output | Show only exit code |
| Very long lines | Truncate or wrap |
| Binary output | Filter non-printable |
| Command killed (SIGTERM) | Show signal info |

---

## Related Units

- **Depends on:** Unit 4.1 (Terminal Mode Schema)
- **Related:** Unit 2.3 (Return to Launcher)
