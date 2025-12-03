# Unit 3.2: Window State Management

> **Phase:** 3 - Niri IPC Integration  
> **Complexity:** Low  
> **Skills:** State machines, IPC

---

## Objective

Manage window floating state based on application mode (launcher vs executing).

---

## Tasks

### 1. SetWindowFloating Commands

```rust
impl NiriClient {
    pub async fn set_floating(&mut self, floating: bool) -> Result<()> {
        let request = if floating {
            r#"{"Action":{"SetWindowFloating":{"id":null,"floating":true}}}"#
        } else {
            r#"{"Action":{"SetWindowFloating":{"id":null,"floating":false}}}"#
        };
        self.send_request(request).await?;
        Ok(())
    }
}
```

### 2. Query Current Window State

```rust
impl NiriClient {
    pub async fn get_focused_window(&mut self) -> Result<Option<WindowInfo>> {
        let request = r#"{"Request":"FocusedWindow"}"#;
        let response = self.send_request(request).await?;
        // Parse window info
        Ok(...)
    }
}
```

### 3. State Transitions

```
┌─────────────┐     execute      ┌─────────────┐
│   Floating  │ ───────────────► │  Unfloated  │
│  (Launcher) │                  │ (Executing) │
└─────────────┘ ◄─────────────── └─────────────┘
                   exit/return
```

---

## Implementation Notes

### Integration with App State

```rust
impl App {
    async fn execute_entry(&mut self, entry: &DesktopEntry) -> Result<()> {
        // 1. Unfloat window
        if self.niri.is_available() {
            self.niri.set_floating(false).await?;
        }
        
        // 2. Execute command
        self.run_command(&entry.exec).await?;
        
        // 3. Re-float window
        if self.niri.is_available() {
            self.niri.set_floating(true).await?;
        }
        
        Ok(())
    }
}
```

### Conditional Unfloating

Some commands should stay floating (quick lookups):

```rust
fn should_unfloat(entry: &DesktopEntry, mode: TerminalMode) -> bool {
    // Check custom field
    if let Some(unfloat) = entry.get("X-DarkwallUnfloatOnRun") {
        return unfloat == "true";
    }
    
    // Default based on mode
    match mode {
        TerminalMode::Oneshot => false,  // Quick commands stay floating
        TerminalMode::Interactive => true,
        TerminalMode::Tui => true,
        TerminalMode::LongRunning => true,
    }
}
```

---

## Niri IPC Commands Reference

### SetWindowFloating

```json
{"Action":{"SetWindowFloating":{"id":null,"floating":true}}}
```

- `id: null` = focused window
- `id: <window_id>` = specific window

### FocusedWindow

```json
{"Request":"FocusedWindow"}
```

Response:
```json
{"ok":{"id":123,"title":"darkwall-drun","app_id":"darkwall-drun"}}
```

---

## Acceptance Criteria

- [ ] Window unfloats when command starts
- [ ] Window re-floats when command ends
- [ ] Quick commands (oneshot) stay floating
- [ ] Custom `X-DarkwallUnfloatOnRun` respected
- [ ] Works without niri (no-op)

---

## Testing

### Unit Tests

```rust
#[test]
fn test_should_unfloat_oneshot() {
    let entry = DesktopEntry::new("ls");
    assert!(!should_unfloat(&entry, TerminalMode::Oneshot));
}

#[test]
fn test_should_unfloat_tui() {
    let entry = DesktopEntry::new("htop");
    assert!(should_unfloat(&entry, TerminalMode::Tui));
}

#[test]
fn test_custom_unfloat_override() {
    let mut entry = DesktopEntry::new("myapp");
    entry.set("X-DarkwallUnfloatOnRun", "false");
    assert!(!should_unfloat(&entry, TerminalMode::Tui));
}
```

### Manual Tests

1. Run `ls` - window should stay floating
2. Run `htop` - window should unfloat, then re-float on exit
3. Test with niri window rules

---

## Related Units

- **Depends on:** Unit 3.1 (IPC Protocol)
- **Related:** Unit 2.4 (Interactive Mode Detection)
