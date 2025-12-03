use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub desktop_entry_dirs: Vec<PathBuf>,
    pub appearance: AppearanceConfig,
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
    /// Fallback mode: "emoji", "nerd", "ascii", "none"
    pub fallback: String,
    /// Force icons over SSH (normally disabled)
    pub force_over_ssh: bool,
}

impl Default for IconsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 32,
            fallback: "emoji".to_string(),
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
}
