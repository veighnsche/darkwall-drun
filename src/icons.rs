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

use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;

/// Icon manager - handles icon loading and caching
/// Must be initialized once at startup before entering raw mode
pub struct IconManager {
    /// The picker determines the graphics protocol and font size
    picker: Option<Picker>,
    /// Cache of loaded icon protocols by entry ID
    cache: HashMap<String, Arc<Mutex<StatefulProtocol>>>,
    /// Icon size in pixels
    icon_size: u16,
    /// Whether graphics are supported
    graphics_supported: bool,
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

        Self {
            picker,
            cache: HashMap::new(),
            icon_size,
            graphics_supported,
        }
    }

    /// Check if graphics icons are supported
    pub fn supports_graphics(&self) -> bool {
        self.graphics_supported
    }

    /// Load an icon for an entry, returning a cached protocol if available
    pub fn load_icon(&mut self, entry_id: &str, icon_name: Option<&str>) -> Option<Arc<Mutex<StatefulProtocol>>> {
        // Check cache first
        if let Some(cached) = self.cache.get(entry_id) {
            return Some(cached.clone());
        }

        // Need a picker and icon name
        let picker = self.picker.as_mut()?;
        let icon_name = icon_name?;

        // Resolve icon path
        let icon_path = resolve_icon_path(icon_name, self.icon_size)?;
        
        // Load the image
        let dyn_img = match image::open(&icon_path) {
            Ok(img) => img,
            Err(e) => {
                tracing::debug!("Failed to load icon {}: {}", icon_path.display(), e);
                return None;
            }
        };

        // Create the protocol
        let protocol = picker.new_resize_protocol(dyn_img);
        let arc = Arc::new(Mutex::new(protocol));
        
        // Cache it
        self.cache.insert(entry_id.to_string(), arc.clone());
        
        Some(arc)
    }

    /// Get a cached icon protocol
    pub fn get_cached(&self, entry_id: &str) -> Option<Arc<Mutex<StatefulProtocol>>> {
        self.cache.get(entry_id).cloned()
    }

    /// Clear the icon cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}


/// Resolve an icon name to a file path using freedesktop-icons
pub fn resolve_icon_path(icon_name: &str, size: u16) -> Option<PathBuf> {
    use freedesktop_icons::lookup;

    // If it's already an absolute path, use it directly
    if icon_name.starts_with('/') {
        let path = PathBuf::from(icon_name);
        if path.exists() {
            return Some(path);
        }
        return None;
    }

    // Try to find the icon in the theme
    lookup(icon_name)
        .with_size(size)
        .with_theme("hicolor")
        .find()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_icon_path() {
        // This may or may not find an icon depending on the system
        // Just verify no panic
        let _path = resolve_icon_path("firefox", 32);
    }
}
