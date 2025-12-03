# DRUN Usage Guide

> A TUI application launcher designed for local and remote (SSH) use.

---

## Quick Start

```bash
# Local usage
drun

# Remote usage (from another machine)
ssh some-host drun

# Or bind to a WM hotkey
# Example for niri:
# Mod+D { spawn "foot" "-e" "drun"; }
```

---

## Design Philosophy

DRUN is a **terminal-agnostic TUI binary**. It:

- Reads **only** XDG `.desktop` files - no custom config formats
- Uses only stdin/stdout/stderr
- Makes no assumptions about terminal emulators
- Works identically whether run locally or via SSH
- Does NOT manage SSH connections or spawn terminals

**The caller decides how to run DRUN** - whether in a local terminal, via SSH, or through a WM keybinding.

---

## Configuration Format

**DRUN's only action source is XDG `.desktop` files.**

There is no `actions.toml`, no JSON registry, no custom format.
If an app has a `.desktop` file, DRUN can run it.

### .desktop File Locations

- `~/.local/share/applications/`
- `/usr/share/applications/`
- `/run/current-system/sw/share/applications/` (NixOS)

### Supported .desktop Fields

| Field | Usage |
|-------|-------|
| `Name` | Displayed in the list |
| `Comment` | Shown as description |
| `Exec` | Command to run |
| `Icon` | Stored (TUI may not display) |
| `Categories` | Used for filtering |
| `Terminal` | If true, runs in current terminal |
| `NoDisplay` | If true, hidden from list |

### Internal Representation

Internally, DRUN converts `.desktop` entries to an `Action` struct.
This is an implementation detail - users never write Actions directly.

---

## Command Line Options

```
drun [OPTIONS]

Options:
  --config <PATH>    Config file path [default: ~/.config/darkwall-drun/config.toml]
  -d, --daemon       Stay open after command execution
  --no-niri          Disable niri IPC integration
  --mouse            Enable mouse support (off by default for SSH compatibility)
  -h, --help         Print help
  -V, --version      Print version
```

---

## Keybindings

### Launcher Mode

| Key | Action |
|-----|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Enter` | Execute selected action |
| `/` | Start filtering |
| `Esc` | Clear filter / Exit |
| `q` | Exit |
| `Ctrl+C` | Exit |
| Any char | Start filtering with that character |

### Executing Mode

| Key | Action |
|-----|--------|
| `Ctrl+C` | Kill process |
| `↑` / `k` | Scroll output up |
| `↓` / `j` | Scroll output down |
| `Ctrl+U` | Scroll up 10 lines |
| `Ctrl+D` | Scroll down 10 lines |
| `g` | Scroll to top |
| `G` | Scroll to bottom |

### Post-Execution Mode

| Key | Action |
|-----|--------|
| `Enter` / `Esc` | Return to launcher |
| `q` / `Ctrl+C` | Exit |

---

## SSH Usage

DRUN is designed to work seamlessly over SSH:

```bash
# Simple remote launch
ssh some-host drun

# With pseudo-terminal allocation (recommended)
ssh -t some-host drun

# Bind to local hotkey (example for sway/niri)
bindsym $mod+d exec foot -e ssh some-host drun
```

### SSH Considerations

1. **Mouse support** is disabled by default (use `--mouse` to enable)
2. **Niri IPC** is auto-disabled when socket isn't found
3. **$TERM** differences are handled by crossterm
4. **Limited colors** work fine (DRUN uses basic colors)

### Remote Execution

When you select an action over SSH:
- The command runs on the **remote host**
- Output is captured and displayed in DRUN
- TUI apps (vim, htop) get full terminal handover

---

## Local vs Remote Behavior

| Aspect | Local | SSH |
|--------|-------|-----|
| Actions loaded from | Local .desktop files | Remote .desktop files |
| Commands run on | Local machine | Remote machine |
| Niri IPC | Available (if running) | Disabled |
| Mouse support | Optional | Disabled by default |
| Terminal | Caller's choice | SSH PTY |

---

## Configuration

See `~/.config/darkwall-drun/config.toml`:

```toml
# Where to find .desktop files
desktop_entry_dirs = [
    "~/.local/share/applications",
    "/usr/share/applications",
]

[appearance]
prompt = "❯ "
selected_prefix = "● "
unselected_prefix = "  "

[niri]
enabled = true
float_on_idle = true
unfloat_on_execute = true

[behavior]
after_command = "return"
preserve_output_lines = 10
```

---

## Integration Examples

### Niri (Wayland Compositor)

```kdl
// ~/.config/niri/config.kdl
binds {
    Mod+D { spawn "foot" "-e" "drun"; }
}

window-rule {
    match app-id="foot" title="drun"
    open-floating true
}
```

### Sway

```
# ~/.config/sway/config
bindsym $mod+d exec foot -e drun
```

### i3

```
# ~/.config/i3/config
bindsym $mod+d exec --no-startup-id alacritty -e drun
```

### Remote via SSH

```bash
# ~/.local/bin/remote-drun
#!/bin/bash
ssh -t myserver drun
```

Then bind `remote-drun` to a hotkey.

---

## Troubleshooting

### "No desktop entries found"

Check that `.desktop` files exist in configured directories:
```bash
ls ~/.local/share/applications/
ls /usr/share/applications/
```

### "Niri IPC not available"

This is normal when:
- Not running under niri
- Running over SSH
- `$NIRI_SOCKET` not set

DRUN works fine without niri - it just won't float/unfloat windows.

### Colors look wrong over SSH

Try setting `$TERM`:
```bash
TERM=xterm-256color ssh -t host drun
```

### Input lag over SSH

This is network latency. DRUN is optimized for responsiveness but can't eliminate network delay.

---

## Architecture

```
┌─────────────────────────────────────────────┐
│                    DRUN                      │
├─────────────────────────────────────────────┤
│  TUI Layer (ratatui + crossterm)            │
│  - Keyboard input                           │
│  - Terminal rendering                       │
│  - No terminal emulator assumptions         │
├─────────────────────────────────────────────┤
│  Action Layer                               │
│  - Action abstraction                       │
│  - Desktop entry loading                    │
│  - Fuzzy matching                           │
├─────────────────────────────────────────────┤
│  Execution Layer                            │
│  - PTY allocation                           │
│  - Output capture                           │
│  - Process management                       │
├─────────────────────────────────────────────┤
│  Optional: Niri IPC                         │
│  - Window floating                          │
│  - Gracefully disabled over SSH             │
└─────────────────────────────────────────────┘
```

---

## See Also

- [Architecture](./ARCHITECTURE.md)
- [Niri Setup](./NIRI_SETUP.md)
- [Custom Fields](./CUSTOM_FIELDS.md)
