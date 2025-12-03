# Phase 6: Icon Display

## Overview

Display application icons in the TUI launcher. This is challenging in terminal environments but possible with modern terminals.

## Current State

The `Entry.icon` field is populated from `.desktop` files but not displayed.

## Approaches

### 1. Kitty Graphics Protocol (Recommended for Kitty users)

Kitty terminal supports inline images via escape sequences.

```rust
// Check if running in Kitty
fn is_kitty() -> bool {
    std::env::var("TERM").map(|t| t.contains("kitty")).unwrap_or(false)
        || std::env::var("KITTY_WINDOW_ID").is_ok()
}

// Render icon using Kitty protocol
fn render_kitty_icon(path: &Path, size: u32) -> Result<String> {
    // Read and base64 encode the image
    // Send via Kitty graphics protocol
    // Returns escape sequence string
}
```

### 2. Sixel Graphics (Wide terminal support)

Sixel is supported by many terminals (xterm, mlterm, foot, etc.)

```rust
fn is_sixel_supported() -> bool {
    // Query terminal capabilities via DECRQSS
}
```

### 3. Unicode/Nerd Font Fallback

For terminals without graphics support, use Nerd Font icons.

```rust
fn icon_to_nerd_font(categories: &[String]) -> &'static str {
    match categories.first().map(|s| s.as_str()) {
        Some("Development") => "\u{f121}",  // 
        Some("Network") => "\u{f0ac}",      // 
        Some("Utility") => "\u{f0ad}",      // 
        Some("System") => "\u{f085}",       // 
        Some("Audio") => "\u{f001}",        // 
        Some("Video") => "\u{f03d}",        // 
        Some("Graphics") => "\u{f1fc}",     // 
        Some("Game") => "\u{f11b}",         // 
        _ => "\u{f15b}",                    //  (default file)
    }
}
```

## Icon Resolution

Desktop entries specify icons by name, not path. Resolution order:

1. Check if `icon` is an absolute path → use directly
2. Search icon theme directories:
   - `$XDG_DATA_HOME/icons/<theme>/`
   - `/usr/share/icons/<theme>/`
   - `/usr/share/pixmaps/`

```rust
fn resolve_icon(name: &str, size: u32, theme: &str) -> Option<PathBuf> {
    // 1. Absolute path?
    let path = PathBuf::from(name);
    if path.is_absolute() && path.exists() {
        return Some(path);
    }
    
    // 2. Search theme directories
    let sizes = [size, 48, 32, 24, 16]; // Preferred sizes
    let extensions = ["svg", "png", "xpm"];
    
    for base in icon_base_dirs() {
        for sz in &sizes {
            for ext in &extensions {
                let path = base.join(theme).join(format!("{sz}x{sz}"))
                    .join("apps").join(format!("{name}.{ext}"));
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }
    
    None
}
```

## Implementation Plan

### Unit 6.1: Icon Resolution

- [ ] Implement `resolve_icon()` function
- [ ] Support SVG, PNG, XPM formats
- [ ] Cache resolved paths

### Unit 6.2: Terminal Capability Detection

- [ ] Detect Kitty terminal
- [ ] Detect Sixel support
- [ ] Fallback to Nerd Fonts

### Unit 6.3: Icon Rendering

- [ ] Kitty graphics protocol implementation
- [ ] Sixel rendering (via `sixel-rs` or similar)
- [ ] Nerd Font category mapping

### Unit 6.4: UI Integration

- [ ] Add icon column to entry list
- [ ] Handle missing icons gracefully
- [ ] Configuration option to disable icons

## Configuration

```toml
[appearance]
show_icons = true
icon_theme = "hicolor"  # or "Papirus", "Adwaita", etc.
icon_size = 24
icon_fallback = "nerd-font"  # or "none"
```

## Dependencies

```toml
[dependencies]
# For image processing
image = "0.24"
# For Sixel (optional)
sixel-rs = { version = "0.3", optional = true }
```

## SSH Considerations

Icons should be disabled or use Nerd Font fallback over SSH:
- Graphics protocols may not work
- Image files may not exist on remote host
- Bandwidth considerations

```rust
fn should_show_icons() -> bool {
    // Disable if SSH_CONNECTION is set (unless explicitly enabled)
    if std::env::var("SSH_CONNECTION").is_ok() {
        return config.appearance.force_icons_over_ssh;
    }
    config.appearance.show_icons
}
```

## Testing

```bash
# Test icon resolution
cargo test icon_resolution

# Visual test in Kitty
TERM=xterm-kitty cargo run

# Test fallback
TERM=dumb cargo run
```
