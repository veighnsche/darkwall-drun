# Unit 4.1: Terminal Mode Schema

> **Phase:** 4 - Advanced Metadata  
> **Complexity:** Medium  
> **Skills:** Schema design, pattern matching

---

## Objective

Define and implement a terminal mode classification system that determines how commands should be executed.

---

## Tasks

### 1. Define `TerminalMode` Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TerminalMode {
    /// Simple command that exits quickly (ls, echo, cat)
    Oneshot,
    
    /// Needs user input (bash, python REPL)
    Interactive,
    
    /// Full-screen TUI application (htop, vim, btop)
    Tui,
    
    /// Long-running process (servers, watch commands)
    LongRunning,
}
```

### 2. Parse from Desktop Entry

```rust
impl DesktopEntry {
    pub fn terminal_mode(&self) -> TerminalMode {
        // 1. Check explicit field
        if let Some(mode) = self.get("X-DarkwallTerminalMode") {
            if let Ok(m) = mode.parse() {
                return m;
            }
        }
        
        // 2. Infer from command
        self.infer_terminal_mode()
    }
}
```

### 3. Inference from Command Patterns

```rust
impl DesktopEntry {
    fn infer_terminal_mode(&self) -> TerminalMode {
        let cmd = self.exec.to_lowercase();
        let binary = cmd.split_whitespace().next().unwrap_or("");
        
        // TUI apps
        if TUI_APPS.contains(&binary) {
            return TerminalMode::Tui;
        }
        
        // Interactive shells/REPLs
        if INTERACTIVE_APPS.contains(&binary) {
            return TerminalMode::Interactive;
        }
        
        // Long-running patterns
        if is_long_running(&cmd) {
            return TerminalMode::LongRunning;
        }
        
        TerminalMode::Oneshot
    }
}
```

---

## Implementation Notes

### Known Application Lists

```rust
const TUI_APPS: &[&str] = &[
    "htop", "btop", "top", "atop",
    "vim", "nvim", "vi", "nano", "emacs",
    "less", "more", "man",
    "mc", "ranger", "nnn", "lf",
    "tmux", "screen",
    "ncdu", "duf",
    "lazygit", "tig",
];

const INTERACTIVE_APPS: &[&str] = &[
    "bash", "zsh", "fish", "sh", "dash",
    "python", "python3", "ipython",
    "node", "deno", "bun",
    "ruby", "irb",
    "ghci", "lua", "perl",
    "sqlite3", "psql", "mysql",
];

fn is_long_running(cmd: &str) -> bool {
    cmd.contains("watch ") ||
    cmd.contains("tail -f") ||
    cmd.contains("journalctl -f") ||
    cmd.starts_with("serve") ||
    cmd.contains("--server") ||
    cmd.contains("-d") && cmd.contains("daemon")
}
```

### Mode Behavior Matrix

| Mode | PTY | Capture | Our UI | Unfloat |
|------|-----|---------|--------|---------|
| Oneshot | Yes | Yes | Split | No |
| Interactive | Yes | Partial | Split | Yes |
| Tui | Passthrough | No | Hidden | Yes |
| LongRunning | Yes | Yes | Split | Yes |

---

## Acceptance Criteria

- [ ] `TerminalMode` enum defined with all variants
- [ ] Parsing from `X-DarkwallTerminalMode` works
- [ ] Inference correctly identifies common apps
- [ ] Unknown commands default to `Oneshot`
- [ ] Mode affects execution behavior

---

## Testing

### Unit Tests

```rust
#[test]
fn test_parse_terminal_mode() {
    assert_eq!("oneshot".parse::<TerminalMode>().unwrap(), TerminalMode::Oneshot);
    assert_eq!("tui".parse::<TerminalMode>().unwrap(), TerminalMode::Tui);
}

#[test]
fn test_infer_htop() {
    let entry = DesktopEntry::new("htop");
    assert_eq!(entry.terminal_mode(), TerminalMode::Tui);
}

#[test]
fn test_infer_python() {
    let entry = DesktopEntry::new("python3");
    assert_eq!(entry.terminal_mode(), TerminalMode::Interactive);
}

#[test]
fn test_infer_ls() {
    let entry = DesktopEntry::new("ls -la");
    assert_eq!(entry.terminal_mode(), TerminalMode::Oneshot);
}

#[test]
fn test_explicit_override() {
    let mut entry = DesktopEntry::new("myapp");
    entry.set("X-DarkwallTerminalMode", "tui");
    assert_eq!(entry.terminal_mode(), TerminalMode::Tui);
}
```

### Manual Tests

1. Run `htop` - should detect as TUI
2. Run `python` - should detect as Interactive
3. Run `ls` - should detect as Oneshot
4. Create custom .desktop with explicit mode

---

## Edge Cases

| Case | Handling |
|------|----------|
| `vim file.txt` | TUI (vim detected) |
| `python script.py` | Oneshot (has argument) |
| `python -i script.py` | Interactive (-i flag) |
| `bash -c "ls"` | Oneshot (non-interactive bash) |
| Unknown binary | Default to Oneshot |

---

## Related Units

- **Depends on:** None
- **Blocks:** Unit 4.3 (Output Preservation), Unit 4.4 (Custom Fields)
- **Related:** Unit 2.4 (Interactive Mode Detection)
