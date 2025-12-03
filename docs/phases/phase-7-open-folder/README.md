# Phase 7: Open Containing Folder

## Overview

Add ability to open the folder containing a `.desktop` file or the application's installation directory.

## Current State

The `Entry.path` field stores the full path to the `.desktop` file but is not used.

## Use Cases

1. **Edit .desktop file** - User wants to modify the desktop entry
2. **Find installation** - User wants to locate where an app is installed
3. **Debug issues** - User wants to inspect the .desktop file contents

## Implementation

### Keybind

Add `o` key in launcher mode to open containing folder:

```rust
// In handle_launcher_keys()
KeyCode::Char('o') if !app.is_filtering() => {
    if let Some(entry) = app.selected_entry() {
        app.open_containing_folder(entry)?;
    }
}
```

### Open Folder Logic

```rust
impl App {
    pub fn open_containing_folder(&self, entry: &Entry) -> Result<()> {
        let folder = entry.path.parent()
            .ok_or_else(|| anyhow::anyhow!("No parent directory"))?;
        
        // Use xdg-open or $VISUAL file manager
        let opener = std::env::var("FILE_MANAGER")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| "xdg-open".to_string());
        
        std::process::Command::new(&opener)
            .arg(folder)
            .spawn()
            .context("Failed to open folder")?;
        
        Ok(())
    }
}
```

### Alternative: Show Path in UI

Instead of (or in addition to) opening, show the path in the status bar:

```rust
// In draw_status_bar()
if let Some(entry) = app.selected_entry() {
    let path_display = entry.path.display();
    // Show truncated path
}
```

### Context Menu (Future)

For more options, implement a context menu:

```
┌─────────────────────────────┐
│ Firefox                     │
├─────────────────────────────┤
│ [Enter] Run                 │
│ [o]     Open folder         │
│ [e]     Edit .desktop       │
│ [c]     Copy command        │
│ [i]     Show info           │
└─────────────────────────────┘
```

## Configuration

```toml
[behavior]
file_manager = "nautilus"  # Override xdg-open
```

## Implementation Checklist

- [ ] Add `open_containing_folder()` method to App
- [ ] Add `o` keybind in launcher mode
- [ ] Update status bar to show keybind hint
- [ ] Handle errors gracefully (folder doesn't exist, no file manager)
- [ ] Add configuration option for file manager
- [ ] Consider adding "edit .desktop" option (`e` key)

## SSH Considerations

Opening folders over SSH is problematic:
- `xdg-open` won't work (no display)
- File manager won't be available

Options:
1. Disable the feature over SSH
2. Print the path to stderr instead
3. Copy path to clipboard (if available)

```rust
fn open_or_print_path(path: &Path) -> Result<()> {
    if std::env::var("SSH_CONNECTION").is_ok() {
        eprintln!("Path: {}", path.display());
        return Ok(());
    }
    // ... normal open logic
}
```
