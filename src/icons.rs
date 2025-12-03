//! Icon loading and display support.
//!
//! TEAM_002: Phase 5, Unit 5.3 - Icons
//!
//! Supports:
//! - Kitty graphics protocol (kitty terminal)
//! - Sixel graphics (foot, mlterm, xterm)
//! - iTerm2 protocol (iTerm2, WezTerm)
//!
//! NO FALLBACKS. Either real images or nothing.

use image::DynamicImage;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::Mutex;

/// Icon manager - handles icon loading and caching
/// Must be initialized once at startup before entering raw mode
pub struct IconManager {
    /// The picker determines the graphics protocol and font size
    picker: Option<Picker>,
    /// Cache of loaded icon protocols by entry ID
    cache: HashMap<String, Arc<Mutex<StatefulProtocol>>>,
    /// Failed icon lookups (don't retry)
    failed: std::collections::HashSet<String>,
    /// Icon size in pixels
    icon_size: u16,
    /// Whether graphics are supported
    graphics_supported: bool,
    /// Icon theme search paths and themes
    icon_lookup: IconLookup,
}

impl IconManager {
    /// Create a new icon manager by querying the terminal
    /// MUST be called before entering raw mode / alternate screen
    pub fn new(icon_size: u16) -> Self {
        // Try to create a picker by querying the terminal
        let picker = match Picker::from_query_stdio() {
            Ok(p) => {
                tracing::info!("Graphics protocol detected: {:?}", p.protocol_type());
                Some(p)
            }
            Err(e) => {
                tracing::debug!("No graphics protocol available: {}", e);
                None
            }
        };

        let graphics_supported = picker.is_some();
        let icon_lookup = IconLookup::new();
        
        tracing::info!("Icon theme: {}, search paths: {}", 
            icon_lookup.theme, 
            icon_lookup.search_paths.len()
        );

        Self {
            picker,
            cache: HashMap::new(),
            failed: std::collections::HashSet::new(),
            icon_size,
            graphics_supported,
            icon_lookup,
        }
    }

    /// Check if graphics icons are supported
    pub fn supports_graphics(&self) -> bool {
        self.graphics_supported
    }

    /// Get a cached icon protocol (non-blocking, for rendering)
    pub fn get_cached(&self, entry_id: &str) -> Option<Arc<Mutex<StatefulProtocol>>> {
        self.cache.get(entry_id).cloned()
    }

    /// Try to load ONE icon that isn't cached yet (call once per frame to avoid blocking)
    /// Returns true if an icon was loaded, false if nothing to load
    pub fn try_load_one<'a>(&mut self, entries: impl Iterator<Item = (&'a str, Option<&'a str>)>) -> bool {
        let picker = match self.picker.as_mut() {
            Some(p) => p,
            None => return false,
        };

        for (entry_id, icon_name) in entries {
            // Skip if already cached or failed
            if self.cache.contains_key(entry_id) || self.failed.contains(entry_id) {
                continue;
            }

            let icon_name = match icon_name {
                Some(n) => n,
                None => {
                    self.failed.insert(entry_id.to_string());
                    continue;
                }
            };

            // Resolve icon path
            let icon_path = match self.icon_lookup.find_icon(icon_name, self.icon_size) {
                Some(p) => p,
                None => {
                    tracing::debug!("Icon not found: {}", icon_name);
                    self.failed.insert(entry_id.to_string());
                    continue;
                }
            };

            // Load the image with transparency support
            let dyn_img = match load_icon_image(&icon_path) {
                Some(img) => img,
                None => {
                    self.failed.insert(entry_id.to_string());
                    continue;
                }
            };

            // Create the protocol and cache it
            let protocol = picker.new_resize_protocol(dyn_img);
            let arc = Arc::new(Mutex::new(protocol));
            self.cache.insert(entry_id.to_string(), arc);
            
            return true; // Only load one per call
        }

        false
    }

    /// Clear the icon cache
    #[allow(dead_code)]
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

