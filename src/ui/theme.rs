//! Theme system for DRUN
//!
//! Unit 5.4.3: Color System
//!
//! Provides:
//! - Theme struct with all UI colors
//! - Built-in presets (darkwall, catppuccin, nord, gruvbox)
//! - Hex color parsing
//! - 256-color fallback

use ratatui::style::Color;

/// Theme colors for the UI
#[derive(Debug, Clone)]
pub struct Theme {
    /// Main background color
    pub background: Color,
    /// Primary text color
    pub foreground: Color,
    /// Background for selected items
    pub selection_bg: Color,
    /// Text color for selected items
    pub selection_fg: Color,
    /// Accent color (borders, highlights)
    pub accent: Color,
    /// Dimmed text (comments, secondary info)
    pub dimmed: Color,
    /// More dimmed text (categories, tertiary info)
    pub dimmed_alt: Color,
    /// Search/filter highlight color
    pub search_highlight: Color,
    /// Success status color (exit code 0)
    pub exit_success: Color,
    /// Failure status color (non-zero exit)
    pub exit_failure: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::darkwall()
    }
}

impl Theme {
    /// Darkwall theme - default, based on rofi config
    pub fn darkwall() -> Self {
        Self {
            background: Color::Rgb(13, 17, 22),       // #0d1116
            foreground: Color::Rgb(229, 234, 241),    // #e5eaf1
            selection_bg: Color::Rgb(20, 28, 42),     // #141c2a
            selection_fg: Color::Rgb(229, 234, 241),  // #e5eaf1
            accent: Color::Rgb(180, 83, 9),           // #b45309 (amber)
            dimmed: Color::Rgb(156, 163, 175),        // #9ca3af
            dimmed_alt: Color::Rgb(107, 114, 128),    // #6b7280
            search_highlight: Color::Rgb(180, 83, 9), // #b45309
            exit_success: Color::Rgb(34, 197, 94),    // #22c55e
            exit_failure: Color::Rgb(239, 68, 68),    // #ef4444
        }
    }

    /// Catppuccin Mocha theme
    pub fn catppuccin_mocha() -> Self {
        Self {
            background: Color::Rgb(30, 30, 46),       // #1e1e2e (base)
            foreground: Color::Rgb(205, 214, 244),    // #cdd6f4 (text)
            selection_bg: Color::Rgb(49, 50, 68),     // #313244 (surface0)
            selection_fg: Color::Rgb(205, 214, 244),  // #cdd6f4 (text)
            accent: Color::Rgb(137, 180, 250),        // #89b4fa (blue)
            dimmed: Color::Rgb(166, 173, 200),        // #a6adc8 (subtext0)
            dimmed_alt: Color::Rgb(147, 153, 178),    // #9399b2 (overlay2)
            search_highlight: Color::Rgb(249, 226, 175), // #f9e2af (yellow)
            exit_success: Color::Rgb(166, 227, 161),  // #a6e3a1 (green)
            exit_failure: Color::Rgb(243, 139, 168),  // #f38ba8 (red)
        }
    }

    /// Catppuccin Latte theme (light)
    pub fn catppuccin_latte() -> Self {
        Self {
            background: Color::Rgb(239, 241, 245),    // #eff1f5 (base)
            foreground: Color::Rgb(76, 79, 105),      // #4c4f69 (text)
            selection_bg: Color::Rgb(204, 208, 218),  // #ccd0da (surface0)
            selection_fg: Color::Rgb(76, 79, 105),    // #4c4f69 (text)
            accent: Color::Rgb(30, 102, 245),         // #1e66f5 (blue)
            dimmed: Color::Rgb(108, 111, 133),        // #6c6f85 (subtext0)
            dimmed_alt: Color::Rgb(140, 143, 161),    // #8c8fa1 (overlay2)
            search_highlight: Color::Rgb(223, 142, 29), // #df8e1d (yellow)
            exit_success: Color::Rgb(64, 160, 43),    // #40a02b (green)
            exit_failure: Color::Rgb(210, 15, 57),    // #d20f39 (red)
        }
    }

    /// Nord theme
    pub fn nord() -> Self {
        Self {
            background: Color::Rgb(46, 52, 64),       // #2e3440 (nord0)
            foreground: Color::Rgb(236, 239, 244),    // #eceff4 (nord6)
            selection_bg: Color::Rgb(67, 76, 94),     // #434c5e (nord2)
            selection_fg: Color::Rgb(236, 239, 244),  // #eceff4 (nord6)
            accent: Color::Rgb(136, 192, 208),        // #88c0d0 (nord8)
            dimmed: Color::Rgb(216, 222, 233),        // #d8dee9 (nord4)
            dimmed_alt: Color::Rgb(76, 86, 106),      // #4c566a (nord3)
            search_highlight: Color::Rgb(235, 203, 139), // #ebcb8b (nord13)
            exit_success: Color::Rgb(163, 190, 140),  // #a3be8c (nord14)
            exit_failure: Color::Rgb(191, 97, 106),   // #bf616a (nord11)
        }
    }

