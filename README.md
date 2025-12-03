# darkwall-drun

A TUI application launcher with niri compositor integration. Part of the Darkwall suite.

## Overview

`darkwall-drun` is a terminal-based application launcher that:
- Parses XDG desktop entries (`.desktop` files)
- Provides fuzzy search/filtering
- Integrates with niri compositor for seamless window state transitions
- Executes commands in-place, transforming between launcher and execution modes

## Quick Start

```bash
# Build
cargo build --release

# Run
./target/release/darkwall-drun

# Or with foot terminal (recommended)
foot --app-id darkwall-drun -e darkwall-drun
```

## Configuration

Copy `config.example.toml` to `~/.config/darkwall-drun/config.toml` and customize.

## Niri Window Rules

Add to your niri config (`~/.config/niri/config.kdl`):

```kdl
window-rule {
    match app-id="darkwall-drun"
    default-column-width { proportion 0.4; }
    open-floating true
}
```

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Execute selected entry |
| `/` | Start filter mode |
| `Esc` | Clear filter / Exit |
| `q` | Quit |
| Any char | Start filtering immediately |

## License

MIT
