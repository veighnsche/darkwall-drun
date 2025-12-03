use anyhow::Result;
use freedesktop_desktop_entry::{DesktopEntry, Iter};
use std::path::{Path, PathBuf};

/// Parsed desktop entry with fields we care about
#[derive(Debug, Clone)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub exec: Option<String>,
    pub icon: Option<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub terminal: bool,
    pub no_display: bool,
    pub path: PathBuf,
}

impl Entry {
    /// Create from freedesktop DesktopEntry
    fn from_desktop_entry(de: &DesktopEntry, path: &Path) -> Option<Self> {
        // The API takes a slice of locale strings, use empty slice for default
        let locales: &[&str] = &[];
        
        let name = de.name(locales)?.to_string();
        let exec = de.exec().map(|s| s.to_string());

        // Skip entries without exec
        if exec.is_none() {
            return None;
        }

        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let generic_name = de.generic_name(locales).map(|s| s.to_string());
        let comment = de.comment(locales).map(|s| s.to_string());
        let icon = de.icon().map(|s| s.to_string());

        let categories = de
            .categories()
            .map(|cats| cats.iter().map(|c| c.to_string()).collect())
            .unwrap_or_default();

        let keywords = de
            .keywords(locales)
            .map(|kws| kws.iter().map(|k| k.to_string()).collect())
            .unwrap_or_default();

        let terminal = de.terminal();
        let no_display = de.no_display();

        Some(Self {
            id,
            name,
            generic_name,
            comment,
            exec,
            icon,
            categories,
            keywords,
            terminal,
            no_display,
            path: path.to_path_buf(),
        })
    }

    /// Get display text for filtering/matching
    pub fn search_text(&self) -> String {
        let mut parts = vec![self.name.clone()];
        if let Some(ref gn) = self.generic_name {
            parts.push(gn.clone());
        }
        if let Some(ref c) = self.comment {
            parts.push(c.clone());
        }
        parts.extend(self.keywords.clone());
        parts.extend(self.categories.clone());
        parts.join(" ")
    }

    /// Get the command to execute (with field codes stripped)
    pub fn command(&self) -> Option<String> {
        self.exec.as_ref().map(|e| {
            // Strip field codes like %f, %F, %u, %U, etc.
            e.split_whitespace()
                .filter(|s| !s.starts_with('%'))
                .collect::<Vec<_>>()
                .join(" ")
        })
    }
}

/// Load all desktop entries from the given directories
pub fn load_all(dirs: &[PathBuf]) -> Result<Vec<Entry>> {
    let mut entries = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for dir in dirs {
        if !dir.exists() {
            tracing::debug!("Skipping non-existent directory: {}", dir.display());
            continue;
        }

        for path in Iter::new(std::iter::once(dir.to_path_buf())) {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    if let Ok(de) = DesktopEntry::from_str(&path, &content, None::<&[&str]>) {
                        if let Some(entry) = Entry::from_desktop_entry(&de, &path) {
                            // Skip NoDisplay entries
                            if entry.no_display {
                                continue;
                            }

                            // Deduplicate by ID (first one wins)
                            if seen_ids.insert(entry.id.clone()) {
                                entries.push(entry);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read {}: {}", path.display(), e);
                }
            }
        }
    }

    // Sort by name
    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(entries)
}
