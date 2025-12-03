use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::ui::theme::{parse_hex_color, Theme};
use crate::ui::layout::GridLayout;
use crate::ui::entry_card::EntryDisplayConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub desktop_entry_dirs: Vec<PathBuf>,
    pub appearance: AppearanceConfig,
    pub theme: ThemeConfig,
    pub niri: NiriConfig,
    pub behavior: BehaviorConfig,
    pub history: HistoryConfig,
    pub icons: IconsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub prompt: String,
    pub selected_prefix: String,
    pub unselected_prefix: String,
    /// Number of columns in the grid layout
    pub columns: u16,
    /// Number of visible rows in the grid layout
    pub visible_rows: u16,
    /// Entry display configuration
    pub entry: EntryDisplayConfigToml,
}

/// TEAM_004: Entry display configuration (TOML-friendly)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EntryDisplayConfigToml {
    /// Show GenericName line
    pub show_generic: bool,
    /// Show Comment line
    pub show_comment: bool,
    /// Show Categories line
    pub show_categories: bool,
}

impl Default for EntryDisplayConfigToml {
    fn default() -> Self {
        Self {
            show_generic: true,
            show_comment: true,
            show_categories: true,
        }
    }
}

impl From<&EntryDisplayConfigToml> for EntryDisplayConfig {
    fn from(toml: &EntryDisplayConfigToml) -> Self {
        Self {
            show_generic: toml.show_generic,
            show_comment: toml.show_comment,
            show_categories: toml.show_categories,
        }
    }
}

/// TEAM_004: Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ThemeConfig {
    /// Use a preset theme as base (darkwall, catppuccin-mocha, catppuccin-latte, nord, gruvbox)
    pub preset: Option<String>,
    /// Custom color overrides
    pub colors: ThemeColors,
}

/// TEAM_004: Custom theme color overrides
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NiriConfig {
    pub enabled: bool,
    pub socket_path: Option<PathBuf>,
    pub float_on_idle: bool,
    pub unfloat_on_execute: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    /// What to do after command exits: "return", "close", "prompt"
    pub after_command: String,
    /// Number of output lines to preserve when returning to launcher
    pub preserve_output_lines: usize,
    /// Show categories in entry list
    pub show_categories: bool,
    /// Show generic name below entry name
    pub show_generic_name: bool,
}

/// TEAM_001: History/frecency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HistoryConfig {
    /// Enable frecency sorting
    pub enabled: bool,
    /// Maximum entries to track
    pub max_entries: usize,
    /// Decay old entries after N days
    pub decay_after_days: u64,
    /// Weight of frecency vs fuzzy match (0.0 - 1.0)
    pub frecency_weight: f64,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 1000,
            decay_after_days: 90,
            frecency_weight: 0.3,
        }
    }
}

/// TEAM_002: Icons configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IconsConfig {
    /// Enable icon display
    pub enabled: bool,
    /// Icon size in pixels (for graphics protocols)
    pub size: u16,
    /// Fallback mode: "none" (graphics only)
    #[allow(dead_code)]
    pub fallback: String,
    /// Force icons over SSH (normally disabled)
    pub force_over_ssh: bool,
}

impl Default for IconsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 32,
            fallback: "none".to_string(),
            force_over_ssh: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            desktop_entry_dirs: vec![
                home.join(".local/share/applications"),
                PathBuf::from("/run/current-system/sw/share/applications"),
                PathBuf::from("/usr/share/applications"),
            ],
            appearance: AppearanceConfig::default(),
            theme: ThemeConfig::default(),
            niri: NiriConfig::default(),
            behavior: BehaviorConfig::default(),
            history: HistoryConfig::default(),
            icons: IconsConfig::default(),
        }
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            prompt: "❯ ".to_string(),
            selected_prefix: "● ".to_string(),
            unselected_prefix: "  ".to_string(),
            columns: 2,
            visible_rows: 5,
            entry: EntryDisplayConfigToml::default(),
        }
    }
}

impl Default for NiriConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            socket_path: None,
            float_on_idle: true,
            unfloat_on_execute: true,
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            after_command: "return".to_string(),
            preserve_output_lines: 10,
            show_categories: true,
            show_generic_name: true,
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let expanded = shellexpand::tilde(path);
        let path = Path::new(expanded.as_ref());

        if path.exists() {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read config from {}", path.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse config from {}", path.display()))
        } else {
            tracing::info!("Config file not found, using defaults");
            Ok(Self::default())
        }
    }

    /// TEAM_004: Resolve theme from preset + color overrides
    pub fn resolve_theme(&self) -> Theme {
        // Start with preset or default
        let mut theme = self.theme.preset
            .as_ref()
            .and_then(|name| Theme::from_preset(name))
            .unwrap_or_default();

        // Apply color overrides
        let colors = &self.theme.colors;
        
        if let Some(ref c) = colors.background {
            if let Ok(color) = parse_hex_color(c) {
                theme.background = color;
            } else {
                tracing::warn!("Invalid background color: {}", c);
            }
        }
        if let Some(ref c) = colors.foreground {
            if let Ok(color) = parse_hex_color(c) {
                theme.foreground = color;
            } else {
                tracing::warn!("Invalid foreground color: {}", c);
            }
        }
        if let Some(ref c) = colors.selection_bg {
            if let Ok(color) = parse_hex_color(c) {
                theme.selection_bg = color;
            } else {
                tracing::warn!("Invalid selection_bg color: {}", c);
            }
        }
        if let Some(ref c) = colors.selection_fg {
            if let Ok(color) = parse_hex_color(c) {
                theme.selection_fg = color;
            } else {
                tracing::warn!("Invalid selection_fg color: {}", c);
            }
        }
        if let Some(ref c) = colors.accent {
            if let Ok(color) = parse_hex_color(c) {
                theme.accent = color;
            } else {
                tracing::warn!("Invalid accent color: {}", c);
            }
        }
        if let Some(ref c) = colors.dimmed {
            if let Ok(color) = parse_hex_color(c) {
                theme.dimmed = color;
            } else {
                tracing::warn!("Invalid dimmed color: {}", c);
            }
        }
        if let Some(ref c) = colors.dimmed_alt {
            if let Ok(color) = parse_hex_color(c) {
                theme.dimmed_alt = color;
            } else {
                tracing::warn!("Invalid dimmed_alt color: {}", c);
            }
        }
        if let Some(ref c) = colors.search_highlight {
            if let Ok(color) = parse_hex_color(c) {
                theme.search_highlight = color;
            } else {
                tracing::warn!("Invalid search_highlight color: {}", c);
            }
        }
        if let Some(ref c) = colors.exit_success {
            if let Ok(color) = parse_hex_color(c) {
                theme.exit_success = color;
            } else {
                tracing::warn!("Invalid exit_success color: {}", c);
            }
        }
        if let Some(ref c) = colors.exit_failure {
            if let Ok(color) = parse_hex_color(c) {
                theme.exit_failure = color;
            } else {
                tracing::warn!("Invalid exit_failure color: {}", c);
            }
        }

        theme
    }

    /// TEAM_004: Get grid layout from config
    pub fn grid_layout(&self) -> GridLayout {
        GridLayout::new(self.appearance.columns, self.appearance.visible_rows)
    }

    /// TEAM_004: Get entry display config
    pub fn entry_display_config(&self) -> EntryDisplayConfig {
        EntryDisplayConfig::from(&self.appearance.entry)
    }
}