    /// Gruvbox dark theme
    pub fn gruvbox() -> Self {
        Self {
            background: Color::Rgb(40, 40, 40),       // #282828 (bg)
            foreground: Color::Rgb(235, 219, 178),    // #ebdbb2 (fg)
            selection_bg: Color::Rgb(60, 56, 54),     // #3c3836 (bg1)
            selection_fg: Color::Rgb(235, 219, 178),  // #ebdbb2 (fg)
            accent: Color::Rgb(215, 153, 33),         // #d79921 (yellow)
            dimmed: Color::Rgb(168, 153, 132),        // #a89984 (gray)
            dimmed_alt: Color::Rgb(146, 131, 116),    // #928374 (gray)
            search_highlight: Color::Rgb(250, 189, 47), // #fabd2f (bright yellow)
            exit_success: Color::Rgb(152, 151, 26),   // #98971a (green)
            exit_failure: Color::Rgb(204, 36, 29),    // #cc241d (red)
        }
    }

    /// Load theme from preset name
    pub fn from_preset(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "darkwall" | "default" => Some(Self::darkwall()),
            "catppuccin-mocha" | "catppuccin_mocha" | "catppuccin" => Some(Self::catppuccin_mocha()),
            "catppuccin-latte" | "catppuccin_latte" => Some(Self::catppuccin_latte()),
            "nord" => Some(Self::nord()),
            "gruvbox" | "gruvbox-dark" | "gruvbox_dark" => Some(Self::gruvbox()),
            _ => None,
        }
    }

    /// Convert to 256-color approximation for limited terminals
    #[allow(dead_code)]
    pub fn to_256_color(&self) -> Self {
        Self {
            background: approximate_256(self.background),
            foreground: approximate_256(self.foreground),
            selection_bg: approximate_256(self.selection_bg),
            selection_fg: approximate_256(self.selection_fg),
            accent: approximate_256(self.accent),
            dimmed: approximate_256(self.dimmed),
            dimmed_alt: approximate_256(self.dimmed_alt),
            search_highlight: approximate_256(self.search_highlight),
            exit_success: approximate_256(self.exit_success),
            exit_failure: approximate_256(self.exit_failure),
        }
    }
}

/// Parse hex color string to Color
/// Supports: #rrggbb, #rgb, rrggbb, rgb
pub fn parse_hex_color(s: &str) -> Result<Color, ColorError> {
    let s = s.trim().trim_start_matches('#');
    
    match s.len() {
        // #rgb -> #rrggbb
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).map_err(|_| ColorError::InvalidHex)?;
            let g = u8::from_str_radix(&s[1..2], 16).map_err(|_| ColorError::InvalidHex)?;
            let b = u8::from_str_radix(&s[2..3], 16).map_err(|_| ColorError::InvalidHex)?;
            Ok(Color::Rgb(r * 17, g * 17, b * 17))
        }
        // #rrggbb
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| ColorError::InvalidHex)?;
            let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| ColorError::InvalidHex)?;
            let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| ColorError::InvalidHex)?;
            Ok(Color::Rgb(r, g, b))
        }
        // #rrggbbaa (alpha ignored)
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| ColorError::InvalidHex)?;
            let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| ColorError::InvalidHex)?;
            let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| ColorError::InvalidHex)?;
            // Alpha (s[6..8]) ignored for TUI
            Ok(Color::Rgb(r, g, b))
        }
        _ => Err(ColorError::InvalidLength),
    }
}

/// Color parsing error
#[derive(Debug, Clone, PartialEq)]
pub enum ColorError {
    InvalidLength,
    InvalidHex,
}

impl std::fmt::Display for ColorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorError::InvalidLength => write!(f, "invalid color length (expected 3, 6, or 8 hex chars)"),
            ColorError::InvalidHex => write!(f, "invalid hex character"),
        }
    }
}

impl std::error::Error for ColorError {}

/// Approximate RGB color to nearest 256-color palette entry
fn approximate_256(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            // 6x6x6 color cube starts at index 16
            // Each axis: 0, 95, 135, 175, 215, 255 -> indices 0-5
            let r_idx = if r < 48 { 0 } else { (r - 35) / 40 };
            let g_idx = if g < 48 { 0 } else { (g - 35) / 40 };
            let b_idx = if b < 48 { 0 } else { (b - 35) / 40 };
            let idx = 16 + 36 * r_idx + 6 * g_idx + b_idx;
            Color::Indexed(idx)
        }
        c => c,
    }
}

/// Serde deserializer for hex colors
#[allow(dead_code)] // Public API for serde(deserialize_with) usage
pub mod serde_color {
    use super::*;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_hex_color(&s).map_err(serde::de::Error::custom)
    }

    pub fn deserialize_option<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => parse_hex_color(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_6() {
        assert_eq!(parse_hex_color("#ff0000"), Ok(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_hex_color("00ff00"), Ok(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_hex_color("#0d1116"), Ok(Color::Rgb(13, 17, 22)));
    }

    #[test]
    fn test_parse_hex_3() {
        assert_eq!(parse_hex_color("#f00"), Ok(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_hex_color("0f0"), Ok(Color::Rgb(0, 255, 0)));
    }

    #[test]
    fn test_parse_hex_8() {
        assert_eq!(parse_hex_color("#ff0000ff"), Ok(Color::Rgb(255, 0, 0)));
    }

    #[test]
    fn test_parse_hex_invalid() {
        assert!(parse_hex_color("invalid").is_err());
        assert!(parse_hex_color("#gg0000").is_err());
        assert!(parse_hex_color("#ff00").is_err());
    }

    #[test]
    fn test_presets() {
        assert!(Theme::from_preset("darkwall").is_some());
        assert!(Theme::from_preset("catppuccin-mocha").is_some());
        assert!(Theme::from_preset("nord").is_some());
        assert!(Theme::from_preset("gruvbox").is_some());
        assert!(Theme::from_preset("nonexistent").is_none());
    }
}
