# Unit 5.4.3: Color System

> **Parent:** Unit 5.4 (Theming)  
> **Complexity:** Low  
> **Skills:** Color parsing, serde  
> **Status:** ✅ COMPLETE - Integrated into draw.rs by TEAM_004

---

## Objective

Implement the Theme struct with color presets and hex color parsing.

---

## Theme Struct

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub accent: Color,
    pub dimmed: Color,
    pub dimmed_alt: Color,
    pub search_highlight: Color,
    pub exit_success: Color,
    pub exit_failure: Color,
}
```

---

## Default Theme (Darkwall)

Based on rofi config:

```rust
impl Default for Theme {
    fn default() -> Self {
        Self::darkwall()
    }
}

impl Theme {
    pub fn darkwall() -> Self {
        Self {
            background: Color::Rgb(13, 17, 22),       // #0d1116
            foreground: Color::Rgb(229, 234, 241),    // #e5eaf1
            selection_bg: Color::Rgb(20, 28, 42),     // #141c2a
            selection_fg: Color::Rgb(229, 234, 241),  // #e5eaf1
            accent: Color::Rgb(180, 83, 9),           // #b45309
            dimmed: Color::Rgb(156, 163, 175),        // #9ca3af
            dimmed_alt: Color::Rgb(107, 114, 128),    // #6b7280
            search_highlight: Color::Rgb(180, 83, 9), // #b45309
            exit_success: Color::Rgb(34, 197, 94),
            exit_failure: Color::Rgb(239, 68, 68),
        }
    }
}
```

---

## Built-in Presets

| Preset | Description |
|--------|-------------|
| `darkwall` | Default (from rofi config) |
| `catppuccin-mocha` | Catppuccin dark |
| `catppuccin-latte` | Catppuccin light |
| `nord` | Nord palette |
| `gruvbox` | Gruvbox dark |

```rust
impl Theme {
    pub fn from_preset(name: &str) -> Option<Self> {
        match name {
            "darkwall" => Some(Self::darkwall()),
            "catppuccin-mocha" => Some(Self::catppuccin_mocha()),
            "catppuccin-latte" => Some(Self::catppuccin_latte()),
            "nord" => Some(Self::nord()),
            "gruvbox" => Some(Self::gruvbox()),
            _ => None,
        }
    }
}
```

---

## Hex Color Parsing

```rust
pub fn parse_hex_color(s: &str) -> Result<Color, ColorError> {
    let s = s.trim_start_matches('#');
    
    if s.len() != 6 && s.len() != 8 {
        return Err(ColorError::InvalidLength);
    }
    
    let r = u8::from_str_radix(&s[0..2], 16)?;
    let g = u8::from_str_radix(&s[2..4], 16)?;
    let b = u8::from_str_radix(&s[4..6], 16)?;
    // Alpha ignored for TUI
    
    Ok(Color::Rgb(r, g, b))
}
```

### Serde Integration

```rust
fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_hex_color(&s).map_err(serde::de::Error::custom)
}
```

---

## Terminal Fallback

For terminals without true color:

```rust
impl Theme {
    pub fn to_256_color(&self) -> Self {
        Self {
            background: approximate_256(self.background),
            foreground: approximate_256(self.foreground),
            // ... etc
        }
    }
}

fn approximate_256(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            // Convert to nearest 256-color
            let idx = 16 + 36 * (r / 51) + 6 * (g / 51) + (b / 51);
            Color::Indexed(idx)
        }
        c => c,
    }
}
```

---

## Acceptance Criteria

- [ ] Theme struct with all required colors
- [ ] Darkwall preset matches rofi config
- [ ] Hex color parsing works (`#rrggbb`)
- [ ] Invalid colors return error (not panic)
- [ ] Preset loading by name
- [ ] 256-color fallback available
