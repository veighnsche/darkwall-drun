# System Requirements

> Minimum and recommended system specifications for darkwall-drun.

---

## Quick Reference

| Tier | Terminal | Icons | Theming | Niri IPC | Frecency |
|------|----------|-------|---------|----------|----------|
| **Minimum** | Any with ANSI | ❌ | Basic 16-color | ❌ | ✅ |
| **Recommended** | Modern (foot, kitty, alacritty) | Emoji fallback | 256-color | ✅ | ✅ |
| **Full Experience** | kitty, foot, WezTerm | Native graphics | True color | ✅ | ✅ |

---

## Minimum Requirements

The bare minimum to run darkwall-drun.

### System

| Component | Requirement |
|-----------|-------------|
| **OS** | Linux (any distribution) |
| **Architecture** | x86_64, aarch64 |
| **Shell** | POSIX-compatible (sh, bash, zsh) |
| **Terminal** | Any with basic ANSI support |

### Environment

| Variable | Required | Notes |
|----------|----------|-------|
| `$HOME` | Yes | For XDG fallbacks |
| `$TERM` | Yes | Any value (e.g., `xterm`, `linux`) |
| `$XDG_DATA_DIRS` | No | Falls back to standard paths |

### Features Available

- ✅ Desktop entry listing and search
- ✅ Fuzzy matching
- ✅ Command execution with PTY
- ✅ Frecency sorting (persisted to `~/.local/state`)
- ✅ Basic 16-color display
- ❌ Icons (no graphics protocol)
- ❌ Niri integration (no compositor)
- ❌ True color theming

### SSH Compatibility

Works fully over SSH with these limitations:
- No Niri IPC (auto-disabled when `$NIRI_SOCKET` absent)
- No graphics-based icons
- Mouse disabled by default

---

## Recommended Requirements

For a good daily-driver experience.

### System

| Component | Requirement |
|-----------|-------------|
| **OS** | Linux with systemd (for XDG paths) |
| **Compositor** | Niri (for window management features) |
| **Terminal** | foot, alacritty, kitty, or WezTerm |

### Environment

| Variable | Required | Notes |
|----------|----------|-------|
| `$XDG_DATA_HOME` | Recommended | `~/.local/share` |
| `$XDG_CONFIG_HOME` | Recommended | `~/.config` |
| `$XDG_STATE_HOME` | Recommended | `~/.local/state` (for history) |
| `$NIRI_SOCKET` | Optional | Auto-detected if running under Niri |

### Terminal Capabilities

| Capability | Minimum | Recommended |
|------------|---------|-------------|
| Colors | 16 | 256 or True Color |
| Unicode | Basic | Full (emoji support) |
| Mouse | Not required | Optional (`--mouse`) |
| Alternate screen | Required | Required |

### Features Available

- ✅ All minimum features
- ✅ 256-color or true color themes
- ✅ Emoji icon fallbacks
- ✅ Niri window float/unfloat
- ❌ Native icon graphics (requires specific terminals)

---

## Full Experience

For all features including native icons.

### System

| Component | Requirement |
|-----------|-------------|
| **OS** | NixOS or modern Linux |
| **Compositor** | Niri |
| **Terminal** | kitty, foot (Sixel), or WezTerm |
| **Icon Theme** | hicolor + application icons installed |

### Terminal Graphics Protocols

| Protocol | Terminals | Quality |
|----------|-----------|---------|
| **Kitty Graphics** | kitty | Best |
| **Sixel** | foot, mlterm, xterm | Good |
| **iTerm2 Inline** | iTerm2, WezTerm | Good |
| **None** | alacritty, others | Emoji fallback |

### Icon Requirements

```
# Icon theme must be installed
~/.local/share/icons/hicolor/
/usr/share/icons/hicolor/
/run/current-system/sw/share/icons/  # NixOS
```

### Features Available

- ✅ All recommended features
- ✅ Native application icons
- ✅ Full theme customization
- ✅ Category grouping with icons

---

## Feature Matrix

### By Feature

