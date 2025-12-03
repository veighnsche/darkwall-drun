# Niri Setup Guide

> TEAM_000: Phase 3, Unit 3.3 - Window Rules Documentation

This guide explains how to configure darkwall-drun with the [niri](https://github.com/YaLTeR/niri) Wayland compositor.

---

## Overview

darkwall-drun integrates with niri's IPC to automatically manage window floating state:

- **Launcher mode**: Window floats (centered, smaller)
- **Execution mode**: Window unfloats (tiles with other windows)
- **Return to launcher**: Window re-floats

---

## Prerequisites

- Niri compositor running
- `$NIRI_SOCKET` environment variable set (automatic in niri sessions)
- A terminal emulator (foot, alacritty, kitty, etc.)

---

## Niri Configuration

Add these window rules to `~/.config/niri/config.kdl`:

### Minimal Setup

```kdl
window-rule {
    match app-id="darkwall-drun"
    open-floating true
}
```

### Recommended Setup

```kdl
window-rule {
    match app-id="darkwall-drun"
    
    // Start floating and centered
    open-floating true
    
    // Floating size (40% of screen width)
    default-column-width { proportion 0.4; }
}
```

### Keybind

Add a keybind to launch darkwall-drun:

```kdl
binds {
    // Launch with Mod+D (like dmenu/rofi)
    Mod+D { spawn "foot" "--app-id" "darkwall-drun" "-e" "darkwall-drun"; }
}
```

---

## Terminal Configuration

### Foot

```bash
foot --app-id darkwall-drun -e darkwall-drun
```

Or create a desktop entry:

```ini
# ~/.local/share/applications/darkwall-drun.desktop
[Desktop Entry]
Name=Darkwall Drun
Exec=foot --app-id darkwall-drun -e darkwall-drun
Type=Application
Terminal=false
```

### Alacritty

```bash
alacritty --class darkwall-drun -e darkwall-drun
```

### Kitty

```bash
kitty --class darkwall-drun darkwall-drun
```

---

## Configuration Options

In `~/.config/darkwall-drun/config.toml`:

```toml
[niri]
# Enable/disable niri integration
enabled = true

# Float window when returning to launcher
float_on_idle = true

# Unfloat window when executing commands
unfloat_on_execute = true
```

---

## Behavior

### Window State Transitions

```
┌─────────────────┐     execute      ┌─────────────────┐
│    Floating     │ ───────────────► │    Unfloated    │
│   (Launcher)    │                  │   (Executing)   │
└─────────────────┘ ◄─────────────── └─────────────────┘
                       exit/return
```

### Terminal Mode Behavior

| Mode | Unfloat? | Description |
|------|----------|-------------|
| Oneshot | No | Quick commands (ls, echo) stay floating |
| Interactive | Yes | REPLs (python, bash) unfloat |
| TUI | Yes* | Full-screen apps (htop, vim) get terminal handover |
| Long-running | Yes | Servers, watch commands unfloat |

*TUI apps get full terminal control, bypassing our UI entirely.

---

## Troubleshooting

### Window Doesn't Float

1. Check app-id matches:
   ```bash
   niri msg windows | grep darkwall
   ```

2. Verify window rule syntax in niri config

3. Check niri logs:
   ```bash
   journalctl --user -u niri
   ```

### IPC Not Working

1. Check socket exists:
   ```bash
   echo $NIRI_SOCKET
   ls -la $NIRI_SOCKET
   ```

2. Test IPC manually:
   ```bash
   echo '{"Request":"Version"}' | nc -U $NIRI_SOCKET
   ```

3. Run darkwall-drun with debug logging:
   ```bash
   RUST_LOG=darkwall_drun=debug darkwall-drun
   ```

### Window Wrong Size

1. Check `default-column-width` in niri config
2. Terminal might override size - check terminal config
3. Try explicit size in foot: `foot -W 80x24 ...`

---

## Running Without Niri

darkwall-drun works without niri - IPC features are simply disabled:

```bash
darkwall-drun --no-niri
```

Or in config:

```toml
[niri]
enabled = false
```

---

## Multi-Monitor

- Launcher appears on the focused monitor
- Execution tiles on the same monitor
- Re-floating returns to the same position

---

## See Also

- [Niri Documentation](https://github.com/YaLTeR/niri/wiki)
- [darkwall-drun README](../README.md)
- [Configuration Guide](./GETTING_STARTED.md)
