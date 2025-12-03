# Testing Strategy

> Testing approach for darkwall-drun.

---

## Test Categories

### Unit Tests

Located in `src/*.rs` as `#[cfg(test)]` modules.

| Module | Coverage Focus |
|--------|----------------|
| `desktop_entry.rs` | Parsing various .desktop formats |
| `config.rs` | Config loading, defaults, overrides |
| `niri.rs` | JSON-RPC parsing, error handling |
| `history.rs` | Frecency calculation, persistence |

### Integration Tests

Located in `tests/` directory.

| Test File | Coverage Focus |
|-----------|----------------|
| `desktop_entry_test.rs` | End-to-end entry loading |
| `niri_test.rs` | IPC with mock socket |
| `execution_test.rs` | Command execution flow |

### Manual Tests

Required for TUI and compositor integration.

---

## Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test desktop_entry

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test '*'
```

---

## Test Fixtures

Located in `tests/fixtures/`:

```
tests/fixtures/
├── valid.desktop           # Standard desktop entry
├── minimal.desktop         # Minimum required fields
├── with_custom_fields.desktop  # X-Darkwall fields
├── invalid_encoding.desktop    # Non-UTF8
├── missing_name.desktop        # Invalid (no Name)
└── complex_exec.desktop        # Field codes, quoting
```

### Example Fixture

```ini
# tests/fixtures/valid.desktop
[Desktop Entry]
Name=Test Application
GenericName=Test
Comment=A test application
Exec=test-app %f
Icon=test-icon
Type=Application
Categories=Utility;Development;
Keywords=test;example;
Terminal=false
```

---

## Unit Test Examples

### Desktop Entry Parsing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_entry() {
        let content = include_str!("../tests/fixtures/valid.desktop");
        let entry = DesktopEntry::parse(content).unwrap();
        
        assert_eq!(entry.name, "Test Application");
        assert_eq!(entry.exec, "test-app %f");
        assert!(entry.categories.contains(&"Utility".to_string()));
    }

    #[test]
    fn test_parse_missing_name() {
        let content = include_str!("../tests/fixtures/missing_name.desktop");
        let result = DesktopEntry::parse(content);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_exec_field_expansion() {
        let entry = DesktopEntry::new("test %f");
        let expanded = entry.expand_exec(Some("/path/to/file"));
        
        assert_eq!(expanded, "test /path/to/file");
    }
}
```

### Fuzzy Matching

```rust
#[test]
fn test_fuzzy_match_exact() {
    let entries = vec![
        DesktopEntry::new("Firefox"),
        DesktopEntry::new("Files"),
    ];
    
    let results = fuzzy_filter(&entries, "firefox");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Firefox");
}

#[test]
fn test_fuzzy_match_partial() {
    let entries = vec![
        DesktopEntry::new("Firefox"),
        DesktopEntry::new("Firewall"),
    ];
    
    let results = fuzzy_filter(&entries, "fire");
    assert_eq!(results.len(), 2);
}
```

### Configuration

```rust
#[test]
fn test_config_defaults() {
    let config = Config::default();
    
    assert!(config.execution.preserve_lines > 0);
    assert!(!config.ui.icons);
}

#[test]
fn test_config_override() {
    let toml = r#"
        [execution]
        preserve_lines = 20
    "#;
    
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.execution.preserve_lines, 20);
}
```

---

## Integration Test Examples

### Niri IPC Mock

```rust
// tests/niri_test.rs
use std::os::unix::net::UnixListener;
use std::thread;

fn mock_niri_server(socket_path: &str) -> thread::JoinHandle<()> {
    let listener = UnixListener::bind(socket_path).unwrap();
    
    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            // Read request, send mock response
            let response = r#"{"ok": null}"#;
            stream.write_all(response.as_bytes()).unwrap();
        }
    })
}

#[test]
fn test_niri_set_floating() {
    let socket_path = "/tmp/test-niri-socket";
    let _server = mock_niri_server(socket_path);
    
    std::env::set_var("NIRI_SOCKET", socket_path);
    
    let mut client = NiriClient::new();
    assert!(client.is_available());
    
    let result = client.set_floating(true);
    assert!(result.is_ok());
}
```

---

## Manual Testing Checklist

### Launcher Mode

- [ ] Entries load from system directories
- [ ] Fuzzy search filters correctly
- [ ] Keyboard navigation works (j/k, arrows)
- [ ] Enter executes selected entry
- [ ] Esc quits application

### Execution Mode

- [ ] Command runs in terminal
- [ ] Output displays correctly
- [ ] ANSI colors render
- [ ] Exit code shows
- [ ] Returns to launcher

### Niri Integration

- [ ] Window floats on startup
- [ ] Window unfloats on execute
- [ ] Window re-floats on return
- [ ] Works without niri

### Terminal Compatibility

Test in:
- [ ] foot
- [ ] alacritty
- [ ] kitty
- [ ] wezterm

---

## Coverage Goals

| Module | Target |
|--------|--------|
| `desktop_entry.rs` | 90% |
| `config.rs` | 85% |
| `niri.rs` | 80% |
| `app.rs` | 70% |
| `ui.rs` | 50% (visual) |

---

## CI Integration

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```
