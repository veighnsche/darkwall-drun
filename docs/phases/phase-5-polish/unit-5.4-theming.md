# Unit 5.4: Theming

> **Phase:** 5 - Polish & Features  
> **Complexity:** Low  
> **Skills:** Configuration, UI styling

---

## Objective

Implement customizable color schemes and border styles.

---

## Tasks

### 1. Color Scheme Configuration

```rust
#[derive(Deserialize)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub border: Color,
    pub search_highlight: Color,
    pub exit_success: Color,
    pub exit_failure: Color,
}
```

### 2. Match System Theme (dark/light)

Detect system preference and apply appropriate theme.

### 3. Border Styles

```rust
#[derive(Deserialize)]
pub enum BorderStyle {
    None,
    Plain,
    Rounded,
    Double,
    Thick,
}
```

---

## Implementation Notes

### Default Themes

```rust
impl Theme {
    pub fn dark() -> Self {
        Self {
            background: Color::Rgb(30, 30, 46),      // Catppuccin base
            foreground: Color::Rgb(205, 214, 244),   // Catppuccin text
            selection_bg: Color::Rgb(69, 71, 90),    // Catppuccin surface1
            selection_fg: Color::Rgb(205, 214, 244),
            border: Color::Rgb(88, 91, 112),         // Catppuccin overlay0
            search_highlight: Color::Rgb(249, 226, 175), // Catppuccin yellow
            exit_success: Color::Rgb(166, 227, 161), // Catppuccin green
            exit_failure: Color::Rgb(243, 139, 168), // Catppuccin red
        }
    }
    
    pub fn light() -> Self {
        Self {
            background: Color::Rgb(239, 241, 245),   // Catppuccin latte base
            foreground: Color::Rgb(76, 79, 105),     // Catppuccin latte text
            selection_bg: Color::Rgb(188, 192, 204),
            selection_fg: Color::Rgb(76, 79, 105),
            border: Color::Rgb(140, 143, 161),
            search_highlight: Color::Rgb(223, 142, 29),
            exit_success: Color::Rgb(64, 160, 43),
            exit_failure: Color::Rgb(210, 15, 57),
        }
    }
}
```

### Configuration File

```toml
# ~/.config/darkwall-drun/theme.toml

[colors]
background = "#1e1e2e"
foreground = "#cdd6f4"
selection_bg = "#45475a"
selection_fg = "#cdd6f4"
border = "#585b70"
search_highlight = "#f9e2af"
exit_success = "#a6e3a1"
exit_failure = "#f38ba8"

[style]
border = "rounded"  # none, plain, rounded, double, thick
```

### System Theme Detection

```rust
fn detect_system_theme() -> ThemeMode {
    // Check common environment variables
    if let Ok(theme) = std::env::var("GTK_THEME") {
        if theme.to_lowercase().contains("dark") {
            return ThemeMode::Dark;
        }
    }
    
    // Check color scheme portal (freedesktop)
    // This would require D-Bus, so simplified here
    
    // Default to dark
    ThemeMode::Dark
}
```

### Applying Theme

```rust
impl App {
    fn style_for(&self, element: UiElement) -> Style {
        match element {
            UiElement::Normal => Style::default()
                .fg(self.theme.foreground)
                .bg(self.theme.background),
            UiElement::Selected => Style::default()
                .fg(self.theme.selection_fg)
                .bg(self.theme.selection_bg),
            UiElement::Border => Style::default()
                .fg(self.theme.border),
            UiElement::SearchMatch => Style::default()
                .fg(self.theme.search_highlight)
                .add_modifier(Modifier::BOLD),
            // ...
        }
    }
}
```

### Border Style Mapping

```rust
fn border_set(style: BorderStyle) -> symbols::border::Set {
    match style {
        BorderStyle::None => symbols::border::PLAIN,  // Will be hidden
        BorderStyle::Plain => symbols::border::PLAIN,
        BorderStyle::Rounded => symbols::border::ROUNDED,
        BorderStyle::Double => symbols::border::DOUBLE,
        BorderStyle::Thick => symbols::border::THICK,
    }
}
```

---

## Built-in Themes

| Theme | Description |
|-------|-------------|
| `dark` | Catppuccin Mocha (default) |
| `light` | Catppuccin Latte |
| `nord` | Nord color palette |
| `gruvbox` | Gruvbox dark |
| `dracula` | Dracula theme |

### Using Built-in Theme

```toml
[theme]
preset = "nord"  # Use built-in theme

# Override specific colors
[theme.colors]
selection_bg = "#88c0d0"
```

---

## Configuration

```toml
[theme]
# Use preset theme
preset = "dark"  # dark, light, nord, gruvbox, dracula

# Or use system theme
# preset = "auto"

# Custom colors override preset
[theme.colors]
# ... color overrides

[theme.style]
border = "rounded"
```

---

## Acceptance Criteria

- [ ] Default dark theme looks good
- [ ] Light theme available
- [ ] Custom colors via config file
- [ ] Border style configurable
- [ ] System theme detection works
- [ ] Invalid colors handled gracefully

---

## Testing

### Unit Tests

```rust
#[test]
fn test_parse_hex_color() {
    let color = parse_color("#ff5500").unwrap();
    assert_eq!(color, Color::Rgb(255, 85, 0));
}

#[test]
fn test_theme_loading() {
    let toml = r#"
        [colors]
        background = "#000000"
        foreground = "#ffffff"
    "#;
    let theme: Theme = toml::from_str(toml).unwrap();
    assert_eq!(theme.background, Color::Rgb(0, 0, 0));
}
```

### Manual Tests

1. Run with default theme - should look good
2. Set `preset = "light"` - should switch to light
3. Override single color - should apply
4. Invalid color value - should use default

---

## Color Format Support

| Format | Example |
|--------|---------|
| Hex | `#ff5500` |
| RGB | `rgb(255, 85, 0)` |
| Named | `red`, `blue`, `green` |
| ANSI | `0-255` |

---

## Related Units

- **Depends on:** None
- **Related:** Unit 5.3 (Icons - theme affects icon display)
