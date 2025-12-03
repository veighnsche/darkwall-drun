# Unit 4.4: Custom Desktop Entry Fields

> **Phase:** 4 - Advanced Metadata  
> **Complexity:** Medium  
> **Skills:** Schema design, documentation

---

## Objective

Define, implement, and document custom X-Darkwall desktop entry fields.

---

## Tasks

### 1. Define Custom Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `X-DarkwallTerminalMode` | enum | inferred | Execution mode |
| `X-DarkwallKeepOutput` | bool | mode-based | Preserve output |
| `X-DarkwallUnfloatOnRun` | bool | mode-based | Unfloat window |
| `X-DarkwallPreserveLines` | int | config | Lines to preserve |

### 2. Implement Field Parsing

```rust
impl DesktopEntry {
    pub fn get_darkwall_field<T: FromStr>(&self, name: &str) -> Option<T> {
        self.get(&format!("X-Darkwall{}", name))
            .and_then(|v| v.parse().ok())
    }
}
```

### 3. Document in README

Create comprehensive documentation for users.

---

## Implementation Notes

### Field Definitions

```rust
pub struct DarkwallMetadata {
    pub terminal_mode: Option<TerminalMode>,
    pub keep_output: Option<bool>,
    pub unfloat_on_run: Option<bool>,
    pub preserve_lines: Option<usize>,
}

impl DarkwallMetadata {
    pub fn from_entry(entry: &DesktopEntry) -> Self {
        Self {
            terminal_mode: entry.get_darkwall_field("TerminalMode"),
            keep_output: entry.get_darkwall_field("KeepOutput"),
            unfloat_on_run: entry.get_darkwall_field("UnfloatOnRun"),
            preserve_lines: entry.get_darkwall_field("PreserveLines"),
        }
    }
}
```

### Example Desktop Entry

```ini
[Desktop Entry]
Name=My TUI App
Exec=my-tui-app
Type=Application
Terminal=true

# Darkwall-specific fields
X-DarkwallTerminalMode=tui
X-DarkwallKeepOutput=false
X-DarkwallUnfloatOnRun=true
```

### NixOS Integration

```nix
# In lib/desktop-entries.nix
darkwallOptions = {
  terminalMode = lib.mkOption {
    type = types.nullOr (types.enum [ "oneshot" "interactive" "tui" "long-running" ]);
    default = null;
    description = "How darkwall-drun should handle this application";
  };
  
  keepOutput = lib.mkOption {
    type = types.nullOr types.bool;
    default = null;
    description = "Whether to preserve output after command exits";
  };
  
  unfloatOnRun = lib.mkOption {
    type = types.nullOr types.bool;
    default = null;
    description = "Whether to unfloat the window during execution";
  };
  
  preserveLines = lib.mkOption {
    type = types.nullOr types.int;
    default = null;
    description = "Number of output lines to preserve";
  };
};

# Usage in desktop entry definition
mkDesktopEntry {
  name = "btop";
  exec = "btop";
  darkwall = {
    terminalMode = "tui";
    keepOutput = false;
  };
}
```

---

## Field Reference

### X-DarkwallTerminalMode

**Values:** `oneshot`, `interactive`, `tui`, `long-running`

| Value | Behavior |
|-------|----------|
| `oneshot` | Quick command, capture output, stay floating |
| `interactive` | Needs input, partial capture, unfloat |
| `tui` | Full screen, no capture, unfloat |
| `long-running` | Server/watch, capture output, unfloat |

### X-DarkwallKeepOutput

**Values:** `true`, `false`

- `true`: Preserve last N lines after exit
- `false`: Clear screen after exit

### X-DarkwallUnfloatOnRun

**Values:** `true`, `false`

- `true`: Unfloat window when command starts
- `false`: Keep window floating during execution

### X-DarkwallPreserveLines

**Values:** positive integer

Overrides global `preserve_lines` config for this entry.

---

## Acceptance Criteria

- [ ] All custom fields parse correctly
- [ ] Invalid values handled gracefully (use defaults)
- [ ] Fields override inferred behavior
- [ ] README documents all fields
- [ ] NixOS module supports fields

---

## Testing

### Unit Tests

```rust
#[test]
fn test_parse_all_fields() {
    let entry = DesktopEntry::from_str(r#"
        [Desktop Entry]
        Name=Test
        Exec=test
        X-DarkwallTerminalMode=tui
        X-DarkwallKeepOutput=false
        X-DarkwallUnfloatOnRun=true
        X-DarkwallPreserveLines=20
    "#).unwrap();
    
    let meta = DarkwallMetadata::from_entry(&entry);
    assert_eq!(meta.terminal_mode, Some(TerminalMode::Tui));
    assert_eq!(meta.keep_output, Some(false));
    assert_eq!(meta.unfloat_on_run, Some(true));
    assert_eq!(meta.preserve_lines, Some(20));
}

#[test]
fn test_invalid_field_ignored() {
    let entry = DesktopEntry::from_str(r#"
        [Desktop Entry]
        Name=Test
        Exec=test
        X-DarkwallTerminalMode=invalid
    "#).unwrap();
    
    let meta = DarkwallMetadata::from_entry(&entry);
    assert!(meta.terminal_mode.is_none());
}
```

### Integration Tests

1. Create test .desktop file with custom fields
2. Launch via darkwall-drun
3. Verify behavior matches field values

---

## Documentation Output

Create `docs/CUSTOM_FIELDS.md`:

```markdown
# Custom Desktop Entry Fields

darkwall-drun supports custom fields in .desktop files to control behavior.

## Fields

### X-DarkwallTerminalMode
...

### X-DarkwallKeepOutput
...

## Examples
...
```

---

## Related Units

- **Depends on:** Unit 4.1 (Terminal Mode Schema)
- **Related:** Unit 4.3 (Output Preservation)
