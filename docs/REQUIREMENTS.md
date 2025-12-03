# DRUN Requirements

> Codified requirements for the DRUN TUI launcher.

---

## Core Principle

**DRUN's only configuration format is XDG `.desktop` entries.**

- DRUN reads `.desktop` files from standard XDG locations
- DRUN does **NOT** introduce custom config formats (no actions.toml, no JSON registry)
- If an app provides a `.desktop` file, DRUN can list and run it
- Users never need to write anything DRUN-specific

---

## Data Model

### External (User-Facing)

```
~/.local/share/applications/*.desktop
/usr/share/applications/*.desktop
/run/current-system/sw/share/applications/*.desktop  (NixOS)
```

Standard XDG Desktop Entry format. No extensions required.

### Internal (DRUN Implementation)

```rust
struct Action {
    id: String,              // filename without .desktop
    name: String,            // Name=
    comment: Option<String>, // Comment=
    exec: String,            // Exec= (with %f, %u stripped)
    icon: Option<String>,    // Icon=
    categories: Vec<String>, // Categories=
    terminal: bool,          // Terminal=
}
```

This is **internal only**. Users never create Actions directly.

---

## SSH-First Model

DRUN must be fully usable remotely:

```bash
ssh some-host drun
```

### Requirements

1. DRUN runs on the host where `.desktop` files live
2. DRUN does **NOT** manage SSH (no tunnels, no remote discovery)
3. TUI works in basic terminals (no kitty/foot assumptions)
4. No mouse support by default (use `--mouse` to enable)
5. Works with limited colors and $TERM variations

### What This Means

| Scenario | Behavior |
|----------|----------|
| `drun` locally | Reads local .desktop files, runs local commands |
| `ssh host drun` | Reads remote .desktop files, runs remote commands |
| Mixed local/remote | Not supported - DRUN is always local to its host |

---

## Terminal Behavior

### DRUN Must NOT

- Spawn a terminal emulator
- Assume specific terminal features (kitty graphics, etc.)
- Require mouse support
- Depend on exotic input sequences

### DRUN Must

- Use only stdin/stdout/stderr
- Handle `Terminal=true` entries by running them directly (DRUN is already in a terminal)
- Gracefully degrade with limited $TERM capabilities

---

## .desktop Parsing Requirements

### Required Fields

| Field | Handling |
|-------|----------|
| `Name` | Required, displayed in list |
| `Exec` | Required, command to run |
| `Type` | Must be `Application` |

### Optional Fields

| Field | Handling |
|-------|----------|
| `Comment` | Shown as description |
| `Icon` | Stored but may not be displayed in TUI |
| `Categories` | Used for grouping/filtering |
| `Terminal` | If true, command runs in current terminal |
| `NoDisplay` | If true, entry is hidden |
| `Hidden` | If true, entry is hidden |

### Exec Field Handling

Strip field codes before execution:
- `%f`, `%F` - file arguments (not used)
- `%u`, `%U` - URL arguments (not used)
- `%i`, `%c`, `%k` - icon/name/path (not used)

### Error Handling

| Condition | Behavior |
|-----------|----------|
| Missing Name | Skip entry, log warning |
| Missing Exec | Skip entry, log warning |
| Invalid syntax | Skip entry, log warning |
| Duplicate ID | First one wins |
| Unreadable file | Skip, continue with others |

---

## What DRUN Does NOT Do

1. **No custom action formats** - only .desktop files
2. **No SSH management** - caller handles SSH
3. **No terminal spawning** - caller provides terminal
4. **No remote aggregation** - actions are always local
5. **No GUI dependencies** - pure TUI

---

## Niri Integration (Optional)

When running under niri compositor:
- Window can float/unfloat during execution
- Auto-disabled when `$NIRI_SOCKET` is absent (e.g., over SSH)

This is purely optional and does not affect core functionality.

---

## Configuration

DRUN has a small config file for **appearance and behavior** only:

```toml
# ~/.config/darkwall-drun/config.toml

# Where to find .desktop files (in addition to XDG defaults)
desktop_entry_dirs = []

[appearance]
prompt = "❯ "

[behavior]
preserve_output_lines = 10
```

This config does **NOT** define actions. Actions come only from `.desktop` files.

---

## Theming Requirements

### Layout

DRUN displays entries in a **grid layout**:

| Property | Value | Notes |
|----------|-------|-------|
| Columns | 2 | Side-by-side entries |
| Visible Rows | 5 | Shows 10 entries total (2×5) |
| Entry Style | Card | Each entry is a distinct visual block |

### Entry Display

Each entry shows **4 lines of information** (multi-line card):

```
┌─────────────────────────────────┐
│ [Icon]  Name (bold)             │
│         Generic Name            │
│         Comment (dimmed)        │
│         Categories (smaller)    │
└─────────────────────────────────┘
```

| Line | Field | Style |
|------|-------|-------|
| 1 | `Name` | Bold, primary foreground |
| 2 | `GenericName` | Normal, primary foreground |
| 3 | `Comment` | Dimmed (gray) |
| 4 | `Categories` | Smaller, dimmed |

### Icon Requirements

| Property | Value |
|----------|-------|
| Size | Large (equivalent to ~48px in GUI) |
| Position | Left of text block |
| Fallback | Text placeholder or none |

### Color Scheme

Default dark theme (user-configurable):

| Element | Default | Notes |
|---------|---------|-------|
| Background | Dark (`#0d1116`) | Main window background |
| Foreground | Light (`#e5eaf1`) | Primary text |
| Selection BG | Slightly lighter (`#141c2a`) | Selected entry background |
| Selection FG | Same as foreground | Selected entry text |
| Accent/Border | Amber (`#b45309`) | Input border, highlights |
| Dimmed Text | Gray (`#9ca3af`, `#6b7280`) | Comments, categories |

### Card Styling

Each entry is rendered as a **card** with:

| Property | Value |
|----------|-------|
| Padding | Generous (12px vertical, 8px horizontal equivalent) |
| Background | Distinct from window background |
| Selection | Background color change on selection |
| Spacing | Gap between cards |

### TUI Constraints

Since DRUN is a TUI, theming must work within terminal limitations:

| Constraint | Handling |
|------------|----------|
| No pixel sizes | Use character cells for sizing |
| Limited colors | Support 256-color and true-color terminals |
| No fonts | Use terminal's font; rely on Unicode for styling |
| No images | Icons via Nerd Font glyphs or Kitty graphics protocol (optional) |

### Configuration

```toml
[appearance]
columns = 2
visible_rows = 5

[appearance.entry]
show_generic = true
show_comment = true
show_categories = true

[theme]
preset = "dark"  # dark, light, nord, gruvbox, dracula

[theme.colors]
background = "#0d1116"
foreground = "#e5eaf1"
selection_bg = "#141c2a"
accent = "#b45309"
dimmed = "#9ca3af"
```

---

## Summary

| Aspect | Requirement |
|--------|-------------|
| Action source | `.desktop` files only |
| Custom formats | Not supported |
| SSH usage | `ssh host drun` |
| Terminal | Caller provides it |
| Mouse | Off by default |
| Niri | Optional, auto-disabled over SSH |
