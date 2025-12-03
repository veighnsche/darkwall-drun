# Unit 4.2: SSH Detection

> **Phase:** 4 - Advanced Metadata  
> **Complexity:** Low  
> **Skills:** String parsing, UI design

---

## Objective

Detect SSH commands and provide appropriate UI feedback (connection spinner, host display).

---

## Tasks

### 1. Parse Command for SSH

```rust
pub struct SshInfo {
    pub user: Option<String>,
    pub host: String,
    pub port: Option<u16>,
    pub command: Option<String>,
}

pub fn parse_ssh_command(cmd: &str) -> Option<SshInfo>;
```

### 2. Show Connection Spinner

```rust
fn render_ssh_connecting(f: &mut Frame, area: Rect, info: &SshInfo) {
    let text = format!("Connecting to {}...", info.host);
    let spinner = Spinner::new()
        .label(&text)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(spinner, area);
}
```

### 3. Handle Connection Failures

- Timeout detection
- Error message display
- Retry option

---

## Implementation Notes

### SSH Command Parsing

```rust
pub fn parse_ssh_command(cmd: &str) -> Option<SshInfo> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    
    // Must start with ssh
    if parts.first() != Some(&"ssh") {
        return None;
    }
    
    let mut info = SshInfo {
        user: None,
        host: String::new(),
        port: None,
        command: None,
    };
    
    let mut i = 1;
    while i < parts.len() {
        match parts[i] {
            "-p" => {
                i += 1;
                info.port = parts.get(i).and_then(|p| p.parse().ok());
            }
            "-l" => {
                i += 1;
                info.user = parts.get(i).map(|s| s.to_string());
            }
            arg if !arg.starts_with('-') => {
                // user@host or just host
                if arg.contains('@') {
                    let parts: Vec<&str> = arg.splitn(2, '@').collect();
                    info.user = Some(parts[0].to_string());
                    info.host = parts[1].to_string();
                } else if info.host.is_empty() {
                    info.host = arg.to_string();
                } else {
                    // Remote command
                    info.command = Some(parts[i..].join(" "));
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    if info.host.is_empty() {
        None
    } else {
        Some(info)
    }
}
```

### UI States

```rust
pub enum SshState {
    Connecting { start_time: Instant },
    Connected,
    Failed { error: String },
    TimedOut,
}
```

### Spinner Widget

```rust
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn render_spinner(f: &mut Frame, area: Rect, frame: usize, label: &str) {
    let spinner_char = SPINNER_FRAMES[frame % SPINNER_FRAMES.len()];
    let text = format!("{} {}", spinner_char, label);
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(paragraph, area);
}
```

---

## Acceptance Criteria

- [ ] SSH commands detected correctly
- [ ] User, host, port parsed from various formats
- [ ] Spinner shows during connection
- [ ] Host name displayed prominently
- [ ] Connection failures show error message

---

## Testing

### Unit Tests

```rust
#[test]
fn test_parse_simple_ssh() {
    let info = parse_ssh_command("ssh example.com").unwrap();
    assert_eq!(info.host, "example.com");
    assert!(info.user.is_none());
}

#[test]
fn test_parse_ssh_with_user() {
    let info = parse_ssh_command("ssh user@example.com").unwrap();
    assert_eq!(info.host, "example.com");
    assert_eq!(info.user, Some("user".to_string()));
}

#[test]
fn test_parse_ssh_with_port() {
    let info = parse_ssh_command("ssh -p 2222 example.com").unwrap();
    assert_eq!(info.port, Some(2222));
}

#[test]
fn test_parse_ssh_with_command() {
    let info = parse_ssh_command("ssh example.com ls -la").unwrap();
    assert_eq!(info.command, Some("ls -la".to_string()));
}

#[test]
fn test_not_ssh() {
    assert!(parse_ssh_command("ls -la").is_none());
}
```

### Manual Tests

1. Run `ssh localhost` - should show spinner
2. Run `ssh user@host` - should show "Connecting to host as user..."
3. Run `ssh nonexistent.host` - should show connection error
4. Run `ssh host command` - should execute and return

---

## SSH Command Formats

| Format | Parsed As |
|--------|-----------|
| `ssh host` | host only |
| `ssh user@host` | user + host |
| `ssh -p 22 host` | host + port |
| `ssh -l user host` | user + host |
| `ssh host cmd` | host + remote command |
| `ssh -t host htop` | host + command (TUI) |

---

## Related Units

- **Depends on:** Unit 4.1 (Terminal Mode - SSH is Interactive/TUI)
- **Related:** Unit 2.2 (Output Capture for SSH output)