| Feature | Minimum | Recommended | Full |
|---------|---------|-------------|------|
| Desktop entry listing | ✅ | ✅ | ✅ |
| Fuzzy search | ✅ | ✅ | ✅ |
| PTY execution | ✅ | ✅ | ✅ |
| TUI handover (vim, htop) | ✅ | ✅ | ✅ |
| Frecency sorting | ✅ | ✅ | ✅ |
| History persistence | ✅ | ✅ | ✅ |
| 16-color theme | ✅ | ✅ | ✅ |
| 256/True color theme | ❌ | ✅ | ✅ |
| Emoji icons | ❌ | ✅ | ✅ |
| Native icons | ❌ | ❌ | ✅ |
| Niri float/unfloat | ❌ | ✅ | ✅ |
| Category grouping | ✅ | ✅ | ✅ |
| Mouse support | Optional | Optional | Optional |

### By Environment

| Environment | Tier | Notes |
|-------------|------|-------|
| SSH session | Minimum | Graphics/Niri auto-disabled |
| Linux console (`/dev/tty`) | Minimum | Very limited colors |
| tmux/screen | Minimum-Recommended | Depends on outer terminal |
| alacritty | Recommended | No graphics protocol |
| foot | Full | Sixel support |
| kitty | Full | Best graphics support |
| WezTerm | Full | iTerm2 protocol |

---

## Niri Integration

### Requirements

| Component | Requirement |
|-----------|-------------|
| **Compositor** | Niri |
| **Socket** | `$NIRI_SOCKET` or `$XDG_RUNTIME_DIR/niri-socket` |

### Features

| Feature | Description |
|---------|-------------|
| **Float on idle** | Window floats when showing launcher |
| **Unfloat on execute** | Window tiles when running command |
| **Auto-disable** | Gracefully disabled over SSH |

### Configuration

```toml
# ~/.config/darkwall-drun/config.toml
[niri]
enabled = true           # Enable Niri integration
float_on_idle = true     # Float window when idle
unfloat_on_execute = true # Unfloat when running command
```

---

## XDG Paths

### Data Locations

| Purpose | Path | Fallback |
|---------|------|----------|
| Desktop entries | `$XDG_DATA_DIRS/applications/` | `/usr/share/applications/` |
| User entries | `$XDG_DATA_HOME/applications/` | `~/.local/share/applications/` |
| NixOS entries | `/run/current-system/sw/share/applications/` | — |

### State & Config

| Purpose | Path | Fallback |
|---------|------|----------|
| Config | `$XDG_CONFIG_HOME/darkwall-drun/` | `~/.config/darkwall-drun/` |
| History | `$XDG_STATE_HOME/darkwall-drun/` | `~/.local/state/darkwall-drun/` |

---

## Troubleshooting

### No desktop entries found

```bash
# Check XDG paths
echo $XDG_DATA_DIRS
ls ~/.local/share/applications/
ls /usr/share/applications/
```

### Niri integration not working

```bash
# Check socket
echo $NIRI_SOCKET
ls $XDG_RUNTIME_DIR/niri*

# Run with debug logging
RUST_LOG=debug drun
```

### Icons not displaying

1. Check terminal supports graphics protocol
2. Verify icon theme is installed
3. Try emoji fallback: `icons = true`, `icon_fallback = "emoji"`

### Colors look wrong

```bash
# Check terminal color support
echo $TERM
tput colors

# Force 256 colors
export TERM=xterm-256color
```

---

## Platform Notes

### NixOS

- Desktop entries in `/run/current-system/sw/share/applications/`
- Icon themes via `gtk.iconTheme` or `qt.iconTheme`
- Niri available in nixpkgs

### Arch Linux

```bash
pacman -S niri foot
# Icons via your preferred theme package
```

### Ubuntu/Debian

```bash
# Niri may need manual installation
# foot available in recent versions
apt install foot
```

---

## Summary

| Use Case | Recommended Setup |
|----------|-------------------|
| **SSH/Remote** | Any terminal, minimum tier |
| **Daily driver** | Niri + foot/alacritty, recommended tier |
| **Power user** | Niri + kitty, full tier |
| **Minimal/Embedded** | Any terminal, minimum tier |
