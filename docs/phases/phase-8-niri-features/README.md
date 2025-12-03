# Phase 9: Niri Window Management Features

## Overview

Expand niri IPC integration with window-aware features and additional controls.

## Current State

Implemented but unused methods in `niri.rs`:
- `is_ok()` - Check if response succeeded
- `is_available()` - Check if niri socket exists
- `focused_window()` - Get info about focused window
- `toggle_floating()` - Toggle float state

## Feature: Health Check UI Indicator

Show niri connection status in the UI.

### Implementation

```rust
// In App
pub fn niri_status(&self) -> NiriStatus {
    match &self.niri {
        None => NiriStatus::Disabled,
        Some(client) if client.is_available() => NiriStatus::Connected,
        Some(_) => NiriStatus::Disconnected,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NiriStatus {
    Disabled,      // --no-niri flag or not configured
    Connected,     // Socket exists and responding
    Disconnected,  // Socket gone (niri crashed?)
}
```

### UI Display

```rust
// In draw_status_bar()
let niri_indicator = match app.niri_status() {
    NiriStatus::Connected => Span::styled("◉", Style::default().fg(Color::Green)),
    NiriStatus::Disconnected => Span::styled("◎", Style::default().fg(Color::Red)),
    NiriStatus::Disabled => Span::raw(""),
};
```

## Feature: Window Info Display

Show information about the current window.

### Use Cases

1. **Debugging** - Verify drun is running in expected window
2. **Context** - Show app_id for scripting purposes

### Implementation

```rust
// Periodic refresh of window info
impl App {
    pub async fn refresh_window_info(&mut self) {
        if let Some(ref niri) = self.niri {
            self.window_info = niri.focused_window().await.ok().flatten();
        }
    }
}

// In UI, show window info in debug mode
if config.debug.show_window_info {
    if let Some(ref info) = app.window_info {
        let text = format!("Window: {} ({})", info.title, info.app_id);
        // Render in corner
    }
}
```

## Feature: Toggle Floating Keybind

Allow user to toggle floating state from within drun.

### Keybind

```rust
// In handle_launcher_keys()
KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
    if let Some(ref niri) = app.niri {
        tokio::spawn({
            let niri = niri.clone();
            async move {
                niri.toggle_floating().await.ok();
            }
        });
    }
}
```

### Behavior

1. **Ctrl+F** toggles floating state
2. Visual feedback: briefly flash border or show indicator
3. Works in any mode (launcher, executing, post-execution)

## Feature: Smart Float/Unfloat

Automatically manage floating state based on context.

### Current Behavior

```rust
// On execute: unfloat (if configured)
if self.config.niri.unfloat_on_execute {
    niri.set_floating(false).await.ok();
}

// On command exit: re-float (if configured)
if self.config.niri.float_on_idle {
    niri.set_floating(true).await.ok();
}
```

### Enhanced Behavior

```rust
// Per-entry override via X-DarkwallUnfloatOnRun
if entry.get_darkwall_bool("UnfloatOnRun").unwrap_or(config.niri.unfloat_on_execute) {
    niri.set_floating(false).await.ok();
}

// Remember original state and restore it
let was_floating = niri.focused_window().await?.map(|w| w.is_floating);
// ... execute command ...
if let Some(true) = was_floating {
    niri.set_floating(true).await.ok();
}
```

## Feature: Workspace Awareness (Future)

Query and potentially switch workspaces.

```rust
// New niri methods
impl NiriClient {
    pub async fn current_workspace(&self) -> Result<WorkspaceInfo> { ... }
    pub async fn switch_workspace(&self, name: &str) -> Result<()> { ... }
}

// Use case: launch app on specific workspace
// X-DarkwallWorkspace=dev
```

## Configuration

```toml
[niri]
enabled = true
socket_path = null  # Auto-detect

# Floating behavior
float_on_idle = true
unfloat_on_execute = true

# UI indicators
show_status_indicator = true
show_window_info = false  # Debug mode

# Keybinds
toggle_float_key = "ctrl+f"
```

## Implementation Checklist

### Unit 9.1: Health Check
- [ ] Add `NiriStatus` enum
- [ ] Add `niri_status()` method to App
- [ ] Show indicator in status bar
- [ ] Periodic health check (every 5s?)

### Unit 9.2: Window Info
- [ ] Add `window_info` field to App
- [ ] Refresh on focus (or periodically)
- [ ] Display in debug mode

### Unit 9.3: Toggle Floating
- [ ] Add Ctrl+F keybind
- [ ] Visual feedback on toggle
- [ ] Handle errors gracefully

### Unit 9.4: Smart Float/Unfloat
- [ ] Remember original float state
- [ ] Per-entry override support
- [ ] Restore state on exit

## Testing

```bash
# Test with niri
niri &
cargo run

# Test without niri (should gracefully degrade)
unset NIRI_SOCKET
cargo run

# Test socket disappearing mid-session
# Kill niri while drun is running
```