/// Load an icon image with proper format handling
fn load_icon_image(path: &Path) -> Option<DynamicImage> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    
    match ext.as_str() {
        "svg" => load_svg(path),
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" => {
            match image::open(path) {
                Ok(img) => Some(img),
                Err(e) => {
                    tracing::debug!("Failed to load {}: {}", path.display(), e);
                    None
                }
            }
        }
        _ => {
            tracing::debug!("Unsupported icon format: {}", ext);
            None
        }
    }
}

/// Load SVG and rasterize it
fn load_svg(path: &Path) -> Option<DynamicImage> {
    use std::fs;
    
    let svg_data = fs::read(path).ok()?;
    
    // Use resvg for SVG rendering
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(&svg_data, &options).ok()?;
    
    let size = tree.size();
    let width = size.width() as u32;
    let height = size.height() as u32;
    
    // Render at a larger size for better quality (128px)
    // ratatui-image will scale down as needed
    let target_size = 128.0;
    let scale = target_size / width.max(height) as f32;
    let scaled_width = (width as f32 * scale).ceil() as u32;
    let scaled_height = (height as f32 * scale).ceil() as u32;
    
    let mut pixmap = tiny_skia::Pixmap::new(scaled_width, scaled_height)?;
    
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    
    // Convert to image::DynamicImage
    let img = image::RgbaImage::from_raw(
        scaled_width,
        scaled_height,
        pixmap.take(),
    )?;
    
    Some(DynamicImage::ImageRgba8(img))
}

/// Icon lookup following freedesktop spec
struct IconLookup {
    /// Icon theme name (from GTK settings)
    theme: String,
    /// Search paths for icons
    search_paths: Vec<PathBuf>,
}

impl IconLookup {
    fn new() -> Self {
        let theme = detect_icon_theme().unwrap_or_else(|| "hicolor".to_string());
        let search_paths = get_icon_search_paths();
        
        tracing::debug!("Icon search paths: {:?}", search_paths);
        
        Self { theme, search_paths }
    }
    
    /// Find an icon by name, searching theme hierarchy
    fn find_icon(&self, name: &str, size: u16) -> Option<PathBuf> {
        // If it's an absolute path, use directly
        if name.starts_with('/') {
            let path = PathBuf::from(name);
            if path.exists() {
                return Some(path);
            }
            return None;
        }
        
        // Build theme search order: current theme -> parent themes -> hicolor
        let themes = self.get_theme_hierarchy();
        tracing::trace!("Looking for icon '{}' in themes: {:?}", name, themes);
        
        // Preferred sizes in order
        let sizes = [
            size.to_string(),
            "scalable".to_string(),
            "64".to_string(),
            "48".to_string(),
            "32".to_string(),
            "24".to_string(),
            "22".to_string(),
            "16".to_string(),
        ];
        
        // Extensions in preference order
        let extensions = ["svg", "png", "xpm"];
        
        // Search each theme
        for theme in &themes {
            for base_path in &self.search_paths {
                let theme_path = base_path.join(theme);
                if !theme_path.exists() {
                    continue;
                }
                
                // Try each size directory
                for size_str in &sizes {
                    // Common subdirectory patterns
                    let subdirs = [
                        format!("{}/apps", size_str),
                        format!("{}x{}/apps", size_str, size_str),
                        format!("{}/categories", size_str),
                        format!("{}x{}/categories", size_str, size_str),
                        format!("{}/mimetypes", size_str),
                        format!("{}x{}/mimetypes", size_str, size_str),
                        format!("{}/places", size_str),
                        format!("{}x{}/places", size_str, size_str),
                        format!("{}/devices", size_str),
                        format!("{}x{}/devices", size_str, size_str),
                        format!("{}/actions", size_str),
                        format!("{}x{}/actions", size_str, size_str),
                        format!("{}/status", size_str),
                        format!("{}x{}/status", size_str, size_str),
                        // Papirus-style paths
                        format!("{}x{}", size_str, size_str),
                        size_str.clone(),
                    ];
                    
                    for subdir in &subdirs {
                        let dir = theme_path.join(subdir);
                        if !dir.exists() {
                            continue;
                        }
                        
                        for ext in &extensions {
                            let icon_path = dir.join(format!("{}.{}", name, ext));
                            if icon_path.exists() {
                                return Some(icon_path);
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback: search pixmaps directories
        for base_path in &self.search_paths {
            let pixmaps = base_path.parent()?.join("pixmaps");
            if pixmaps.exists() {
                for ext in &extensions {
                    let icon_path = pixmaps.join(format!("{}.{}", name, ext));
                    if icon_path.exists() {
                        return Some(icon_path);
                    }
                }
            }
        }
        
        None
    }
    
    /// Get theme hierarchy (current theme + inherited themes + hicolor)
    fn get_theme_hierarchy(&self) -> Vec<String> {
        let mut themes = vec![self.theme.clone()];
        
        // Add parent themes based on common patterns
        // Papirus-Dark -> Papirus -> hicolor
        if self.theme.ends_with("-Dark") || self.theme.ends_with("-dark") {
            let base = self.theme.trim_end_matches("-Dark").trim_end_matches("-dark");
            if !themes.contains(&base.to_string()) {
                themes.push(base.to_string());
            }
        }
        if self.theme.ends_with("-Light") || self.theme.ends_with("-light") {
            let base = self.theme.trim_end_matches("-Light").trim_end_matches("-light");
            if !themes.contains(&base.to_string()) {
                themes.push(base.to_string());
            }
        }
        
        // Always include these fallbacks
        for fallback in &["Adwaita", "breeze", "hicolor"] {
            if !themes.contains(&fallback.to_string()) {
                themes.push(fallback.to_string());
            }
        }
        
        themes
    }
}

/// Detect icon theme from GTK settings
fn detect_icon_theme() -> Option<String> {
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))?;
    
    // Try GTK 4, then GTK 3
    for gtk_version in &["gtk-4.0", "gtk-3.0"] {
        let settings_path = config_home.join(gtk_version).join("settings.ini");
        if let Ok(content) = std::fs::read_to_string(&settings_path) {
            for line in content.lines() {
                if line.starts_with("gtk-icon-theme-name=") {
                    return Some(line.trim_start_matches("gtk-icon-theme-name=").to_string());
                }
            }
        }
    }
    
    None
}

/// Get all icon search paths from XDG_DATA_DIRS
fn get_icon_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // User icons first
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".icons"));
        paths.push(home.join(".local/share/icons"));
    }
    
