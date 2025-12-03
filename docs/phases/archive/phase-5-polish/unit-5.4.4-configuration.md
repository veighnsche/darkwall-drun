# Unit 5.4.4: Theme Configuration

> **Parent:** Unit 5.4 (Theming)  
> **Complexity:** Low  
> **Skills:** TOML parsing, serde, config merging  
> **Status:** ✅ COMPLETE - config.rs updated by TEAM_004

---

## Objective

Load theme and layout configuration from TOML file with preset support and custom overrides.

---

## Config File Location

```
~/.config/darkwall-drun/config.toml
```

---

## Full Configuration Schema

```toml
# ~/.config/darkwall-drun/config.toml

[appearance]
prompt = "❯ "
columns = 2
visible_rows = 5

[appearance.entry]
show_generic = true
show_comment = true
show_categories = true

[theme]
# Use a preset theme as base
preset = "darkwall"  # darkwall, catppuccin-mocha, catppuccin-latte, nord, gruvbox

# Override specific colors (optional)
[theme.colors]
background = "#0d1116"
foreground = "#e5eaf1"
selection_bg = "#141c2a"
selection_fg = "#e5eaf1"
accent = "#b45309"
dimmed = "#9ca3af"
dimmed_alt = "#6b7280"
search_highlight = "#b45309"
exit_success = "#22c55e"
exit_failure = "#ef4444"

[theme.style]
border = "rounded"  # none, plain, rounded, double, thick
```

---

## Implementation

### Config Structs

```rust
#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub appearance: AppearanceConfig,
    pub theme: ThemeConfig,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub prompt: String,
    pub columns: u16,
    pub visible_rows: u16,
    pub entry: EntryDisplayConfig,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            prompt: "❯ ".into(),
            columns: 2,
            visible_rows: 5,
            entry: EntryDisplayConfig::default(),
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct EntryDisplayConfig {
    pub show_generic: bool,
    pub show_comment: bool,
    pub show_categories: bool,
}

impl Default for EntryDisplayConfig {
    fn default() -> Self {
        Self {
            show_generic: true,
            show_comment: true,
            show_categories: true,
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ThemeConfig {
    pub preset: Option<String>,
    pub colors: Option<ThemeColors>,
    pub style: ThemeStyle,
}

#[derive(Deserialize)]
pub struct ThemeColors {
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub selection_bg: Option<String>,
    pub selection_fg: Option<String>,
    pub accent: Option<String>,
    pub dimmed: Option<String>,
    pub dimmed_alt: Option<String>,
    pub search_highlight: Option<String>,
    pub exit_success: Option<String>,
    pub exit_failure: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ThemeStyle {
    pub border: BorderStyle,
}
```

### Config Loading

```rust
impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let path = dirs::config_dir()
            .ok_or(ConfigError::NoConfigDir)?
            .join("darkwall-drun/config.toml");

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Resolve theme from preset + overrides
    pub fn resolve_theme(&self) -> Theme {
        // Start with preset or default
        let mut theme = self.theme.preset
            .as_ref()
            .and_then(|name| Theme::from_preset(name))
            .unwrap_or_default();

        // Apply color overrides
        if let Some(colors) = &self.theme.colors {
            if let Some(c) = &colors.background {
                if let Ok(color) = parse_hex_color(c) {
                    theme.background = color;
                }
            }
            // ... repeat for each color field
        }

        theme
    }
}
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Could not find config directory")]
    NoConfigDir,
    
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
    
    #[error("Invalid color value: {0}")]
    InvalidColor(String),
}
```

---

## Validation

| Field | Validation |
|-------|------------|
| `columns` | 1-10, default 2 |
| `visible_rows` | 1-20, default 5 |
| `colors.*` | Valid hex color or skip |
| `preset` | Known preset name or ignore |
| `border` | Valid enum variant or default |

### Graceful Degradation

Invalid values should log a warning and use defaults, not fail:

```rust
fn parse_columns(value: u16) -> u16 {
    if value == 0 || value > 10 {
        tracing::warn!("Invalid columns value {}, using default 2", value);
        2
    } else {
        value
    }
}
```

---

## CLI Overrides

Config can be overridden via CLI flags:

```bash
drun --columns 3 --rows 4 --theme nord
```

Priority: CLI > Config file > Defaults

---

## Acceptance Criteria

- [ ] Config loads from `~/.config/darkwall-drun/config.toml`
- [ ] Missing config file uses defaults
- [ ] Preset themes load correctly
- [ ] Color overrides apply on top of preset
- [ ] Invalid values use defaults (don't crash)
- [ ] CLI flags override config file
- [ ] Layout config (columns, rows) works
- [ ] Entry display toggles work
