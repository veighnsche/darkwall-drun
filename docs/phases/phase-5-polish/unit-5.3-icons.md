# Unit 5.3: Icons (Optional)

> **Phase:** 5 - Polish & Features  
> **Complexity:** Low  
> **Skills:** Image handling, terminal capabilities  
> **Status:** � Complete - Emoji fallback working, Kitty graphics prepared

---

## Current State

The `Entry.icon` field is populated from `.desktop` files but not displayed.
The field is marked `#[allow(dead_code)]` with a note about future icon display.

---

## Objective

Display application icons in the TUI for terminals that support inline images.

---

## Tasks

### 1. Load Icons via freedesktop-icons

```rust
use freedesktop_icons::lookup;

pub fn find_icon(icon_name: &str, size: u16) -> Option<PathBuf> {
    lookup(icon_name)
        .with_size(size)
        .with_theme("hicolor")
        .find()
}
```

### 2. Render in TUI (if supported)

Check terminal capabilities and render appropriately.

### 3. Fallback to Text Indicators

```rust
fn icon_fallback(entry: &DesktopEntry) -> &'static str {
    match entry.main_category() {
        Some("Network") => "🌐",
        Some("Development") => "💻",
        Some("Graphics") => "🎨",
        Some("AudioVideo") => "🎵",
        Some("Game") => "🎮",
        Some("Office") => "📄",
        Some("Utility") => "🔧",
        Some("System") => "⚙️",
        _ => "📦",
    }
}
```

---

## Implementation Notes

### Terminal Image Protocols

| Protocol | Terminals |
|----------|-----------|
| Kitty Graphics | kitty |
| iTerm2 Inline | iTerm2, WezTerm |
| Sixel | foot, mlterm, xterm |
| None | alacritty, most others |

### Detection

```rust
pub enum ImageProtocol {
    Kitty,
    ITerm2,
    Sixel,
    None,
}

pub fn detect_image_protocol() -> ImageProtocol {
    // Check TERM and TERM_PROGRAM
    if std::env::var("KITTY_WINDOW_ID").is_ok() {
        return ImageProtocol::Kitty;
    }
    
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        if term_program == "iTerm.app" || term_program == "WezTerm" {
            return ImageProtocol::ITerm2;
        }
    }
    
    // Sixel detection is more complex
    // Would need to query terminal
    
    ImageProtocol::None
}
```

### Ratatui Image Widget

Using `ratatui-image` crate:

```toml
[dependencies]
ratatui-image = { version = "1.0", optional = true }
image = { version = "0.25", optional = true }

[features]
icons = ["ratatui-image", "image"]
```

```rust
#[cfg(feature = "icons")]
fn render_icon(f: &mut Frame, area: Rect, icon_path: &Path) {
    use ratatui_image::{Image, Resize};
    
    if let Ok(img) = image::open(icon_path) {
        let image = Image::new(&img)
            .resize(Resize::Fit);
        f.render_widget(image, area);
    }
}
```

### Layout with Icons

```
┌─────────────────────────────┐
│ > search                    │
├─────────────────────────────┤
│ [🦊] Firefox                │
│ [📁] Files                  │
│ [⚙️] Settings               │
└─────────────────────────────┘
```

Icon area: 2-3 characters wide (for emoji/text fallback) or 16-32px (for images).

---

## Configuration

```toml
[ui]
# Enable icons
icons = true

# Icon size in pixels (for image protocols)
icon_size = 16

# Fallback mode: "emoji", "ascii", "none"
icon_fallback = "emoji"

# Force specific protocol (auto-detect if not set)
# icon_protocol = "sixel"
```

---

## Acceptance Criteria

- [x] Icons load from freedesktop icon theme (prepared, not yet rendered)
- [ ] Kitty graphics protocol works (if in kitty) - prepared, needs `--features graphics`
- [x] Emoji fallback works in all terminals
- [x] No crashes if icon not found
- [x] Feature can be disabled at compile time (`--features graphics`)

---

## Testing

### Unit Tests

```rust
#[test]
fn test_icon_lookup() {
    // May fail if no icon theme installed
    let path = find_icon("firefox", 16);
    // Just verify no panic
}

#[test]
fn test_icon_fallback() {
    let entry = DesktopEntry::new("Firefox").with_category("Network");
    assert_eq!(icon_fallback(&entry), "🌐");
}
```

### Manual Tests

1. Run in kitty - should show actual icons
2. Run in alacritty - should show emoji fallback
3. Run with `icons = false` - should show no icons
4. Entry with missing icon - should use fallback

---

## Caveats

- **Performance:** Loading icons can be slow; consider caching
- **Theme issues:** Icon themes vary; may need fallback chain
- **Terminal support:** Most terminals don't support images
- **Feature flag:** Make this optional to reduce binary size

---

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

---

## Nerd Font Alternative

For terminals without graphics support, use Nerd Font icons:

```rust
fn icon_to_nerd_font(categories: &[String]) -> &'static str {
    match categories.first().map(|s| s.as_str()) {
        Some("Development") => "\u{f121}",  // 
        Some("Network") => "\u{f0ac}",      // 
        Some("Utility") => "\u{f0ad}",      // 
        Some("System") => "\u{f085}",       // 
        Some("Audio") => "\u{f001}",        // 
        Some("Video") => "\u{f03d}",        // 
        Some("Graphics") => "\u{f1fc}",     // 
        Some("Game") => "\u{f11b}",         // 
        _ => "\u{f15b}",                    //  (default file)
    }
}
```

---

## Related Units

- **Depends on:** None (optional feature)
- **Related:** Unit 5.4 (Theming - icon colors)