    // XDG_DATA_HOME
    if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
        paths.push(PathBuf::from(data_home).join("icons"));
    }
    
    // XDG_DATA_DIRS
    if let Ok(data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in data_dirs.split(':') {
            if !dir.is_empty() {
                paths.push(PathBuf::from(dir).join("icons"));
            }
        }
    }
    
    // Standard fallbacks
    paths.push(PathBuf::from("/usr/share/icons"));
    paths.push(PathBuf::from("/usr/local/share/icons"));
    paths.push(PathBuf::from("/run/current-system/sw/share/icons")); // NixOS
    
    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    paths.retain(|p| seen.insert(p.clone()));
    
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_icon_theme() {
        let theme = detect_icon_theme();
        println!("Detected theme: {:?}", theme);
    }
    
    #[test]
    fn test_get_icon_search_paths() {
        let paths = get_icon_search_paths();
        println!("Search paths ({}):", paths.len());
        for p in &paths {
            println!("  {} {}", if p.exists() { "✓" } else { "✗" }, p.display());
        }
        assert!(!paths.is_empty());
    }
    
    #[test]
    fn test_icon_lookup() {
        let lookup = IconLookup::new();
        println!("Theme: {}", lookup.theme);
        println!("Theme hierarchy: {:?}", lookup.get_theme_hierarchy());
        
        // Test some common icons
        for icon in &["firefox", "chromium", "org.kde.ark", "utilities-terminal"] {
            let path = lookup.find_icon(icon, 64);
            println!("  {}: {:?}", icon, path);
        }
    }
    
    #[test]
    fn test_svg_loading() {
        let lookup = IconLookup::new();
        if let Some(path) = lookup.find_icon("firefox", 64) {
            println!("Loading SVG: {}", path.display());
            match load_icon_image(&path) {
                Some(img) => {
                    println!("  Loaded! Size: {}x{}", img.width(), img.height());
                }
                None => {
                    println!("  Failed to load!");
                }
            }
        } else {
            println!("Firefox icon not found");
        }
    }
}
