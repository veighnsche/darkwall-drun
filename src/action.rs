//! Internal Action representation for DRUN.
//!
//! # Data Model
//!
//! DRUN's **only** configuration format is XDG `.desktop` entries.
//! This module defines the internal `Action` struct that DRUN uses
//! after parsing `.desktop` files.
//!
//! ```text
//! .desktop file (user's config)
//!        │
//!        ▼
//!   desktop_entry.rs (parser)
//!        │
//!        ▼
//!   Action struct (internal model)
//!        │
//!        ▼
//!   TUI / execution
//! ```
//!
//! # Field Synergy with NixOS actions.nix
//!
//! When `.desktop` files are generated from NixOS config, fields flow like this:
//!
//! ```text
//! actions.nix field  →  .desktop field  →  Action field
//! ─────────────────────────────────────────────────────────
//! id                 →  filename         →  id
//! name               →  Name             →  name
//! description        →  Comment          →  comment
//! command            →  Exec             →  exec
//! icon               →  Icon             →  icon
//! categories         →  Categories       →  categories
//! keywords           →  Keywords         →  keywords
//! terminal           →  Terminal         →  terminal
//! ```
//!
//! OS-only fields (waitForKey, interactive, notify) are handled by wrapper
//! scripts on the Nix side and don't appear in Action.
//!
//! # Important
//!
//! - The `Action` struct is **internal only** - users never write Actions directly.
//! - DRUN does **NOT** support custom config formats (no actions.toml, no JSON registry).
//! - If an app has a `.desktop` file, DRUN can run it. That's the contract.
//!
//! # SSH Usage
//!
//! ```text
//! ssh some-host drun
//! ```
//!
//! DRUN runs on the remote host, reads that host's `.desktop` files,
//! and executes commands there. No SSH management, no remote aggregation.

use crate::desktop_entry::Entry;

/// Internal representation of a launchable action.
///
/// This struct is populated from `.desktop` files only.
/// It reflects exactly what the XDG Desktop Entry spec defines - no more.
///
/// Users never create Actions directly; they create `.desktop` files.
#[derive(Debug, Clone)]
pub struct Action {
    /// Desktop entry ID (filename without .desktop)
    pub id: String,
    /// Name field from .desktop
    pub name: String,
    /// Comment field from .desktop (used for search)
    pub comment: Option<String>,
    /// Exec field (with %f, %u, etc. stripped)
    pub exec: String,
    /// Icon field (optional, may not be used in TUI)
    pub icon: Option<String>,
    /// Categories field (used for search and grouping)
    pub categories: Vec<String>,
    /// Keywords field from .desktop (used for search)
    /// TEAM_005: Added to improve search - aligns with XDG spec and Nix actions.nix
    pub keywords: Vec<String>,
    /// Whether Terminal=true in .desktop
    pub terminal: bool,
}

impl Action {
    /// Convert a desktop entry to an Action.
    ///
    /// Returns None if the entry has no valid Exec command.
    pub fn from_entry(entry: &Entry) -> Option<Self> {
        let exec = entry.command()?;
        Some(Self {
            id: entry.id.clone(),
            name: entry.name.clone(),
            comment: entry.comment.clone(),
            exec,
            icon: entry.icon.clone(),
            categories: entry.categories.clone(),
            keywords: entry.keywords.clone(),
            terminal: entry.terminal,
        })
    }

    /// Get text for fuzzy matching
    /// TEAM_005: Now includes keywords for better search alignment with Nix actions
    pub fn search_text(&self) -> String {
        let mut parts = vec![self.name.as_str()];
        if let Some(ref comment) = self.comment {
            parts.push(comment.as_str());
        }
        for kw in &self.keywords {
            parts.push(kw.as_str());
        }
        for cat in &self.categories {
            parts.push(cat.as_str());
        }
        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_text() {
        let action = Action {
            id: "firefox".to_string(),
            name: "Firefox".to_string(),
            comment: Some("Web Browser".to_string()),
            exec: "firefox".to_string(),
            icon: None,
            categories: vec!["Network".to_string()],
            keywords: vec!["internet".to_string(), "browse".to_string()],
            terminal: false,
        };

        let text = action.search_text();
        assert!(text.contains("Firefox"));
        assert!(text.contains("Web Browser"));
        assert!(text.contains("Network"));
        // TEAM_005: Keywords now included in search
        assert!(text.contains("internet"));
        assert!(text.contains("browse"));
    }
}
